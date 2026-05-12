use std::sync::Arc;

use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::api::core::v1::Event;
use k8s_openapi::api::core::v1::ObjectReference;
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::api::core::v1::Service;
use k8s_openapi::api::storage::v1::StorageClass;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::ObjectList;
use rmcp::model::CallToolResult;
use rmcp::model::JsonObject;
use rmcp::model::ListToolsResult;
use rmcp::model::Meta;
use rmcp::model::Tool;
use rmcp::model::ToolAnnotations;
use serde_json::Value;
use serde_json::json;

use crate::catalog::ExtensionCatalog;
use crate::catalog::ExtensionDefinition;
use crate::helm;
use crate::helm::HelmChartRef;
use crate::helm::HelmInstallPlan;
use crate::k8s::HelmReleaseDiscovery;
use crate::k8s::HelmReleaseSummary;
use crate::k8s::KubernetesClient;
use crate::k8s::PodExecError;
use crate::k8s::PodLogParams;
use crate::k8s::ResourceListParams;
use crate::vault::SecretObject;
use crate::vault::VaultClient;
use crate::vault::VaultError;
use crate::vault::VaultPath;
use crate::vault::WriteMode;

use super::ToolDefinition;
use super::supernode;
use super::vault;
use super::workloads;

const CARDANO_NODE_CHART_NAME: &str = "cardano-node";
const CARDANO_NODE_RELAY_EXTENSION_ID: &str = "cardano-node-relay";
const WORKLOAD_METRICS_CONTAINER: &str = "cardano-node";
const WORKLOAD_METRICS_SCRIPT: &str = "/opt/metis/bin/metrics.sh";

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
            .copied()
            .collect();

        Self {
            definitions: Arc::new(definitions),
        }
    }

    pub fn list(&self) -> ListToolsResult {
        ListToolsResult::with_all_items(
            self.definitions
                .iter()
                .map(|definition| tool_from_definition(*definition))
                .collect(),
        )
    }

    pub fn get(&self, name: &str) -> Option<ToolDefinition> {
        self.definitions
            .iter()
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
            "vault.runtime.metadata.get" => vault_runtime_metadata_get(arguments).await,
            "vault.runtime.write" => vault_runtime_write(arguments, WriteMode::Replace).await,
            "vault.runtime.patch" => vault_runtime_write(arguments, WriteMode::Patch).await,
            "workloads.list" => workloads_list(arguments).await,
            "workloads.get" => workloads_get(arguments).await,
            "workloads.logs.get" => workloads_logs_get(arguments).await,
            "workloads.metrics.get" => workloads_metrics_get(arguments, catalog).await,
            "workloads.install" => workloads_install(arguments, catalog).await,
            _ => self.not_implemented_result(definition),
        }
    }
}

async fn vault_runtime_metadata_get(arguments: Option<&JsonObject>) -> CallToolResult {
    let path = match runtime_vault_path(arguments) {
        Ok(path) => path,
        Err(error) => return error,
    };
    let client = match VaultClient::from_env() {
        Ok(client) => client,
        Err(error) => return vault_error("vault.runtime.metadata.get", error),
    };

    match client.runtime_metadata(&path).await {
        Ok(metadata) => success(json!({
            "path": metadata.path,
            "exists": metadata.exists,
            "keyNames": metadata.key_names,
            "keyNamesAvailable": metadata.key_names_available,
            "currentVersion": metadata.current_version,
        })),
        Err(error) => vault_error("vault.runtime.metadata.get", error),
    }
}

async fn vault_runtime_write(
    arguments: Option<&JsonObject>,
    default_mode: WriteMode,
) -> CallToolResult {
    let path = match runtime_vault_path(arguments) {
        Ok(path) => path,
        Err(error) => return error,
    };
    let secret = match secret_argument(arguments) {
        Ok(secret) => secret,
        Err(error) => return error,
    };
    let mode = match write_mode(arguments, default_mode) {
        Ok(mode) => mode,
        Err(error) => return vault_error("vault.runtime.write", error),
    };
    let client = match VaultClient::from_env() {
        Ok(client) => client,
        Err(error) => return vault_error("vault.runtime.write", error),
    };

    match client.write_runtime_secret(&path, &secret, mode).await {
        Ok(receipt) => success(json!({
            "path": receipt.path,
            "writtenKeys": receipt.written_keys,
            "version": receipt.version,
            "secretValuesReturned": false,
        })),
        Err(error) => vault_error("vault.runtime.write", error),
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
            "storageClasses": storage_classes.items.iter().map(storage_class_summary).collect::<Vec<_>>(),
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
            "events": events.items.iter().map(event_summary).collect::<Vec<_>>(),
        })),
        Err(error) => kube_error("cluster.events.list", error),
    }
}

