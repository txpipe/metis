# Apex Fusion Relay Setup

## Goal

Install and validate an Apex Fusion relay with MCP tools only.

## Required Inputs

- release name
- namespace
- network: `vector-testnet`, `prime-testnet`, or `prime-mainnet`
- storage class selected from MCP
- optional direct chart values from the catalog schema

## Workflow

1. Call `supernode.status.get`.
2. Call `extensions.catalog.get` with `extensionId=apex-fusion-relay`.
3. Call `cluster.storage_classes.list` and ask the operator to choose one.
4. Call `workloads.list` to identify same-network workloads and naming conflicts.
5. Call `workloads.install` with `dryRun=true` and direct `apex-fusion-relay` chart values.
6. Review the dry-run result with the operator.
7. Call `workloads.install` with `dryRun=false` only after approval.
8. Validate with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.

## Minimal Configuration

```json
{
  "node": {
    "network": "vector-testnet"
  },
  "persistence": {
    "storageClass": "<storage-class>",
    "size": "20Gi"
  }
}
```

## Rules

- Use `apex-fusion-relay` for new relays.
- Do not use non-MCP commands.
