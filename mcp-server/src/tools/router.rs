use std::sync::Arc;

use rmcp::model::{CallToolResult, JsonObject, ListToolsResult, Meta, Tool, ToolAnnotations};
use serde_json::{Value, json};

use crate::{
    catalog::ExtensionCatalog,
    helm::{self, HelmChartRef, HelmInstallPlan},
    k8s::{HelmReleaseDiscovery, KubernetesClient, ResourceListParams},
    vault::{SecretObject, VaultClient, VaultError, VaultPath, WriteMode},
};

use super::{
    ToolDefinition, common::kube_error, common::pod_exec_error, common::success,
    common::tool_error, common::vault_error, hydra, k8s_summaries, supernode, vault, workloads,
    workloads::install, workloads::logs, workloads::metrics, workloads::outputs,
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
            "vault.runtime.metadata.get" => vault_runtime_metadata_get(arguments).await,
            "vault.runtime.write" => vault_runtime_write(arguments, WriteMode::Replace).await,
            "vault.runtime.patch" => vault_runtime_write(arguments, WriteMode::Patch).await,
            "hydra.keys.generate" => hydra::generate_keys(arguments).await,
            "workloads.list" => workloads_list(arguments, catalog).await,
            "workloads.get" => workloads_get(arguments, catalog).await,
            "workloads.logs.get" => logs::get(arguments).await,
            "workloads.metrics.get" => workloads_metrics_get(arguments, catalog).await,
            "workloads.install" => workloads_install(arguments, catalog).await,
            "workloads.upgrade" => workloads::upgrade::upgrade(arguments, catalog).await,
            "workloads.delete" => workloads::delete::delete(arguments, catalog).await,
            "dolos.snapshot.refresh" => workloads::dolos::snapshot_refresh(arguments).await,
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
        "resolvedConfiguration": resolution.configuration,
        "helmValues": helm_values,
        "availableStorageClasses": resolution.available_storage_classes,
        "recommendedStorageClasses": resolution.recommended_storage_classes,
        "helm": helm_result,
        "notes": [
            "Helm upgrade --install completed successfully",
            "raw Helm values are rejected by the extension configuration schema"
        ],
    }))
}

async fn workloads_list(
    arguments: Option<&JsonObject>,
    catalog: &ExtensionCatalog,
) -> CallToolResult {
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
        Ok(items) => k8s_summaries::deployment_summaries(items, include_control_plane),
        Err(error) => return kube_error("workloads.list", error),
    };
    let stateful_sets = match client
        .list_stateful_sets(namespace.as_deref(), &params)
        .await
    {
        Ok(items) => k8s_summaries::stateful_set_summaries(items, include_control_plane),
        Err(error) => return kube_error("workloads.list", error),
    };
    let pods = match client.list_pods(namespace.as_deref(), &params).await {
        Ok(items) => k8s_summaries::pod_summaries(items, include_control_plane),
        Err(error) => return kube_error("workloads.list", error),
    };
    let services = match client.list_services(namespace.as_deref(), &params).await {
        Ok(items) => items,
        Err(error) => return kube_error("workloads.list", error),
    };
    let workload_outputs = helm_releases
        .iter()
        .map(|release| {
            json!({
                "namespace": release.namespace,
                "name": release.name,
                "outputs": outputs::outputs_for_release(
                    &release.namespace,
                    &release.name,
                    Some(release),
                    &services.items,
                    catalog,
                ),
            })
        })
        .filter(|entry| {
            entry
                .pointer("/outputs")
                .and_then(Value::as_array)
                .is_some_and(|outputs| !outputs.is_empty())
        })
        .collect::<Vec<_>>();

    success(json!({
        "namespace": namespace,
        "source": "kubernetes-api+helm-secrets",
        "helmReleases": helm_releases,
        "deployments": deployments,
        "statefulSets": stateful_sets,
        "pods": pods,
        "services": k8s_summaries::service_summaries(services, include_control_plane),
        "workloadOutputs": workload_outputs,
    }))
}

