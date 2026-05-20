# Cardano Relay Setup

## Goal

Install and validate a Cardano relay with MCP tools only.

## Required Inputs

- release name
- namespace
- Cardano network
- storage class selected from MCP
- optional resource, image, service, or topology values from the catalog schema

## Workflow

1. Call `supernode.status.get`.
2. Call `extensions.catalog.get` with `extensionId=cardano-relay`.
3. Call `cluster.storage_classes.list` and ask the operator to choose one.
4. Call `workloads.list` to identify same-network workloads and naming conflicts.
5. Call `workloads.install` with `dryRun=true` and direct `cardano-relay` chart values.
6. Review the dry-run result with the operator.
7. Call `workloads.install` with `dryRun=false` only after approval.
8. Validate with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.

## Minimal Configuration

```json
{
  "node": {
    "network": "preview"
  },
  "persistence": {
    "storageClass": "<storage-class>",
    "size": "80Gi"
  }
}
```

## Rules

- Use `cardano-relay` for new relays.
- Do not use flat legacy fields such as `network`, `storageClass`, `pvcSize`, or `imageTag`.
- Do not use non-MCP cluster or Helm commands.
