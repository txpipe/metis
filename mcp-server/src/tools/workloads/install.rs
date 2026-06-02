use std::collections::BTreeSet;

use k8s_openapi::api::storage::v1::StorageClass;
use rmcp::model::{CallToolResult, JsonObject};
use serde_json::Value;
use serde_json::json;

use crate::catalog::{ExtensionCatalog, ExtensionDefinition};
use crate::helm::{self, HelmChartRef, HelmInstallPlan};
use crate::k8s::ResourceListParams;
use crate::k8s::{HelmReleaseDiscovery, KubernetesClient};

use super::registry;
use crate::tools::args::{optional_bool, required_object, required_string};
use crate::tools::common::kube_error;
use crate::tools::common::success;
use crate::tools::common::tool_error;
use crate::tools::k8s_summaries::storage_class_summary;
use crate::tools::schema_validation::{annotated_field_paths, validate_configuration_schema};

const TOOL_NAME: &str = "workloads.install";

pub(crate) async fn install(
    arguments: Option<&JsonObject>,
    catalog: &ExtensionCatalog,
) -> CallToolResult {
    let extension_id = match required_string(arguments, "extensionId") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let release_name = match required_string(arguments, "releaseName") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let namespace = match required_string(arguments, "namespace") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let dry_run = optional_bool(arguments, "dryRun").unwrap_or(true);

    let configuration = match required_object(arguments, "configuration") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let extension = match catalog.get(&extension_id) {
        Some(extension) => extension,
        None => {
            return tool_error(
                "unknown_extension",
                format!("extension not found: {extension_id}"),
                json!({ "extensionId": extension_id }),
            );
        }
    };

    if let Err(error) = validate_configuration_schema(&configuration, &extension.configuration) {
        return error;
    }

    if !uses_direct_helm_values(extension)
        && configuration.get("namespace").and_then(Value::as_str) != Some(namespace.as_str())
    {
        return tool_error(
            "invalid_arguments",
            "configuration.namespace must match namespace",
            json!({ "namespace": namespace, "configurationNamespace": configuration.get("namespace") }),
        );
    }

    let resolved_configuration = apply_defaults(extension, Value::Object(configuration));
    let resolution =
        match resolve_configuration(extension, &namespace, resolved_configuration, dry_run).await {
            Ok(resolution) => resolution,
            Err(error) => return error,
        };
    let helm_values = planned_helm_values(extension, &release_name, &resolution.configuration);
    let chart = HelmChartRef {
        chart: extension.chart.clone(),
        version: extension.default_version.clone(),
    };

    if dry_run {
        let mut notes = vec![
            "dry-run planning only; no Kubernetes or Helm mutation was performed".to_string(),
            "raw Helm values are rejected by the extension configuration schema".to_string(),
        ];
        let missing_dependencies = missing_dependency_ids(&resolution.dependencies);
        if !missing_dependencies.is_empty() {
            notes.push(format!(
                "required extension dependencies are not currently deployed: {}",
                missing_dependencies.join(", ")
            ));
        }
        if resolution.dependency_check_skipped {
            notes.push(
                "dependency deployment checks were skipped because Kubernetes discovery was unavailable during dry-run"
                    .to_string(),
            );
        }

        return success(json!({
            "action": "install",
            "dryRun": true,
            "wouldMutate": false,
            "release": {
                "name": release_name,
                "namespace": namespace,
            },
            "extension": {
                "id": extension.id,
                "name": extension.name,
                "version": extension.default_version,
            },
            "chart": chart,
            "resolvedConfiguration": resolution.configuration,
            "helmValues": helm_values,
            "availableStorageClasses": resolution.available_storage_classes,
            "recommendedStorageClasses": resolution.recommended_storage_classes,
            "dependencies": resolution.dependencies,
            "notes": notes,
        }));
    }

    let plan = HelmInstallPlan {
        release_name: release_name.clone(),
        namespace: namespace.clone(),
        chart: chart.clone(),
        values: helm_values.clone(),
    };
    let helm_result = match helm::install(&plan).await {
        Ok(result) => result,
        Err(error) => {
            let helm_details = match &error {
                helm::HelmInstallError::Failed {
                    status,
                    stdout,
                    stderr,
                } => json!({
                    "tool": TOOL_NAME,
                    "extensionId": extension.id,
                    "releaseName": release_name,
                    "namespace": namespace,
                    "status": status,
                    "stdout": stdout,
                    "stderr": stderr,
                }),
                _ => json!({
                    "tool": TOOL_NAME,
                    "extensionId": extension.id,
                    "releaseName": release_name,
                    "namespace": namespace,
                }),
            };
            return tool_error("helm_install_failed", error.to_string(), helm_details);
        }
    };

    success(json!({
        "action": "install",
        "dryRun": false,
        "wouldMutate": true,
        "release": {
            "name": release_name,
            "namespace": namespace,
        },
        "extension": {
            "id": extension.id,
            "name": extension.name,
            "version": extension.default_version,
        },
        "chart": chart,
        "resolvedConfiguration": resolution.configuration,
        "helmValues": helm_values,
        "availableStorageClasses": resolution.available_storage_classes,
        "recommendedStorageClasses": resolution.recommended_storage_classes,
        "dependencies": resolution.dependencies,
        "helm": helm_result,
        "notes": [
            "Helm upgrade --install completed successfully",
            "raw Helm values are rejected by the extension configuration schema"
        ],
    }))
}

