# Workload Output Discovery

## Goal

Discover workload outputs with MCP tools first and describe the current local access boundary.

## Workflow

1. Ask for namespace and workload name if unknown.
2. Call `workloads.get`.
3. Present the returned outputs by name, protocol, scope, service, port, and URL.
4. If the user asks to expose an output locally, explain that MCP does not currently provide a long-running port-forward tool.
5. If operator-approved local access is explicitly allowed, identify the exact output to target and describe a scoped port-forward as a local fallback.

## Rules

- Prefer MCP outputs and typed MCP tools over local tunnels.
- Only describe a port-forward when the operator explicitly approves local access for that workload.
- Keep any port-forward guidance scoped to the specific service port returned by MCP.
- Use only the output data returned by `workloads.get`.
