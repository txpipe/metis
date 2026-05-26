# Midnight Boot Node Operations

## Goal

Guide MCP-first checks for a Midnight node operating as a boot node and use a scoped local port-forward only when API inspection is needed.

## Workflow

1. Call `workloads.get` for the Midnight workload.
2. Confirm the workload exposes the `p2p` output and that the `node.bootnodes` configuration matches the intended environment.
3. Call `workloads.metrics.get`.
4. Call `workloads.logs.get`.
5. Call `cluster.events.list` for the workload namespace.
6. If the operator needs API-level confirmation and explicitly approves local access, read `supernode://skills/workload-output-port-forward`, target the `rpc` output, and use a scoped local port-forward for read-only JSON-RPC checks.

## What To Check

- the node has peer connectivity through `healthPeers` or `peers`
- the node is not stuck in a restart loop
- logs show steady peer discovery or network participation
- the `p2p` output is present and matches the expected service port

## Limits

- MCP can confirm the `p2p` output and in-cluster workload state.
- MCP cannot prove internet reachability of the boot node's exposed P2P endpoint unless a future MCP tool adds that check.

## Optional RPC Checks

If the operator explicitly approves local access, use a scoped local port-forward to the `rpc` output and run read-only calls such as:

- `system_chain`
- `system_health`
- `system_syncState`
- `system_version`

## Rules

- Treat a boot node as a specialized Midnight node, not as a separate extension.
- Do not use local port-forwarding unless the operator explicitly approves it.
- Keep any local API use read-only and scoped to the approved Midnight workload.
