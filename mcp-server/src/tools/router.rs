use std::sync::Arc;

use rmcp::model::{CallToolResult, JsonObject, ListToolsResult, Meta, Tool, ToolAnnotations};
use serde_json::{Value, json};

use crate::{
    catalog::{ExtensionCatalog, extension_summary},
    k8s::{KubernetesClient, ResourceListParams},
    vault::WriteMode,
};

use super::{
    ToolDefinition,
    args::{optional_string, optional_u32, required_string},
    common::kube_error,
    common::success,
    common::tool_error,
    hydra, k8s_summaries, supernode, vault, workloads,
    workloads::{install, logs},
};

#[derive(Debug, Clone)]
pub struct ToolRouter {
    definitions: Arc<Vec<ToolDefinition>>,
}

impl ToolRouter {
    pub fn new() -> Self {
        let definitions = supernode::definitions()
            .iter()
            .chain(workloads::definitions())
            .chain(vault::definitions())
            .chain(hydra::definitions())
            .copied()
            .collect();

        Self {
            definitions: Arc::new(definitions),
        }
    }

    pub fn list_with_dynamic(&self, dynamic_definitions: &[ToolDefinition]) -> ListToolsResult {
        ListToolsResult::with_all_items(
            self.definitions
                .iter()
                .chain(dynamic_definitions.iter())
                .map(|definition| tool_from_definition(*definition))
                .collect(),
        )
    }

    pub fn get_with_dynamic(
        &self,
        name: &str,
        dynamic_definitions: &[ToolDefinition],
    ) -> Option<ToolDefinition> {
        self.definitions
            .iter()
            .chain(dynamic_definitions.iter())
            .find(|definition| definition.name == name)
            .copied()
    }

    pub fn not_implemented_result(&self, definition: ToolDefinition) -> CallToolResult {
        CallToolResult::structured_error(json!({
            "error": "not_implemented",
            "tool": definition.name,
            "message": "Tool execution is not implemented in this incremental step.",
        }))
    }

    pub async fn call(
        &self,
        definition: ToolDefinition,
        arguments: Option<&JsonObject>,
        catalog: &ExtensionCatalog,
    ) -> CallToolResult {
        match definition.name {
            "supernode.status.get" => supernode_status(catalog).await,
            "cluster.storage_classes.list" => storage_classes_list(arguments).await,
            "cluster.events.list" => events_list(arguments).await,
            "extensions.catalog.list" => catalog_list(catalog),
            "extensions.catalog.get" => catalog_get(arguments, catalog),
            "vault.runtime.metadata.get" => vault::runtime_metadata_get(arguments).await,
            "vault.runtime.write" => vault::runtime_write(arguments, WriteMode::Replace).await,
            "vault.runtime.patch" => vault::runtime_write(arguments, WriteMode::Patch).await,
            "hydra.keys.generate" => hydra::generate_keys(arguments).await,
            "workloads.list" => workloads::list::list(arguments, catalog).await,
            "workloads.get" => workloads::get::get(arguments, catalog).await,
            "workloads.logs.get" => logs::get(arguments).await,
            "workloads.metrics.get" => workloads::metrics::get(arguments, catalog).await,
            "workloads.install" => install::install(arguments, catalog).await,
            "workloads.upgrade" => workloads::upgrade::upgrade(arguments, catalog).await,
            "workloads.delete" => workloads::delete::delete(arguments, catalog).await,
            "dolos.snapshot.refresh" => workloads::dolos::snapshot_refresh(arguments).await,
            _ => self.not_implemented_result(definition),
        }
    }
}

async fn supernode_status(catalog: &ExtensionCatalog) -> CallToolResult {
    let kubernetes = match KubernetesClient::try_default().await {
        Ok(client) => match client
            .list_namespaces(&ResourceListParams {
                limit: Some(1),
                ..Default::default()
            })
            .await
        {
            Ok(namespaces) => json!({
                "connected": true,
                "namespaceSampleCount": namespaces.items.len(),
            }),
            Err(error) => json!({
                "connected": false,
                "error": error.to_string(),
            }),
        },
        Err(error) => json!({
            "connected": false,
            "error": error.to_string(),
        }),
    };

    success(json!({
        "status": "ok",
        "catalogExtensionCount": catalog.len(),
        "kubernetes": kubernetes,
    }))
}

