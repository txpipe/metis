use std::collections::BTreeSet;

use crate::catalog::ExtensionCatalog;
use crate::catalog::ExtensionDefinition;
use crate::k8s::HelmReleaseSummary;

pub(crate) const CARDANO_NODE_CHART_NAME: &str = "cardano-node";
pub(crate) const CARDANO_NODE_RELAY_EXTENSION_ID: &str = "cardano-node-relay";
pub(crate) const CARDANO_NODE_METRICS_CONTAINER: &str = "cardano-node";
pub(crate) const DOLOS_CHART_NAME: &str = "dolos";
pub(crate) const DOLOS_EXTENSION_ID: &str = "dolos";
pub(crate) const DOLOS_METRICS_CONTAINER: &str = "dolos";
pub(crate) const HYDRA_NODE_CHART_NAME: &str = "hydra-node";
pub(crate) const HYDRA_NODE_EXTENSION_ID: &str = "hydra-node";
pub(crate) const HYDRA_NODE_METRICS_CONTAINER: &str = "hydra-node";

pub(crate) fn extension_for_release<'a>(
    release: &HelmReleaseSummary,
    catalog: &'a ExtensionCatalog,
) -> Option<&'a ExtensionDefinition> {
    catalog.get(extension_id_for_chart(release.chart.name.as_deref())?)
}

pub(crate) fn extension_id_for_chart(chart_name: Option<&str>) -> Option<&'static str> {
    match chart_name {
        Some(CARDANO_NODE_CHART_NAME) => Some(CARDANO_NODE_RELAY_EXTENSION_ID),
        Some(DOLOS_CHART_NAME) => Some(DOLOS_EXTENSION_ID),
        Some(HYDRA_NODE_CHART_NAME) => Some(HYDRA_NODE_EXTENSION_ID),
        _ => None,
    }
}

pub(crate) fn installed_extension_ids(releases: &[HelmReleaseSummary]) -> BTreeSet<String> {
    releases
        .iter()
        .filter(|release| release.status.as_deref() == Some("deployed"))
        .filter_map(|release| extension_id_for_chart(release.chart.name.as_deref()))
        .map(str::to_string)
        .collect()
}
