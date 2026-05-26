# Midnight Node Deployment

## Goal

Deploy and validate a Midnight node with MCP tools first, using a scoped local port-forward only when deeper RPC inspection is required.

## Required Inputs

- release name
- namespace
- Midnight environment
- storage class selected from MCP
- approved Vault path for the Cardano DB Sync connection string
- approved Vault path for the Midnight node key
- optional node image tag when deploying Preprod or Mainnet
- optional bootnodes override for non-default environments

## Workflow

1. Call `supernode.status.get`.
2. Call `extensions.catalog.get` with `extensionId=midnight`.
3. Call `cluster.storage_classes.list`.
4. Call `workloads.list` and inspect deployed `cardano-db-sync` workloads.
5. Confirm the approved `cardano-db-sync` dependency is already deployed before planning a Midnight install.
6. Ask the operator to approve the exact Vault runtime path that stores the DB Sync PostgreSQL connection string under `connection`.
7. Ask the operator to approve the exact Vault runtime path that stores the Midnight node key under `node.key`.
8. Call `workloads.install` with `dryRun=true` and direct `midnight` chart values.
9. Review the dry-run result with the operator, including dependency status and storage class validation.
10. Call `workloads.install` with `dryRun=false` only after approval.
11. Validate with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.
12. If deeper API validation is needed and the operator explicitly approves local access, read `supernode://skills/workload-output-port-forward`, target the `rpc` output, and use a scoped local port-forward for read-only JSON-RPC checks.

## Minimal Configuration

```json
{
  "node": {
    "network": "preview"
  },
  "persistence": {
    "storageClass": "<storage-class>"
  },
  "dbSync": {
    "vaultStaticSecret": {
      "path": "runtime/midnight/preview/dbsync"
    }
  },
  "nodeKey": {
    "vaultStaticSecret": {
      "path": "runtime/midnight/preview/node-key"
    }
  }
}
```

## Validation Checks

- `workloads.metrics.get` should show Midnight peer and sync data without collection errors.
- `workloads.logs.get` should not show repeated DB Sync connection failures.
- `workloads.logs.get` should show peer discovery and normal sync progress.
- `cluster.events.list` should be clear of repeated container restarts, PVC errors, or Vault sync failures.

## Optional RPC Checks

If the operator explicitly approves local access, use a scoped local port-forward to the `rpc` output for read-only checks such as:

- `system_chain`
- `system_health`
- `system_syncState`
- `system_version`
- `rpc_methods`

## Rules

- Midnight depends on a deployed `cardano-db-sync` workload. Do not proceed if the dependency is missing.
- Do not ask the user to paste secret values in chat.
- Do not use local port-forwarding unless the operator explicitly approves it.
- Keep any local API use read-only and scoped to the approved Midnight workload.