fn catalog_list(catalog: &ExtensionCatalog) -> CallToolResult {
    success(json!({
        "extensions": catalog.list().collect::<Vec<_>>(),
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

async fn workloads_install(
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

    if configuration.get("namespace").and_then(Value::as_str) != Some(namespace.as_str()) {
        return tool_error(
            "invalid_arguments",
            "configuration.namespace must match namespace",
            json!({ "namespace": namespace, "configurationNamespace": configuration.get("namespace") }),
        );
    }

    let resolved_configuration = apply_extension_defaults(extension, Value::Object(configuration));
    let helm_values = planned_helm_values(extension, &release_name, &resolved_configuration);
    let chart = HelmChartRef {
        chart: extension.chart.clone(),
        version: extension.default_version.clone(),
    };

    if dry_run {
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
            "resolvedConfiguration": resolved_configuration,
            "helmValues": helm_values,
            "notes": [
                "dry-run planning only; no Kubernetes or Helm mutation was performed",
                "raw Helm values are rejected by the extension configuration schema"
            ],
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
                    "tool": "workloads.install",
                    "extensionId": extension.id,
                    "releaseName": release_name,
                    "namespace": namespace,
                    "status": status,
                    "stdout": stdout,
                    "stderr": stderr,
                }),
                _ => json!({
                    "tool": "workloads.install",
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
        "resolvedConfiguration": resolved_configuration,
        "helmValues": helm_values,
        "helm": helm_result,
        "notes": [
            "Helm upgrade --install completed successfully",
            "raw Helm values are rejected by the extension configuration schema"
        ],
    }))
}

async fn workloads_list(arguments: Option<&JsonObject>) -> CallToolResult {
    let namespace = optional_string(arguments, "namespace");
    let include_control_plane = optional_bool(arguments, "includeControlPlane").unwrap_or(false);
    let params = list_params(arguments, Some(200));
    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error("workloads.list", error),
    };
    let helm_releases = match HelmReleaseDiscovery::new(client.clone())
        .list_latest(namespace.as_deref(), include_control_plane)
        .await
    {
        Ok(releases) => releases,
        Err(error) => {
            return tool_error(
                "helm_release_discovery_error",
                error.to_string(),
                json!({ "tool": "workloads.list" }),
            );
        }
    };

    let deployments = match client.list_deployments(namespace.as_deref(), &params).await {
        Ok(items) => deployment_summaries(items, include_control_plane),
        Err(error) => return kube_error("workloads.list", error),
    };
    let stateful_sets = match client
        .list_stateful_sets(namespace.as_deref(), &params)
        .await
    {
        Ok(items) => stateful_set_summaries(items, include_control_plane),
        Err(error) => return kube_error("workloads.list", error),
    };
    let pods = match client.list_pods(namespace.as_deref(), &params).await {
        Ok(items) => pod_summaries(items, include_control_plane),
        Err(error) => return kube_error("workloads.list", error),
    };
    let services = match client.list_services(namespace.as_deref(), &params).await {
        Ok(items) => service_summaries(items, include_control_plane),
        Err(error) => return kube_error("workloads.list", error),
    };

    success(json!({
        "namespace": namespace,
        "source": "kubernetes-api+helm-secrets",
        "helmReleases": helm_releases,
        "deployments": deployments,
        "statefulSets": stateful_sets,
        "pods": pods,
        "services": services,
    }))
}

async fn workloads_get(arguments: Option<&JsonObject>) -> CallToolResult {
    let namespace = match required_string(arguments, "namespace") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let name = match required_string(arguments, "name") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error("workloads.get", error),
    };
    let helm_release = match HelmReleaseDiscovery::new(client.clone())
        .get_latest(&namespace, &name)
        .await
    {
        Ok(release) => release,
        Err(error) => {
            return tool_error(
                "helm_release_discovery_error",
                error.to_string(),
                json!({ "tool": "workloads.get", "namespace": namespace, "name": name }),
            );
        }
    };

    let deployment = match get_optional(client.get_deployment(&namespace, &name).await) {
        Ok(value) => value.map(|deployment| deployment_summary(&deployment)),
        Err(error) => return kube_error("workloads.get", error),
    };
    let stateful_set = match get_optional(client.get_stateful_set(&namespace, &name).await) {
        Ok(value) => value.map(|stateful_set| stateful_set_summary(&stateful_set)),
        Err(error) => return kube_error("workloads.get", error),
    };
    let pod = match get_optional(client.get_pod(&namespace, &name).await) {
        Ok(value) => value.map(|pod| pod_summary(&pod)),
        Err(error) => return kube_error("workloads.get", error),
    };
    let services = match client
        .list_services(
            Some(&namespace),
            &ResourceListParams {
                label_selector: Some(format!("app.kubernetes.io/instance={name}")),
                ..Default::default()
            },
        )
        .await
    {
        Ok(items) => service_summaries(items, true),
        Err(error) => return kube_error("workloads.get", error),
    };
    let pods = match client
        .list_pods(
            Some(&namespace),
            &ResourceListParams {
                label_selector: Some(format!("app.kubernetes.io/instance={name}")),
                ..Default::default()
            },
        )
        .await
    {
        Ok(items) => pod_summaries(items, true),
        Err(error) => return kube_error("workloads.get", error),
    };

    if deployment.is_none()
        && stateful_set.is_none()
        && pod.is_none()
        && helm_release.is_none()
        && pods.is_empty()
        && services.is_empty()
    {
        return tool_error(
            "not_found",
            format!("workload not found: {namespace}/{name}"),
            json!({ "namespace": namespace, "name": name }),
        );
    }

    success(json!({
        "namespace": namespace,
        "name": name,
        "source": "kubernetes-api+helm-secrets",
        "helmRelease": helm_release,
        "deployment": deployment,
        "statefulSet": stateful_set,
        "pod": pod,
        "relatedPods": pods,
        "relatedServices": services,
    }))
}

