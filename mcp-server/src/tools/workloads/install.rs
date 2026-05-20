use k8s_openapi::api::storage::v1::StorageClass;
use rmcp::model::CallToolResult;
use serde_json::Value;
use serde_json::json;

use crate::catalog::ExtensionDefinition;
use crate::k8s::KubernetesClient;
use crate::k8s::ResourceListParams;

use super::registry;
use crate::tools::common::kube_error;
use crate::tools::common::tool_error;
use crate::tools::k8s_summaries::storage_class_summary;

#[derive(Debug, Clone)]
pub(crate) struct InstallResolution {
    pub configuration: Value,
    pub available_storage_classes: Vec<Value>,
    pub recommended_storage_classes: Vec<String>,
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
        });
    }

    let client = match KubernetesClient::try_default().await {
        Ok(client) => Some(client),
        Err(_) if dry_run => None,
        Err(error) => return Err(kube_error("workloads.install", error)),
    };

    let mut available_storage_classes = vec![];
    let mut recommended_storage_classes = vec![];

    if let Some(client) = &client {
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

        let Some(storage_class) =
            input_string_at(&resolved_configuration, &["persistence", "storageClass"])
                .filter(|value| !value.trim().is_empty())
        else {
            return Err(tool_error(
                "invalid_extension_configuration",
                "missing required extension configuration value: persistence.storageClass",
                json!({ "field": "persistence.storageClass" }),
            ));
        };

        if !storage_classes
            .items
            .iter()
            .any(|candidate| candidate.metadata.name.as_deref() == Some(storage_class))
        {
            return Err(tool_error(
                "invalid_extension_configuration",
                "unsupported storage class for extension configuration field: persistence.storageClass",
                json!({
                    "field": "persistence.storageClass",
                    "actualValue": storage_class,
                    "availableStorageClasses": available_storage_classes,
                    "recommendedStorageClasses": recommended_storage_classes,
                }),
            ));
        }
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

fn uses_direct_helm_values(extension: &ExtensionDefinition) -> bool {
    matches!(
        extension.id.as_str(),
        registry::DOLOS_EXTENSION_ID
            | registry::CARDANO_RELAY_EXTENSION_ID
            | registry::CARDANO_BLOCK_PRODUCER_EXTENSION_ID
            | registry::APEX_FUSION_RELAY_EXTENSION_ID
            | registry::APEX_FUSION_BLOCK_PRODUCER_EXTENSION_ID
            | registry::HYDRA_NODE_EXTENSION_ID
    )
}

fn input_string_at<'a>(inputs: &'a Value, path: &[&str]) -> Option<&'a str> {
    path.iter()
        .try_fold(inputs, |current, segment| current.get(*segment))
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
