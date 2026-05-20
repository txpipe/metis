use std::collections::BTreeSet;

use crate::catalog::ExtensionCatalog;
use crate::catalog::ExtensionDefinition;
use crate::k8s::HelmReleaseSummary;

pub(crate) const APEX_FUSION_RELAY_CHART_NAME: &str = "apex-fusion-relay";
pub(crate) const APEX_FUSION_RELAY_EXTENSION_ID: &str = "apex-fusion-relay";
pub(crate) const APEX_FUSION_BLOCK_PRODUCER_CHART_NAME: &str = "apex-fusion-block-producer";
pub(crate) const APEX_FUSION_BLOCK_PRODUCER_EXTENSION_ID: &str = "apex-fusion-block-producer";
pub(crate) const CARDANO_RELAY_CHART_NAME: &str = "cardano-relay";
pub(crate) const CARDANO_RELAY_EXTENSION_ID: &str = "cardano-relay";
pub(crate) const CARDANO_BLOCK_PRODUCER_CHART_NAME: &str = "cardano-block-producer";
pub(crate) const CARDANO_BLOCK_PRODUCER_EXTENSION_ID: &str = "cardano-block-producer";
pub(crate) const DOLOS_CHART_NAME: &str = "dolos";
pub(crate) const DOLOS_EXTENSION_ID: &str = "dolos";
pub(crate) const HYDRA_NODE_CHART_NAME: &str = "hydra-node";
pub(crate) const HYDRA_NODE_EXTENSION_ID: &str = "hydra-node";

pub(crate) fn extension_for_release<'a>(
    release: &HelmReleaseSummary,
    catalog: &'a ExtensionCatalog,
) -> Option<&'a ExtensionDefinition> {
    catalog.get(extension_id_for_chart(release.chart.name.as_deref())?)
}

pub(crate) fn extension_id_for_chart(chart_name: Option<&str>) -> Option<&'static str> {
    match chart_name {
        Some(APEX_FUSION_RELAY_CHART_NAME) => Some(APEX_FUSION_RELAY_EXTENSION_ID),
        Some(APEX_FUSION_BLOCK_PRODUCER_CHART_NAME) => {
            Some(APEX_FUSION_BLOCK_PRODUCER_EXTENSION_ID)
        }
        Some(CARDANO_RELAY_CHART_NAME) => Some(CARDANO_RELAY_EXTENSION_ID),
        Some(CARDANO_BLOCK_PRODUCER_CHART_NAME) => Some(CARDANO_BLOCK_PRODUCER_EXTENSION_ID),
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
