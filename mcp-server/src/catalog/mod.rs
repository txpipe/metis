mod apex_fusion_block_producer;
mod apex_fusion_relay;
mod cardano_block_producer;
mod cardano_relay;
mod dolos;
pub mod extension;
mod hydra_node;
mod schema;

use std::collections::BTreeMap;

use serde::Serialize;

pub use extension::ExtensionConfiguration;
pub use extension::ExtensionDefinition;
pub use extension::ExtensionId;
pub use extension::ExtensionMetrics;
pub use extension::ExtensionMetricsCollection;
pub use extension::ExtensionOutputDefinition;
pub use extension::ExtensionSecretDefinition;

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionCatalog {
    extensions: BTreeMap<ExtensionId, ExtensionDefinition>,
}

impl ExtensionCatalog {
    pub fn embedded() -> Self {
        Self::from_extensions([
            apex_fusion_block_producer::definition(),
            apex_fusion_relay::definition(),
            cardano_block_producer::definition(),
            cardano_relay::definition(),
            dolos::definition(),
            hydra_node::definition(),
        ])
    }

    pub fn from_extensions(extensions: impl IntoIterator<Item = ExtensionDefinition>) -> Self {
        let extensions = extensions
            .into_iter()
            .map(|extension| (extension.id.clone(), extension))
            .collect();
        Self { extensions }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn embedded_catalog_contains_cardano_extensions_dolos_and_hydra() {
        let catalog = ExtensionCatalog::embedded();

        assert_eq!(catalog.len(), 6);
        assert!(catalog.get("apex-fusion-relay").is_some());
        assert!(catalog.get("apex-fusion-block-producer").is_some());
        assert!(catalog.get("cardano-relay").is_some());
        assert!(catalog.get("cardano-block-producer").is_some());
        assert!(catalog.get("cardano-node").is_none());
        assert!(catalog.get("dolos").is_some());
        assert!(catalog.get("hydra-node").is_some());
    }

    #[test]
    fn relay_extension_exposes_domain_contract() {
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
        let cases = [
            ("cardano-relay", "cardano-node"),
            ("cardano-block-producer", "cardano-node"),
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
        }
    }

    #[test]
    fn dolos_extension_exposes_domain_contract() {
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
        let catalog = ExtensionCatalog::embedded();
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
