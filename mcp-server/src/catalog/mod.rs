pub mod extension;
mod oci;
pub mod source;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub use extension::ExtensionDefinition;
pub use extension::ExtensionId;
pub use extension::ExtensionOutputDefinition;

const CATALOG_SCHEMA_VERSION: &str = "supernode.extensionCatalog/v1";
const TRUSTED_OCI_REGISTRY: &str = "oci.supernode.store";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionCatalogDocument {
    pub schema_version: String,
    pub extensions: Vec<ExtensionDefinition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionCatalog {
    extensions: BTreeMap<ExtensionId, ExtensionDefinition>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSummary<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub versions: &'a [String],
    pub default_version: &'a str,
    pub dependencies: &'a [ExtensionId],
    pub outputs: &'a [ExtensionOutputDefinition],
    pub chart: &'a str,
}

pub fn extension_summary(extension: &ExtensionDefinition) -> ExtensionSummary<'_> {
    ExtensionSummary {
        id: &extension.id,
        name: &extension.name,
        description: &extension.description,
        versions: &extension.versions,
        default_version: &extension.default_version,
        dependencies: &extension.dependencies,
        outputs: &extension.outputs,
        chart: &extension.chart,
    }
}

impl ExtensionCatalog {
    pub fn bundled() -> Self {
        Self::from_json_str(include_str!("../../../catalog/extension-catalog.json"))
            .expect("bundled extension catalog must be valid")
    }

    pub fn from_extensions(extensions: impl IntoIterator<Item = ExtensionDefinition>) -> Self {
        let extensions = extensions
            .into_iter()
            .map(|extension| (extension.id.clone(), extension))
            .collect();
        Self { extensions }
    }

    pub fn from_json_str(payload: &str) -> Result<Self, CatalogLoadError> {
        Self::from_json_str_with_trust(payload, false)
    }

    pub fn from_json_str_with_trust(
        payload: &str,
        allow_untrusted: bool,
    ) -> Result<Self, CatalogLoadError> {
        let document = serde_json::from_str::<ExtensionCatalogDocument>(payload)?;
        Self::from_document_with_trust(document, allow_untrusted)
    }

    pub fn from_document_with_trust(
        document: ExtensionCatalogDocument,
        allow_untrusted: bool,
    ) -> Result<Self, CatalogLoadError> {
        if document.schema_version != CATALOG_SCHEMA_VERSION {
            return Err(CatalogLoadError::UnsupportedSchemaVersion(
                document.schema_version,
            ));
        }

        validate_extensions(&document.extensions, allow_untrusted)?;
        Ok(Self::from_extensions(document.extensions))
    }

    #[cfg(test)]
    pub fn testing() -> Self {
        Self::bundled()
    }

    pub fn list(&self) -> impl Iterator<Item = &ExtensionDefinition> {
        self.extensions.values()
    }

    pub fn get(&self, extension_id: &str) -> Option<&ExtensionDefinition> {
        self.extensions.get(extension_id)
    }