#[derive(Debug, Clone)]
pub(crate) struct InstallResolution {
    pub configuration: Value,
    pub available_storage_classes: Vec<Value>,
    pub recommended_storage_classes: Vec<String>,
    pub dependencies: Vec<Value>,
    pub dependency_check_skipped: bool,
}

pub(crate) async fn resolve_configuration(
    extension: &ExtensionDefinition,
    _namespace: &str,
    resolved_configuration: Value,
    dry_run: bool,
) -> Result<InstallResolution, CallToolResult> {
    if !uses_direct_helm_values(extension) {
        return Ok(InstallResolution {
            configuration: resolved_configuration,
            available_storage_classes: vec![],
            recommended_storage_classes: vec![],
            dependencies: vec![],
            dependency_check_skipped: false,
        });
    }

    let client = match KubernetesClient::try_default().await {
        Ok(client) => Some(client),
        Err(_) if dry_run => None,
        Err(error) => return Err(kube_error("workloads.install", error)),
    };

    let mut available_storage_classes = vec![];
    let mut recommended_storage_classes = vec![];
    let mut dependencies = vec![];
    let mut dependency_check_skipped = false;

    if let Some(client) = &client {
        let installed_extension_ids = HelmReleaseDiscovery::new(client.clone())
            .list_latest(None, false)
            .await
            .map(|releases| {
                releases
                    .into_iter()
                    .filter(|release| release.status.as_deref() == Some("deployed"))
                    .filter_map(|release| release.chart.name)
                    .collect::<BTreeSet<_>>()
            })
            .map_err(|error| {
                tool_error(
                    "helm_release_discovery_error",
                    error.to_string(),
                    json!({ "tool": TOOL_NAME, "extensionId": extension.id }),
                )
            })?;
        dependencies = dependency_status(extension, &installed_extension_ids);
        let missing_dependencies = missing_dependency_ids(&dependencies);
        if !dry_run && !missing_dependencies.is_empty() {
            return Err(tool_error(
                "missing_extension_dependencies",
                format!(
                    "required extension dependencies are not deployed: {}",
                    missing_dependencies.join(", ")
                ),
                json!({
                    "extensionId": extension.id,
                    "dependencies": dependencies,
                    "missingDependencies": missing_dependencies,
                }),
            ));
        }

        let storage_classes = client
            .list_storage_classes(&ResourceListParams::default())
            .await
            .map_err(|error| kube_error("workloads.install", error))?;
        available_storage_classes = storage_classes
            .items
            .iter()
            .map(storage_class_summary)
            .collect();
        recommended_storage_classes = recommended_storage_class_names(&storage_classes.items);

        for field in annotated_field_paths(
            &extension.configuration,
            "x-supernodeRole",
            "storageClass",
        ) {
            let Some(storage_class) =
                input_string_at_path(&resolved_configuration, &field)
                    .filter(|value| !value.trim().is_empty())
            else {
                return Err(tool_error(
                    "invalid_extension_configuration",
                    format!("missing required extension configuration value: {field}"),
                    json!({ "field": field }),
                ));
            };

            if !storage_classes
                .items
                .iter()
                .any(|candidate| candidate.metadata.name.as_deref() == Some(storage_class))
            {
                return Err(tool_error(
                    "invalid_extension_configuration",
                    format!("unsupported storage class for extension configuration field: {field}"),
                    json!({
                        "field": field,
                        "actualValue": storage_class,
                        "availableStorageClasses": available_storage_classes,
                        "recommendedStorageClasses": recommended_storage_classes,
                    }),
                ));
            }
        }
    } else if !extension.dependencies.is_empty() {
        dependency_check_skipped = true;
        dependencies = unknown_dependency_status(extension);
    }

    if extension.id == registry::DOLOS_EXTENSION_ID
        && input_string_at(&resolved_configuration, &["config", "upstreamAddress"])
            .is_none_or(|value| value.trim().is_empty())
    {
        return Err(tool_error(
            "invalid_extension_configuration",
            "missing required extension configuration value: config.upstreamAddress",
            json!({
                "field": "config.upstreamAddress",
                "reason": "MCP does not auto-resolve Dolos upstream relays; run workloads.list to inspect same-network Cardano relay candidates",
            }),
        ));
    }

    if matches!(
        extension.id.as_str(),
        registry::CARDANO_BLOCK_PRODUCER_EXTENSION_ID
            | registry::APEX_FUSION_BLOCK_PRODUCER_EXTENSION_ID
    ) && resolved_configuration
        .pointer("/relays/count")
        .and_then(Value::as_i64)
        == Some(0)
        && resolved_configuration
            .pointer("/relays/trusted")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
    {
        return Err(tool_error(
            "invalid_extension_configuration",
            "missing required extension configuration value: relays.trusted",
            json!({
                "field": "relays.trusted",
                "reason": "relays.trusted is required when relays.count=0; run workloads.list to inspect same-network relay candidates",
            }),
        ));
    }

    Ok(InstallResolution {
        configuration: resolved_configuration,
        available_storage_classes,
        recommended_storage_classes,
        dependencies,
        dependency_check_skipped,
    })
}