async fn workloads_logs_get(arguments: Option<&JsonObject>) -> CallToolResult {
    let namespace = match required_string(arguments, "namespace") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let workload = match required_string(arguments, "workload") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let container = optional_string(arguments, "container");
    let tail_lines = optional_i64(arguments, "tailLines");
    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error("workloads.logs.get", error),
    };

    let pod_name = match find_log_pod(&client, &namespace, &workload).await {
        Ok(Some(pod_name)) => pod_name,
        Ok(None) => {
            return tool_error(
                "not_found",
                format!("no pod found for workload: {namespace}/{workload}"),
                json!({ "namespace": namespace, "workload": workload }),
            );
        }
        Err(error) => return kube_error("workloads.logs.get", error),
    };
    let params = PodLogParams {
        container: container.clone(),
        tail_lines,
        ..Default::default()
    };

    match client.pod_logs(&namespace, &pod_name, &params).await {
        Ok(logs) => success(json!({
            "namespace": namespace,
            "workload": workload,
            "pod": pod_name,
            "container": container,
            "tailLines": params.to_kube().tail_lines,
            "logs": logs,
        })),
        Err(error) => kube_error("workloads.logs.get", error),
    }
}

async fn workloads_metrics_get(
    arguments: Option<&JsonObject>,
    catalog: &ExtensionCatalog,
) -> CallToolResult {
    let namespace = match required_string(arguments, "namespace") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let workload = match required_string(arguments, "workload") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error("workloads.metrics.get", error),
    };
    let helm_release = match HelmReleaseDiscovery::new(client.clone())
        .get_latest(&namespace, &workload)
        .await
    {
        Ok(Some(release)) => release,
        Ok(None) => {
            return tool_error(
                "not_found",
                format!("workload not found: {namespace}/{workload}"),
                json!({ "namespace": namespace, "workload": workload }),
            );
        }
        Err(error) => {
            return tool_error(
                "helm_release_discovery_error",
                error.to_string(),
                json!({ "tool": "workloads.metrics.get", "namespace": namespace, "workload": workload }),
            );
        }
    };
    let extension = match metrics_extension_for_release(&helm_release, catalog) {
        Some(extension) => extension,
        None => {
            return tool_error(
                "unsupported_metrics_workload",
                "metrics are only supported for catalog-managed Cardano node relay workloads",
                json!({
                    "namespace": namespace,
                    "workload": workload,
                    "chart": helm_release.chart,
                }),
            );
        }
    };
    let pod_name = match find_log_pod(&client, &namespace, &workload).await {
        Ok(Some(pod_name)) => pod_name,
        Ok(None) => {
            return tool_error(
                "not_found",
                format!("no pod found for workload: {namespace}/{workload}"),
                json!({ "namespace": namespace, "workload": workload }),
            );
        }
        Err(error) => return kube_error("workloads.metrics.get", error),
    };
    let output = match client
        .pod_exec_capture(
            &namespace,
            &pod_name,
            WORKLOAD_METRICS_CONTAINER,
            &[WORKLOAD_METRICS_SCRIPT],
        )
        .await
    {
        Ok(output) => output,
        Err(error) => return pod_exec_error("workloads.metrics.get", error),
    };
    let metrics = match parse_metrics_payload(&output.stdout) {
        Ok(metrics) => metrics,
        Err(error) => {
            return tool_error(
                "invalid_metrics_payload",
                format!("metrics script did not return valid JSON: {error}"),
                json!({
                    "namespace": namespace,
                    "workload": workload,
                    "pod": pod_name,
                    "container": WORKLOAD_METRICS_CONTAINER,
                }),
            );
        }
    };

    success(json!({
        "namespace": namespace,
        "workload": workload,
        "pod": pod_name,
        "container": WORKLOAD_METRICS_CONTAINER,
        "source": format!("pod-exec:{WORKLOAD_METRICS_SCRIPT}"),
        "extension": {
            "id": extension.id,
            "name": extension.name,
            "version": extension.default_version,
        },
        "helmRelease": helm_release,
        "metrics": metrics,
        "metricsSchema": extension.metrics,
        "stderr": if output.stderr.trim().is_empty() { Value::Null } else { Value::String(output.stderr) },
    }))
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

