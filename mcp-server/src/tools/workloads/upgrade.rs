use rmcp::model::{CallToolResult, JsonObject};
use serde_json::{Value, json};

use crate::catalog::ExtensionCatalog;
use crate::helm::{self, HelmChartRef, HelmUpgradePlan};
use crate::k8s::{HelmReleaseDiscovery, KubernetesClient};
use crate::tools::args::{optional_bool, required_object, required_string};
use crate::tools::common::{kube_error, success, tool_error};
use crate::tools::schema_validation::validate_configuration_schema;

use super::{install, registry};

const TOOL_NAME: &str = "workloads.upgrade";

pub(crate) async fn upgrade(
    arguments: Option<&JsonObject>,
    catalog: &ExtensionCatalog,
) -> CallToolResult {
    let namespace = match required_string(arguments, "namespace") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let release_name = match required_string(arguments, "releaseName") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let dry_run = optional_bool(arguments, "dryRun").unwrap_or(true);
    let configuration = match required_object(arguments, "configuration") {
        Ok(value) => value,
        Err(error) => return error,
    };

    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error(TOOL_NAME, error),
    };
    let release = match HelmReleaseDiscovery::new(client)
        .get_latest(&namespace, &release_name)
        .await
    {
        Ok(Some(release)) => release,
        Ok(None) => {
            return tool_error(
                "not_found",
                format!("workload release not found: {namespace}/{release_name}"),
                json!({ "namespace": namespace, "releaseName": release_name }),
            );
        }
        Err(error) => {
            return tool_error(
                "helm_release_discovery_error",
                error.to_string(),
                json!({ "tool": TOOL_NAME, "namespace": namespace, "releaseName": release_name }),
            );
        }
    };
    let extension = match registry::extension_for_release(&release, catalog) {
        Some(extension) => extension,
        None => {
            return tool_error(
                "unsupported_workload",
                "workloads.upgrade only upgrades catalog-managed extension releases",
                json!({
                    "namespace": namespace,
                    "releaseName": release_name,
                    "chart": release.chart,
                }),
            );
        }
    };

    if let Err(error) = validate_configuration_schema(&configuration, &extension.configuration) {
        return error;
    }
    if !install::uses_direct_helm_values(extension)
        && configuration.get("namespace").and_then(Value::as_str) != Some(namespace.as_str())
    {
        return tool_error(
            "invalid_arguments",
            "configuration.namespace must match namespace",
            json!({ "namespace": namespace, "configurationNamespace": configuration.get("namespace") }),
        );
    }

    let resolved_configuration = install::apply_defaults(extension, Value::Object(configuration));
    let resolution = match install::resolve_configuration(
        extension,
        &namespace,
        resolved_configuration,
        dry_run,
    )
    .await
    {
        Ok(resolution) => resolution,
        Err(error) => return error,
    };
    let helm_values =
        install::planned_helm_values(extension, &release_name, &resolution.configuration);
    let chart = HelmChartRef {
        chart: extension.chart.clone(),
        version: extension.default_version.clone(),
    };

    if dry_run {
        return success(json!({
            "action": "upgrade",
            "dryRun": true,
            "wouldMutate": false,
            "release": {
                "name": release_name,
                "namespace": namespace,
                "current": release,
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
            "notes": [
                "dry-run planning only; no Kubernetes or Helm mutation was performed",
                "raw Helm values are rejected by the extension configuration schema",
                "upgrade requires an existing release and will not install missing releases"
            ],
        }));
    }

    let helm_result = match helm::upgrade(&HelmUpgradePlan {
        release_name: release_name.clone(),
        namespace: namespace.clone(),
        chart: chart.clone(),
        values: helm_values.clone(),
    })
    .await
    {
        Ok(result) => result,
        Err(error) => {
            let helm_details = match &error {
                helm::HelmUpgradeError::Failed {
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
            return tool_error("helm_upgrade_failed", error.to_string(), helm_details);
        }
    };

    success(json!({
        "action": "upgrade",
        "dryRun": false,
        "wouldMutate": true,
        "release": {
            "name": release_name,
            "namespace": namespace,
            "previous": release,
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
        "helm": helm_result,
        "notes": [
            "Helm upgrade completed successfully",
            "raw Helm values are rejected by the extension configuration schema"
        ],
    }))
}

#[cfg(test)]
mod tests {
    use crate::policy::{ApprovalClass, Scope};
    use crate::tools::workloads;

    use super::*;

    #[test]
    fn workload_upgrade_definition_is_mutating_but_not_destructive() {
        let definition = workloads::definitions()
            .iter()
            .find(|definition| definition.name == "workloads.upgrade")
            .unwrap();

        assert!(!definition.destructive);
        assert!(!definition.read_only);
        assert_eq!(definition.required_scope, Scope::WorkloadsUpgrade);
        assert_eq!(definition.approval_class, ApprovalClass::Mutation);
    }

    #[test]
    fn missing_configuration_returns_invalid_arguments() {
        let mut arguments = JsonObject::new();
        arguments.insert("namespace".to_string(), Value::String("hydra".to_string()));
        arguments.insert(
            "releaseName".to_string(),
            Value::String("hydra-offline".to_string()),
        );

        let error = required_object(Some(&arguments), "configuration").unwrap_err();

        assert_eq!(
            error
                .structured_content
                .as_ref()
                .and_then(|content| content.get("error")),
            Some(&Value::String("invalid_arguments".to_string()))
        );
    }

    #[test]
    fn schema_validation_rejects_unknown_values() {
        let schema = json!({
            "type": "object",
            "properties": { "namespace": { "type": "string" } },
            "additionalProperties": false
        });
        let mut values = JsonObject::new();
        values.insert("namespace".to_string(), Value::String("hydra".to_string()));
        values.insert("rawValues".to_string(), json!({}));

        let error = validate_configuration_schema(&values, &schema).unwrap_err();

        assert_eq!(
            error
                .structured_content
                .as_ref()
                .and_then(|content| content.get("error")),
            Some(&Value::String(
                "invalid_extension_configuration".to_string()
            ))
        );
    }
}
