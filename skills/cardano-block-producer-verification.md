# Cardano Block Producer Verification

## Goal

Verify block-producer readiness using MCP tools only.

## Workflow

1. Call `workloads.get` for the producer.
2. Call `workloads.metrics.get` for producer metrics.
3. Call `workloads.logs.get` for recent producer logs if metrics show errors.
4. Call `cluster.events.list` for namespace events if the workload is not healthy.
5. If Dolos is installed for the same network, use `dolos.snapshot.refresh` when external chain-view refresh is needed.

## Verify From MCP

- workload is healthy
- Vault-backed runtime material is mounted according to workload status
- debug-mode or active forging mode is as expected
- KES and op-cert fields are present when exposed by metrics
- peer and sync fields are healthy
- schedule fields are present when exposed by metrics

## Limits

MCP metrics can show readiness and local producer signals. If MCP does not expose canonical block outcome confirmation, say so and do not claim the pool produced accepted blocks.

## Rules

- Do not use non-MCP commands.
- Do not overstate what MCP metrics prove.