async fn find_log_pod(
    client: &KubernetesClient,
    namespace: &str,
    workload: &str,
) -> Result<Option<String>, kube::Error> {
    if let Some(pod) = get_optional(client.get_pod(namespace, workload).await)? {
        return Ok(pod.metadata.name);
    }

    let pods = client
        .list_pods(
            Some(namespace),
            &ResourceListParams {
                label_selector: Some(format!("app.kubernetes.io/instance={workload}")),
                ..Default::default()
            },
        )
        .await?;

    let running_pod = pods
        .items
        .iter()
        .find(|pod| {
            pod.status
                .as_ref()
                .and_then(|status| status.phase.as_deref())
                == Some("Running")
        })
        .and_then(|pod| pod.metadata.name.clone());

    if running_pod.is_some() {
        return Ok(running_pod);
    }

    Ok(pods
        .items
        .iter()
        .find(|pod| pod.metadata.deletion_timestamp.is_none())
        .and_then(|pod| pod.metadata.name.clone()))
}

fn success(value: Value) -> CallToolResult {
    CallToolResult::structured(value)
}

fn tool_error(code: &str, message: impl Into<String>, details: Value) -> CallToolResult {
    CallToolResult::structured_error(json!({
        "error": code,
        "message": message.into(),
        "details": details,
    }))
}

fn kube_error(tool: &str, error: kube::Error) -> CallToolResult {
    tool_error(
        "kubernetes_error",
        error.to_string(),
        json!({ "tool": tool }),
    )
}

fn pod_exec_error(tool: &str, error: PodExecError) -> CallToolResult {
    let code = match &error {
        PodExecError::Timeout { .. } => "pod_exec_timeout",
        PodExecError::OutputTooLarge { .. } => "pod_exec_output_too_large",
        PodExecError::CommandFailed { .. } => "pod_exec_command_failed",
        PodExecError::Kubernetes(_) => "kubernetes_error",
        PodExecError::MissingStream(_)
        | PodExecError::Read { .. }
        | PodExecError::RemoteCommand(_) => "pod_exec_error",
    };

    tool_error(code, error.to_string(), json!({ "tool": tool }))
}

fn metrics_extension_for_release<'a>(
    release: &HelmReleaseSummary,
    catalog: &'a ExtensionCatalog,
) -> Option<&'a ExtensionDefinition> {
    match release.chart.name.as_deref() {
        Some(CARDANO_NODE_CHART_NAME) => catalog.get(CARDANO_NODE_RELAY_EXTENSION_ID),
        _ => None,
    }
}

fn parse_metrics_payload(payload: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(payload.trim())
}

fn vault_error(tool: &str, error: VaultError) -> CallToolResult {
    let code = match &error {
        VaultError::Path(_) => "vault_path_not_allowed",
        VaultError::MissingConfig(_) | VaultError::InvalidConfig(_) => "vault_not_configured",
        VaultError::RootTokenRejected => "vault_root_token_rejected",
        VaultError::InvalidWriteMode => "invalid_arguments",
        VaultError::SecretValue(_) => "invalid_secret_values",
        VaultError::Status(_) | VaultError::Http(_) | VaultError::TokenFile(_) => "vault_error",
    };

    tool_error(code, error.to_string(), json!({ "tool": tool }))
}

fn get_optional<T>(result: Result<T, kube::Error>) -> Result<Option<T>, kube::Error> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(kube::Error::Api(error)) if error.code == 404 => Ok(None),
        Err(error) => Err(error),
    }
}

fn list_params(arguments: Option<&JsonObject>, default_limit: Option<u32>) -> ResourceListParams {
    ResourceListParams {
        label_selector: optional_string(arguments, "labelSelector"),
        field_selector: optional_string(arguments, "fieldSelector"),
        limit: optional_u32(arguments, "limit").or(default_limit),
    }
}

fn required_object(
    arguments: Option<&JsonObject>,
    name: &str,
) -> Result<JsonObject, CallToolResult> {
    match arguments.and_then(|arguments| arguments.get(name)) {
        Some(Value::Object(value)) => Ok(value.clone()),
        Some(value) => Err(tool_error(
            "invalid_arguments",
            format!("expected object argument: {name}"),
            json!({ "argument": name, "actualType": value_type_name(value) }),
        )),
        None => Err(tool_error(
            "invalid_arguments",
            format!("missing required object argument: {name}"),
            json!({ "argument": name }),
        )),
    }
}

