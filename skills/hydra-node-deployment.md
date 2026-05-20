# Hydra Node Deployment

## Goal

Deploy and validate a Hydra node with MCP tools only.

## Required Inputs

- release name
- namespace
- storage class selected from MCP
- offline or online mode
- Hydra signing key runtime path
- Hydra verification key entries
- online Cardano socket proxy target when online mode is used

## Workflow

1. Call `supernode.status.get`.
2. Call `extensions.catalog.get` with `extensionId=hydra-node`.
3. Call `cluster.storage_classes.list`.
4. Use `hydra.keys.generate` when the operator wants MCP to generate Hydra keys.
5. If the operator provides existing runtime material through MCP, use `vault.runtime.write`; do not ask for secret values in chat.
6. For online mode, call `workloads.list` and ask the operator to approve the Cardano socket proxy target.
7. Call `workloads.install` with `dryRun=true` and direct `hydra-node` chart values.
8. Review the dry-run result with the operator.
9. Call `workloads.install` with `dryRun=false` only after approval.
10. Validate with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.

## Offline Configuration Pattern

```json
{
  "persistence": {
    "storageClass": "<storage-class>"
  },
  "keys": {
    "hydraSigning": {
      "vaultStaticSecret": {
        "path": "runtime/hydra/demo/hydra-signing"
      }
    },
    "hydraVerification": {
      "items": [
        {
          "filename": "hydra.vk",
          "value": "<public hydra verification key>"
        }
      ]
    }
  }
}
```

## Online Rules

- Use `node.cardanoSocketProxy` for online Cardano connectivity.
- MCP does not auto-discover the proxy target.
- Do not use unsupported manual socket mount values such as `node.extraVolumes` or `node.extraVolumeMounts`.

## Rules

- Do not read or echo signing key values in chat.
- Do not use non-MCP commands.
