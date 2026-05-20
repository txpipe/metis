use serde_json::json;

use super::{
    ExtensionConfiguration, ExtensionDefinition, ExtensionMetrics, ExtensionMetricsCollection,
    ExtensionOutputDefinition,
};
use crate::catalog::schema::nullable_number;

pub(super) fn definition() -> ExtensionDefinition {
    ExtensionDefinition::new(
        "apex-fusion-relay",
        "Apex Fusion Relay",
        "An Apex Fusion relay node workload for participating in Vector and Prime network topology without block-producing keys.",
        vec!["0.1.0"],
        "0.1.0",
        configuration_schema(),
        vec![],
        vec![],
        metrics_schema(),
        outputs(),
        "oci://oci.supernode.store/extensions/apex-fusion-relay".to_string(),
    )
    .with_metrics_collection(ExtensionMetricsCollection::metrics_script("apex-fusion"))
}

fn configuration_schema() -> ExtensionConfiguration {
    serde_json::from_str(include_str!(
        "../../../extensions/apex-fusion-relay/values.schema.json"
    ))
    .expect("embedded Apex Fusion relay values schema must be valid JSON")
}

pub(super) fn outputs() -> Vec<ExtensionOutputDefinition> {
    vec![
        ExtensionOutputDefinition::new(
            "n2n",
            "Apex Fusion node-to-node networking endpoint for relay peer connectivity.",
            "n2n",
            "TCP",
        ),
        ExtensionOutputDefinition::new(
            "n2c",
            "Apex Fusion node-to-client endpoint for local clients through the chart proxy.",
            "n2c",
            "TCP",
        ),
    ]
}

pub(super) fn metrics_schema() -> ExtensionMetrics {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Apex Fusion Relay Metrics",
        "type": "object",
        "required": ["type", "errors"],
        "properties": {
            "type": { "const": "apex-fusion" },
            "blockHeight": nullable_number("Latest block number observed by the node."),
            "epoch": nullable_number("Current epoch observed by the node."),
            "slotNum": nullable_number("Absolute slot number observed by the node."),
            "peersIncoming": nullable_number("Incoming peer connection count."),
            "peersOutgoing": nullable_number("Outgoing peer connection count."),
            "errors": { "type": "array", "items": { "type": "string" } }
        },
        "additionalProperties": true
    })
}
