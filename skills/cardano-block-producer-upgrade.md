# Cardano Block Producer Upgrade

## Goal

Install or upgrade a Cardano block producer with MCP tools only.

## Required Inputs

- release name
- namespace
- Cardano network
- pool ID
- producer runtime Vault path
- producer storage class
- relay storage class when `relays.count > 0`
- either managed relay count or explicit trusted relay targets

## Workflow

1. Call `supernode.status.get`.
2. Call `extensions.catalog.get` with `extensionId=cardano-block-producer`.
3. Call `cluster.storage_classes.list`.
4. Call `workloads.list` to inspect same-network relay candidates.
5. If using existing relays, ask the operator to approve the exact `relays.trusted` addresses.
6. Call `vault.runtime.metadata.get` for the runtime path when validating existing material.
7. If material must be written or updated, use `vault.runtime.write` or `vault.runtime.patch`; do not ask for secret values in chat.
8. Call `workloads.install` or `workloads.upgrade` with `dryRun=true` and direct `cardano-block-producer` chart values.
9. Review the dry-run result with the operator.
10. Call the same workload mutation with `dryRun=false` only after approval.
11. Validate with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.

## Debug Configuration Pattern

```json
{
  "node": {
    "network": "preview"
  },
  "persistence": {
    "storageClass": "<storage-class>",
    "size": "80Gi"
  },
  "relayPersistence": {
    "storageClass": "<storage-class>",
    "size": "80Gi"
  },
  "blockProducer": {
    "debug": true,
    "poolId": "<pool-id>",
    "vaultStaticSecret": {
      "path": "runtime/cardano-node/<network>-<pool-slug>/block-producer"
    }
  },
  "relays": {
    "count": 1
  }
}
```

## Existing Relay Pattern

```json
{
  "relays": {
    "count": 0,
    "trusted": [
      {
        "address": "<relay-service>.<namespace>.svc.cluster.local",
        "port": 3000,
        "valency": 1
      }
    ],
    "useLedgerAfterSlot": -1
  }
}
```

## Activation

Use `workloads.upgrade` with `dryRun=true`, changing only `blockProducer.debug` from `true` to `false`. Run live only after operator approval.

## Rules

- Do not use old `node.blockProducer.*` fields with `cardano-block-producer`.
- MCP does not auto-resolve trusted relays.
- Do not use non-MCP cluster, Helm, Vault CLI, or Cardano CLI commands.
