use serde_json::json;

use super::{
    ExtensionConfiguration, ExtensionDefinition, ExtensionMetrics, ExtensionOutputDefinition,
    ExtensionSecretDefinition,
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
            Some("mode == online"),
            "runtime",
            "cardano-signing-key",
            true,
            vec!["vaultStaticSecret"],
        ),
        ExtensionSecretDefinition::new(
            "blockfrostProjectId",
            "Optional Blockfrost project identifier if a future chart profile uses Blockfrost instead of a Cardano node socket.",
            false,
            Some("cardanoBackend.mode == blockfrost"),
            "runtime",
            "blockfrost-project-id",
            true,
            vec!["vaultStaticSecret"],
        ),
        ExtensionSecretDefinition::new(
            "tlsKey",
            "Optional Hydra API TLS private key when TLS is enabled for the unauthenticated Hydra API endpoint.",
            false,
            Some("api.tls == true"),
            "runtime",
            "tls-private-key",
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

fn configuration_schema() -> ExtensionConfiguration {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Hydra Node Configuration",
        "type": "object",
        "required": ["namespace", "storageClass", "mode", "hydraSigningKey", "hydraVerificationKeys"],
        "properties": {
            "namespace": {
                "type": "string",
                "description": "Kubernetes namespace where the Hydra node workload will be installed.",
                "minLength": 1
            },
            "storageClass": {
                "type": "string",
                "description": "StorageClass used for the Hydra persistence PVC.",
                "minLength": 1
            },
            "mode": {
                "type": "string",
                "description": "Run without L1 connectivity for local experiments, or online against Cardano.",
                "enum": ["offline", "online"],
                "default": "offline"
            },
            "network": {
                "type": "string",
                "description": "Cardano network for online mode.",
                "enum": ["mainnet", "preprod", "preview"]
            },
            "nodeId": {
                "type": "string",
                "description": "Unique Hydra node identifier. Mirror nodes must use unique node IDs.",
                "default": "hydra-node-1",
                "minLength": 1
            },
            "imageTag": {
                "type": "string",
                "description": "Hydra node image tag.",
                "default": "2.1.0"
            },
            "pvcSize": {
                "type": "string",
                "description": "Requested size for Hydra persistence.",
                "default": "5Gi",
                "pattern": "^[0-9]+(Mi|Gi|Ti)$"
            },
            "exposeLoadBalancer": {
                "type": "boolean",
                "description": "Expose the Hydra API, P2P, and monitoring service as a LoadBalancer instead of ClusterIP.",
                "default": false
            },
            "contestationPeriod": {
                "type": "string",
                "description": "Hydra contestation period, for example 43200s. Mainnet should use at least 12 hours.",
                "default": "43200s"
            },
            "depositPeriod": {
                "type": "string",
                "description": "Optional deposit period used for incremental commits, for example 7200s."
            },
            "unsyncedPeriod": {
                "type": "integer",
                "description": "Optional seconds after which the node considers itself out of sync with L1.",
                "minimum": 1
            },
            "peers": {
                "type": "array",
                "description": "Static Hydra peer endpoints. All participants must agree on topology.",
                "items": {
                    "type": "object",
                    "required": ["host", "port"],
                    "properties": {
                        "host": { "type": "string", "minLength": 1 },
                        "port": { "type": "integer", "minimum": 1, "maximum": 65535 }
                    },
                    "additionalProperties": false
                },
                "default": []
            },
            "offline": {
                "type": "object",
                "description": "Offline Hydra head parameters used for local or CI experiments without Cardano L1.",
                "properties": {
                    "headSeed": {
                        "type": "string",
                        "description": "Hexadecimal offline head seed shared by participants.",
                        "pattern": "^[0-9a-fA-F]+$",
                        "default": "0001"
                    },
                    "initialUtxo": { "type": "object", "description": "Initial Hydra UTxO JSON." },
                    "protocolParameters": { "type": "object", "description": "Hydra ledger protocol parameters JSON." },
                    "ledgerGenesis": { "type": "object", "description": "Optional Shelley genesis JSON for offline time semantics." }
                },
                "additionalProperties": false,
                "default": { "headSeed": "0001" }
            },
            "cardanoBackend": {
                "type": "object",
                "description": "Online Cardano backend configuration.",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["autoRelay", "socketProxy", "mountedSocket", "blockfrost"]
                    },
                    "upstreamAddress": { "type": "string", "minLength": 1 },
                    "socketPath": { "type": "string", "default": "/ipc/node.socket" },
                    "blockfrostProjectId": { "$ref": "#/$defs/secretRef" },
                    "startChainFrom": { "type": "string" },
                    "hydraScriptsTxId": { "type": "string", "minLength": 1 }
                },
                "additionalProperties": false
            },
            "hydraSigningKey": { "$ref": "#/$defs/secretRef" },
            "hydraVerificationKeys": {
                "type": "array",
                "description": "Hydra verification key files for all parties, including this node.",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "required": ["filename"],
                    "properties": {
                        "filename": { "type": "string", "minLength": 1 },
                        "value": { "type": "string", "description": "Public verification key payload. This is not secret material." }
                    },
                    "additionalProperties": false
                }
            },
            "cardanoSigningKey": { "$ref": "#/$defs/secretRef" },
            "cardanoVerificationKey": {
                "type": "object",
                "description": "Cardano verification key for this online Hydra participant.",
                "properties": {
                    "filename": { "type": "string", "default": "cardano.vk" },
                    "value": { "type": "string", "description": "Public verification key payload. This is not secret material." },
                    "existingConfigMap": { "type": "string", "minLength": 1 }
                },
                "additionalProperties": false
            },
            "resources": {
                "type": "object",
                "description": "Kubernetes resource requests and limits for the Hydra node container.",
                "properties": {
                    "requests": { "$ref": "#/$defs/resourceList" },
                    "limits": { "$ref": "#/$defs/resourceList" }
                },
                "additionalProperties": false
            }
        },
        "$defs": {
            "secretRef": {
                "type": "object",
                "description": "Reference to pre-staged runtime secret material. Secret values are not accepted in catalog-driven lifecycle inputs.",
                "required": ["source"],
                "properties": {
                    "source": { "type": "string", "enum": ["vaultStaticSecret"] },
                    "vaultPath": { "type": "string", "pattern": "^runtime/", "description": "Runtime Vault path without kv/data prefix." },
                    "key": { "type": "string", "description": "Secret key expected in the Vault record and synced Kubernetes Secret.", "minLength": 1 }
                },
                "additionalProperties": false
            },
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
