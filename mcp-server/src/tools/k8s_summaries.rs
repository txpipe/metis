use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::api::core::v1::Container;
use k8s_openapi::api::core::v1::ContainerState;
use k8s_openapi::api::core::v1::ContainerStatus;
use k8s_openapi::api::core::v1::Event;
use k8s_openapi::api::core::v1::ObjectReference;
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::api::core::v1::Service;
use k8s_openapi::api::storage::v1::StorageClass;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::ObjectList;
use serde_json::Value;
use serde_json::json;

pub(crate) fn deployment_summary(deployment: &Deployment) -> Value {
    json!({
        "kind": "Deployment",
        "metadata": metadata_summary(&deployment.metadata),
        "replicas": deployment.spec.as_ref().and_then(|spec| spec.replicas),
        "readyReplicas": deployment.status.as_ref().and_then(|status| status.ready_replicas),
        "availableReplicas": deployment.status.as_ref().and_then(|status| status.available_replicas),
    })
}

pub(crate) fn stateful_set_summary(stateful_set: &StatefulSet) -> Value {
    json!({
        "kind": "StatefulSet",
        "metadata": metadata_summary(&stateful_set.metadata),
        "replicas": stateful_set.spec.as_ref().and_then(|spec| spec.replicas),
        "readyReplicas": stateful_set.status.as_ref().and_then(|status| status.ready_replicas),
        "currentReplicas": stateful_set.status.as_ref().and_then(|status| status.current_replicas),
    })
}

pub(crate) fn pod_summary(pod: &Pod) -> Value {
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

pub(crate) fn pod_log_target_summaries(pods: &[Pod]) -> Vec<Value> {
    pods.iter().map(pod_log_target_summary).collect()
}

pub(crate) fn workload_diagnostics(pods: &[Pod]) -> Value {
    let active_pod_count = pods
        .iter()
        .filter(|pod| pod.metadata.deletion_timestamp.is_none())
        .count();
    let terminating_pod_count = pods
        .iter()
        .filter(|pod| pod.metadata.deletion_timestamp.is_some())
        .count();

    let containers_with_restarts = pods
        .iter()
        .flat_map(restarted_container_diagnostics)
        .collect::<Vec<_>>();
    let total_restart_count = containers_with_restarts
        .iter()
        .filter_map(|container| container["restartCount"].as_i64())
        .sum::<i64>();
    let oom_killed = containers_with_restarts.iter().any(|container| {
        container["lastTerminationReason"].as_str() == Some("OOMKilled")
    });

    json!({
        "workloadState": derive_workload_state(pods, &containers_with_restarts),
        "activePodCount": active_pod_count,
        "terminatingPodCount": terminating_pod_count,
        "totalRestartCount": total_restart_count,
        "containersWithRestarts": containers_with_restarts,
        "oomKilled": oom_killed,
    })
}

pub(crate) fn selected_container_diagnostics(
    pod: &Pod,
    container_name: &str,
    requested_previous: bool,
) -> Value {
    let status = pod.status.as_ref().and_then(|status| {
        status
            .container_statuses
            .as_deref()
            .into_iter()
            .chain(status.init_container_statuses.as_deref())
            .flatten()
            .find(|status| status.name == container_name)
    });
    let last_termination_reason = status
        .and_then(|status| status.last_state.as_ref())
        .and_then(|state| state.terminated.as_ref())
        .and_then(|terminated| terminated.reason.clone());

    json!({
        "pod": pod.metadata.name,
        "container": container_name,
        "requestedPrevious": requested_previous,
        "ready": status.map(|status| status.ready),
        "restartCount": status.map(|status| status.restart_count),
        "state": status.and_then(|status| container_state(status.state.as_ref())),
        "stateDetails": status.and_then(|status| container_state_details(status.state.as_ref())),
        "lastState": status.and_then(|status| container_state(status.last_state.as_ref())),
        "lastStateDetails": status.and_then(|status| container_state_details(status.last_state.as_ref())),
        "lastTerminationReason": last_termination_reason,
        "previousLogsAvailable": status.is_some_and(|status| status.restart_count > 0),
    })
}

fn pod_log_target_summary(pod: &Pod) -> Value {
    let container_statuses = pod
        .status
        .as_ref()
        .and_then(|status| status.container_statuses.as_deref());
    let init_container_statuses = pod
        .status
        .as_ref()
        .and_then(|status| status.init_container_statuses.as_deref());

    json!({
        "pod": pod.metadata.name,
        "phase": pod.status.as_ref().and_then(|status| status.phase.clone()),
        "deleting": pod.metadata.deletion_timestamp.is_some(),
        "containers": pod.spec.as_ref().map(|spec| {
            container_log_targets(&spec.containers, container_statuses)
        }).unwrap_or_default(),
        "initContainers": pod.spec.as_ref().and_then(|spec| spec.init_containers.as_ref()).map(|containers| {
            container_log_targets(containers, init_container_statuses)
        }).unwrap_or_default(),
    })
}

fn container_log_targets(
    containers: &[Container],
    statuses: Option<&[ContainerStatus]>,
) -> Vec<Value> {
    containers
        .iter()
        .map(|container| {
            let status = statuses
                .and_then(|statuses| statuses.iter().find(|status| status.name == container.name));
            json!({
                "name": container.name,
                "ready": status.map(|status| status.ready),
                "restartCount": status.map(|status| status.restart_count),
                "state": status.and_then(|status| container_state(status.state.as_ref())),
                "stateDetails": status.and_then(|status| container_state_details(status.state.as_ref())),
                "lastState": status.and_then(|status| container_state(status.last_state.as_ref())),
                "lastStateDetails": status.and_then(|status| container_state_details(status.last_state.as_ref())),
            })
        })
        .collect()
}

fn container_state(state: Option<&ContainerState>) -> Option<&'static str> {
    let state = state?;
    if state.running.is_some() {
        Some("running")
    } else if state.waiting.is_some() {
        Some("waiting")
    } else if state.terminated.is_some() {
        Some("terminated")
    } else {
        None
    }
}

