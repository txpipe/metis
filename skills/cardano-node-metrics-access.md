# Cardano Node Metrics Access

## Goal

Read Cardano or Apex Fusion workload metrics using MCP tools only.

## Workflow

1. Ask for namespace and workload name if unknown.
2. Call `workloads.get` to confirm the workload and status.
3. Call `workloads.metrics.get`.
4. If metrics are missing or stale, call `workloads.logs.get` and `cluster.events.list`.

## Interpretation

- Use returned sync, resource, peer, KES, op-cert, and forging fields as the MCP source of truth.
- If raw Prometheus text is needed but `workloads.metrics.get` does not expose it, state that MCP does not currently expose raw metric scraping and stop.

## Rules

- Do not exec into pods.
- Do not query local container ports.
- Do not use non-MCP commands.