    pub fn len(&self) -> usize {
        self.extensions.len()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogLoadError {
    #[error("extension catalog JSON is invalid: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("extension catalog JSON is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("unsupported extension catalog schema version: {0}")]
    UnsupportedSchemaVersion(String),
    #[error("invalid extension catalog: {0}")]
    InvalidCatalog(String),
    #[error("missing extension catalog OCI reference")]
    MissingOciReference,
    #[error("untrusted extension catalog OCI reference: {0}")]
    UntrustedCatalogReference(String),
    #[error("untrusted extension chart OCI reference: {0}")]
    UntrustedChartReference(String),
    #[error("failed to load extension catalog from OCI: {0}")]
    Oci(#[from] oci::OciCatalogError),
}

fn validate_extensions(
    extensions: &[ExtensionDefinition],
    allow_untrusted: bool,
) -> Result<(), CatalogLoadError> {
    let mut ids = BTreeSet::new();
    for extension in extensions {
        if extension.id.trim().is_empty() {
            return invalid_catalog("extension id must not be empty");
        }
        if !ids.insert(extension.id.as_str()) {
            return invalid_catalog(format!("duplicate extension id: {}", extension.id));
        }
        if extension.name.trim().is_empty() {
            return invalid_catalog(format!(
                "extension name must not be empty: {}",
                extension.id
            ));
        }
        if extension.versions.is_empty() {
            return invalid_catalog(format!(
                "extension versions must not be empty: {}",
                extension.id
            ));
        }
        if !extension
            .versions
            .iter()
            .any(|version| version == &extension.default_version)
        {
            return invalid_catalog(format!(
                "extension defaultVersion must be listed in versions: {}",
                extension.id
            ));
        }
        if !extension.configuration.is_object() {
            return invalid_catalog(format!(
                "extension configuration schema must be an object: {}",
                extension.id
            ));
        }
        if !extension.metrics.is_object() {
            return invalid_catalog(format!(
                "extension metrics schema must be an object: {}",
                extension.id
            ));
        }
        if !extension.chart.starts_with("oci://") {
            return invalid_catalog(format!(
                "extension chart must be an OCI reference: {}",
                extension.id
            ));
        }
        if chart_basename(&extension.chart) != Some(extension.id.as_str()) {
            return invalid_catalog(format!(
                "extension id must match OCI chart basename: {}",
                extension.id
            ));
        }
        if !allow_untrusted && !is_trusted_extension_chart(&extension.chart, &extension.id) {
            return Err(CatalogLoadError::UntrustedChartReference(
                extension.chart.clone(),
            ));
        }
        if let Some(metrics_collection) = &extension.metrics_collection {
            if metrics_collection.container.trim().is_empty() {
                return invalid_catalog(format!(
                    "extension metrics collection container must not be empty: {}",
                    extension.id
                ));
            }
            if metrics_collection.command.is_empty() {
                return invalid_catalog(format!(
                    "extension metrics collection command must not be empty: {}",
                    extension.id
                ));
            }
            if metrics_collection
                .pod_label_selector
                .as_deref()
                .is_some_and(|selector| selector.trim().is_empty())
            {
                return invalid_catalog(format!(
                    "extension metrics collection pod label selector must not be empty: {}",
                    extension.id
                ));
            }
        }
        for output in &extension.outputs {
            if output.name.trim().is_empty() || output.port_name.trim().is_empty() {
                return invalid_catalog(format!(
                    "extension output names must not be empty: {}",
                    extension.id
                ));
            }
        }
    }

    for extension in extensions {
        for dependency in &extension.dependencies {
            if !ids.contains(dependency.as_str()) {
                return invalid_catalog(format!(
                    "extension dependency is not in catalog: {} -> {}",
                    extension.id, dependency
                ));
            }
        }
    }

    Ok(())
}

fn is_trusted_extension_chart(chart: &str, extension_id: &str) -> bool {
    let Some(rest) = chart.strip_prefix("oci://") else {
        return false;
    };
    let Some((registry, repository)) = rest.split_once('/') else {
        return false;
    };

    registry == TRUSTED_OCI_REGISTRY
        && strip_oci_reference_suffix(repository) == format!("extensions/{extension_id}")
}

fn strip_oci_reference_suffix(repository: &str) -> &str {
    let without_digest = repository
        .split_once('@')
        .map_or(repository, |(repo, _)| repo);
    let last_slash = without_digest.rfind('/');
    if let Some(index) = without_digest.rfind(':')
        && last_slash
            .map(|last_slash| index > last_slash)
            .unwrap_or(true)
    {
        return &without_digest[..index];
    }

    without_digest
}

fn chart_basename(chart: &str) -> Option<&str> {
    strip_oci_reference_suffix(chart.strip_prefix("oci://")?.trim_end_matches('/'))
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
}

fn invalid_catalog(message: impl Into<String>) -> Result<(), CatalogLoadError> {
    Err(CatalogLoadError::InvalidCatalog(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn bundled_catalog_contains_cardano_extensions_dolos_and_hydra() {
        let catalog = ExtensionCatalog::testing();

        assert_eq!(catalog.len(), 7);
        assert!(catalog.get("apex-fusion-relay").is_some());
        assert!(catalog.get("apex-fusion-block-producer").is_some());
        assert!(catalog.get("cardano-relay").is_some());
        assert!(catalog.get("cardano-block-producer").is_some());
        assert!(catalog.get("cardano-db-sync").is_some());
        assert!(catalog.get("cardano-node").is_none());
        assert!(catalog.get("dolos").is_some());
        assert!(catalog.get("hydra-node").is_some());
    }

    #[test]
    fn catalog_json_rejects_duplicate_extension_ids() {
        let extension = ExtensionCatalog::testing().get("dolos").unwrap().clone();
        let document = ExtensionCatalogDocument {
            schema_version: CATALOG_SCHEMA_VERSION.to_string(),
            extensions: vec![extension.clone(), extension],
        };

        let error = ExtensionCatalog::from_document_with_trust(document, false).unwrap_err();

        assert!(matches!(error, CatalogLoadError::InvalidCatalog(_)));
    }

    #[test]
    fn catalog_json_requires_default_version_to_be_listed() {
        let mut extension = ExtensionCatalog::testing().get("dolos").unwrap().clone();
        extension.default_version = "9.9.9".to_string();
        let document = ExtensionCatalogDocument {
            schema_version: CATALOG_SCHEMA_VERSION.to_string(),
            extensions: vec![extension],
        };

        let error = ExtensionCatalog::from_document_with_trust(document, false).unwrap_err();

        assert!(matches!(error, CatalogLoadError::InvalidCatalog(_)));
    }

    #[test]
    fn catalog_json_requires_extension_id_to_match_chart_basename() {
        let mut extension = ExtensionCatalog::testing().get("dolos").unwrap().clone();
        extension.chart = "oci://oci.supernode.store/extensions/not-dolos".to_string();
        let document = ExtensionCatalogDocument {
            schema_version: CATALOG_SCHEMA_VERSION.to_string(),
            extensions: vec![extension],
        };

        let error = ExtensionCatalog::from_document_with_trust(document, false).unwrap_err();

        assert!(matches!(error, CatalogLoadError::InvalidCatalog(_)));
    }

    #[test]
    fn catalog_json_accepts_chart_basename_with_tag() {
        let mut extension = ExtensionCatalog::testing().get("dolos").unwrap().clone();
        extension.chart = "oci://oci.supernode.store/extensions/dolos:1.2.3".to_string();
        let document = ExtensionCatalogDocument {
            schema_version: CATALOG_SCHEMA_VERSION.to_string(),
            extensions: vec![extension],
        };

        let catalog = ExtensionCatalog::from_document_with_trust(document, false).unwrap();

        assert!(catalog.get("dolos").is_some());
    }

    #[test]
    fn catalog_json_accepts_chart_basename_with_digest() {
        let mut extension = ExtensionCatalog::testing().get("dolos").unwrap().clone();
        extension.chart =
            "oci://oci.supernode.store/extensions/dolos@sha256:0123456789abcdef".to_string();
        let document = ExtensionCatalogDocument {
            schema_version: CATALOG_SCHEMA_VERSION.to_string(),
            extensions: vec![extension],
        };

        let catalog = ExtensionCatalog::from_document_with_trust(document, false).unwrap();

        assert!(catalog.get("dolos").is_some());
    }

    #[test]
    fn catalog_json_rejects_untrusted_chart_references_by_default() {
        let mut extension = ExtensionCatalog::testing().get("dolos").unwrap().clone();
        extension.chart = "oci://evil.example/extensions/dolos".to_string();
        let document = ExtensionCatalogDocument {
            schema_version: CATALOG_SCHEMA_VERSION.to_string(),
            extensions: vec![extension],
        };

        let error = ExtensionCatalog::from_document_with_trust(document, false).unwrap_err();

        assert!(matches!(
            error,
            CatalogLoadError::UntrustedChartReference(_)
        ));
    }

    #[test]
    fn catalog_json_allows_untrusted_chart_references_when_explicit() {
        let mut extension = ExtensionCatalog::testing().get("dolos").unwrap().clone();
        extension.chart = "oci://evil.example/extensions/dolos".to_string();
        let document = ExtensionCatalogDocument {
            schema_version: CATALOG_SCHEMA_VERSION.to_string(),
            extensions: vec![extension],
        };

        let catalog = ExtensionCatalog::from_document_with_trust(document, true).unwrap();

        assert!(catalog.get("dolos").is_some());
    }

    #[test]
    fn relay_extension_exposes_domain_contract() {
        let catalog = ExtensionCatalog::testing();
        let extension = catalog.get("cardano-relay").unwrap();

        assert_eq!(extension.name, "Cardano Relay");
        assert_eq!(extension.default_version, "0.1.0");
        assert!(extension.versions.contains(&"0.1.0".to_string()));
        assert_eq!(extension.configuration.get("type"), Some(&json!("object")));
        assert_eq!(extension.metrics.get("type"), Some(&json!("object")));
        assert_eq!(extension.outputs.len(), 2);
        assert!(extension.secrets.is_empty());
        assert!(extension.dependencies.is_empty());
    }

    #[test]
    fn relay_configuration_does_not_expose_power_user_config_override() {
        let catalog = ExtensionCatalog::testing();
        let configuration = &catalog.get("cardano-relay").unwrap().configuration;
        let properties = configuration
            .get("properties")
            .and_then(Value::as_object)
            .unwrap();

        assert!(properties.contains_key("node"));
        assert!(properties.contains_key("service"));
        assert!(properties.contains_key("persistence"));
        assert!(properties.contains_key("resources"));
        assert!(
            configuration
                .pointer("/properties/image/properties/pullPolicy")
                .is_none()
        );
        assert!(
            configuration
                .pointer("/properties/persistence/properties/enabled")
                .is_none()
        );
        assert!(
            configuration
                .pointer("/properties/node/properties/blockProducer")
                .is_none()
        );
        assert!(!properties.contains_key("rawValues"));
    }

    #[test]
    fn block_producer_configuration_exposes_debug_and_relays() {
        let catalog = ExtensionCatalog::testing();
        let configuration = &catalog.get("cardano-block-producer").unwrap().configuration;

        assert_eq!(
            configuration.pointer("/properties/blockProducer/properties/debug/type"),
            Some(&json!("boolean"))
        );
        assert_eq!(
            configuration.pointer("/properties/relays/properties/count/type"),
            Some(&json!("integer"))
        );
        assert!(
            configuration
                .pointer("/properties/node/properties/blockProducer")
                .is_none()
        );
    }

    #[test]
    fn relay_metrics_schema_describes_script_output_fields() {
        let catalog = ExtensionCatalog::testing();
        let metrics = &catalog.get("cardano-relay").unwrap().metrics;
        let properties = metrics
            .get("properties")
            .and_then(Value::as_object)
            .unwrap();
        let required = metrics.get("required").and_then(Value::as_array).unwrap();

        assert!(required.contains(&json!("role")));
        assert!(properties.contains_key("role"));
        assert!(properties.contains_key("epochLength"));
        assert!(properties.contains_key("kesExpirationTime"));
        assert!(properties.contains_key("scheduledLeaderCount"));
        assert!(properties.contains_key("nextLeaderTimeRemainingSeconds"));
    }

    #[test]
    fn extensions_define_metrics_collection_metadata() {
        let catalog = ExtensionCatalog::testing();
        let cases = [
            ("cardano-relay", "cardano-node"),
            ("cardano-block-producer", "cardano-node"),
            ("cardano-db-sync", "postgres"),
            ("apex-fusion-relay", "apex-fusion"),
            ("apex-fusion-block-producer", "apex-fusion"),
            ("dolos", "dolos"),
            ("hydra-node", "hydra-node"),
        ];

        for (extension_id, container) in cases {
            let metrics_collection = catalog
                .get(extension_id)
                .unwrap()
                .metrics_collection
                .as_ref()
                .unwrap();

            assert_eq!(metrics_collection.container, container);
            assert_eq!(
                metrics_collection.command,
                vec!["/opt/metis/bin/metrics.sh"]
            );

            if extension_id == "cardano-db-sync" {
                assert_eq!(
                    metrics_collection.pod_label_selector.as_deref(),
                    Some("app.kubernetes.io/component=postgres")
                );
            } else {
                assert_eq!(metrics_collection.pod_label_selector, None);
            }
        }
    }

    #[test]
    fn cardano_db_sync_extension_exposes_domain_contract() {
        let catalog = ExtensionCatalog::testing();
        let extension = catalog.get("cardano-db-sync").unwrap();

        assert_eq!(extension.name, "Cardano DB Sync");
        assert_eq!(extension.default_version, "0.1.0");
        assert!(extension.versions.contains(&"0.1.0".to_string()));
        assert_eq!(extension.configuration.get("type"), Some(&json!("object")));
        assert_eq!(extension.metrics.get("type"), Some(&json!("object")));
        assert_eq!(extension.outputs.len(), 1);
        assert_eq!(extension.secrets.len(), 1);
        assert!(extension.dependencies.is_empty());
    }

    #[test]
    fn cardano_db_sync_metrics_schema_describes_database_sync_fields() {
        let catalog = ExtensionCatalog::testing();
        let metrics = &catalog.get("cardano-db-sync").unwrap().metrics;
        let properties = metrics
            .get("properties")
            .and_then(Value::as_object)
            .unwrap();
        let required = metrics.get("required").and_then(Value::as_array).unwrap();

        assert!(required.contains(&json!("type")));
        assert!(required.contains(&json!("errors")));
        assert!(properties.contains_key("blockHeight"));
        assert!(properties.contains_key("slotNum"));
        assert!(properties.contains_key("epoch"));
        assert!(properties.contains_key("dbSizeBytes"));
        assert!(properties.contains_key("latestBlockAgeSeconds"));
    }

    #[test]
    fn dolos_extension_exposes_domain_contract() {
        let catalog = ExtensionCatalog::testing();
        let extension = catalog.get("dolos").unwrap();

        assert_eq!(extension.name, "Dolos");
        assert_eq!(extension.default_version, "0.1.0");
        assert!(extension.versions.contains(&"0.1.0".to_string()));
        assert_eq!(extension.configuration.get("type"), Some(&json!("object")));
        assert_eq!(extension.metrics.get("type"), Some(&json!("object")));
        assert_eq!(extension.outputs.len(), 4);
        assert!(extension.secrets.is_empty());
        assert!(extension.dependencies.is_empty());
    }

    #[test]
    fn dolos_configuration_only_exposes_safe_cardano_fields() {
        let catalog = ExtensionCatalog::testing();
        let configuration = &catalog.get("dolos").unwrap().configuration;
        let properties = configuration
            .get("properties")
            .and_then(Value::as_object)
            .unwrap();

        assert!(properties.contains_key("dolos"));
        assert!(properties.contains_key("config"));
        assert!(properties.contains_key("persistence"));
        assert!(properties.contains_key("resources"));
        assert_eq!(
            configuration.pointer("/properties/dolos/properties/network/type"),
            Some(&json!("string"))
        );
        assert_eq!(
            configuration.pointer("/properties/config/properties/upstreamAddress/type"),
            Some(&json!("string"))
        );
        assert_eq!(
            configuration.pointer("/properties/persistence/properties/storageClass/type"),
            Some(&json!("string"))
        );
        assert!(
            configuration
                .pointer("/properties/config/properties/existingConfigMap")
                .is_none()
        );
        assert!(
            configuration
                .pointer("/properties/dolos/properties/env")
                .is_none()
        );
        assert!(
            configuration
                .pointer("/properties/image/properties/pullPolicy")
                .is_none()
        );
        assert!(
            configuration
                .pointer("/properties/persistence/properties/enabled")
                .is_none()
        );
        assert!(
            configuration
                .pointer("/properties/config/properties/presets")
                .is_none()
        );
        assert!(!properties.contains_key("rawValues"));
    }

    #[test]
    fn dolos_metrics_schema_describes_basic_minibf_fields() {
        let catalog = ExtensionCatalog::testing();
        let metrics = &catalog.get("dolos").unwrap().metrics;
        let properties = metrics
            .get("properties")
            .and_then(Value::as_object)
            .unwrap();
        let required = metrics.get("required").and_then(Value::as_array).unwrap();

        assert!(required.contains(&json!("type")));
        assert!(required.contains(&json!("errors")));
        assert!(properties.contains_key("blockHeight"));
        assert!(properties.contains_key("epoch"));
        assert!(properties.contains_key("slotNum"));
    }

    #[test]
    fn hydra_extension_exposes_domain_contract() {
        let catalog = ExtensionCatalog::testing();
        let extension = catalog.get("hydra-node").unwrap();

        assert_eq!(extension.name, "Hydra Node");
        assert_eq!(extension.default_version, "0.2.0");
        assert!(extension.versions.contains(&"0.2.0".to_string()));
        assert_eq!(extension.configuration.get("type"), Some(&json!("object")));
        assert_eq!(extension.metrics.get("type"), Some(&json!("object")));
        assert_eq!(extension.outputs.len(), 4);
        assert_eq!(extension.secrets.len(), 2);
        assert!(extension.dependencies.is_empty());
    }

    #[test]
    fn hydra_extension_describes_runtime_secret_metadata() {
        let catalog = ExtensionCatalog::testing();
        let extension = catalog.get("hydra-node").unwrap();

        let hydra_signing = extension
            .secrets
            .iter()
            .find(|secret| secret.name == "hydraSigningKey")
            .unwrap();

        assert!(hydra_signing.required);
        assert_eq!(hydra_signing.scope, "runtime");
        assert!(hydra_signing.write_only);
        assert!(
            hydra_signing
                .accepted_sources
                .contains(&"vaultStaticSecret".to_string())
        );
        assert_eq!(hydra_signing.accepted_sources.len(), 1);
    }

    #[test]
    fn hydra_metrics_schema_describes_api_and_prometheus_fields() {
        let catalog = ExtensionCatalog::testing();
        let metrics = &catalog.get("hydra-node").unwrap().metrics;
        let properties = metrics
            .get("properties")
            .and_then(Value::as_object)
            .unwrap();

        assert!(properties.contains_key("headStatus"));
        assert!(properties.contains_key("snapshotNumber"));
        assert!(properties.contains_key("peersConnected"));
        assert!(properties.contains_key("txConfirmationTimeMsAvg"));
    }

    #[test]
    fn extension_outputs_describe_exposed_endpoints_for_llms() {
        let catalog = ExtensionCatalog::testing();
        let relay = catalog.get("cardano-relay").unwrap();
        let dolos = catalog.get("dolos").unwrap();
        let hydra = catalog.get("hydra-node").unwrap();

        assert_eq!(relay.outputs[0].name, "n2n");
        assert_eq!(relay.outputs[1].name, "n2c");
        assert!(relay.outputs[0].description.contains("node-to-node"));

        assert_eq!(dolos.outputs[0].name, "trp");
        assert_eq!(dolos.outputs[1].name, "blockfrost");
        assert_eq!(dolos.outputs[2].name, "kupo");
        assert_eq!(dolos.outputs[3].name, "utxorpc");
        assert_eq!(dolos.outputs[3].protocol, "gRPC");

        assert_eq!(hydra.outputs[0].name, "api");
        assert_eq!(hydra.outputs[1].name, "ws");
        assert_eq!(hydra.outputs[1].protocol, "WebSocket");
        assert_eq!(hydra.outputs[2].name, "p2p");
        assert_eq!(hydra.outputs[3].name, "monitoring");
    }
}
