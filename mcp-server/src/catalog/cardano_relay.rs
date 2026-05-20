use serde_json::json;

use super::{
    ExtensionConfiguration, ExtensionDefinition, ExtensionMetrics, ExtensionMetricsCollection,
    ExtensionOutputDefinition,
};
use crate::catalog::schema::{nullable_boolean, nullable_number, nullable_string};

pub(super) fn definition() -> ExtensionDefinition {
    ExtensionDefinition::new(
        "cardano-relay",
        "Cardano Relay",
        "A Cardano relay node workload for participating in Cardano network topology without block-producing keys.",
        vec!["0.1.0"],
        "0.1.0",
        configuration_schema(),
        vec![],
        vec![],
        metrics_schema(),
        outputs(),
        "oci://oci.supernode.store/extensions/cardano-relay".to_string(),
    )
    .with_metrics_collection(ExtensionMetricsCollection::metrics_script("cardano-node"))
}

pub(super) fn outputs() -> Vec<ExtensionOutputDefinition> {
    vec![
        ExtensionOutputDefinition::new(
            "n2n",
            "Cardano node-to-node networking endpoint for relay peer connectivity.",
            "n2n",
            "TCP",
        ),
        ExtensionOutputDefinition::new(
            "n2c",
            "Cardano node-to-client endpoint for local clients through the chart proxy.",
            "n2c",
            "TCP",
        ),
    ]
}

pub(super) fn metrics_schema() -> ExtensionMetrics {
    metrics_schema_for("Cardano Relay Metrics", "relay")
}

pub(super) fn block_producer_metrics_schema() -> ExtensionMetrics {
    metrics_schema_for("Cardano Block Producer Metrics", "block-producer")
}

