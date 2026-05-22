use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::api::core::v1::PersistentVolumeClaim;
use rmcp::model::CallToolResult;
use rmcp::model::JsonObject;
use serde_json::Value;
use serde_json::json;
use tokio::time::Duration;
use tokio::time::Instant;

use crate::k8s::HelmReleaseDiscovery;
use crate::k8s::KubernetesClient;
use crate::k8s::ResourceListParams;
use crate::policy::ApprovalClass;
use crate::policy::Scope;
use crate::tools::ToolDefinition;
use crate::tools::args::{optional_bool, required_string};
use crate::tools::common::kube_error;
use crate::tools::common::success;
use crate::tools::common::tool_error;

use super::registry;

const SNAPSHOT_REFRESH_WAIT_TIMEOUT_SECONDS: u64 = 120;
const SNAPSHOT_REFRESH_WAIT_INTERVAL_SECONDS: u64 = 2;

pub(crate) fn definitions() -> &'static [ToolDefinition] {
    &[ToolDefinition {
        name: "dolos.snapshot.refresh",
        title: "Refresh Dolos Snapshot",
        description: "Delete the managed Dolos data PVC and restart the StatefulSet so Dolos downloads a fresh snapshot.",
        required_scope: Scope::WorkloadsDelete,
        approval_class: ApprovalClass::Destructive,
        read_only: false,
        destructive: true,
        input_schema: r#"{"type":"object","required":["namespace","releaseName"],"properties":{"namespace":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"releaseName":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"dryRun":{"type":"boolean"},"approvalId":{"type":"string"}},"additionalProperties":false}"#,
    }]
}

pub(crate) async fn snapshot_refresh(arguments: Option<&JsonObject>) -> CallToolResult {
    let namespace = match required_string(arguments, "namespace") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let release_name = match required_string(arguments, "releaseName") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let dry_run = optional_bool(arguments, "dryRun").unwrap_or(true);
    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error("dolos.snapshot.refresh", error),
    };

    let release = match HelmReleaseDiscovery::new(client.clone())
        .get_latest(&namespace, &release_name)
        .await
    {
        Ok(Some(release)) => release,
        Ok(None) => {
            return tool_error(
                "not_found",
                format!("Dolos release not found: {namespace}/{release_name}"),
                json!({ "namespace": namespace, "releaseName": release_name }),
            );
        }
        Err(error) => {
            return tool_error(
                "helm_release_discovery_error",
                error.to_string(),
                json!({ "tool": "dolos.snapshot.refresh", "namespace": namespace, "releaseName": release_name }),
            );
        }
    };

    if release.chart.name.as_deref() != Some(registry::DOLOS_EXTENSION_ID) {
        return tool_error(
            "unsupported_workload",
            "snapshot refresh is only supported for Dolos workloads",
            json!({
                "namespace": namespace,
                "releaseName": release_name,
                "chart": release.chart,
            }),
        );
    }

    let stateful_set = match find_dolos_stateful_set(&client, &namespace, &release_name).await {
        Ok(Some(stateful_set)) => stateful_set,
        Ok(None) => {
            return tool_error(
                "not_found",
                format!("Dolos StatefulSet not found: {namespace}/{release_name}"),
                json!({ "namespace": namespace, "releaseName": release_name }),
            );
        }
        Err(error) => return kube_error("dolos.snapshot.refresh", error),
    };
    let stateful_set_name = stateful_set.metadata.name.clone().unwrap_or_default();
    let original_replicas = stateful_set
        .spec
        .as_ref()
        .and_then(|spec| spec.replicas)
        .unwrap_or(1);
    let pvc_names = managed_pvc_names(&stateful_set);
    if pvc_names.is_empty() {
        return tool_error(
            "invalid_workload_state",
            "Dolos StatefulSet does not define managed volume claim templates",
            json!({
                "namespace": namespace,
                "releaseName": release_name,
                "statefulSet": stateful_set_name,
            }),
        );
    }
    let existing_pvcs = match existing_pvcs(&client, &namespace, &pvc_names).await {
        Ok(pvcs) => pvcs,
        Err(error) => return kube_error("dolos.snapshot.refresh", error),
    };

    let plan = json!({
        "statefulSet": stateful_set_name,
        "originalReplicas": original_replicas,
        "scaleDownReplicas": 0,
        "scaleUpReplicas": original_replicas,
        "managedPvcs": pvc_names,
        "existingPvcs": existing_pvcs.iter().map(pvc_summary).collect::<Vec<_>>(),
    });

    if dry_run {
        return success(json!({
            "action": "refresh-dolos-snapshot",
            "dryRun": true,
            "wouldMutate": false,
            "namespace": namespace,
            "releaseName": release_name,
            "plan": plan,
            "notes": [
                "dry-run planning only; no Kubernetes mutation was performed",
                "actual execution scales the StatefulSet to zero, deletes managed data PVCs, then restores the original replica count"
            ],
        }));
    }

    if let Err(error) = client
        .scale_stateful_set(&namespace, &stateful_set_name, 0)
        .await
    {
        return kube_error("dolos.snapshot.refresh", error);
    }
    if let Err(error) = wait_for_dolos_pods_absent(&client, &namespace, &release_name).await {
        let _ = client
            .scale_stateful_set(&namespace, &stateful_set_name, original_replicas)
            .await;
        return error;
    }
    for pvc_name in &pvc_names {
        if let Err(error) = client
            .delete_persistent_volume_claim(&namespace, pvc_name)
            .await
        {
            let _ = client
                .scale_stateful_set(&namespace, &stateful_set_name, original_replicas)
                .await;
            return kube_error("dolos.snapshot.refresh", error);
        }
    }
    if let Err(error) = client
        .scale_stateful_set(&namespace, &stateful_set_name, original_replicas)
        .await
    {
        return kube_error("dolos.snapshot.refresh", error);
    }

    success(json!({
        "action": "refresh-dolos-snapshot",
        "dryRun": false,
        "wouldMutate": true,
        "namespace": namespace,
        "releaseName": release_name,
        "plan": plan,
        "deletedPvcs": pvc_names,
        "notes": [
            "Dolos StatefulSet was scaled down before PVC deletion",
            "Dolos StatefulSet replica count was restored so Kubernetes can create a new PVC and pod"
        ],
    }))
}

