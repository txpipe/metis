# Supernode Dashboard Discovery

## Goal

Find dashboard-related control-plane outputs using MCP tools only.

## Workflow

1. Call `workloads.list` with `includeControlPlane=true`.
2. Find the `control-plane` workload or dashboard-related workload returned by MCP.
3. Call `workloads.get` for that workload.
4. Present any dashboard, Grafana, or Prometheus outputs returned by MCP.
5. If the user asks to open a local tunnel, explain that MCP does not currently provide a long-running port-forward tool and stop.

## Rules

- Do not generate local port-forward commands.
- Do not use non-MCP cluster commands.
