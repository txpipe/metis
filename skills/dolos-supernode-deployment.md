# Dolos Supernode Deployment

## Goal

Deploy and validate Dolos with MCP tools only.

## Required Inputs

- release name
- namespace
- network
- storage class selected from MCP
- explicit upstream relay address

## Workflow

1. Call `supernode.status.get`.
2. Call `extensions.catalog.get` with `extensionId=dolos`.
3. Call `cluster.storage_classes.list` and ask the operator to choose one.
4. Call `workloads.list` to inspect same-network relay candidates.
5. Ask the operator to approve the exact `config.upstreamAddress`.
6. Call `workloads.install` with `dryRun=true` and direct Dolos chart values.
7. Review the dry-run result with the operator.
8. Call `workloads.install` with `dryRun=false` only after approval.
9. Validate with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.
10. Use `dolos.snapshot.refresh` when the operator needs a Dolos snapshot refresh.

## Minimal Configuration

```json
{
  "dolos": {
    "network": "cardano-preview"
  },
  "persistence": {
    "storageClass": "<storage-class>",
    "size": "50Gi"
  },
  "config": {
    "upstreamAddress": "<relay-service>.<namespace>.svc.cluster.local:3000"
  }
}
```

## Rules

- MCP does not auto-resolve Dolos upstreams.
- Do not use old flat Dolos fields.
- Do not use non-MCP commands.
