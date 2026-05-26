# Midnight Node Operations

## Goal

Inspect Midnight node health and operating state with MCP tools first, falling back to a scoped local RPC port-forward only when needed.

## Workflow

1. Call `workloads.get` for the Midnight workload.
2. Review the `rpc`, `ws`, `p2p`, and `metrics` outputs.
3. Call `workloads.metrics.get`.
4. Call `workloads.logs.get`.
5. Call `cluster.events.list` for the workload namespace.
6. If the operator needs API-level confirmation and explicitly approves local access, read `supernode://skills/workload-output-port-forward`, target the `rpc` output, and use a scoped local port-forward for read-only JSON-RPC checks.

## What To Check

- `metrics.chain` matches the expected Midnight environment.
- `metrics.nodeVersion` matches the intended release line for the environment.
- `metrics.shouldHavePeers` is true for networked nodes.
- `metrics.healthPeers` or `metrics.peers` is non-zero after startup.
- `metrics.syncing` trends toward false as the node catches up.
- `metrics.errors` is empty or only contains transient startup warnings.

## Common Failure Patterns

- DB Sync dependency missing or not ready:
  - logs show connection failures
  - metrics remain sparse
  - node does not progress
- Wrong network or image version:
  - chain identity is unexpected
  - logs show compatibility or handshake failures
- Bootnode or peer issues:
  - peer counts stay at zero
  - logs show repeated dial or discovery failures
- Storage or restart issues:
  - events show PVC mount failures or restart loops

## Optional RPC Checks

If the operator explicitly approves local access, use a scoped local port-forward to the `rpc` output and run read-only JSON-RPC checks such as:

- `system_chain`
- `system_health`
- `system_syncState`
- `system_version`
- `rpc_methods`

Correlate API responses with `workloads.metrics.get` and `workloads.logs.get` before concluding the node is healthy.

## Rules

- Prefer `workloads.metrics.get` and `workloads.logs.get` before local API inspection.
- Do not use local port-forwarding unless the operator explicitly approves it.
- Keep any local API use read-only.
- Do not use non-MCP mutation paths for Midnight operations.
