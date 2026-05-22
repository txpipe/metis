use rmcp::model::{CallToolResult, JsonObject};
use serde_json::{Value, json};

use crate::catalog::{ExtensionCatalog, extension_summary};
use crate::k8s::{HelmReleaseDiscovery, KubernetesClient, ResourceListParams};
use crate::tools::args::{optional_bool, optional_string};
use crate::tools::common::{kube_error, success, tool_error};

use super::registry;

pub(crate) async fn list(
    arguments: Option<&JsonObject>,
    catalog: &ExtensionCatalog,
) -> CallToolResult {
    let namespace = optional_string(arguments, "namespace");
    let include_control_plane = optional_bool(arguments, "includeControlPlane").unwrap_or(false);
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

    let workloads = helm_releases
        .iter()
        .map(|release| workload_summary(release, catalog))
        .collect::<Vec<_>>();

    success(json!({
        "namespace": namespace,
        "source": "kubernetes-api+helm-secrets",
        "workloads": workloads,
    }))
}

pub(crate) fn workload_summary(
    release: &crate::k8s::HelmReleaseSummary,
    catalog: &ExtensionCatalog,
) -> Value {
    let catalog_extension =
        registry::extension_for_release(release, catalog).map(extension_summary);

    json!({
        "namespace": release.namespace,
        "name": release.name,
        "status": release.status,
        "revision": release.revision,
        "chart": release.chart,
        "appVersion": release.app_version,
        "updated": release.updated,
        "catalogExtension": catalog_extension,
    })
}

fn get_optional<T>(result: Result<T, kube::Error>) -> Result<Option<T>, kube::Error> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(kube::Error::Api(error)) if error.code == 404 => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) async fn find_workload_pod(
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