async fn storage_classes_list(arguments: Option<&JsonObject>) -> CallToolResult {
    let params = list_params(arguments, Some(100));
    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error("cluster.storage_classes.list", error),
    };

    match client.list_storage_classes(&params).await {
        Ok(storage_classes) => success(json!({
            "storageClasses": storage_classes.items.iter().map(k8s_summaries::storage_class_summary).collect::<Vec<_>>(),
        })),
        Err(error) => kube_error("cluster.storage_classes.list", error),
    }
}

async fn events_list(arguments: Option<&JsonObject>) -> CallToolResult {
    let params = list_params(arguments, Some(100));
    let namespace = optional_string(arguments, "namespace");
    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error("cluster.events.list", error),
    };

    match client.list_events(namespace.as_deref(), &params).await {
        Ok(events) => success(json!({
            "namespace": namespace,
            "events": events.items.iter().map(k8s_summaries::event_summary).collect::<Vec<_>>(),
        })),
        Err(error) => kube_error("cluster.events.list", error),
    }
}

fn catalog_list(catalog: &ExtensionCatalog) -> CallToolResult {
    success(json!({
        "extensions": catalog.list().map(extension_summary).collect::<Vec<_>>(),
    }))
}

fn catalog_get(arguments: Option<&JsonObject>, catalog: &ExtensionCatalog) -> CallToolResult {
    let extension_id = match required_string(arguments, "extensionId") {
        Ok(value) => value,
        Err(error) => return error,
    };

    match catalog.get(&extension_id) {
        Some(extension) => success(json!({ "extension": extension })),
        None => tool_error(
            "not_found",
            format!("extension not found: {extension_id}"),
            json!({ "extensionId": extension_id }),
        ),
    }
}

fn tool_from_definition(definition: ToolDefinition) -> Tool {
    Tool::new(
        definition.name,
        definition.description,
        input_schema(definition.input_schema),
    )
    .with_title(definition.title)
    .with_annotations(
        ToolAnnotations::with_title(definition.title)
            .read_only(definition.read_only)
            .destructive(definition.destructive)
            .idempotent(definition.read_only)
            .open_world(false),
    )
    .with_meta(tool_meta(definition))
}

fn input_schema(schema: &str) -> JsonObject {
    match serde_json::from_str::<Value>(schema).expect("tool input schema must be valid JSON") {
        Value::Object(object) => object,
        _ => panic!("tool input schema must be a JSON object"),
    }
}

fn tool_meta(definition: ToolDefinition) -> Meta {
    let mut meta = JsonObject::new();
    meta.insert(
        "requiredScope".to_string(),
        serde_json::to_value(definition.required_scope)
            .expect("serializing required scope must not fail"),
    );
    meta.insert(
        "approvalClass".to_string(),
        serde_json::to_value(definition.approval_class)
            .expect("serializing approval class must not fail"),
    );
    meta.insert(
        "approvalRequired".to_string(),
        Value::Bool(definition.approval_class.requires_approval()),
    );
    Meta(meta)
}

fn list_params(arguments: Option<&JsonObject>, default_limit: Option<u32>) -> ResourceListParams {
    ResourceListParams {
        label_selector: optional_string(arguments, "labelSelector"),
        field_selector: optional_string(arguments, "fieldSelector"),
        limit: optional_u32(arguments, "limit").or(default_limit),
    }
}