fn required_string(arguments: Option<&JsonObject>, name: &str) -> Result<String, CallToolResult> {
    optional_string(arguments, name).ok_or_else(|| {
        tool_error(
            "invalid_arguments",
            format!("missing required string argument: {name}"),
            json!({ "argument": name }),
        )
    })
}

fn validate_configuration_schema(
    values: &JsonObject,
    schema: &Value,
) -> Result<(), CallToolResult> {
    let schema = schema.as_object().ok_or_else(|| {
        tool_error(
            "catalog_schema_error",
            "extension configuration schema must be an object schema",
            json!({}),
        )
    })?;
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            tool_error(
                "catalog_schema_error",
                "extension configuration schema must define properties",
                json!({}),
            )
        })?;

    if schema.get("additionalProperties").and_then(Value::as_bool) == Some(false) {
        for key in values.keys() {
            if !properties.contains_key(key) {
                return Err(tool_error(
                    "invalid_extension_configuration",
                    format!("unknown extension configuration value: {key}"),
                    json!({ "field": key }),
                ));
            }
        }
    }

    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for field in required.iter().filter_map(Value::as_str) {
            if !values.contains_key(field) {
                return Err(tool_error(
                    "invalid_extension_configuration",
                    format!("missing required extension configuration value: {field}"),
                    json!({ "field": field }),
                ));
            }
        }
    }

    for (key, value) in values {
        if let Some(property_schema) = properties.get(key) {
            validate_property_value(key, value, property_schema)?;
        }
    }

    Ok(())
}

fn validate_property_value(
    name: &str,
    value: &Value,
    schema: &Value,
) -> Result<(), CallToolResult> {
    if let Some(expected_type) = schema.get("type").and_then(Value::as_str) {
        let matches = match expected_type {
            "boolean" => value.is_boolean(),
            "integer" => value.as_i64().is_some(),
            "number" => value.as_f64().is_some(),
            "object" => value.is_object(),
            "string" => value.is_string(),
            _ => true,
        };

        if !matches {
            return Err(tool_error(
                "invalid_extension_configuration",
                format!("invalid type for extension configuration value: {name}"),
                json!({
                    "field": name,
                    "expectedType": expected_type,
                    "actualType": value_type_name(value),
                }),
            ));
        }
    }

    if let Some(allowed_values) = schema.get("enum").and_then(Value::as_array)
        && !allowed_values.iter().any(|allowed| allowed == value)
    {
        return Err(tool_error(
            "invalid_extension_configuration",
            format!("unsupported value for extension configuration field: {name}"),
            json!({
                "field": name,
                "allowedValues": allowed_values,
                "actualValue": value,
            }),
        ));
    }

    Ok(())
}

fn merge_defaults(defaults: &Value, values: Value) -> Value {
    match (defaults, values) {
        (Value::Object(defaults), Value::Object(mut values)) => {
            for (key, default_value) in defaults {
                let value = values
                    .remove(key)
                    .map(|value| merge_defaults(default_value, value))
                    .unwrap_or_else(|| default_value.clone());
                values.insert(key.clone(), value);
            }
            Value::Object(values)
        }
        (_, values) => values,
    }
}

fn apply_extension_defaults(extension: &ExtensionDefinition, inputs: Value) -> Value {
    let defaults = match extension.id.as_str() {
        "cardano-node-relay" => cardano_node_relay_defaults(input_string(&inputs, "network")),
        _ => json!({}),
    };

    merge_defaults(&defaults, inputs)
}

fn cardano_node_relay_defaults(network: Option<&str>) -> Value {
    let (pvc_size, cpu_request, memory_request, cpu_limit, memory_limit) = match network {
        Some("mainnet") => ("250Gi", "2", "8Gi", "4", "16Gi"),
        Some("preprod") => ("120Gi", "1", "4Gi", "2", "8Gi"),
        _ => ("80Gi", "500m", "2Gi", "1", "4Gi"),
    };

    json!({
        "topology": { "mode": "image-default" },
        "exposeLoadBalancer": false,
        "imageTag": "11.0.1",
        "resources": {
            "requests": {
                "cpu": cpu_request,
                "memory": memory_request,
            },
            "limits": {
                "cpu": cpu_limit,
                "memory": memory_limit,
            },
        },
        "pvcSize": pvc_size,
    })
}

