# Workload Output Port Forward

## Goal

Help a user expose one discovered workload output locally with `kubectl port-forward` after discovering the output through MCP.

## Important Constraint

MCP can discover workload outputs, but it cannot hold open a local `kubectl port-forward` session for the user. The user must run the generated `kubectl` command locally.

## Discovery Workflow

1. Ask for the workload namespace and workload name if they are not already known.
2. Call `workloads.get` for that workload.
3. Inspect `outputs` in the workload status.
4. If there are multiple outputs, ask the user which one they want to expose locally.
5. Prefer the `internal` entry for the selected output name when building a port-forward command.

Expected output fields from MCP:

- `name`
- `description`
- `scope`
- `namespace`
- `serviceName`
- `serviceType`
- `portName`
- `port`
- `protocol`
- `url`

## Kubernetes Context

Use the Kubernetes context that corresponds to the Supernode cluster.

If the context name is unknown, ask the user to confirm it before generating or running commands. Do not assume the context name is literally `supernode`.

Command template:

```bash
kubectl --context <supernode-context> -n <namespace> port-forward service/<serviceName> <localPort>:<remotePort>
```

## Port Selection Rules

- Use the selected workload output's `port` as the remote port.
- By default, use the same number for the local port.
- If that local port is already taken or the user wants a different local port, ask for an alternate local port.
- Do not hardcode ports by workload type. Always take them from the discovered workload status.

## Generic Procedure

After the user selects an output, produce a command like:

```bash
kubectl --context <supernode-context> -n <namespace> port-forward service/<serviceName> <localPort>:<remotePort>
```

Where:

- `<namespace>` comes from workload status
- `<serviceName>` comes from the selected output
- `<remotePort>` comes from the selected output's `port`
- `<localPort>` defaults to the same value unless the user requests otherwise

## Local Access Guidance

After the port-forward is established, tell the user how to reach it locally based on `protocol`:

- `HTTP`: `http://127.0.0.1:<localPort>`
- `gRPC`: `127.0.0.1:<localPort>` or `grpc://127.0.0.1:<localPort>` depending on the client
- `TCP`: `127.0.0.1:<localPort>`

Do not assume extra URL paths such as `/blocks/latest` unless the user asks for a specific API call or the selected output description explicitly requires one.

## Example Agent Behavior

1. Read `workloads.get` for the workload.
2. Present the discovered `outputs` by name, protocol, scope, and URL.
3. Ask which output to expose if more than one is available.
4. Ask which Kubernetes context corresponds to the Supernode cluster if it is not already known.
5. Build the exact `kubectl --context ... port-forward ...` command from the selected output.
6. Explain the local address the user should hit after the tunnel is up.
