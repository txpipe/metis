# Hydra Node Troubleshooting

## Goal

Troubleshoot Hydra node startup, networking, metrics, and head progress with MCP tools only.

## Workflow

1. Call `workloads.get` for the Hydra release.
2. Call `workloads.logs.get` for recent Hydra logs.
3. Call `workloads.metrics.get` for the derived Hydra metrics payload.
4. Call `cluster.events.list` for scheduling, mount, probe, image, or PVC errors.
5. Call `vault.runtime.metadata.get` for runtime paths when secret sync is suspected.

## Check From MCP

- workload health
- event errors
- log errors
- metric collection errors
- peer connection counts
- head status
- snapshot status
- configured outputs

## Limits

If direct Hydra HTTP or WebSocket interaction is required, state that MCP does not currently expose Hydra API operation tools and stop.

## Rules

- Do not call Hydra HTTP or WebSocket APIs directly.
- Do not generate port-forward commands.
- Do not use non-MCP commands.
