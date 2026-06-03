use k8s_openapi::api::core::v1::Pod;
use rmcp::model::CallToolResult;
use rmcp::model::JsonObject;
use serde_json::json;

use crate::k8s::KubernetesClient;
use crate::k8s::PodLogParams;
use crate::k8s::ResourceListParams;
use crate::tools::args::{optional_bool, optional_i64, optional_string, required_string};
use crate::tools::common::kube_error;
use crate::tools::common::success;
use crate::tools::common::tool_error;
use crate::tools::k8s_summaries;

pub(crate) async fn get(arguments: Option<&JsonObject>) -> CallToolResult {
    let namespace = match required_string(arguments, "namespace") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let workload = match required_string(arguments, "workload") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let pod = optional_string(arguments, "pod");
    let container = optional_string(arguments, "container");
    let tail_lines = optional_i64(arguments, "tailLines");
    let since_seconds = optional_i64(arguments, "sinceSeconds");
    let previous = optional_bool(arguments, "previous").unwrap_or(false);
    let timestamps = optional_bool(arguments, "timestamps").unwrap_or(false);
    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error("workloads.logs.get", error),
    };

    let pods = match workload_log_pods(&client, &namespace, &workload).await {
        Ok(pods) if pods.is_empty() => {
            return tool_error(
                "not_found",
                format!("no pod found for workload: {namespace}/{workload}"),
                json!({ "namespace": namespace, "workload": workload }),
            );
        }
        Ok(pods) => pods,
        Err(error) => return kube_error("workloads.logs.get", error),
    };
    let target = match resolve_log_target(
        &namespace,
        &workload,
        &pods,
        pod.as_deref(),
        container.as_deref(),
    ) {
        Ok(target) => target,
        Err(error) => return error,
    };
    let params = PodLogParams {
        container: Some(target.container.clone()),
        previous,
        tail_lines,
        since_seconds,
        timestamps,
    };

    match client.pod_logs(&namespace, &target.pod, &params).await {
        Ok(logs) => success(json!({
            "namespace": namespace,
            "workload": workload,
            "pod": target.pod,
            "container": target.container,
            "tailLines": params.to_kube().tail_lines,
            "sinceSeconds": since_seconds,
            "previous": previous,
            "timestamps": timestamps,
            "diagnostics": selected_target_diagnostics(&pods, &target, previous),
            "logs": logs,
        })),
        Err(error) => kube_error("workloads.logs.get", error),
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ResolvedLogTarget {
    pod: String,
    container: String,
}

fn resolve_log_target(
    namespace: &str,
    workload: &str,
    pods: &[Pod],
    requested_pod: Option<&str>,
    requested_container: Option<&str>,
) -> Result<ResolvedLogTarget, CallToolResult> {
    let available_targets = || k8s_summaries::pod_log_target_summaries(pods);
    let pod = if let Some(requested_pod) = requested_pod {
        pods.iter()
            .find(|pod| pod.metadata.name.as_deref() == Some(requested_pod))
            .ok_or_else(|| {
                tool_error(
                    "invalid_log_target",
                    format!("pod is not part of workload: {namespace}/{workload}/{requested_pod}"),
                    json!({
                        "namespace": namespace,
                        "workload": workload,
                        "pod": requested_pod,
                        "availableTargets": available_targets(),
                    }),
                )
            })?
    } else {
        let active_pods = pods
            .iter()
            .filter(|pod| pod.metadata.deletion_timestamp.is_none())
            .collect::<Vec<_>>();
        match active_pods.as_slice() {
            [pod] => *pod,
            [] => {
                return Err(tool_error(
                    "not_found",
                    format!("no active pod found for workload: {namespace}/{workload}"),
                    json!({
                        "namespace": namespace,
                        "workload": workload,
                        "availableTargets": available_targets(),
                    }),
                ));
            }
            _ => {
                return Err(tool_error(
                    "ambiguous_log_target",
                    "workload has multiple active pods; specify the pod argument",
                    json!({
                        "namespace": namespace,
                        "workload": workload,
                        "availableTargets": available_targets(),
                    }),
                ));
            }
        }
    };
    let pod_name = pod.metadata.name.clone().unwrap_or_default();
    let containers = loggable_container_names(pod);
    let container = if let Some(requested_container) = requested_container {
        if !containers
            .iter()
            .any(|container| container == requested_container)
        {
            return Err(tool_error(
                "invalid_log_target",
                format!(
                    "container is not part of pod: {namespace}/{pod_name}/{requested_container}"
                ),
                json!({
                    "namespace": namespace,
                    "workload": workload,
                    "pod": pod_name,
                    "container": requested_container,
                    "availableTargets": available_targets(),
                }),
            ));
        }
        requested_container.to_string()
    } else {
        match containers.as_slice() {
            [container] => container.clone(),
            [] => {
                return Err(tool_error(
                    "not_found",
                    format!("no loggable container found for pod: {namespace}/{pod_name}"),
                    json!({
                        "namespace": namespace,
                        "workload": workload,
                        "pod": pod_name,
                        "availableTargets": available_targets(),
                    }),
                ));
            }
            _ => {
                return Err(tool_error(
                    "ambiguous_log_target",
                    "pod has multiple loggable containers; specify the container argument",
                    json!({
                        "namespace": namespace,
                        "workload": workload,
                        "pod": pod_name,
                        "availableTargets": available_targets(),
                    }),
                ));
            }
        }
    };

    Ok(ResolvedLogTarget {
        pod: pod_name,
        container,
    })
}

fn loggable_container_names(pod: &Pod) -> Vec<String> {
    let mut containers = pod
        .spec
        .as_ref()
        .map(|spec| {
            spec.containers
                .iter()
                .map(|container| container.name.clone())
                .chain(
                    spec.init_containers
                        .as_ref()
                        .into_iter()
                        .flatten()
                        .map(|container| container.name.clone()),
                )
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    containers.sort();
    containers
}

fn selected_target_diagnostics(
    pods: &[Pod],
    target: &ResolvedLogTarget,
    requested_previous: bool,
) -> serde_json::Value {
    pods.iter()
        .find(|pod| pod.metadata.name.as_deref() == Some(target.pod.as_str()))
        .map(|pod| {
            k8s_summaries::selected_container_diagnostics(pod, &target.container, requested_previous)
        })
        .unwrap_or_else(|| {
            json!({
                "pod": target.pod,
                "container": target.container,
                "requestedPrevious": requested_previous,
                "ready": null,
                "restartCount": null,
                "state": null,
                "stateDetails": null,
                "lastState": null,
                "lastStateDetails": null,
                "lastTerminationReason": null,
                "previousLogsAvailable": false,
            })
        })
}

async fn workload_log_pods(
    client: &KubernetesClient,
    namespace: &str,
    workload: &str,
) -> Result<Vec<Pod>, kube::Error> {
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
    Ok(pods)
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
    use k8s_openapi::api::core::v1::Container;
    use k8s_openapi::api::core::v1::ContainerState;
    use k8s_openapi::api::core::v1::ContainerStateRunning;
    use k8s_openapi::api::core::v1::ContainerStateTerminated;
    use k8s_openapi::api::core::v1::PodSpec;
    use k8s_openapi::api::core::v1::PodStatus;
    use k8s_openapi::api::core::v1::ContainerStatus;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use serde_json::Value;
    use serde_json::json;

    use super::*;

    #[test]
    fn log_target_auto_selects_single_pod_and_container() {
        let pods = vec![pod_with_containers("relay-0", &["cardano-node"], &[])];

        let target = resolve_log_target("cardano", "relay", &pods, None, None).unwrap();

        assert_eq!(
            target,
            ResolvedLogTarget {
                pod: "relay-0".to_string(),
                container: "cardano-node".to_string(),
            }
        );
    }

    #[test]
    fn log_target_requires_pod_when_workload_has_multiple_active_pods() {
        let pods = vec![
            pod_with_containers("relay-0", &["cardano-node"], &[]),
            pod_with_containers("relay-1", &["cardano-node"], &[]),
        ];

        let error = resolve_log_target("cardano", "relay", &pods, None, None).unwrap_err();

        assert_eq!(error.is_error, Some(true));
        assert_eq!(
            error
                .structured_content
                .as_ref()
                .and_then(|value| value.get("error")),
            Some(&Value::String("ambiguous_log_target".to_string()))
        );
        assert!(
            error
                .structured_content
                .as_ref()
                .and_then(|value| value.pointer("/details/availableTargets"))
                .is_some()
        );
    }

    #[test]
    fn log_target_requires_container_when_pod_has_multiple_containers() {
        let pods = vec![pod_with_containers(
            "relay-0",
            &["cardano-node", "metrics-sidecar"],
            &[],
        )];

        let error = resolve_log_target("cardano", "relay", &pods, None, None).unwrap_err();

        assert_eq!(
            error
                .structured_content
                .as_ref()
                .and_then(|value| value.get("error")),
            Some(&Value::String("ambiguous_log_target".to_string()))
        );
    }

    #[test]
    fn log_target_rejects_container_not_in_selected_pod() {
        let pods = vec![pod_with_containers("relay-0", &["cardano-node"], &[])];

        let error = resolve_log_target("cardano", "relay", &pods, Some("relay-0"), Some("missing"))
            .unwrap_err();

        assert_eq!(
            error
                .structured_content
                .as_ref()
                .and_then(|value| value.get("error")),
            Some(&Value::String("invalid_log_target".to_string()))
        );
    }

    #[test]
    fn selected_target_diagnostics_report_previous_container_death_for_normal_container() {
        let pods = vec![pod_with_statuses(
            "relay-0",
            &["cardano-node"],
            &[],
            vec![ContainerStatus {
                name: "cardano-node".to_string(),
                ready: true,
                restart_count: 2,
                state: Some(ContainerState {
                    running: Some(ContainerStateRunning::default()),
                    ..Default::default()
                }),
                last_state: Some(ContainerState {
                    terminated: Some(ContainerStateTerminated {
                        exit_code: 137,
                        reason: Some("OOMKilled".to_string()),
                        message: Some("container ran out of memory".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            vec![],
        )];

        let target = resolve_log_target("cardano", "relay", &pods, Some("relay-0"), Some("cardano-node"))
            .unwrap();

        assert_eq!(
            selected_target_diagnostics(&pods, &target, true),
            json!({
                "pod": "relay-0",
                "container": "cardano-node",
                "requestedPrevious": true,
                "ready": true,
                "restartCount": 2,
                "state": "running",
                "stateDetails": {
                    "type": "running"
                },
                "lastState": "terminated",
                "lastStateDetails": {
                    "type": "terminated",
                    "reason": "OOMKilled",
                    "message": "container ran out of memory",
                    "exitCode": 137
                },
                "lastTerminationReason": "OOMKilled",
                "previousLogsAvailable": true,
            })
        );
    }

    #[test]
    fn selected_target_diagnostics_also_work_for_init_container_if_selected() {
        let pods = vec![pod_with_statuses(
            "relay-0",
            &["cardano-node"],
            &["bootstrap"],
            vec![],
            vec![ContainerStatus {
                name: "bootstrap".to_string(),
                ready: false,
                restart_count: 1,
                state: Some(ContainerState {
                    terminated: Some(ContainerStateTerminated {
                        exit_code: 1,
                        reason: Some("Error".to_string()),
                        message: Some("init failed".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                last_state: Some(ContainerState {
                    terminated: Some(ContainerStateTerminated {
                        exit_code: 1,
                        reason: Some("Error".to_string()),
                        message: Some("init failed".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }],
        )];

        let target = resolve_log_target("cardano", "relay", &pods, Some("relay-0"), Some("bootstrap"))
            .unwrap();

        assert_eq!(
            selected_target_diagnostics(&pods, &target, false),
            json!({
                "pod": "relay-0",
                "container": "bootstrap",
                "requestedPrevious": false,
                "ready": false,
                "restartCount": 1,
                "state": "terminated",
                "stateDetails": {
                    "type": "terminated",
                    "reason": "Error",
                    "message": "init failed",
                    "exitCode": 1
                },
                "lastState": "terminated",
                "lastStateDetails": {
                    "type": "terminated",
                    "reason": "Error",
                    "message": "init failed",
                    "exitCode": 1
                },
                "lastTerminationReason": "Error",
                "previousLogsAvailable": true,
            })
        );
    }

    fn pod_with_containers(name: &str, containers: &[&str], init_containers: &[&str]) -> Pod {
        pod_with_statuses(name, containers, init_containers, vec![], vec![])
    }

    fn pod_with_statuses(
        name: &str,
        containers: &[&str],
        init_containers: &[&str],
        container_statuses: Vec<ContainerStatus>,
        init_container_statuses: Vec<ContainerStatus>,
    ) -> Pod {
        Pod {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: containers.iter().map(|name| container(name)).collect(),
                init_containers: (!init_containers.is_empty())
                    .then(|| init_containers.iter().map(|name| container(name)).collect()),
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                container_statuses: (!container_statuses.is_empty()).then_some(container_statuses),
                init_container_statuses: (!init_container_statuses.is_empty())
                    .then_some(init_container_statuses),
                ..Default::default()
            }),
        }
    }

    fn container(name: &str) -> Container {
        Container {
            name: name.to_string(),
            ..Default::default()
        }
    }
}
