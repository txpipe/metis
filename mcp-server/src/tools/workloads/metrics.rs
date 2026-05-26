use rmcp::model::{CallToolResult, JsonObject};
use serde_json::{Value, json};

use crate::catalog::ExtensionCatalog;
use crate::k8s::{HelmReleaseDiscovery, KubernetesClient};
use crate::tools::args::required_string;
use crate::tools::common::{kube_error, pod_exec_error, success, tool_error};

use super::{list::find_workload_pod, registry};

pub(crate) async fn get(
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
    let extension = match registry::extension_for_release(&helm_release, catalog) {
        Some(extension) => extension,
        None => {
            return tool_error(
                "unsupported_metrics_workload",
                "metrics are only supported for catalog-managed workloads",
                json!({
                    "namespace": namespace,
                    "workload": workload,
                    "chart": helm_release.chart,
                }),
            );
        }
    };
    let metrics_collection = match extension.metrics_collection.as_ref() {
        Some(metrics_collection) => metrics_collection,
        None => {
            return tool_error(
                "unsupported_metrics_workload",
                "this extension does not define metrics collection metadata",
                json!({
                    "namespace": namespace,
                    "workload": workload,
                    "extensionId": extension.id,
                    "chart": helm_release.chart,
                }),
            );
        }
    };
    let pod_name = match find_workload_pod(
        &client,
        &namespace,
        &workload,
        metrics_collection.pod_label_selector.as_deref(),
    )
    .await
    {
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
    let command = metrics_collection
        .command
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let output = match client
        .pod_exec_capture(
            &namespace,
            &pod_name,
            &metrics_collection.container,
            &command,
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
                    "container": metrics_collection.container,
                }),
            );
        }
    };

    success(json!({
        "namespace": namespace,
        "workload": workload,
        "pod": pod_name,
        "container": metrics_collection.container,
        "source": format!("pod-exec:{}", metrics_collection.command.join(" ")),
        "extension": {
            "id": extension.id,
            "name": extension.name,
            "version": extension.default_version,
        },
        "helmRelease": helm_release,
        "metrics": metrics,
        "stderr": if output.stderr.trim().is_empty() { Value::Null } else { Value::String(output.stderr) },
    }))
}
