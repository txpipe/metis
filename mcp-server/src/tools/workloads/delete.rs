use std::collections::BTreeSet;

use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::api::core::v1::PersistentVolumeClaim;
use rmcp::model::{CallToolResult, JsonObject};
use serde_json::{Value, json};

use crate::catalog::ExtensionCatalog;
use crate::helm::{self, HelmUninstallPlan};
use crate::k8s::{HelmReleaseDiscovery, KubernetesClient, ResourceListParams};
use crate::tools::common::{kube_error, success, tool_error};

use super::registry;

const TOOL_NAME: &str = "workloads.delete";

pub(crate) async fn delete(
    arguments: Option<&JsonObject>,
    catalog: &ExtensionCatalog,
) -> CallToolResult {
    let namespace = match required_string(arguments, "namespace") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let release_name = match required_string(arguments, "releaseName") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let delete_pvcs = optional_bool(arguments, "deletePvcs").unwrap_or(false);
    let dry_run = optional_bool(arguments, "dryRun").unwrap_or(true);
    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error(TOOL_NAME, error),
    };

    let release = match HelmReleaseDiscovery::new(client.clone())
        .get_latest(&namespace, &release_name)
        .await
    {
        Ok(Some(release)) => release,
        Ok(None) => {
            return tool_error(
                "not_found",
                format!("workload release not found: {namespace}/{release_name}"),
                json!({ "namespace": namespace, "releaseName": release_name }),
            );
        }
        Err(error) => {
            return tool_error(
                "helm_release_discovery_error",
                error.to_string(),
                json!({ "tool": TOOL_NAME, "namespace": namespace, "releaseName": release_name }),
            );
        }
    };

    let extension = match registry::extension_for_release(&release, catalog) {
        Some(extension) => extension,
        None => {
            return tool_error(
                "unsupported_workload",
                "workloads.delete only deletes catalog-managed extension releases",
                json!({
                    "namespace": namespace,
                    "releaseName": release_name,
                    "chart": release.chart,
                }),
            );
        }
    };

    let pvc_candidates = match pvc_candidates(&client, &namespace, &release_name).await {
        Ok(pvcs) => pvcs,
        Err(error) => return kube_error(TOOL_NAME, error),
    };

    let plan = json!({
        "helmRelease": release,
        "extension": {
            "id": extension.id,
            "name": extension.name,
            "version": extension.default_version,
        },
        "deletePvcs": delete_pvcs,
        "preservedPvcs": if delete_pvcs { Vec::<Value>::new() } else { pvc_candidates.iter().map(pvc_summary).collect::<Vec<_>>() },
        "pvcsToDelete": if delete_pvcs { pvc_candidates.iter().map(pvc_summary).collect::<Vec<_>>() } else { Vec::<Value>::new() },
    });

    if dry_run {
        return success(json!({
            "action": "delete",
            "dryRun": true,
            "wouldMutate": false,
            "namespace": namespace,
            "releaseName": release_name,
            "plan": plan,
            "notes": [
                "dry-run planning only; no Kubernetes or Helm mutation was performed",
                "PVCs are preserved by default; set deletePvcs=true to delete candidate PVCs after Helm uninstall"
            ],
        }));
    }

    let helm_result = match helm::uninstall(&HelmUninstallPlan {
        release_name: release_name.clone(),
        namespace: namespace.clone(),
    })
    .await
    {
        Ok(result) => result,
        Err(error) => {
            let helm_details = match &error {
                helm::HelmUninstallError::Failed {
                    status,
                    stdout,
                    stderr,
                } => json!({
                    "tool": TOOL_NAME,
                    "releaseName": release_name,
                    "namespace": namespace,
                    "status": status,
                    "stdout": stdout,
                    "stderr": stderr,
                }),
                _ => json!({
                    "tool": TOOL_NAME,
                    "releaseName": release_name,
                    "namespace": namespace,
                }),
            };
            return tool_error("helm_uninstall_failed", error.to_string(), helm_details);
        }
    };

    let mut deleted_pvcs = Vec::new();
    if delete_pvcs {
        for pvc_name in pvc_candidates.iter().filter_map(pvc_name) {
            match client
                .delete_persistent_volume_claim(&namespace, pvc_name)
                .await
            {
                Ok(()) => deleted_pvcs.push(pvc_name.to_string()),
                Err(kube::Error::Api(error)) if error.code == 404 => {}
                Err(error) => return kube_error(TOOL_NAME, error),
            }
        }
    }

    success(json!({
        "action": "delete",
        "dryRun": false,
        "wouldMutate": true,
        "namespace": namespace,
        "releaseName": release_name,
        "plan": plan,
        "helm": helm_result,
        "deletedPvcs": deleted_pvcs,
        "notes": if delete_pvcs {
            vec![
                "Helm release was uninstalled successfully",
                "Candidate PVCs were deleted after Helm uninstall",
            ]
        } else {
            vec![
                "Helm release was uninstalled successfully",
                "PVCs were preserved; deletePvcs defaults to false",
            ]
        },
    }))
}

