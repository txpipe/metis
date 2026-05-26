# Skills

These skills are MCP-only operating guides for agents working with Metis.

## Rule

Use MCP tools first. Do not tell the user to run `helm`, `vault`, `cardano-cli`, `curl`, or Docker from these skills.

If MCP does not expose a required workload API directly, a skill may describe an operator-approved, scoped local port-forward to a specific workload output for read-only inspection. Keep that fallback narrow, prefer MCP outputs discovered through `workloads.get`, and do not use port-forwards for secret handling or arbitrary cluster access.

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
- Use split extension IDs for new installs: `cardano-relay`, `cardano-block-producer`, `cardano-db-sync`, `apex-fusion-relay`, `apex-fusion-block-producer`, `dolos`, `hydra-node`, and `midnight`.
- Use `extensions.catalog.get` as the source of truth for configuration shape.
- Pass direct chart values to `workloads.install` and `workloads.upgrade`.
- Start mutating tools with `dryRun: true`; run live only after the operator approves the returned plan.
- Do not auto-resolve trusted relays or upstreams. Use `workloads.list`, propose same-network candidates, and pass the operator-approved address explicitly.
- Do not ask users to paste secret values in chat. If runtime material must be written, use `vault.runtime.write` or `vault.runtime.patch` through the MCP client.
