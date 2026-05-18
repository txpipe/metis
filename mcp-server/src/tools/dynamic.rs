use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::k8s::HelmReleaseDiscovery;
use crate::k8s::KubernetesClient;

use super::ToolDefinition;
use super::workloads;

#[derive(Debug, thiserror::Error)]
pub(crate) enum DynamicToolError {
    #[error("kubernetes error: {0}")]
    Kubernetes(#[from] kube::Error),
    #[error("helm release discovery error: {0}")]
    HelmRelease(#[from] crate::k8s::HelmReleaseError),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DynamicToolState {
    definitions: Arc<RwLock<Arc<Vec<ToolDefinition>>>>,
}

impl DynamicToolState {
    pub(crate) async fn definitions(&self) -> Arc<Vec<ToolDefinition>> {
        self.definitions.read().await.clone()
    }

    pub(crate) async fn signature(&self) -> BTreeSet<&'static str> {
        tool_signature(&self.definitions().await)
    }

    pub(crate) async fn refresh(&self) -> bool {
        let definitions = match discover_definitions().await {
            Ok(definitions) => definitions,
            Err(error) => {
                tracing::debug!(%error, "failed to refresh dynamic MCP tools");
                Vec::new()
            }
        };

        let mut current = self.definitions.write().await;
        let changed = tool_signature(current.as_ref()) != tool_signature(&definitions);
        if changed {
            *current = Arc::new(definitions);
        }
        changed
    }
}

async fn discover_definitions() -> Result<Vec<ToolDefinition>, DynamicToolError> {
    let client = KubernetesClient::try_default().await?;
    let releases = HelmReleaseDiscovery::new(client)
        .list_latest(None, true)
        .await?;
    let installed_extension_ids = workloads::registry::installed_extension_ids(&releases);

    Ok(workloads::dynamic_definitions(&installed_extension_ids))
}

fn tool_signature(definitions: &[ToolDefinition]) -> BTreeSet<&'static str> {
    definitions
        .iter()
        .map(|definition| definition.name)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::k8s::HelmChartSummary;
    use crate::k8s::HelmReleaseSummary;

    use super::*;

    #[test]
    fn dynamic_workload_tools_follow_installed_extensions() {
        let releases = vec![HelmReleaseSummary {
            name: "dolos-preview".to_string(),
            namespace: "cardano".to_string(),
            revision: 1,
            status: Some("deployed".to_string()),
            chart: HelmChartSummary {
                name: Some("dolos".to_string()),
                version: Some("0.1.0".to_string()),
            },
            app_version: None,
            description: None,
            updated: None,
            secret_name: None,
            config: None,
        }];
        let installed_extension_ids = workloads::registry::installed_extension_ids(&releases);

        let definitions = workloads::dynamic_definitions(&installed_extension_ids);

        assert!(
            definitions
                .iter()
                .any(|definition| definition.name == "dolos.snapshot.refresh")
        );
    }
}
