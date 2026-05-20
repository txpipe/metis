# Cardano SPO KES Rotation

## Goal

Handle only the MCP-supported deployment portion of a KES rotation.

## Boundary

MCP does not currently generate KES keys or issue operational certificates. The operator must complete those steps outside MCP and provide approved runtime material through the MCP client when updating Vault.

## Workflow

1. Call `workloads.get` and `workloads.metrics.get` for the producer.
2. Call `vault.runtime.metadata.get` for the runtime path.
3. If the operator is ready to stage new runtime material, call `vault.runtime.patch` or `vault.runtime.write`; do not ask for secret values in chat.
4. Call `workloads.upgrade` with `dryRun=true` if a workload restart or config change is needed.
5. Run live `workloads.upgrade` only after approval.
6. Validate with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.

## Rules

- Do not generate keys.
- Do not issue op certs.
- Do not use non-MCP commands.