fn container_state_details(state: Option<&ContainerState>) -> Option<Value> {
    let state = state?;
    if state.running.is_some() {
        Some(json!({
            "type": "running",
        }))
    } else if let Some(waiting) = state.waiting.as_ref() {
        let mut details = serde_json::Map::from_iter([(
            "type".to_string(),
            Value::String("waiting".into()),
        )]);
        if let Some(reason) = waiting.reason.clone() {
            details.insert("reason".into(), Value::String(reason));
        }
        if let Some(message) = waiting.message.clone() {
            details.insert("message".into(), Value::String(message));
        }
        Some(Value::Object(details))
    } else {
        state.terminated.as_ref().map(|terminated| {
            let mut details = serde_json::Map::from_iter([(
                "type".to_string(),
                Value::String("terminated".into()),
            )]);
            if let Some(reason) = terminated.reason.clone() {
                details.insert("reason".into(), Value::String(reason));
            }
            if let Some(message) = terminated.message.clone() {
                details.insert("message".into(), Value::String(message));
            }
            details.insert("exitCode".into(), json!(terminated.exit_code));
            Value::Object(details)
        })
    }
}

fn restarted_container_diagnostics<'a>(pod: &'a Pod) -> impl Iterator<Item = Value> + 'a {
    let pod_name = pod.metadata.name.clone();
    pod.status
        .iter()
        .flat_map(|status| {
            status
                .container_statuses
                .iter()
                .chain(status.init_container_statuses.iter())
        })
        .flat_map(move |statuses| {
            let pod_name = pod_name.clone();
            statuses.iter().filter_map(move |status| {
                if status.restart_count > 0 {
                    let last_termination_reason = status
                        .last_state
                        .as_ref()
                        .and_then(|state| state.terminated.as_ref())
                        .and_then(|terminated| terminated.reason.clone());
                    Some(json!({
                        "pod": pod_name,
                        "container": status.name,
                        "ready": status.ready,
                        "restartCount": status.restart_count,
                        "state": container_state(status.state.as_ref()),
                        "stateDetails": container_state_details(status.state.as_ref()),
                        "lastState": container_state(status.last_state.as_ref()),
                        "lastStateDetails": container_state_details(status.last_state.as_ref()),
                        "lastTerminationReason": last_termination_reason,
                        "previousLogsAvailable": status.restart_count > 0,
                    }))
                } else {
                    None
                }
            })
        })
}

