use serde_json::json;

use super::{
    ExtensionConfiguration, ExtensionDefinition, ExtensionMetrics, ExtensionMetricsCollection,
    ExtensionOutputDefinition, ExtensionSecretDefinition,
};
use crate::catalog::schema::{nullable_number, nullable_string};

pub(super) fn definition() -> ExtensionDefinition {
    ExtensionDefinition::new(
        "hydra-node",
        "Hydra Node",
        "A Hydra Head protocol node for operating Cardano L2 state channels with low-latency off-chain transactions and L1 settlement.",
        vec!["0.2.0"],
        "0.2.0",
        configuration_schema(),
        secrets(),
        vec![],
        metrics_schema(),
        outputs(),
        "oci://oci.supernode.store/extensions/hydra-node".to_string(),
    )
    .with_metrics_collection(ExtensionMetricsCollection::metrics_script("hydra-node"))
}

fn configuration_schema() -> ExtensionConfiguration {
    serde_json::from_str(include_str!(
        "../../../extensions/hydra-node/values.schema.json"
    ))
    .expect("embedded Hydra node values schema must be valid JSON")
}

fn secrets() -> Vec<ExtensionSecretDefinition> {
    vec![
        ExtensionSecretDefinition::new(
            "hydraSigningKey",
            "Hydra Ed25519 signing key used by this node to sign snapshots. Values must be supplied through runtime Vault sync and are never echoed by MCP.",
            true,
            None,
            "runtime",
            "hydra-signing-key",
            true,
            vec!["vaultStaticSecret"],
        ),
        ExtensionSecretDefinition::new(
            "cardanoSigningKey",
            "Cardano signing key used by online Hydra nodes to pay L1 fuel and drive head lifecycle transactions.",
            false,
            Some("keys.cardano.enabled == true"),
            "runtime",
            "cardano-signing-key",
            true,
            vec!["vaultStaticSecret"],
        ),
    ]
}

fn outputs() -> Vec<ExtensionOutputDefinition> {
    vec![
        ExtensionOutputDefinition::new("api", "Hydra HTTP API endpoint.", "api", "HTTP"),
        ExtensionOutputDefinition::new(
            "ws",
            "Hydra WebSocket client-input and server-output endpoint.",
            "api",
            "WebSocket",
        ),
        ExtensionOutputDefinition::new(
            "p2p",
            "Hydra node-to-node peer networking endpoint.",
            "p2p",
            "TCP",
        ),
        ExtensionOutputDefinition::new(
            "monitoring",
            "Hydra Prometheus metrics endpoint.",
            "monitoring",
            "HTTP",
        ),
    ]
}

fn metrics_schema() -> ExtensionMetrics {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Hydra Node Metrics",
        "type": "object",
        "required": ["type", "errors"],
        "properties": {
            "type": { "const": "hydra-node" },
            "mode": nullable_string("Hydra node mode as configured by the chart: offline or online."),
            "headStatus": nullable_string("Latest head state tag reported by the Hydra HTTP API."),
            "headId": nullable_string("Current Hydra head identifier when a head is known."),
            "hydraNodeVersion": nullable_string("Hydra node version reported by the HTTP API."),
            "currentSlot": nullable_number("Current chain slot reported by the Hydra API when available."),
            "chainSyncedStatus": nullable_string("Hydra chain sync status when connected to L1."),
            "peersConnected": nullable_number("Connected Hydra peers from the Prometheus metrics endpoint."),
            "pendingDeposits": nullable_number("Number of pending deposit transaction IDs returned by /commits."),
            "snapshotNumber": nullable_number("Latest confirmed snapshot number when available."),
            "snapshotVersion": nullable_number("Latest confirmed snapshot version when available."),
            "confirmedUtxoCount": nullable_number("Number of entries in the latest confirmed snapshot UTxO."),
            "confirmedLovelace": nullable_number("Sum of lovelace in the latest confirmed snapshot UTxO."),
            "lastSeenSnapshotTag": nullable_string("Tag returned by /snapshot/last-seen for diagnosing in-flight snapshot consensus."),
            "requestedTx": nullable_number("Total requested L2 transactions from hydra_head_requested_tx."),
            "confirmedTx": nullable_number("Total confirmed L2 transactions from hydra_head_confirmed_tx."),
            "inputs": nullable_number("Total processed head inputs from hydra_head_inputs."),
            "txConfirmationTimeMsCount": nullable_number("Confirmation-time histogram sample count."),
            "txConfirmationTimeMsSum": nullable_number("Confirmation-time histogram sample sum in milliseconds."),
            "txConfirmationTimeMsAvg": nullable_number("Derived average confirmation time in milliseconds."),
            "errors": {
                "type": "array",
                "description": "Warnings or collection errors emitted by the metrics script.",
                "items": { "type": "string" }
            }
        },
        "additionalProperties": false
    })
}
