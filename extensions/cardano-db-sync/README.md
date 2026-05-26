# Cardano DB Sync Helm Chart

This chart deploys Cardano DB Sync with a chart-managed PostgreSQL database. It
is intended as reusable Supernode infrastructure for Midnight and other
Cardano/partner-chain workloads that need a DB Sync PostgreSQL source.

## Features

- PostgreSQL StatefulSet with persistent storage.
- Cardano DB Sync StatefulSet with persistent state storage.
- Configurable Cardano network: `preview`, `preprod`, or `mainnet`.
- Opinionated Cardano node connectivity through a built-in `socat` socket proxy
  to an approved Cardano relay `host:port`.
- Vault-only credentials. VaultStaticSecret creates the internal Kubernetes
  Secret consumed by Postgres, DB Sync, and downstream Midnight workloads.
- `values.schema.json` as the public Helm, MCP, and LLM configuration contract.

## Minimal Configuration

```yaml
dbSync:
  network: preview
  persistence:
    storageClass: <storage-class>
postgres:
  persistence:
    storageClass: <storage-class>
cardanoNode:
  upstreamAddress: cardano-relay.default.svc.cluster.local:3000
credentials:
  vaultStaticSecret:
    path: runtime/cardano-db-sync/preview/postgres
```

For MCP workflows, inspect candidate Cardano relay workloads with
`workloads.list` and ask the operator to approve the exact `host:port` value for
`cardanoNode.upstreamAddress`.

## Vault Record

The Vault record at `credentials.vaultStaticSecret.path` must include these
keys by default:

```json
{
  "username": "cexplorer",
  "password": "<postgres-password>",
  "database": "cexplorer",
  "connection": "psql://cexplorer:<postgres-password>@<release>-cardano-db-sync-postgres:5432/cexplorer"
}
```

`username`, `password`, and `database` are consumed by the Postgres and DB Sync
pods. `connection` is intended for Midnight workloads that need the PostgreSQL
connection string.

## Testing

```shell
helm lint . -f ci/values-default.yaml
helm lint . -f ci/values-socket-proxy.yaml

helm template cardano-db-sync . -f ci/values-default.yaml
helm template cardano-db-sync . -f ci/values-socket-proxy.yaml
```

## Key Values

| Value | Description | Default |
|-------|-------------|---------|
| `dbSync.network` | Cardano network DB Sync follows | `preview` |
| `dbSync.image.tag` | Cardano DB Sync image tag | `13.6.0.4` |
| `postgres.image.tag` | PostgreSQL image tag | `15.3` |
| `postgres.persistence.storageClass` | StorageClass for PostgreSQL data | empty |
| `dbSync.persistence.storageClass` | StorageClass for DB Sync state | empty |
| `cardanoNode.upstreamAddress` | Approved Cardano relay endpoint in `host:port` form | empty |
| `cardanoNode.socketPath` | Unix socket path exposed inside the DB Sync pod | `/node-ipc/node.socket` |
| `credentials.vaultStaticSecret.path` | Vault path containing PostgreSQL credentials | empty |

Consult `values.schema.json` for the full public configuration contract.
