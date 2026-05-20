use super::{
    ExtensionConfiguration, ExtensionDefinition, ExtensionMetricsCollection,
    ExtensionOutputDefinition, ExtensionSecretDefinition,
};

pub(super) fn definition() -> ExtensionDefinition {
    ExtensionDefinition::new(
        "apex-fusion-block-producer",
        "Apex Fusion Block Producer",
        "A private Apex Fusion block producer workload with optional managed relays and Vault-synced runtime producer material.",
        vec!["0.1.0"],
        "0.1.0",
        configuration_schema(),
        secrets(),
        vec![],
        super::apex_fusion_relay::metrics_schema(),
        outputs(),
        "oci://oci.supernode.store/extensions/apex-fusion-block-producer".to_string(),
    )
    .with_metrics_collection(ExtensionMetricsCollection::metrics_script("apex-fusion"))
}

fn configuration_schema() -> ExtensionConfiguration {
    serde_json::from_str(include_str!(
        "../../../extensions/apex-fusion-block-producer/values.schema.json"
    ))
    .expect("embedded Apex Fusion block producer values schema must be valid JSON")
}

fn secrets() -> Vec<ExtensionSecretDefinition> {
    vec![ExtensionSecretDefinition::new(
        "blockProducerRuntime",
        "Runtime Apex Fusion producer material synced from Vault: kes.skey, vrf.skey, and op.cert. Cold keys and counters must not be mounted into the producer pod.",
        true,
        None,
        "runtime",
        "apex-fusion-block-producer-runtime",
        true,
        vec!["vaultStaticSecret"],
    )]
}

fn outputs() -> Vec<ExtensionOutputDefinition> {
    super::apex_fusion_relay::outputs()
}
