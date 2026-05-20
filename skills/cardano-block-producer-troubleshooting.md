# Cardano Block Producer Troubleshooting

## Goal

Troubleshoot a Cardano block producer using MCP tools only.

## Workflow

1. Call `workloads.get` for the producer.
2. Call `workloads.metrics.get` for producer metrics.
3. Call `workloads.logs.get` for bounded recent logs.
4. Call `cluster.events.list` for the producer namespace.
5. Call `workloads.get` and `workloads.metrics.get` for trusted relay workloads.
6. If Dolos is installed for the same network, call `dolos.snapshot.refresh` when an external chain-view refresh is needed.

## Check From MCP

- producer mode and debug/active state
- sync health
- peer health
- KES and op-cert status
- schedule fields
- local forging/adoption fields when exposed
- relay workload health
- recent Kubernetes events returned by MCP

## Limits

If MCP does not expose raw sockets, raw topology files, or canonical block outcome tracking, state that limitation and stop instead of giving shell commands.

## Rules

- Do not exec into pods.
- Do not query external HTTP APIs directly.
- Do not use non-MCP commands.
