use rmcp::model::{CallToolResult, JsonObject};
use serde_json::Value;
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

    let outputs = outputs::outputs_for_release(
        &namespace,
        &name,
        helm_release.as_ref(),
        &services.items,
        catalog,
    );

    success(build_workload_response(WorkloadResponse {
        namespace: &namespace,
        name: &name,
        helm_release: helm_release.map(|value| json!(value)),
        deployment,
        stateful_set,
        pod,
        pods,
        pod_items: &pod_items,
        related_services: k8s_summaries::service_summaries(services.clone(), true),
        outputs: outputs.into_iter().map(|value| json!(value)).collect(),
    }))
}

struct WorkloadResponse<'a> {
    namespace: &'a str,
    name: &'a str,
    helm_release: Option<Value>,
    deployment: Option<Value>,
    stateful_set: Option<Value>,
    pod: Option<Value>,
    pods: Vec<Value>,
    pod_items: &'a [k8s_openapi::api::core::v1::Pod],
    related_services: Vec<Value>,
    outputs: Vec<Value>,
}

fn build_workload_response(response: WorkloadResponse<'_>) -> Value {
    json!({
        "namespace": response.namespace,
        "name": response.name,
        "source": "kubernetes-api+helm-secrets",
        "helmRelease": response.helm_release,
        "deployment": response.deployment,
        "statefulSet": response.stateful_set,
        "pod": response.pod,
        "relatedPods": response.pods,
        "diagnostics": k8s_summaries::workload_diagnostics(response.pod_items),
        "logTargets": k8s_summaries::pod_log_target_summaries(response.pod_items),
        "relatedServices": response.related_services,
        "outputs": response.outputs,
    })
}

fn get_optional<T>(result: Result<T, kube::Error>) -> Result<Option<T>, kube::Error> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(kube::Error::Api(error)) if error.code == 404 => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{
        Container, ContainerState, ContainerStateRunning, ContainerStateTerminated,
        ContainerStatus, Pod, PodSpec, PodStatus,
    };
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use serde_json::Value;

    #[test]
    fn response_includes_additive_diagnostics_and_existing_fields() {
        let pod_items = vec![Pod {
            metadata: ObjectMeta {
                name: Some("demo-pod".into()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "app".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some("Running".into()),
                container_statuses: Some(vec![ContainerStatus {
                    name: "app".into(),
                    ready: true,
                    restart_count: 3,
                    state: Some(ContainerState {
                        running: Some(ContainerStateRunning::default()),
                        ..Default::default()
                    }),
                    last_state: Some(ContainerState {
                        terminated: Some(ContainerStateTerminated {
                            exit_code: 137,
                            reason: Some("OOMKilled".into()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        }];

        let response = build_workload_response(WorkloadResponse {
            namespace: "default",
            name: "demo",
            helm_release: None,
            deployment: None,
            stateful_set: None,
            pod: None,
            pods: pod_items.iter().map(k8s_summaries::pod_summary).collect::<Vec<_>>(),
            pod_items: &pod_items,
            related_services: Vec::<Value>::new(),
            outputs: Vec::<Value>::new(),
        });

        assert_eq!(response.pointer("/diagnostics/totalRestartCount"), Some(&json!(3)));
        assert_eq!(
            response.pointer("/diagnostics/containersWithRestarts/0/pod"),
            Some(&json!("demo-pod"))
        );
        assert_eq!(
            response.pointer("/diagnostics/containersWithRestarts/0/container"),
            Some(&json!("app"))
        );
        assert_eq!(
            response.pointer("/diagnostics/containersWithRestarts/0/lastTerminationReason"),
            Some(&json!("OOMKilled"))
        );
        assert_eq!(response.pointer("/diagnostics/oomKilled"), Some(&json!(true)));
        assert!(response.pointer("/logTargets/0/containers/0/name").is_some());
        assert!(response.pointer("/relatedPods/0/metadata/name").is_some());
    }
}
