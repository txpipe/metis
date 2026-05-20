# Cardano SPO Pool Update

## Goal

Handle only MCP-supported checks around a Cardano stake pool update.

## Boundary

MCP does not currently build, sign, or submit Cardano stake pool update transactions. If the operator asks for those steps, state that MCP does not support them and stop.

## MCP Workflow

1. Call `workloads.get` and `workloads.metrics.get` for the producer.
2. Call `workloads.get` and `workloads.metrics.get` for relays involved in publication data.
3. Call `vault.runtime.metadata.get` if runtime material may be affected.
4. If runtime material must change after the on-chain update, use `vault.runtime.patch` or `vault.runtime.write` through MCP.
5. Use `workloads.upgrade` with `dryRun=true` for any matching workload configuration change.
6. Apply live only after operator approval.
7. Validate with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.

## Rules

- Do not construct or submit ledger transactions.
- Do not ask for signing keys.
- Do not use non-MCP commands.
