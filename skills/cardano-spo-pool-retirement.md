# Cardano SPO Pool Retirement

## Goal

Handle only MCP-supported checks around a Cardano stake pool retirement.

## Boundary

MCP does not currently build, sign, or submit Cardano retirement transactions. If the operator asks for those steps, state that MCP does not support them and stop.

## MCP Workflow

1. Call `workloads.get` and `workloads.metrics.get` for the producer.
2. Call `workloads.get` and `workloads.metrics.get` for relays that may remain online after retirement.
3. Use `workloads.upgrade` with `dryRun=true` for any workload-mode change the operator requests.
4. Use `workloads.delete` with `dryRun=true` only when the operator explicitly asks to remove a workload.
5. Apply live changes only after operator approval.
6. Validate with `workloads.list`, `workloads.get`, `workloads.logs.get`, and `cluster.events.list`.

## Rules

- Do not construct or submit retirement transactions.
- Do not delete workloads or PVCs unless explicitly approved through MCP dry-run review.
- Do not use non-MCP commands.
