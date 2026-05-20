use serde_json::json;

use super::{
    ExtensionConfiguration, ExtensionDefinition, ExtensionMetrics, ExtensionMetricsCollection,
    ExtensionOutputDefinition,
};
use crate::catalog::schema::nullable_number;

pub(super) fn definition() -> ExtensionDefinition {
    ExtensionDefinition::new(
        "dolos",
        "Dolos",
        "A Dolos chain data service workload for serving Cardano chain data APIs from the supernode cluster.",
        vec!["0.1.0"],
        "0.1.0",
        configuration_schema(),
        vec![],
        vec![],
        metrics_schema(),
        outputs(),
        "oci://oci.supernode.store/extensions/dolos".to_string(),
    )
    .with_metrics_collection(ExtensionMetricsCollection::metrics_script("dolos"))
}

fn outputs() -> Vec<ExtensionOutputDefinition> {
    vec![
        ExtensionOutputDefinition::new("trp", "Dolos TRP HTTP endpoint.", "trp", "HTTP"),
        ExtensionOutputDefinition::new(
            "blockfrost",
            "Blockfrost-compatible minibf HTTP endpoint.",
            "minibf",
            "HTTP",
        ),
        ExtensionOutputDefinition::new(
            "kupo",
            "Kupo-compatible minikupo HTTP endpoint.",
            "minikupo",
            "HTTP",
        ),
        ExtensionOutputDefinition::new("utxorpc", "UTxO RPC gRPC endpoint.", "grpc", "gRPC"),
    ]
}

fn configuration_schema() -> ExtensionConfiguration {
    serde_json::from_str(include_str!("../../../extensions/dolos/values.schema.json"))
        .expect("embedded Dolos values schema must be valid JSON")
}

fn metrics_schema() -> ExtensionMetrics {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Dolos Metrics",
        "type": "object",
        "required": ["type", "errors"],
        "properties": {
            "type": { "const": "dolos" },
            "blockHeight": nullable_number("Latest block height served by Dolos."),
            "epoch": nullable_number("Current epoch served by Dolos."),
            "slotNum": nullable_number("Latest block slot served by Dolos."),
            "errors": {
                "type": "array",
                "description": "Warnings or collection errors emitted by the metrics script.",
                "items": { "type": "string" }
            }
        },
        "additionalProperties": false
    })
}