impl Default for ToolRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::ExtensionCatalog;
    use crate::k8s::{HelmChartSummary, HelmReleaseSummary};

    use super::*;

    #[test]
    fn lists_mvp_tools_with_policy_metadata() {
        let router = ToolRouter::new();

        let tools = router.list_with_dynamic(&[]).tools;

        let install = tools
            .iter()
            .find(|tool| tool.name == "workloads.install")
            .unwrap();
        assert_eq!(
            install.meta.as_ref().unwrap().0.get("requiredScope"),
            Some(&Value::String("workloads-install".to_string()))
        );
        assert_eq!(
            install.meta.as_ref().unwrap().0.get("approvalClass"),
            Some(&Value::String("mutation".to_string()))
        );
    }

    #[test]
    fn does_not_include_extension_specific_install_tools() {
        let router = ToolRouter::new();

        let tools = router.list_with_dynamic(&[]).tools;

        assert!(
            !tools
                .iter()
                .any(|tool| tool.name == "cardano.relay.install")
        );
        assert!(
            !tools
                .iter()
                .any(|tool| tool.name == "cardano.producer.verify")
        );
        assert!(!tools.iter().any(|tool| tool.name == "dolos.deploy"));
        assert!(tools.iter().any(|tool| tool.name == "workloads.install"));
    }

    #[test]
    fn can_lookup_tool_definition() {
        let router = ToolRouter::new();

        let definition = router.get_with_dynamic("workloads.delete", &[]).unwrap();

        assert!(definition.destructive);
    }

    #[test]
    fn workloads_logs_schema_exposes_pod_and_container_selection() {
        let router = ToolRouter::new();
        let definition = router.get_with_dynamic("workloads.logs.get", &[]).unwrap();

        assert!(definition.input_schema.contains("pod"));
        assert!(definition.input_schema.contains("container"));
        assert!(definition.input_schema.contains("previous"));
        assert!(definition.input_schema.contains("sinceSeconds"));
        assert!(definition.input_schema.contains("timestamps"));
    }

    #[test]
    fn dynamic_tool_definitions_are_listed_and_resolved_when_supplied() {
        let router = ToolRouter::new();
        let dynamic = workloads::dolos::definitions();

        assert!(
            router
                .get_with_dynamic("dolos.snapshot.refresh", &[])
                .is_none()
        );
        assert!(
            router
                .list_with_dynamic(dynamic)
                .tools
                .iter()
                .any(|tool| tool.name == "dolos.snapshot.refresh")
        );
        assert!(
            router
                .get_with_dynamic("dolos.snapshot.refresh", dynamic)
                .is_some()
        );
    }

    #[tokio::test]
    async fn executes_catalog_get_tool() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router
            .get_with_dynamic("extensions.catalog.get", &[])
            .unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "extensionId".to_string(),
            Value::String("cardano-relay".to_string()),
        );

        let result = router.call(definition, Some(&arguments), &catalog).await;

        assert_eq!(result.is_error, Some(false));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.pointer("/extension/id")),
            Some(&Value::String("cardano-relay".to_string()))
        );
        let content = result.structured_content.as_ref().unwrap();
        assert!(content.pointer("/extension/configuration").is_some());
        assert!(content.pointer("/extension/metrics").is_some());
    }

    #[tokio::test]
    async fn catalog_list_returns_summaries_without_large_schemas() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router
            .get_with_dynamic("extensions.catalog.list", &[])
            .unwrap();

        let result = router.call(definition, None, &catalog).await;

        assert_eq!(result.is_error, Some(false));
        let content = result.structured_content.as_ref().unwrap();
        let dolos = content
            .pointer("/extensions")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .find(|extension| extension.pointer("/id") == Some(&json!("dolos")))
            .unwrap();
        assert_eq!(dolos.pointer("/name"), Some(&json!("Dolos")));
        assert!(dolos.get("configuration").is_none());
        assert!(dolos.get("metrics").is_none());
        assert!(dolos.get("metricsCollection").is_none());
        assert!(dolos.get("secrets").is_none());
        assert!(
            dolos
                .pointer("/outputs")
                .and_then(Value::as_array)
                .is_some_and(|outputs| outputs
                    .iter()
                    .any(|output| { output.pointer("/name") == Some(&json!("kupo")) }))
        );
    }

    #[test]
    fn workload_summary_uses_catalog_extension_summary() {
        let catalog = ExtensionCatalog::testing();
        let release = HelmReleaseSummary {
            name: "dolos-preview".to_string(),
            namespace: "cardano".to_string(),
            revision: 1,
            status: Some("deployed".to_string()),
            chart: HelmChartSummary {
                name: Some("dolos".to_string()),
                version: Some("0.1.0".to_string()),
            },
            app_version: Some("1.1.1".to_string()),
            description: None,
            updated: None,
            secret_name: None,
            config: None,
        };

        let summary = workloads::list::workload_summary(&release, &catalog);

        assert_eq!(summary.pointer("/name"), Some(&json!("dolos-preview")));
        assert_eq!(
            summary.pointer("/catalogExtension/id"),
            Some(&json!("dolos"))
        );
        assert!(summary.pointer("/catalogExtension/configuration").is_none());
        assert!(summary.pointer("/catalogExtension/metrics").is_none());
        assert!(
            summary
                .pointer("/catalogExtension/metricsCollection")
                .is_none()
        );
        assert!(
            summary
                .pointer("/catalogExtension/outputs")
                .and_then(Value::as_array)
                .is_some_and(|outputs| outputs
                    .iter()
                    .any(|output| { output.pointer("/name") == Some(&json!("kupo")) }))
        );
    }

    #[tokio::test]
    async fn missing_required_argument_returns_tool_error() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router
            .get_with_dynamic("extensions.catalog.get", &[])
            .unwrap();

        let result = router.call(definition, None, &catalog).await;

        assert_eq!(result.is_error, Some(true));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("error")),
            Some(&Value::String("invalid_arguments".to_string()))
        );
    }

    #[tokio::test]
    async fn workloads_metrics_get_dispatches_to_argument_validation() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router
            .get_with_dynamic("workloads.metrics.get", &[])
            .unwrap();

        let result = router.call(definition, None, &catalog).await;

        assert_eq!(result.is_error, Some(true));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("error")),
            Some(&Value::String("invalid_arguments".to_string()))
        );
    }

    #[tokio::test]
    async fn workloads_upgrade_dispatches_to_argument_validation() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router.get_with_dynamic("workloads.upgrade", &[]).unwrap();

        let result = router.call(definition, None, &catalog).await;

        assert_eq!(result.is_error, Some(true));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("error")),
            Some(&Value::String("invalid_arguments".to_string()))
        );
    }

    #[tokio::test]
    async fn workloads_install_dry_run_returns_validated_plan() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router.get_with_dynamic("workloads.install", &[]).unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "extensionId".to_string(),
            Value::String("cardano-relay".to_string()),
        );
        arguments.insert(
            "releaseName".to_string(),
            Value::String("relay-preview".to_string()),
        );
        arguments.insert(
            "namespace".to_string(),
            Value::String("cardano".to_string()),
        );
        arguments.insert("dryRun".to_string(), Value::Bool(true));
        arguments.insert(
            "configuration".to_string(),
            json!({
                "node": {
                    "network": "preview",
                },
                "persistence": {
                    "storageClass": "standard",
                    "size": "80Gi",
                },
            }),
        );

        let result = router.call(definition, Some(&arguments), &catalog).await;

        assert_eq!(result.is_error, Some(false));
        let content = result.structured_content.as_ref().unwrap();
        assert_eq!(content.pointer("/wouldMutate"), Some(&Value::Bool(false)));
        assert_eq!(
            content.pointer("/extension/id"),
            Some(&Value::String("cardano-relay".to_string()))
        );
        assert_eq!(
            content.pointer("/chart/chart"),
            Some(&Value::String(
                "oci://oci.supernode.store/extensions/cardano-relay".to_string()
            ))
        );
        assert_eq!(
            content.pointer("/helmValues/node/network"),
            Some(&Value::String("preview".to_string()))
        );
        assert_eq!(
            content.pointer("/helmValues/persistence/size"),
            Some(&Value::String("80Gi".to_string()))
        );
    }

    #[tokio::test]
    async fn workloads_install_rejects_unknown_raw_helm_values() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router.get_with_dynamic("workloads.install", &[]).unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "extensionId".to_string(),
            Value::String("cardano-relay".to_string()),
        );
        arguments.insert(
            "releaseName".to_string(),
            Value::String("relay-preview".to_string()),
        );
        arguments.insert(
            "namespace".to_string(),
            Value::String("cardano".to_string()),
        );
        arguments.insert(
            "configuration".to_string(),
            json!({
                "node": {
                    "network": "preview",
                },
                "persistence": {
                    "storageClass": "standard",
                },
                "rawValues": { "node": { "replicas": 10 } },
            }),
        );

        let result = router.call(definition, Some(&arguments), &catalog).await;

        assert_eq!(result.is_error, Some(true));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("error")),
            Some(&Value::String(
                "invalid_extension_configuration".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn dolos_install_dry_run_returns_validated_plan() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router.get_with_dynamic("workloads.install", &[]).unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "extensionId".to_string(),
            Value::String("dolos".to_string()),
        );
        arguments.insert(
            "releaseName".to_string(),
            Value::String("dolos-preview".to_string()),
        );
        arguments.insert(
            "namespace".to_string(),
            Value::String("cardano".to_string()),
        );
        arguments.insert("dryRun".to_string(), Value::Bool(true));
        arguments.insert(
            "configuration".to_string(),
            json!({
                "dolos": {
                    "network": "cardano-preview",
                },
                "persistence": {
                    "storageClass": "standard",
                    "size": "50Gi",
                },
                "config": {
                    "upstreamAddress": "relay-preview-cardano-relay.cardano.svc.cluster.local:3000",
                },
            }),
        );

        let result = router.call(definition, Some(&arguments), &catalog).await;

        assert_eq!(result.is_error, Some(false));
        let content = result.structured_content.as_ref().unwrap();
        assert_eq!(content.pointer("/wouldMutate"), Some(&Value::Bool(false)));
        assert_eq!(content.pointer("/extension/id"), Some(&json!("dolos")));
        assert_eq!(
            content.pointer("/chart/chart"),
            Some(&json!("oci://oci.supernode.store/extensions/dolos"))
        );
        assert_eq!(
            content.pointer("/helmValues/dolos/network"),
            Some(&json!("cardano-preview"))
        );
        assert_eq!(
            content.pointer("/helmValues/config/upstreamAddress"),
            Some(&json!(
                "relay-preview-cardano-relay.cardano.svc.cluster.local:3000"
            ))
        );
        assert_eq!(
            content.pointer("/helmValues/persistence/storageClass"),
            Some(&json!("standard"))
        );
        assert_eq!(
            content.pointer("/helmValues/persistence/size"),
            Some(&json!("50Gi"))
        );
    }

    #[tokio::test]
    async fn hydra_install_dry_run_passes_chart_values_directly() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router.get_with_dynamic("workloads.install", &[]).unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "extensionId".to_string(),
            Value::String("hydra-node".to_string()),
        );
        arguments.insert(
            "releaseName".to_string(),
            Value::String("hydra-offline".to_string()),
        );
        arguments.insert("namespace".to_string(), Value::String("hydra".to_string()));
        arguments.insert("dryRun".to_string(), Value::Bool(true));
        arguments.insert(
            "configuration".to_string(),
            json!({
                "persistence": {
                    "storageClass": "standard",
                    "size": "5Gi"
                },
                "keys": {
                    "hydraSigning": {
                        "vaultStaticSecret": {
                            "path": "runtime/hydra/offline/hydra-signing"
                        }
                    },
                    "hydraVerification": {
                        "items": [
                            {
                                "filename": "hydra.vk",
                                "value": "public-hydra-verification-key"
                            }
                        ]
                    }
                }
            }),
        );

        let result = router.call(definition, Some(&arguments), &catalog).await;

        assert_eq!(result.is_error, Some(false));
        let content = result.structured_content.as_ref().unwrap();
        assert_eq!(content.pointer("/extension/id"), Some(&json!("hydra-node")));
        assert_eq!(
            content.pointer("/helmValues/displayName"),
            Some(&json!("hydra-offline"))
        );
        assert_eq!(
            content.pointer("/helmValues/persistence/storageClass"),
            Some(&json!("standard"))
        );
        assert_eq!(
            content.pointer("/helmValues/keys/hydraSigning/vaultStaticSecret/path"),
            Some(&json!("runtime/hydra/offline/hydra-signing"))
        );
        assert_eq!(content.pointer("/helmValues/namespace"), None);
    }

    #[tokio::test]
    async fn midnight_install_dry_run_reports_declared_dependency() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router.get_with_dynamic("workloads.install", &[]).unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "extensionId".to_string(),
            Value::String("midnight".to_string()),
        );
        arguments.insert(
            "releaseName".to_string(),
            Value::String("midnight-preview".to_string()),
        );
        arguments.insert(
            "namespace".to_string(),
            Value::String("midnight".to_string()),
        );
        arguments.insert("dryRun".to_string(), Value::Bool(true));
        arguments.insert(
            "configuration".to_string(),
            json!({
                "node": {
                    "network": "preview"
                },
                "persistence": {
                    "storageClass": "standard"
                },
                "dbSync": {
                    "vaultStaticSecret": {
                        "path": "runtime/midnight/preview/dbsync"
                    }
                },
                "nodeKey": {
                    "vaultStaticSecret": {
                        "path": "runtime/midnight/preview/node-key"
                    }
                }
            }),
        );

        let result = router.call(definition, Some(&arguments), &catalog).await;

        assert_eq!(result.is_error, Some(false));
        let content = result.structured_content.as_ref().unwrap();
        assert_eq!(content.pointer("/extension/id"), Some(&json!("midnight")));
        assert_eq!(
            content.pointer("/dependencies/0/extensionId"),
            Some(&json!("cardano-db-sync"))
        );
        assert_eq!(
            content.pointer("/helmValues/displayName"),
            Some(&json!("midnight-preview"))
        );
    }

    #[test]
    fn workloads_install_schema_does_not_expose_approval_id() {
        let router = ToolRouter::new();
        let definition = router.get_with_dynamic("workloads.install", &[]).unwrap();

        assert!(definition.input_schema.contains("configuration"));
        assert!(!definition.input_schema.contains("approvalId"));
    }

    #[test]
    fn workloads_install_tool_schema_advertises_configuration_argument() {
        let router = ToolRouter::new();
        let tool = router
            .list_with_dynamic(&[])
            .tools
            .into_iter()
            .find(|tool| tool.name == "workloads.install")
            .unwrap();
        let schema = Value::Object((*tool.input_schema).clone());

        assert_eq!(
            schema
                .get("required")
                .and_then(Value::as_array)
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>(),
            vec!["extensionId", "releaseName", "namespace", "configuration"]
        );
        assert_eq!(
            schema.pointer("/properties/configuration/type"),
            Some(&json!("object"))
        );
        assert_eq!(
            schema.pointer("/properties/configuration/additionalProperties"),
            Some(&json!(true))
        );
        assert!(
            schema
                .pointer("/properties/configuration/description")
                .and_then(Value::as_str)
                .is_some_and(|description| description.contains("Required"))
        );
    }

    #[tokio::test]
    async fn vault_runtime_tool_rejects_non_runtime_path_before_client_setup() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::testing();
        let definition = router.get_with_dynamic("vault.runtime.write", &[]).unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "path".to_string(),
            Value::String("operator/root".to_string()),
        );
        arguments.insert("values".to_string(), serde_json::json!({ "key": "secret" }));

        let result = router.call(definition, Some(&arguments), &catalog).await;

        assert_eq!(result.is_error, Some(true));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("error")),
            Some(&Value::String("vault_path_not_allowed".to_string()))
        );
        assert!(!serde_json::to_string(&result).unwrap().contains("secret"));
    }
}