async fn workloads_get(
    arguments: Option<&JsonObject>,
    catalog: &ExtensionCatalog,
) -> CallToolResult {
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
        Ok(value) => value.map(|deployment| k8s_summaries::deployment_summary(&deployment)),
        Err(error) => return kube_error("workloads.get", error),
    };
    let stateful_set = match get_optional(client.get_stateful_set(&namespace, &name).await) {
        Ok(value) => value.map(|stateful_set| k8s_summaries::stateful_set_summary(&stateful_set)),
        Err(error) => return kube_error("workloads.get", error),
    };
    let pod = match get_optional(client.get_pod(&namespace, &name).await) {
        Ok(value) => value.map(|pod| k8s_summaries::pod_summary(&pod)),
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
        Ok(items) => items,
        Err(error) => return kube_error("workloads.get", error),
    };
    let pod_items = match client
        .list_pods(
            Some(&namespace),
            &ResourceListParams {
                label_selector: Some(format!("app.kubernetes.io/instance={name}")),
                ..Default::default()
            },
        )
        .await
    {
        Ok(items) => items.items,
        Err(error) => return kube_error("workloads.get", error),
    };
    let pods = pod_items
        .iter()
        .map(k8s_summaries::pod_summary)
        .collect::<Vec<_>>();

    if deployment.is_none()
        && stateful_set.is_none()
        && pod.is_none()
        && helm_release.is_none()
        && pods.is_empty()
        && services.items.is_empty()
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
        "logTargets": k8s_summaries::pod_log_target_summaries(&pod_items),
        "relatedServices": k8s_summaries::service_summaries(services.clone(), true),
        "outputs": outputs::outputs_for_release(
            &namespace,
            &name,
            helm_release.as_ref(),
            &services.items,
            catalog,
        ),
    }))
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
    let target = match metrics::target_for_release(&helm_release, catalog) {
        Some(target) => target,
        None => {
            return tool_error(
                "unsupported_metrics_workload",
                "metrics are only supported for catalog-managed Cardano node relay, Dolos, and Hydra node workloads",
                json!({
                    "namespace": namespace,
                    "workload": workload,
                    "chart": helm_release.chart,
                }),
            );
        }
    };
    let pod_name = match find_workload_pod(&client, &namespace, &workload).await {
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
            target.container,
            &[metrics::SCRIPT_PATH],
        )
        .await
    {
        Ok(output) => output,
        Err(error) => return pod_exec_error("workloads.metrics.get", error),
    };
    let metrics = match serde_json::from_str::<Value>(output.stdout.trim()) {
        Ok(metrics) => metrics,
        Err(error) => {
            return tool_error(
                "invalid_metrics_payload",
                format!("metrics script did not return valid JSON: {error}"),
                json!({
                    "namespace": namespace,
                    "workload": workload,
                    "pod": pod_name,
                    "container": target.container,
                }),
            );
        }
    };

    success(json!({
        "namespace": namespace,
        "workload": workload,
        "pod": pod_name,
        "container": target.container,
        "source": format!("pod-exec:{}", metrics::SCRIPT_PATH),
        "extension": {
            "id": target.extension.id,
            "name": target.extension.name,
            "version": target.extension.default_version,
        },
        "helmRelease": helm_release,
        "metrics": metrics,
        "metricsSchema": target.extension.metrics,
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

async fn find_workload_pod(
    client: &KubernetesClient,
    namespace: &str,
    workload: &str,
) -> Result<Option<String>, kube::Error> {
    let mut pods = client
        .list_pods(
            Some(namespace),
            &ResourceListParams {
                label_selector: Some(format!("app.kubernetes.io/instance={workload}")),
                ..Default::default()
            },
        )
        .await?
        .items;

    if pods.is_empty()
        && let Some(pod) = get_optional(client.get_pod(namespace, workload).await)?
    {
        pods.push(pod);
    }

    pods.sort_by(|left, right| left.metadata.name.cmp(&right.metadata.name));

    let running_pod = pods
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
        .iter()
        .find(|pod| pod.metadata.deletion_timestamp.is_none())
        .and_then(|pod| pod.metadata.name.clone()))
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

fn optional_u32(arguments: Option<&JsonObject>, name: &str) -> Option<u32> {
    arguments?
        .get(name)?
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
}

impl Default for ToolRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::ExtensionCatalog;

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
        let catalog = ExtensionCatalog::embedded();
        let definition = router
            .get_with_dynamic("extensions.catalog.get", &[])
            .unwrap();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
        let definition = router.get_with_dynamic("workloads.install", &[]).unwrap();
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
        let definition = router.get_with_dynamic("workloads.install", &[]).unwrap();
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

    #[tokio::test]
    async fn dolos_install_dry_run_returns_validated_plan() {
        let router = ToolRouter::new();
        let catalog = ExtensionCatalog::embedded();
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
                "network": "cardano-preview",
                "namespace": "cardano",
                "storageClass": "standard",
                "upstreamAddress": "relay-preview-cardano-node.cardano.svc.cluster.local:3000",
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
                "relay-preview-cardano-node.cardano.svc.cluster.local:3000"
            ))
        );
        assert_eq!(
            content.pointer("/helmValues/image/tag"),
            Some(&json!("v1.1.1"))
        );
        assert_eq!(
            content.pointer("/helmValues/persistence/size"),
            Some(&json!("50Gi"))
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
        let catalog = ExtensionCatalog::embedded();
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