fn planned_helm_values(
    extension: &ExtensionDefinition,
    release_name: &str,
    inputs: &Value,
) -> Value {
    let mut helm_values = JsonObject::new();
    helm_values.insert(
        "displayName".to_string(),
        Value::String(release_name.to_string()),
    );

    if let Some(storage_class) = input_string(inputs, "storageClass") {
        insert_path(
            &mut helm_values,
            &["persistence", "storageClass"],
            Value::String(storage_class.to_string()),
        );
    }

    if extension.id == "cardano-node-relay" {
        if let Some(network) = input_string(inputs, "network") {
            insert_path(
                &mut helm_values,
                &["node", "network"],
                Value::String(network.to_string()),
            );
        }
        if let Some(pvc_size) = input_string(inputs, "pvcSize") {
            insert_path(
                &mut helm_values,
                &["persistence", "size"],
                Value::String(pvc_size.to_string()),
            );
        }
        if let Some(image_tag) = input_string(inputs, "imageTag") {
            insert_path(
                &mut helm_values,
                &["image", "tag"],
                Value::String(image_tag.to_string()),
            );
        }
        if inputs
            .get("exposeLoadBalancer")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            insert_path(
                &mut helm_values,
                &["service", "type"],
                Value::String("LoadBalancer".to_string()),
            );
        }
        if let Some(resources) = inputs.get("resources") {
            helm_values.insert("resources".to_string(), resources.clone());
        }
        if let Some(topology) = inputs.get("topology") {
            insert_path(&mut helm_values, &["node", "topology"], topology.clone());
        }
        insert_path(
            &mut helm_values,
            &["node", "blockProducer", "enabled"],
            Value::Bool(false),
        );
    }

    Value::Object(helm_values)
}

fn insert_path(root: &mut JsonObject, path: &[&str], value: Value) {
    let Some((last, parents)) = path.split_last() else {
        return;
    };
    let mut current = root;

    for parent in parents {
        current = current
            .entry((*parent).to_string())
            .or_insert_with(|| Value::Object(JsonObject::new()))
            .as_object_mut()
            .expect("planned Helm value parent must be an object");
    }

    current.insert((*last).to_string(), value);
}

fn input_string<'a>(inputs: &'a Value, name: &str) -> Option<&'a str> {
    inputs.get(name).and_then(Value::as_str)
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn runtime_vault_path(arguments: Option<&JsonObject>) -> Result<VaultPath, CallToolResult> {
    let path = required_string(arguments, "path")?;
    VaultPath::runtime(&path).map_err(|error| {
        tool_error(
            "vault_path_not_allowed",
            error.to_string(),
            json!({ "path": path }),
        )
    })
}

fn secret_argument(arguments: Option<&JsonObject>) -> Result<SecretObject, CallToolResult> {
    let value = arguments
        .and_then(|arguments| arguments.get("values").or_else(|| arguments.get("data")))
        .cloned()
        .ok_or_else(|| {
            tool_error(
                "invalid_arguments",
                "missing required secret object argument: values",
                json!({ "argument": "values" }),
            )
        })?;

    SecretObject::new(value).map_err(|error| {
        tool_error(
            "invalid_secret_values",
            error.to_string(),
            json!({ "argument": "values" }),
        )
    })
}

fn write_mode(
    arguments: Option<&JsonObject>,
    default_mode: WriteMode,
) -> Result<WriteMode, VaultError> {
    match optional_string(arguments, "mode") {
        Some(mode) => WriteMode::parse(Some(&mode)),
        None => Ok(default_mode),
    }
}

