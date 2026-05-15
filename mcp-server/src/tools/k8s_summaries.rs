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

pub(crate) fn deployment_summaries(
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

pub(crate) fn deployment_summary(deployment: &Deployment) -> Value {
    json!({
        "kind": "Deployment",
        "metadata": metadata_summary(&deployment.metadata),
        "replicas": deployment.spec.as_ref().and_then(|spec| spec.replicas),
        "readyReplicas": deployment.status.as_ref().and_then(|status| status.ready_replicas),
        "availableReplicas": deployment.status.as_ref().and_then(|status| status.available_replicas),
    })
}

pub(crate) fn stateful_set_summaries(
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

pub(crate) fn stateful_set_summary(stateful_set: &StatefulSet) -> Value {
    json!({
        "kind": "StatefulSet",
        "metadata": metadata_summary(&stateful_set.metadata),
        "replicas": stateful_set.spec.as_ref().and_then(|spec| spec.replicas),
        "readyReplicas": stateful_set.status.as_ref().and_then(|status| status.ready_replicas),
        "currentReplicas": stateful_set.status.as_ref().and_then(|status| status.current_replicas),
    })
}

pub(crate) fn pod_summaries(pods: ObjectList<Pod>, include_control_plane: bool) -> Vec<Value> {
    pods.items
        .iter()
        .filter(|pod| include_control_plane || !is_control_plane(&pod.metadata))
        .map(pod_summary)
        .collect()
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
                "lastState": status.and_then(|status| container_state(status.last_state.as_ref())),
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
