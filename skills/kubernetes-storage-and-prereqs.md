# Kubernetes Storage And Prereqs

## Goal

Validate cluster readiness before installs or upgrades using MCP tools only.

## Workflow

1. Call `supernode.status.get`.
2. Call `cluster.storage_classes.list`.
3. Call `cluster.events.list` for recent scheduling, provisioning, image, probe, or mount problems.
4. Call `workloads.list` to understand existing releases.
5. Call `workloads.get` for any workload involved in the planned operation.

## Checks

- The MCP server can reach Kubernetes.
- A real storage class is selected from `cluster.storage_classes.list`.
- Existing workloads are healthy enough to use as dependencies.
- Recent events do not show unresolved scheduling or PVC failures.

## Rules

- Do not assume storage class names.
- Do not proceed with install or upgrade when MCP reports unresolved PVC, scheduling, or control-plane failures.
- Do not use non-MCP cluster commands from this skill.
