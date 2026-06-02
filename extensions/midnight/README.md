# Midnight Helm Chart

This chart deploys a Supernode-opinionated Midnight node on Kubernetes.

The chart follows the current Midnight node documentation: Midnight nodes are
deployed separately from Cardano DB Sync and consume an externally provided
Cardano DB Sync PostgreSQL endpoint. Use the `cardano-db-sync`
extension when you want Supernode to manage that dependency.

## Features

- Single-replica StatefulSet with persistent storage for Midnight chain data.
- Opinionated RPC node defaults for RPC, WebSocket, P2P, and Prometheus access.
- External Cardano DB Sync connection derived from a workload reference plus VaultStaticSecret credentials.
- Stable node key from VaultStaticSecret.
- Fixed ClusterIP Service for RPC, P2P, and metrics ports.
- PodMonitor for Prometheus scraping.
- `values.schema.json` as the public Helm, MCP, and LLM configuration contract.

## Current Midnight Versions

The Midnight docs compatibility matrix lists these current node versions:

| Network | Node version |
|---------|--------------|
| Preview | `0.22.5` |
| Preprod | `0.22.2` |
| Mainnet | `0.22.1` |

The chart defaults to Preview and image tag `0.22.5`. Pin `image.tag` when
deploying Preprod or Mainnet.

## Minimal Configuration

```yaml
node:
  network: preview
persistence:
  storageClass: <storage-class>
dbSync:
  workload:
    releaseName: <cardano-db-sync-release>
    namespace: <cardano-db-sync-namespace>
  vaultStaticSecret:
    path: runtime/midnight/preview/dbsync
nodeKey:
  vaultStaticSecret:
    path: runtime/midnight/preview/node-key
```

The DB Sync Vault record must contain `username`, `password`, and `database`
keys. The chart derives a PostgreSQL libpq connection string locally from
those credentials and the referenced `cardano-db-sync` workload endpoint. Set
`dbSync.workload.postgresServiceName` only when the DB Sync workload uses
custom `nameOverride` or `fullnameOverride` values. The node key Vault record
must contain a `node.key` key.

## External DB Sync

Midnight requires Cardano DB Sync to be fully synchronized. This chart does not
deploy DB Sync directly. Point `dbSync.workload` at the approved
`cardano-db-sync` workload, and point `dbSync.vaultStaticSecret.path` at a Vault
record containing the approved DB Sync PostgreSQL `username`, `password`, and
`database` values. The chart derives the PostgreSQL libpq connection string
locally from those credentials and the referenced workload endpoint.

If the referenced DB Sync workload customizes resource naming, set
`dbSync.workload.postgresServiceName` to the actual PostgreSQL Service name.
Otherwise Midnight uses the standard
`<releaseName>-cardano-db-sync-postgres` convention.

For MCP workflows, inspect installed workloads with `workloads.list`, find the
approved `cardano-db-sync` workload for the same Cardano/Midnight environment,
then use its release name and namespace in `dbSync.workload` and the matching
runtime Vault path for the DB Sync credentials.

## Bootnodes

The default values include the documented Midnight Preview bootnodes. For
Preprod use the bootnodes from the Midnight node operator docs:

```yaml
node:
  network: preprod
  bootnodes:
    - /dns/bootnode-1.preprod.midnight.network/tcp/30333/ws/p2p/12D3KooWQxxUgq7ndPfAaCFNbAxtcKYxrAzTxDfRGNktF75SxdX5
    - /dns/bootnode-2.preprod.midnight.network/tcp/30333/ws/p2p/12D3KooWNrUBs22FfmgjqFMa9ZqKED2jnxwsXWw5E4q2XVwN35TJ
```

Mainnet bootnodes were not published in the node operator docs at the time this
chart was updated.

## Testing

```shell
helm lint . -f ci/values-vault-static-secret.yaml

helm template midnight . -f ci/values-vault-static-secret.yaml
```

## Key Values

| Value | Description | Default |
|-------|-------------|---------|
| `image.tag` | Midnight node image tag | `0.22.5` |
| `node.network` | Midnight environment represented by the workload | `preview` |
| `node.cfgPreset` | `CFG_PRESET` passed to the image entrypoint | `testnet-02` |
| `node.bootnodes` | Bootnode multiaddresses | Preview bootnodes |
| `dbSync.workload.releaseName` | Helm release name of the `cardano-db-sync` workload | empty |
| `dbSync.workload.namespace` | Namespace of the `cardano-db-sync` workload | empty |
| `dbSync.workload.postgresServiceName` | Optional override for customized DB Sync PostgreSQL Service names | empty |
| `dbSync.workload.postgresPort` | PostgreSQL Service port on the `cardano-db-sync` workload | `5432` |
| `dbSync.vaultStaticSecret.path` | Vault path containing DB Sync `username`/`password`/`database` | empty |
| `nodeKey.vaultStaticSecret.path` | Vault path containing Midnight node key | empty |
| `persistence.storageClass` | StorageClass for chain data | empty |
| `service.*Port` | Fixed ClusterIP Service ports | `9944`, `30333`, `9615` |

Consult `values.schema.json` for the full public configuration contract.
