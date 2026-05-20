use super::{
    ExtensionConfiguration, ExtensionDefinition, ExtensionMetricsCollection,
    ExtensionOutputDefinition, ExtensionSecretDefinition,
};

pub(super) fn definition() -> ExtensionDefinition {
    ExtensionDefinition::new(
        "cardano-block-producer",
        "Cardano Block Producer",
        "A private Cardano block producer workload with optional managed relays and Vault-synced runtime producer material.",
        vec!["0.1.0"],
        "0.1.0",
        configuration_schema(),
        secrets(),
        vec![],
        super::cardano_relay::block_producer_metrics_schema(),
        outputs(),
        "oci://oci.supernode.store/extensions/cardano-block-producer".to_string(),
    )
    .with_metrics_collection(ExtensionMetricsCollection::metrics_script("cardano-node"))
}

fn configuration_schema() -> ExtensionConfiguration {
    serde_json::from_str(include_str!(
        "../../../extensions/cardano-block-producer/values.schema.json"
    ))
    .expect("embedded Cardano block producer values schema must be valid JSON")
}

fn secrets() -> Vec<ExtensionSecretDefinition> {
    vec![ExtensionSecretDefinition::new(
        "blockProducerRuntime",
        "Runtime producer material synced from Vault: kes.skey, vrf.skey, and op.cert. Cold keys and counters must not be mounted into the producer pod.",
        true,
        None,
        "runtime",
        "cardano-block-producer-runtime",
        true,
        vec!["vaultStaticSecret"],
    )]
}

fn outputs() -> Vec<ExtensionOutputDefinition> {
    super::cardano_relay::outputs()
}
