# Apex Fusion Block Producer Deployment

## Goal

Install or upgrade an Apex Fusion block producer with MCP tools only.

## Required Inputs

- release name
- namespace
- network: `vector-testnet`, `prime-testnet`, or `prime-mainnet`
- pool ID
- producer runtime Vault path
- producer storage class
- relay storage class when `relays.count > 0`
- either managed relay count or explicit trusted relay targets

## Workflow

1. Call `supernode.status.get`.
2. Call `extensions.catalog.get` with `extensionId=apex-fusion-block-producer`.
3. Call `cluster.storage_classes.list`.
4. Call `workloads.list` to inspect same-network relay candidates.
5. If using existing relays, ask the operator to approve the exact `relays.trusted` addresses.
6. Call `vault.runtime.metadata.get` for the runtime path when validating existing material.
7. If material must be written or updated, use `vault.runtime.write` or `vault.runtime.patch`; do not ask for secret values in chat.
8. Call `workloads.install` or `workloads.upgrade` with `dryRun=true` and direct `apex-fusion-block-producer` chart values.
9. Review the dry-run result with the operator.
10. Call the same workload mutation with `dryRun=false` only after approval.
11. Validate with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.

## Debug Configuration Pattern

```json
{
  "node": {
    "network": "vector-testnet"
  },
  "persistence": {
    "storageClass": "<storage-class>",
    "size": "20Gi"
  },
  "relayPersistence": {
    "storageClass": "<storage-class>",
    "size": "20Gi"
  },
  "blockProducer": {
    "debug": true,
    "poolId": "<pool-id>",
    "vaultStaticSecret": {
      "path": "runtime/apex-fusion/<network>-<pool-slug>/block-producer"
    }
  },
  "relays": {
    "count": 1
  }
}
```

## Activation

Use `workloads.upgrade` with `dryRun=true`, changing only `blockProducer.debug` from `true` to `false`. Run live only after operator approval.

## Rules

- MCP does not auto-resolve trusted relays.
- Do not use non-MCP commands.