fn metrics_schema_for(title: &str, role: &str) -> ExtensionMetrics {
    let mut properties = serde_json::Map::new();
    properties.insert("type".to_string(), json!({ "const": "cardano-node" }));
    properties.insert("role".to_string(), json!({ "const": role }));
    properties.insert(
        "blockHeight".to_string(),
        nullable_number("Latest block number observed by the node."),
    );
    properties.insert(
        "epoch".to_string(),
        nullable_number("Current epoch observed by the node."),
    );
    properties.insert(
        "slotNum".to_string(),
        nullable_number("Absolute slot number observed by the node across the chain timeline."),
    );
    properties.insert(
        "slotInEpoch".to_string(),
        nullable_number("Current slot within the active epoch observed by the node."),
    );
    properties.insert(
        "epochProgressPercent".to_string(),
        nullable_number("Percentage of the current epoch completed from slot-in-epoch and Shelley genesis epoch length."),
    );
    properties.insert(
        "epochTimeRemainingSeconds".to_string(),
        nullable_number(
            "Approximate time remaining in the current epoch derived from Shelley genesis timing.",
        ),
    );
    properties.insert(
        "tipRefSlot".to_string(),
        nullable_number(
            "Reference chain tip computed from the Shelley genesis system start and slot length.",
        ),
    );
    properties.insert(
        "tipDiffSlots".to_string(),
        nullable_number("Difference between the computed reference tip and the node tip."),
    );
    properties.insert(
        "syncPercent".to_string(),
        nullable_number("Estimated sync percentage against the computed reference tip."),
    );
    properties.insert(
        "density".to_string(),
        nullable_number("Recent chain density reported by the node, expressed as a percentage."),
    );
    properties.insert(
        "forks".to_string(),
        nullable_number("Number of chain forks the node has observed since startup."),
    );
    properties.insert(
        "txProcessed".to_string(),
        nullable_number("Total transactions processed by the node since startup."),
    );
    properties.insert(
        "pendingTx".to_string(),
        nullable_number("Transactions currently in the mempool."),
    );
    properties.insert(
        "pendingTxBytes".to_string(),
        nullable_number("Buffered mempool transaction size when available."),
    );
    properties.insert(
        "nodeVersion".to_string(),
        nullable_string("Cardano node build version reported by the metrics endpoint."),
    );
    properties.insert(
        "nodeRevision".to_string(),
        nullable_string("Cardano node build revision reported by the metrics endpoint."),
    );
    properties.insert(
        "forgingEnabled".to_string(),
        nullable_boolean("Whether this node currently has forging enabled."),
    );
    properties.insert(
        "peersIncoming".to_string(),
        nullable_number("Active inbound node connections."),
    );
    properties.insert(
        "peersOutgoing".to_string(),
        nullable_number("Active outbound node connections."),
    );
    properties.insert(
        "connectionUniDir".to_string(),
        nullable_number("Current unidirectional connection count."),
    );
    properties.insert(
        "connectionBiDir".to_string(),
        nullable_number("Current bidirectional connection count."),
    );
    properties.insert(
        "connectionDuplex".to_string(),
        nullable_number("Current full duplex connection count."),
    );
    properties.insert(
        "inboundGovernorWarm".to_string(),
        nullable_number("Inbound governor warm connection count reported by the node."),
    );
    properties.insert(
        "inboundGovernorHot".to_string(),
        nullable_number("Inbound governor hot connection count reported by the node."),
    );
    properties.insert(
        "peerSelectionCold".to_string(),
        nullable_number("Peer selection cold state count for outbound connections."),
    );
    properties.insert(
        "peerSelectionWarm".to_string(),
        nullable_number("Peer selection warm state count for outbound connections."),
    );
    properties.insert(
        "peerSelectionHot".to_string(),
        nullable_number("Peer selection hot state count for outbound connections."),
    );
    properties.insert(
        "lastBlockDelaySeconds".to_string(),
        nullable_number("Latest observed block propagation delay."),
    );
    properties.insert(
        "blocksServed".to_string(),
        nullable_number("Blocks served to peers by this node since startup."),
    );
    properties.insert(
        "blocksLate".to_string(),
        nullable_number("Blocks observed later than five seconds by the block fetch client."),
    );
    properties.insert(
        "blocksWithin1s".to_string(),
        nullable_number("Percentage of observed blocks arriving within 1 second."),
    );
    properties.insert(
        "blocksWithin3s".to_string(),
        nullable_number("Percentage of observed blocks arriving within 3 seconds."),
    );
    properties.insert(
        "blocksWithin5s".to_string(),
        nullable_number("Percentage of observed blocks arriving within 5 seconds."),
    );
    properties.insert(
        "memLiveBytes".to_string(),
        nullable_number("Live RTS memory currently retained by the node process."),
    );
    properties.insert(
        "memHeapBytes".to_string(),
        nullable_number("Heap memory currently reserved by the node RTS."),
    );
    properties.insert(
        "gcMinorCount".to_string(),
        nullable_number("Number of minor garbage collections since startup."),
    );
    properties.insert(
        "gcMajorCount".to_string(),
        nullable_number("Number of major garbage collections since startup."),
    );
    properties.insert(
        "epochLength".to_string(),
        nullable_number("Number of slots in the current Cardano epoch from Shelley genesis."),
    );
    properties.insert(
        "slotLength".to_string(),
        nullable_number("Slot duration in seconds from Shelley genesis."),
    );
    properties.insert(
        "systemStartUnix".to_string(),
        nullable_number("Shelley system start timestamp as Unix seconds."),
    );
    properties.insert(
        "kesPeriod".to_string(),
        nullable_number("Current KES period reported by the node, when available."),
    );
    properties.insert(
        "kesRemaining".to_string(),
        nullable_number("Remaining KES periods before key expiry, when available."),
    );
    properties.insert(
        "kesExpirationSeconds".to_string(),
        nullable_number("Approximate seconds until KES key expiry, when available."),
    );
    properties.insert(
        "kesExpirationTime".to_string(),
        nullable_string("Estimated KES key expiry time as an ISO-8601 timestamp, when available."),
    );
    properties.insert(
        "opCertOnDisk".to_string(),
        nullable_number("Operational certificate counter found on disk, when available."),
    );
    properties.insert(
        "opCertOnChain".to_string(),
        nullable_number("Operational certificate counter observed on chain, when available."),
    );
    properties.insert(
        "leaderCount".to_string(),
        nullable_number(
            "Slots where the node was leader since startup, for block-producing nodes.",
        ),
    );
    properties.insert(
        "adoptedCount".to_string(),
        nullable_number(
            "Forged blocks adopted by the chain since startup, for block-producing nodes.",
        ),
    );
    properties.insert(
        "forgedCount".to_string(),
        nullable_number("Blocks forged by the node since startup, for block-producing nodes."),
    );
    properties.insert(
        "aboutToLeadCount".to_string(),
        nullable_number(
            "Times the node was about to lead a slot since startup, for block-producing nodes.",
        ),
    );
    properties.insert(
        "invalidCount".to_string(),
        nullable_number("Derived count of forged blocks that were not adopted, clamped at zero."),
    );
    properties.insert(
        "missedSlots".to_string(),
        nullable_number("Slots missed by the node since startup, when available."),
    );
    properties.insert(
        "scheduledLeaderCount".to_string(),
        nullable_number(
            "Leadership slots scheduled for the current epoch, for block-producing nodes.",
        ),
    );
    properties.insert(
        "scheduledIdealCount".to_string(),
        nullable_number("Expected leadership slots for the current epoch based on active stake."),
    );
    properties.insert(
        "scheduledLuckPercent".to_string(),
        nullable_number("Scheduled leader slots as a percentage of ideal expected slots."),
    );
    properties.insert(
        "nextLeaderSlot".to_string(),
        nullable_number("Next scheduled leadership slot number, for block-producing nodes."),
    );
    properties.insert(
        "nextLeaderTime".to_string(),
        nullable_string("Next scheduled leadership slot time as an ISO-8601 timestamp."),
    );
    properties.insert(
        "nextLeaderTimeRemainingSeconds".to_string(),
        nullable_number("Approximate seconds until the next scheduled leadership slot."),
    );
    properties.insert(
        "errors".to_string(),
        json!({
            "type": "array",
            "description": "Warnings or collection errors emitted by the metrics script.",
            "items": { "type": "string" }
        }),
    );

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": title,
        "type": "object",
        "required": ["type", "role", "errors"],
        "properties": properties,
        "additionalProperties": false
    })
}

fn configuration_schema() -> ExtensionConfiguration {
    serde_json::from_str(include_str!(
        "../../../extensions/cardano-relay/values.schema.json"
    ))
    .expect("embedded Cardano relay values schema must be valid JSON")
}