async fn pvc_candidates(
    client: &KubernetesClient,
    namespace: &str,
    release_name: &str,
) -> Result<Vec<PersistentVolumeClaim>, kube::Error> {
    let stateful_sets = client
        .list_stateful_sets(
            Some(namespace),
            &ResourceListParams {
                label_selector: Some(format!("app.kubernetes.io/instance={release_name}")),
                ..Default::default()
            },
        )
        .await?;
    let managed_prefixes = managed_pvc_prefixes(&stateful_sets.items);
    let all_pvcs = client
        .list_persistent_volume_claims(Some(namespace), &ResourceListParams::default())
        .await?;
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();

    for pvc in all_pvcs.items {
        let Some(name) = pvc_name(&pvc) else {
            continue;
        };
        let label_match = pvc
            .metadata
            .labels
            .as_ref()
            .and_then(|labels| labels.get("app.kubernetes.io/instance"))
            .is_some_and(|value| value == release_name);
        let prefix_match = managed_prefixes
            .iter()
            .any(|prefix| name.starts_with(prefix));

        if (label_match || prefix_match) && seen.insert(name.to_string()) {
            candidates.push(pvc);
        }
    }

    candidates.sort_by(|left, right| left.metadata.name.cmp(&right.metadata.name));
    Ok(candidates)
}

fn managed_pvc_prefixes(stateful_sets: &[StatefulSet]) -> Vec<String> {
    stateful_sets
        .iter()
        .flat_map(|stateful_set| {
            let stateful_set_name = stateful_set.metadata.name.as_deref().unwrap_or_default();
            stateful_set
                .spec
                .as_ref()
                .and_then(|spec| spec.volume_claim_templates.as_ref())
                .into_iter()
                .flatten()
                .filter_map(move |template| {
                    let claim_name = template.metadata.name.as_deref()?;
                    Some(format!("{claim_name}-{stateful_set_name}-"))
                })
        })
        .collect()
}

fn pvc_summary(pvc: &PersistentVolumeClaim) -> Value {
    json!({
        "name": pvc.metadata.name,
        "namespace": pvc.metadata.namespace,
        "phase": pvc.status.as_ref().and_then(|status| status.phase.clone()),
    })
}

fn pvc_name(pvc: &PersistentVolumeClaim) -> Option<&str> {
    pvc.metadata.name.as_deref()
}

fn required_string(arguments: Option<&JsonObject>, name: &str) -> Result<String, CallToolResult> {
    arguments
        .and_then(|arguments| arguments.get(name))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            tool_error(
                "invalid_arguments",
                format!("missing required string argument: {name}"),
                json!({ "argument": name }),
            )
        })
}

fn optional_bool(arguments: Option<&JsonObject>, name: &str) -> Option<bool> {
    arguments
        .and_then(|arguments| arguments.get(name))
        .and_then(Value::as_bool)
}

#[cfg(test)]
mod tests {
    use k8s_openapi::api::apps::v1::StatefulSetSpec;
    use k8s_openapi::api::core::v1::PersistentVolumeClaimSpec;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    use crate::policy::{ApprovalClass, Scope};
    use crate::tools::workloads;

    use super::*;

    #[test]
    fn workload_delete_definition_is_destructive() {
        let definition = workloads::definitions()
            .iter()
            .find(|definition| definition.name == "workloads.delete")
            .unwrap();

        assert!(definition.destructive);
        assert_eq!(definition.required_scope, Scope::WorkloadsDelete);
        assert_eq!(definition.approval_class, ApprovalClass::Destructive);
        assert!(definition.input_schema.contains("dryRun"));
        assert!(definition.input_schema.contains("deletePvcs"));
    }

    #[test]
    fn managed_pvc_prefixes_follow_stateful_set_templates() {
        let stateful_set = StatefulSet {
            metadata: ObjectMeta {
                name: Some("hydra-offline-hydra-node".to_string()),
                ..Default::default()
            },
            spec: Some(StatefulSetSpec {
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
            managed_pvc_prefixes(&[stateful_set]),
            vec!["data-hydra-offline-hydra-node-".to_string()]
        );
    }

    #[test]
    fn missing_release_name_returns_invalid_arguments() {
        let mut arguments = JsonObject::new();
        arguments.insert("namespace".to_string(), Value::String("hydra".to_string()));

        let error = required_string(Some(&arguments), "releaseName").unwrap_err();

        assert_eq!(
            error
                .structured_content
                .as_ref()
                .and_then(|content| content.get("error")),
            Some(&Value::String("invalid_arguments".to_string()))
        );
    }
}
