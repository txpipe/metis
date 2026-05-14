use serde_json::json;

use super::{
    ExtensionConfiguration, ExtensionDefinition, ExtensionMetrics, ExtensionOutputDefinition,
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
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Dolos Configuration",
        "type": "object",
        "required": ["network", "namespace", "storageClass"],
        "properties": {
            "network": {
                "type": "string",
                "description": "Cardano network Dolos should index.",
                "enum": ["cardano-mainnet", "cardano-preprod", "cardano-preview"],
                "default": "cardano-preview"
            },
            "namespace": {
                "type": "string",
                "description": "Kubernetes namespace where the Dolos workload will be installed.",
                "minLength": 1
            },
            "storageClass": {
                "type": "string",
                "description": "StorageClass used for the Dolos data PVC.",
                "minLength": 1
            },
            "upstreamAddress": {
                "type": "string",
                "description": "Trusted Cardano relay address for Dolos sync. If omitted, MCP will try to discover a same-network relay already installed in the cluster.",
                "minLength": 1
            },
            "exposeLoadBalancer": {
                "type": "boolean",
                "description": "Expose the Dolos service as a Kubernetes LoadBalancer instead of ClusterIP.",
                "default": false
            },
            "imageTag": {
                "type": "string",
                "description": "Dolos image tag.",
                "default": "v1.1.1"
            },
            "resources": {
                "type": "object",
                "description": "Kubernetes resource requests and limits for the Dolos container.",
                "properties": {
                    "requests": { "$ref": "#/$defs/resourceList" },
                    "limits": { "$ref": "#/$defs/resourceList" }
                },
                "additionalProperties": false
            },
            "pvcSize": {
                "type": "string",
                "description": "Requested size for the Dolos data PVC.",
                "pattern": "^[0-9]+(Mi|Gi|Ti)$"
            }
        },
        "$defs": {
            "resourceList": {
                "type": "object",
                "properties": {
                    "cpu": { "type": "string", "minLength": 1 },
                    "memory": { "type": "string", "minLength": 1 }
                },
                "additionalProperties": false
            }
        },
        "additionalProperties": false
    })
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
