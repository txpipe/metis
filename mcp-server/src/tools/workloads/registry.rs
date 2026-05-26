use std::collections::BTreeSet;

use crate::catalog::ExtensionCatalog;
use crate::catalog::ExtensionDefinition;
use crate::k8s::HelmReleaseSummary;

pub(crate) const APEX_FUSION_RELAY_EXTENSION_ID: &str = "apex-fusion-relay";
pub(crate) const APEX_FUSION_BLOCK_PRODUCER_EXTENSION_ID: &str = "apex-fusion-block-producer";
pub(crate) const CARDANO_RELAY_EXTENSION_ID: &str = "cardano-relay";
pub(crate) const CARDANO_BLOCK_PRODUCER_EXTENSION_ID: &str = "cardano-block-producer";
pub(crate) const CARDANO_DB_SYNC_EXTENSION_ID: &str = "cardano-db-sync";
pub(crate) const DOLOS_EXTENSION_ID: &str = "dolos";
pub(crate) const HYDRA_NODE_EXTENSION_ID: &str = "hydra-node";
pub(crate) const MIDNIGHT_EXTENSION_ID: &str = "midnight";

pub(crate) fn extension_for_release<'a>(
    release: &HelmReleaseSummary,
    catalog: &'a ExtensionCatalog,
) -> Option<&'a ExtensionDefinition> {
    catalog.get(release.chart.name.as_deref()?)
}

pub(crate) fn installed_extension_ids(
    releases: &[HelmReleaseSummary],
    catalog: &ExtensionCatalog,
) -> BTreeSet<String> {
    releases
        .iter()
        .filter(|release| release.status.as_deref() == Some("deployed"))
        .filter_map(|release| release.chart.name.as_deref())
        .filter(|extension_id| catalog.get(extension_id).is_some())
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::k8s::HelmChartSummary;

    use super::*;

    #[test]
    fn release_chart_name_is_the_extension_id() {
        let catalog = ExtensionCatalog::testing();
        let release = release("cardano-relay", "deployed");

        let extension = extension_for_release(&release, &catalog).unwrap();

        assert_eq!(extension.id, "cardano-relay");
    }

    #[test]
    fn unknown_chart_names_are_not_catalog_managed() {
        let catalog = ExtensionCatalog::testing();
        let release = release("not-in-catalog", "deployed");

        assert!(extension_for_release(&release, &catalog).is_none());
    }

    #[test]
    fn installed_extension_ids_are_catalog_backed_chart_names() {
        let catalog = ExtensionCatalog::testing();
        let releases = vec![
            release("dolos", "deployed"),
            release("not-in-catalog", "deployed"),
            release("hydra-node", "failed"),
        ];

        let installed = installed_extension_ids(&releases, &catalog);

        assert_eq!(installed, BTreeSet::from(["dolos".to_string()]));
    }

    fn release(chart_name: &str, status: &str) -> HelmReleaseSummary {
        HelmReleaseSummary {
            name: format!("{chart_name}-preview"),
            namespace: "preview".to_string(),
            revision: 1,
            status: Some(status.to_string()),
            chart: HelmChartSummary {
                name: Some(chart_name.to_string()),
                version: Some("0.1.0".to_string()),
            },
            app_version: None,
            description: None,
            updated: None,
            secret_name: None,
            config: None,
        }
    }
}