fn optional_string(arguments: Option<&JsonObject>, name: &str) -> Option<String> {
    arguments?
        .get(name)?
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

fn optional_bool(arguments: Option<&JsonObject>, name: &str) -> Option<bool> {
    arguments?.get(name)?.as_bool()
}

fn optional_i64(arguments: Option<&JsonObject>, name: &str) -> Option<i64> {
    arguments?.get(name)?.as_i64()
}

fn optional_u32(arguments: Option<&JsonObject>, name: &str) -> Option<u32> {
    arguments?
        .get(name)?
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
}

fn deployment_summaries(
    deployments: ObjectList<Deployment>,
    include_control_plane: bool,
) -> Vec<Value> {
    deployments
        .items
        .iter()
        .filter(|deployment| include_control_plane || !is_control_plane(&deployment.metadata))
        .map(deployment_summary)
        .collect()
}

fn deployment_summary(deployment: &Deployment) -> Value {
    json!({
        "kind": "Deployment",
        "metadata": metadata_summary(&deployment.metadata),
        "replicas": deployment.spec.as_ref().and_then(|spec| spec.replicas),
        "readyReplicas": deployment.status.as_ref().and_then(|status| status.ready_replicas),
        "availableReplicas": deployment.status.as_ref().and_then(|status| status.available_replicas),
    })
}

fn stateful_set_summaries(
    stateful_sets: ObjectList<StatefulSet>,
    include_control_plane: bool,
) -> Vec<Value> {
    stateful_sets
        .items
        .iter()
        .filter(|stateful_set| include_control_plane || !is_control_plane(&stateful_set.metadata))
        .map(stateful_set_summary)
        .collect()
}

fn stateful_set_summary(stateful_set: &StatefulSet) -> Value {
    json!({
        "kind": "StatefulSet",
        "metadata": metadata_summary(&stateful_set.metadata),
        "replicas": stateful_set.spec.as_ref().and_then(|spec| spec.replicas),
        "readyReplicas": stateful_set.status.as_ref().and_then(|status| status.ready_replicas),
        "currentReplicas": stateful_set.status.as_ref().and_then(|status| status.current_replicas),
    })
}

fn pod_summaries(pods: ObjectList<Pod>, include_control_plane: bool) -> Vec<Value> {
    pods.items
        .iter()
        .filter(|pod| include_control_plane || !is_control_plane(&pod.metadata))
        .map(pod_summary)
        .collect()
}

fn pod_summary(pod: &Pod) -> Value {
    json!({
        "kind": "Pod",
        "metadata": metadata_summary(&pod.metadata),
        "phase": pod.status.as_ref().and_then(|status| status.phase.clone()),
        "podIp": pod.status.as_ref().and_then(|status| status.pod_ip.clone()),
        "nodeName": pod.spec.as_ref().and_then(|spec| spec.node_name.clone()),
        "containers": pod.spec.as_ref().map(|spec| {
            spec.containers
                .iter()
                .map(|container| container.name.clone())
                .collect::<Vec<_>>()
        }).unwrap_or_default(),
    })
}

fn service_summaries(services: ObjectList<Service>, include_control_plane: bool) -> Vec<Value> {
    services
        .items
        .iter()
        .filter(|service| include_control_plane || !is_control_plane(&service.metadata))
        .map(service_summary)
        .collect()
}

fn service_summary(service: &Service) -> Value {
    json!({
        "kind": "Service",
        "metadata": metadata_summary(&service.metadata),
        "type": service.spec.as_ref().and_then(|spec| spec.type_.clone()),
        "clusterIp": service.spec.as_ref().and_then(|spec| spec.cluster_ip.clone()),
        "ports": service.spec.as_ref().map(|spec| {
            spec.ports.clone().unwrap_or_default().into_iter().map(|port| json!({
                "name": port.name,
                "port": port.port,
                "protocol": port.protocol,
            })).collect::<Vec<_>>()
        }).unwrap_or_default(),
    })
}

fn storage_class_summary(storage_class: &StorageClass) -> Value {
    json!({
        "metadata": metadata_summary(&storage_class.metadata),
        "provisioner": storage_class.provisioner,
        "reclaimPolicy": storage_class.reclaim_policy,
        "volumeBindingMode": storage_class.volume_binding_mode,
        "allowVolumeExpansion": storage_class.allow_volume_expansion,
        "isDefault": storage_class.metadata.annotations.as_ref().is_some_and(|annotations| {
            annotations.get("storageclass.kubernetes.io/is-default-class").is_some_and(|value| value == "true")
                || annotations.get("storageclass.beta.kubernetes.io/is-default-class").is_some_and(|value| value == "true")
        }),
    })
}

fn event_summary(event: &Event) -> Value {
    json!({
        "metadata": metadata_summary(&event.metadata),
        "type": event.type_,
        "reason": event.reason,
        "message": event.message,
        "count": event.count,
        "involvedObject": object_reference_summary(&event.involved_object),
        "firstTimestamp": event.first_timestamp,
        "lastTimestamp": event.last_timestamp,
    })
}

fn metadata_summary(metadata: &ObjectMeta) -> Value {
    json!({
        "name": metadata.name,
        "namespace": metadata.namespace,
        "labels": metadata.labels,
        "creationTimestamp": metadata.creation_timestamp,
    })
}

fn object_reference_summary(reference: &ObjectReference) -> Value {
    json!({
        "kind": reference.kind,
        "namespace": reference.namespace,
        "name": reference.name,
        "uid": reference.uid,
    })
}

fn is_control_plane(metadata: &ObjectMeta) -> bool {
    metadata.name.as_deref() == Some("control-plane")
        || metadata.namespace.as_deref() == Some("control-plane")
        || metadata.labels.as_ref().is_some_and(|labels| {
            labels
                .get("app.kubernetes.io/name")
                .is_some_and(|value| value == "control-plane")
                || labels
                    .get("app.kubernetes.io/instance")
                    .is_some_and(|value| value == "control-plane")
        })
}

impl Default for ToolRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::ExtensionCatalog;
    use crate::k8s::HelmChartSummary;
    use crate::k8s::HelmReleaseSummary;

    use super::*;

    #[test]
    fn lists_mvp_tools_with_policy_metadata() {
        let router = ToolRouter::new();

        let tools = router.list().tools;

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

        let tools = router.list().tools;

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

        let definition = router.get("workloads.delete").unwrap();

        assert!(definition.destructive);
    }

    #[tokio::test]
    async fn executes_catalog_get_tool() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::embedded();
        let definition = router.get("extensions.catalog.get").unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "extensionId".to_string(),
            Value::String("cardano-node-relay".to_string()),
        );

        let result = router.call(definition, Some(&arguments), &catalog).await;

        assert_eq!(result.is_error, Some(false));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.pointer("/extension/id")),
            Some(&Value::String("cardano-node-relay".to_string()))
        );
    }

    #[tokio::test]
    async fn missing_required_argument_returns_tool_error() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::embedded();
        let definition = router.get("extensions.catalog.get").unwrap();

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
        let catalog = ExtensionCatalog::embedded();
        let definition = router.get("workloads.metrics.get").unwrap();

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

    #[test]
    fn metrics_extension_resolves_cardano_node_chart() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release_with_chart(Some("cardano-node"));

        let extension = metrics_extension_for_release(&release, &catalog).unwrap();

        assert_eq!(extension.id, "cardano-node-relay");
    }

    #[test]
    fn metrics_extension_rejects_unsupported_chart() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release_with_chart(Some("dolos"));

        assert!(metrics_extension_for_release(&release, &catalog).is_none());
    }

    #[test]
    fn metrics_payload_parser_accepts_script_json() {
        let metrics = parse_metrics_payload(
            r#"
            {"type":"cardano-node","role":"relay","errors":[]}
            "#,
        )
        .unwrap();

        assert_eq!(metrics.pointer("/type"), Some(&json!("cardano-node")));
        assert_eq!(metrics.pointer("/role"), Some(&json!("relay")));
    }

    #[test]
    fn metrics_payload_parser_rejects_invalid_json() {
        assert!(parse_metrics_payload("not json").is_err());
    }

    #[tokio::test]
    async fn non_planning_mutation_tool_execution_remains_explicitly_unimplemented() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::embedded();
        let definition = router.get("workloads.upgrade").unwrap();

        let result = router.call(definition, None, &catalog).await;

        assert_eq!(result.is_error, Some(true));
        assert_eq!(
            result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("error")),
            Some(&Value::String("not_implemented".to_string()))
        );
    }

    #[tokio::test]
    async fn workloads_install_dry_run_returns_validated_plan() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::embedded();
        let definition = router.get("workloads.install").unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "extensionId".to_string(),
            Value::String("cardano-node-relay".to_string()),
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
                "network": "preview",
                "namespace": "cardano",
                "storageClass": "standard",
            }),
        );

        let result = router.call(definition, Some(&arguments), &catalog).await;

        assert_eq!(result.is_error, Some(false));
        let content = result.structured_content.as_ref().unwrap();
        assert_eq!(content.pointer("/wouldMutate"), Some(&Value::Bool(false)));
        assert_eq!(
            content.pointer("/extension/id"),
            Some(&Value::String("cardano-node-relay".to_string()))
        );
        assert_eq!(
            content.pointer("/chart/chart"),
            Some(&Value::String(
                "oci://oci.supernode.store/extensions/cardano-node".to_string()
            ))
        );
        assert_eq!(
            content.pointer("/helmValues/node/network"),
            Some(&Value::String("preview".to_string()))
        );
        assert_eq!(
            content.pointer("/helmValues/node/blockProducer/enabled"),
            Some(&Value::Bool(false))
        );
        assert_eq!(
            content.pointer("/helmValues/persistence/size"),
            Some(&Value::String("80Gi".to_string()))
        );
    }

    #[tokio::test]
    async fn workloads_install_rejects_unknown_raw_helm_values() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::embedded();
        let definition = router.get("workloads.install").unwrap();
        let mut arguments = JsonObject::new();
        arguments.insert(
            "extensionId".to_string(),
            Value::String("cardano-node-relay".to_string()),
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
                "network": "preview",
                "namespace": "cardano",
                "storageClass": "standard",
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

    #[test]
    fn workloads_install_schema_does_not_expose_approval_id() {
        let router = ToolRouter::new();
        let definition = router.get("workloads.install").unwrap();

        assert!(definition.input_schema.contains("configuration"));
        assert!(!definition.input_schema.contains("approvalId"));
    }

    #[tokio::test]
    async fn vault_runtime_tool_rejects_non_runtime_path_before_client_setup() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::embedded();
        let definition = router.get("vault.runtime.write").unwrap();
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

    fn helm_release_with_chart(chart_name: Option<&str>) -> HelmReleaseSummary {
        HelmReleaseSummary {
            name: "relay-preview".to_string(),
            namespace: "cardano".to_string(),
            revision: 1,
            status: Some("deployed".to_string()),
            chart: HelmChartSummary {
                name: chart_name.map(str::to_string),
                version: Some("0.1.0".to_string()),
            },
            app_version: Some("11.0.1".to_string()),
            description: None,
            updated: None,
            secret_name: Some("sh.helm.release.v1.relay-preview.v1".to_string()),
        }
    }
}