async fn find_dolos_stateful_set(
    client: &KubernetesClient,
    namespace: &str,
    release_name: &str,
) -> Result<Option<StatefulSet>, kube::Error> {
    let stateful_sets = client
        .list_stateful_sets(
            Some(namespace),
            &ResourceListParams {
                label_selector: Some(format!(
                    "app.kubernetes.io/instance={release_name},app.kubernetes.io/name=dolos"
                )),
                ..Default::default()
            },
        )
        .await?;

    Ok(stateful_sets
        .items
        .into_iter()
        .min_by(|left, right| left.metadata.name.cmp(&right.metadata.name)))
}

pub(crate) fn managed_pvc_names(stateful_set: &StatefulSet) -> Vec<String> {
    let Some(stateful_set_name) = stateful_set.metadata.name.as_deref() else {
        return vec![];
    };
    let replicas = stateful_set
        .spec
        .as_ref()
        .and_then(|spec| spec.replicas)
        .unwrap_or(1)
        .max(1);

    stateful_set
        .spec
        .as_ref()
        .and_then(|spec| spec.volume_claim_templates.as_ref())
        .map(|templates| {
            templates
                .iter()
                .flat_map(|template| {
                    let claim_name = template.metadata.name.as_deref().unwrap_or_default();
                    (0..replicas)
                        .map(move |ordinal| format!("{claim_name}-{stateful_set_name}-{ordinal}"))
                })
                .collect()
        })
        .unwrap_or_default()
}

async fn existing_pvcs(
    client: &KubernetesClient,
    namespace: &str,
    pvc_names: &[String],
) -> Result<Vec<PersistentVolumeClaim>, kube::Error> {
    let mut pvcs = Vec::new();

    for pvc_name in pvc_names {
        match client
            .get_persistent_volume_claim(namespace, pvc_name)
            .await
        {
            Ok(pvc) => pvcs.push(pvc),
            Err(kube::Error::Api(error)) if error.code == 404 => {}
            Err(error) => return Err(error),
        }
    }

    Ok(pvcs)
}

async fn wait_for_dolos_pods_absent(
    client: &KubernetesClient,
    namespace: &str,
    release_name: &str,
) -> Result<(), CallToolResult> {
    let deadline = Instant::now() + Duration::from_secs(SNAPSHOT_REFRESH_WAIT_TIMEOUT_SECONDS);
    let params = ResourceListParams {
        label_selector: Some(format!(
            "app.kubernetes.io/instance={release_name},app.kubernetes.io/name=dolos"
        )),
        ..Default::default()
    };

    loop {
        let pods = client
            .list_pods(Some(namespace), &params)
            .await
            .map_err(|error| kube_error("dolos.snapshot.refresh", error))?;
        if pods.items.is_empty() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(tool_error(
                "workload_restart_timeout",
                "timed out waiting for Dolos pods to terminate after scaling StatefulSet to zero",
                json!({
                    "namespace": namespace,
                    "releaseName": release_name,
                    "remainingPods": pods.items.iter().filter_map(|pod| pod.metadata.name.clone()).collect::<Vec<_>>(),
                    "timeoutSeconds": SNAPSHOT_REFRESH_WAIT_TIMEOUT_SECONDS,
                }),
            ));
        }
        tokio::time::sleep(Duration::from_secs(SNAPSHOT_REFRESH_WAIT_INTERVAL_SECONDS)).await;
    }
}

fn pvc_summary(pvc: &PersistentVolumeClaim) -> Value {
    json!({
        "name": pvc.metadata.name,
        "namespace": pvc.metadata.namespace,
        "phase": pvc.status.as_ref().and_then(|status| status.phase.clone()),
    })
}

#[cfg(test)]
mod tests {
    use k8s_openapi::api::core::v1::PersistentVolumeClaim;
    use k8s_openapi::api::core::v1::PersistentVolumeClaimSpec;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    use super::*;

    #[test]
    fn definition_is_destructive_and_requires_workload_delete_scope() {
        let definition = definitions().first().unwrap();

        assert_eq!(definition.name, "dolos.snapshot.refresh");
        assert!(definition.destructive);
        assert_eq!(definition.required_scope, Scope::WorkloadsDelete);
        assert_eq!(definition.approval_class, ApprovalClass::Destructive);
    }

    #[test]
    fn managed_pvc_names_follow_stateful_set_volume_claim_template_names() {
        let stateful_set = StatefulSet {
            metadata: ObjectMeta {
                name: Some("dolos-preview".to_string()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::apps::v1::StatefulSetSpec {
                replicas: Some(2),
                volume_claim_templates: Some(vec![PersistentVolumeClaim {
                    metadata: ObjectMeta {
                        name: Some("data".to_string()),
                        ..Default::default()
                    },
                    spec: Some(PersistentVolumeClaimSpec::default()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(
            managed_pvc_names(&stateful_set),
            vec![
                "data-dolos-preview-0".to_string(),
                "data-dolos-preview-1".to_string(),
            ]
        );
    }
}