pub(crate) fn apply_defaults(extension: &ExtensionDefinition, inputs: Value) -> Value {
    let _ = extension;
    inputs
}

pub(crate) fn planned_helm_values(
    extension: &ExtensionDefinition,
    release_name: &str,
    inputs: &Value,
) -> Value {
    if uses_direct_helm_values(extension) {
        let mut helm_values = serde_json::Map::new();
        if let Some(values) = inputs.as_object() {
            helm_values.extend(values.clone());
        }
        helm_values.insert(
            "displayName".to_string(),
            Value::String(release_name.to_string()),
        );
        return Value::Object(helm_values);
    }

    let mut helm_values = serde_json::Map::new();
    helm_values.insert(
        "displayName".to_string(),
        Value::String(release_name.to_string()),
    );
    Value::Object(helm_values)
}

pub(crate) fn uses_direct_helm_values(extension: &ExtensionDefinition) -> bool {
    matches!(
        extension.id.as_str(),
        registry::DOLOS_EXTENSION_ID
            | registry::CARDANO_RELAY_EXTENSION_ID
            | registry::CARDANO_DB_SYNC_EXTENSION_ID
            | registry::CARDANO_BLOCK_PRODUCER_EXTENSION_ID
            | registry::APEX_FUSION_RELAY_EXTENSION_ID
            | registry::APEX_FUSION_BLOCK_PRODUCER_EXTENSION_ID
            | registry::HYDRA_NODE_EXTENSION_ID
            | registry::MIDNIGHT_EXTENSION_ID
    )
}

