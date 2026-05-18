use crate::catalog::ExtensionCatalog;
use crate::catalog::ExtensionDefinition;
use crate::k8s::HelmReleaseSummary;

use super::registry;

pub(crate) const SCRIPT_PATH: &str = "/opt/metis/bin/metrics.sh";

#[derive(Clone, Copy)]
pub(crate) struct MetricsTarget<'a> {
    pub extension: &'a ExtensionDefinition,
    pub container: &'static str,
}

pub(crate) fn target_for_release<'a>(
    release: &HelmReleaseSummary,
    catalog: &'a ExtensionCatalog,
) -> Option<MetricsTarget<'a>> {
    match release.chart.name.as_deref() {
        Some(registry::CARDANO_NODE_CHART_NAME) => catalog
            .get(registry::CARDANO_NODE_RELAY_EXTENSION_ID)
            .map(|extension| MetricsTarget {
                extension,
                container: registry::CARDANO_NODE_METRICS_CONTAINER,
            }),
        Some(registry::DOLOS_CHART_NAME) => {
            catalog
                .get(registry::DOLOS_EXTENSION_ID)
                .map(|extension| MetricsTarget {
                    extension,
                    container: registry::DOLOS_METRICS_CONTAINER,
                })
        }
        Some(registry::HYDRA_NODE_CHART_NAME) => catalog
            .get(registry::HYDRA_NODE_EXTENSION_ID)
            .map(|extension| MetricsTarget {
                extension,
                container: registry::HYDRA_NODE_METRICS_CONTAINER,
            }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::ExtensionCatalog;
    use crate::k8s::HelmChartSummary;
    use crate::k8s::HelmReleaseSummary;

    use super::*;

    #[test]
    fn resolves_cardano_node_chart() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("cardano-node"));

        let target = target_for_release(&release, &catalog).unwrap();

        assert_eq!(target.extension.id, "cardano-node-relay");
        assert_eq!(target.container, registry::CARDANO_NODE_METRICS_CONTAINER);
    }

    #[test]
    fn resolves_dolos_chart() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("dolos"));

        let target = target_for_release(&release, &catalog).unwrap();

        assert_eq!(target.extension.id, "dolos");
        assert_eq!(target.container, registry::DOLOS_METRICS_CONTAINER);
    }

    #[test]
    fn resolves_hydra_node_chart() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("hydra-node"));

        let target = target_for_release(&release, &catalog).unwrap();

        assert_eq!(target.extension.id, "hydra-node");
        assert_eq!(target.container, registry::HYDRA_NODE_METRICS_CONTAINER);
    }

    #[test]
    fn rejects_unsupported_chart() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("midnight"));

        assert!(target_for_release(&release, &catalog).is_none());
    }

    fn helm_release(chart_name: Option<&str>) -> HelmReleaseSummary {
        HelmReleaseSummary {
            name: "relay-preview".to_string(),
            namespace: "cardano".to_string(),
            revision: 1,
            status: Some("deployed".to_string()),
            chart: HelmChartSummary {
                name: chart_name.map(str::to_string),
                version: Some("0.1.0".to_string()),
            },
            app_version: Some("11.0.1".to_string()),
            description: None,
            updated: None,
            secret_name: Some("sh.helm.release.v1.relay-preview.v1".to_string()),
            config: None,
        }
    }
}
