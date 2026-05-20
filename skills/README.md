# Skills

These skills are MCP-only operating guides for agents working with Metis.

## Rule

Use MCP tools only. Do not tell the user to run `kubectl`, `helm`, `vault`, `cardano-cli`, `curl`, Docker, or local port-forward commands from these skills.

If a requested action cannot be completed with the MCP tools below, say that MCP does not currently expose that operation and stop for operator direction.

## MCP Tools

- `supernode.status.get`
- `extensions.catalog.list`
- `extensions.catalog.get`
- `cluster.storage_classes.list`
- `cluster.events.list`
- `workloads.list`
- `workloads.get`
- `workloads.logs.get`
- `workloads.metrics.get`
- `workloads.install`
- `workloads.upgrade`
- `workloads.delete`
- `vault.runtime.metadata.get`
- `vault.runtime.write`
- `vault.runtime.patch`
- `hydra.keys.generate`
- `dolos.snapshot.refresh`

## Workload Rules

- Use split extension IDs for new installs: `cardano-relay`, `cardano-block-producer`, `apex-fusion-relay`, `apex-fusion-block-producer`, `dolos`, and `hydra-node`.
- Use `extensions.catalog.get` as the source of truth for configuration shape.
- Pass direct chart values to `workloads.install` and `workloads.upgrade`.
- Start mutating tools with `dryRun: true`; run live only after the operator approves the returned plan.
- Do not auto-resolve trusted relays or upstreams. Use `workloads.list`, propose same-network candidates, and pass the operator-approved address explicitly.
- Do not ask users to paste secret values in chat. If runtime material must be written, use `vault.runtime.write` or `vault.runtime.patch` through the MCP client.
