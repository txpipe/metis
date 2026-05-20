# Hydra Head Operations

## Goal

Define the MCP boundary for Hydra head operations.

## Boundary

MCP currently exposes Hydra deployment, discovery, logs, metrics, events, Vault runtime operations, and `hydra.keys.generate`. It does not expose direct Hydra head lifecycle tools such as init, commit, close, contest, fanout, decommit, or transaction submission.

## Workflow

1. Call `workloads.get` for the Hydra workload.
2. Call `workloads.metrics.get` for head status and snapshot-related fields.
3. Call `workloads.logs.get` for recent Hydra logs.
4. Present the MCP-observed state to the operator.
5. If the operator asks for a direct head lifecycle action, state that MCP does not currently expose that operation and stop.

## Rules

- Do not call Hydra HTTP or WebSocket APIs directly.
- Do not generate port-forward commands.
- Do not use non-MCP commands.
