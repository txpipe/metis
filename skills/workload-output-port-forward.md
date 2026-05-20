# Workload Output Discovery

## Goal

Discover workload outputs using MCP tools only.

## Workflow

1. Ask for namespace and workload name if unknown.
2. Call `workloads.get`.
3. Present the returned outputs by name, protocol, scope, service, port, and URL.
4. If the user asks to expose an output locally, explain that MCP does not currently provide a long-running port-forward tool and stop.

## Rules

- Do not generate local port-forward commands.
- Do not use non-MCP networking commands.
- Use only the output data returned by `workloads.get`.