fn derive_workload_state(pods: &[Pod], containers_with_restarts: &[Value]) -> &'static str {
    if pods.iter().any(|pod| {
        pod.status
            .as_ref()
            .and_then(|status| status.phase.as_deref())
            .is_some_and(|phase| phase == "Failed")
    }) {
        "failed"
    } else if pods.is_empty()
        || pods.iter().all(|pod| pod.metadata.deletion_timestamp.is_some())
        || pods.iter().any(|pod| {
            pod.metadata.deletion_timestamp.is_none()
                && pod
                    .status
                    .as_ref()
                    .and_then(|status| status.phase.as_deref())
                    .is_none_or(|phase| phase != "Running")
        })
        || !containers_with_restarts.is_empty()
    {
        "degraded"
    } else {
        "running"
    }
}

pub(crate) fn service_summaries(
    services: ObjectList<Service>,
    include_control_plane: bool,
) -> Vec<Value> {
    services
        .items
        .iter()
        .filter(|service| include_control_plane || !is_control_plane(&service.metadata))
        .map(service_summary)
        .collect()
}

pub(crate) fn service_summary(service: &Service) -> Value {
    json!({
        "kind": "Service",
        "metadata": metadata_summary(&service.metadata),
        "type": service.spec.as_ref().and_then(|spec| spec.type_.clone()),
        "clusterIp": service.spec.as_ref().and_then(|spec| spec.cluster_ip.clone()),
        "loadBalancerIngress": load_balancer_ingress_summary(service),
        "ports": service.spec.as_ref().map(|spec| {
            spec.ports.clone().unwrap_or_default().into_iter().map(|port| json!({
                "name": port.name,
                "port": port.port,
                "protocol": port.protocol,
            })).collect::<Vec<_>>()
        }).unwrap_or_default(),
    })
}

pub(crate) fn storage_class_summary(storage_class: &StorageClass) -> Value {
    json!({
        "metadata": metadata_summary(&storage_class.metadata),
        "provisioner": storage_class.provisioner,
        "reclaimPolicy": storage_class.reclaim_policy,
        "volumeBindingMode": storage_class.volume_binding_mode,
        "allowVolumeExpansion": storage_class.allow_volume_expansion,
        "isDefault": is_default_storage_class(storage_class),
    })
}