fn dependency_status(
    extension: &ExtensionDefinition,
    installed_extension_ids: &BTreeSet<String>,
) -> Vec<Value> {
    extension
        .dependencies
        .iter()
        .map(|dependency| {
            json!({
                "extensionId": dependency,
                "installed": installed_extension_ids.contains(dependency),
            })
        })
        .collect()
}

fn unknown_dependency_status(extension: &ExtensionDefinition) -> Vec<Value> {
    extension
        .dependencies
        .iter()
        .map(|dependency| {
            json!({
                "extensionId": dependency,
                "installed": Value::Null,
            })
        })
        .collect()
}

fn missing_dependency_ids(dependencies: &[Value]) -> Vec<String> {
    dependencies
        .iter()
        .filter(|dependency| dependency.get("installed") == Some(&Value::Bool(false)))
        .filter_map(|dependency| dependency.get("extensionId").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect()
}

fn input_string_at<'a>(inputs: &'a Value, path: &[&str]) -> Option<&'a str> {
    path.iter()
        .try_fold(inputs, |current, segment| current.get(*segment))
        .and_then(Value::as_str)
}

fn input_string_at_path<'a>(inputs: &'a Value, path: &str) -> Option<&'a str> {
    path.split('.')
        .try_fold(inputs, |current, segment| current.get(segment))
        .and_then(Value::as_str)
}

fn recommended_storage_class_names(storage_classes: &[StorageClass]) -> Vec<String> {
    let mut defaults = storage_classes
        .iter()
        .filter(|storage_class| {
            storage_class
                .metadata
                .annotations
                .as_ref()
                .is_some_and(|annotations| {
                    annotations
                        .get("storageclass.kubernetes.io/is-default-class")
                        .is_some_and(|value| value == "true")
                        || annotations
                            .get("storageclass.beta.kubernetes.io/is-default-class")
                            .is_some_and(|value| value == "true")
                })
        })
        .filter_map(|storage_class| storage_class.metadata.name.clone())
        .collect::<Vec<_>>();

    defaults.sort();
    if !defaults.is_empty() {
        return defaults;
    }

    let mut names = storage_classes
        .iter()
        .filter_map(|storage_class| storage_class.metadata.name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use crate::catalog::ExtensionCatalog;
    use crate::tools::schema_validation::annotated_field_paths;

    #[test]
    fn cardano_db_sync_discovers_nested_storage_class_fields_from_annotations() {
        let catalog = ExtensionCatalog::testing();
        let extension = catalog.get("cardano-db-sync").unwrap();

        assert_eq!(
            annotated_field_paths(&extension.configuration, "x-supernodeRole", "storageClass"),
            vec![
                "dbSync.persistence.storageClass".to_string(),
                "postgres.persistence.storageClass".to_string(),
            ]
        );
    }

    #[test]
    fn relay_discovers_top_level_storage_class_field_from_annotations() {
        let catalog = ExtensionCatalog::testing();
        let extension = catalog.get("cardano-relay").unwrap();

        assert_eq!(
            annotated_field_paths(&extension.configuration, "x-supernodeRole", "storageClass"),
            vec!["persistence.storageClass".to_string()]
        );
    }

    #[test]
    fn cardano_block_producer_discovers_only_always_required_storage_class_fields() {
        let catalog = ExtensionCatalog::testing();
        let extension = catalog.get("cardano-block-producer").unwrap();

        assert_eq!(
            annotated_field_paths(&extension.configuration, "x-supernodeRole", "storageClass"),
            vec!["persistence.storageClass".to_string()]
        );
    }
}
