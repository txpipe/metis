mod cardano_node_relay;
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
pub use extension::ExtensionOutputDefinition;
pub use extension::ExtensionSecretDefinition;

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionCatalog {
    extensions: BTreeMap<ExtensionId, ExtensionDefinition>,
}

impl ExtensionCatalog {
    pub fn embedded() -> Self {
        Self::from_extensions([
            cardano_node_relay::definition(),
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
    fn embedded_catalog_contains_cardano_node_relay_dolos_and_hydra() {
        let catalog = ExtensionCatalog::embedded();

        assert_eq!(catalog.len(), 3);
        assert!(catalog.get("cardano-node-relay").is_some());
        assert!(catalog.get("cardano-node").is_none());
        assert!(catalog.get("dolos").is_some());
        assert!(catalog.get("hydra-node").is_some());
    }

    #[test]
    fn relay_extension_exposes_domain_contract() {
        let catalog = ExtensionCatalog::embedded();
        let extension = catalog.get("cardano-node-relay").unwrap();

        assert_eq!(extension.name, "Cardano Node Relay");
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
        let properties = catalog
            .get("cardano-node-relay")
            .unwrap()
            .configuration
            .get("properties")
            .and_then(Value::as_object)
            .unwrap();

        assert!(properties.contains_key("topology"));
        assert!(properties.contains_key("exposeLoadBalancer"));
        assert!(properties.contains_key("imageTag"));
        assert!(properties.contains_key("resources"));
        assert!(properties.contains_key("pvcSize"));
        assert!(!properties.contains_key("config"));
    }

    #[test]
    fn relay_metrics_schema_describes_script_output_fields() {
        let catalog = ExtensionCatalog::embedded();
        let metrics = &catalog.get("cardano-node-relay").unwrap().metrics;
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
        let properties = catalog
            .get("dolos")
            .unwrap()
            .configuration
            .get("properties")
            .and_then(Value::as_object)
            .unwrap();

        assert!(properties.contains_key("network"));
        assert!(properties.contains_key("storageClass"));
        assert!(properties.contains_key("upstreamAddress"));
        assert!(properties.contains_key("imageTag"));
        assert!(properties.contains_key("resources"));
        assert!(properties.contains_key("pvcSize"));
        assert!(!properties.contains_key("bootstrapEnabled"));
        assert!(!properties.contains_key("config"));
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
        assert_eq!(extension.secrets.len(), 4);
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
        let relay = catalog.get("cardano-node-relay").unwrap();
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
