use rmcp::model::{CallToolResult, JsonObject};
use serde_json::json;

use crate::catalog::ExtensionCatalog;
use crate::k8s::{HelmReleaseDiscovery, KubernetesClient, ResourceListParams};
use crate::tools::args::required_string;
use crate::tools::common::{kube_error, success, tool_error};
use crate::tools::{k8s_summaries, workloads::outputs};

pub(crate) async fn get(
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

fn get_optional<T>(result: Result<T, kube::Error>) -> Result<Option<T>, kube::Error> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(kube::Error::Api(error)) if error.code == 404 => Ok(None),
        Err(error) => Err(error),
    }
}