pub(crate) fn event_summary(event: &Event) -> Value {
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

fn load_balancer_ingress_summary(service: &Service) -> Vec<Value> {
    service
        .status
        .as_ref()
        .and_then(|status| status.load_balancer.as_ref())
        .and_then(|load_balancer| load_balancer.ingress.as_ref())
        .map(|ingress| {
            ingress
                .iter()
                .map(|entry| {
                    json!({
                        "ip": entry.ip,
                        "hostname": entry.hostname,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn is_default_storage_class(storage_class: &StorageClass) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::ContainerStateRunning;
    use k8s_openapi::api::core::v1::ContainerStateTerminated;
    use k8s_openapi::api::core::v1::ContainerStateWaiting;
    use k8s_openapi::api::core::v1::PodSpec;
    use k8s_openapi::api::core::v1::PodStatus;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
    use std::str::FromStr;
    use serde_json::json;

    #[test]
    fn workload_diagnostics_report_restarts_and_oom_kills() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("api-abc123".into()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "api".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some("Running".into()),
                container_statuses: Some(vec![ContainerStatus {
                    name: "api".into(),
                    ready: true,
                    restart_count: 3,
                    state: Some(ContainerState {
                        running: Some(ContainerStateRunning::default()),
                        ..Default::default()
                    }),
                    last_state: Some(ContainerState {
                        terminated: Some(ContainerStateTerminated {
                            reason: Some("OOMKilled".into()),
                            exit_code: 137,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            })
        };

        let diagnostics = workload_diagnostics(&[pod]);

        assert_eq!(diagnostics["workloadState"], json!("degraded"));
        assert_eq!(diagnostics["activePodCount"], json!(1));
        assert_eq!(diagnostics["terminatingPodCount"], json!(0));
        assert_eq!(diagnostics["totalRestartCount"], json!(3));
        assert_eq!(diagnostics["oomKilled"], json!(true));
        assert_eq!(
            diagnostics["containersWithRestarts"],
            json!([{
                "pod": "api-abc123",
                "container": "api",
                "ready": true,
                "restartCount": 3,
                "state": "running",
                "stateDetails": {
                    "type": "running"
                },
                "lastState": "terminated",
                "lastStateDetails": {
                    "type": "terminated",
                    "reason": "OOMKilled",
                    "exitCode": 137
                },
                "lastTerminationReason": "OOMKilled",
                "previousLogsAvailable": true
            }])
        );
    }

    #[test]
    fn container_log_target_summaries_include_detailed_state_information() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("worker-0".into()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "worker".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            status: Some(PodStatus {
                container_statuses: Some(vec![ContainerStatus {
                    name: "worker".into(),
                    ready: false,
                    restart_count: 1,
                    state: Some(ContainerState {
                        terminated: Some(ContainerStateTerminated {
                            reason: Some("Error".into()),
                            message: Some("crash loop".into()),
                            exit_code: 1,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    last_state: Some(ContainerState {
                        terminated: Some(ContainerStateTerminated {
                            reason: Some("Completed".into()),
                            exit_code: 0,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            })
        };

        let summaries = pod_log_target_summaries(&[pod]);

        assert_eq!(
            summaries[0]["containers"][0],
            json!({
                "name": "worker",
                "ready": false,
                "restartCount": 1,
                "state": "terminated",
                "stateDetails": {
                    "type": "terminated",
                    "reason": "Error",
                    "message": "crash loop",
                    "exitCode": 1
                },
                "lastState": "terminated",
                "lastStateDetails": {
                    "type": "terminated",
                    "reason": "Completed",
                    "exitCode": 0
                }
            })
        );
    }

    #[test]
    fn workload_diagnostics_marks_pending_pods_without_restarts_as_degraded() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("api-pending".into()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "api".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some("Pending".into()),
                container_statuses: Some(vec![ContainerStatus {
                    name: "api".into(),
                    ready: false,
                    state: Some(ContainerState {
                        waiting: Some(ContainerStateWaiting {
                            reason: Some("ContainerCreating".into()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            })
        };

        let diagnostics = workload_diagnostics(&[pod]);

        assert_eq!(diagnostics["workloadState"], json!("degraded"));
        assert_eq!(diagnostics["totalRestartCount"], json!(0));
    }

    #[test]
    fn workload_diagnostics_includes_init_container_restarts_and_oom_kills() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("api-init".into()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "api".into(),
                    ..Default::default()
                }],
                init_containers: Some(vec![Container {
                    name: "init-db".into(),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some("Pending".into()),
                init_container_statuses: Some(vec![ContainerStatus {
                    name: "init-db".into(),
                    ready: false,
                    restart_count: 2,
                    state: Some(ContainerState {
                        waiting: Some(ContainerStateWaiting {
                            reason: Some("CrashLoopBackOff".into()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    last_state: Some(ContainerState {
                        terminated: Some(ContainerStateTerminated {
                            reason: Some("OOMKilled".into()),
                            exit_code: 137,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            })
        };

        let diagnostics = workload_diagnostics(&[pod]);

        assert_eq!(diagnostics["totalRestartCount"], json!(2));
        assert_eq!(diagnostics["oomKilled"], json!(true));
        assert_eq!(
            diagnostics["containersWithRestarts"],
            json!([{
                "pod": "api-init",
                "container": "init-db",
                "ready": false,
                "restartCount": 2,
                "state": "waiting",
                "stateDetails": {
                    "type": "waiting",
                    "reason": "CrashLoopBackOff"
                },
                "lastState": "terminated",
                "lastStateDetails": {
                    "type": "terminated",
                    "reason": "OOMKilled",
                    "exitCode": 137
                },
                "lastTerminationReason": "OOMKilled",
                "previousLogsAvailable": true
            }])
        );
    }

    #[test]
    fn workload_diagnostics_marks_empty_or_terminating_only_workloads_as_degraded() {
        assert_eq!(workload_diagnostics(&[])["workloadState"], json!("degraded"));

        let terminating_pod = Pod {
            metadata: ObjectMeta {
                name: Some("api-old".into()),
                deletion_timestamp: Some(Time(
                    k8s_openapi::jiff::Timestamp::from_str("2024-01-01T00:00:00Z").unwrap(),
                )),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Running".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let diagnostics = workload_diagnostics(&[terminating_pod]);

        assert_eq!(diagnostics["workloadState"], json!("degraded"));
        assert_eq!(diagnostics["activePodCount"], json!(0));
        assert_eq!(diagnostics["terminatingPodCount"], json!(1));
    }
}
