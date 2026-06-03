# General Workload Troubleshooting

## Goal

Troubleshoot a workload with MCP tools first, using workload status, logs, metrics, events, and dependency inspection before suggesting configuration changes.

## Workflow

1. Call `workloads.get` for the workload.
2. Inspect `diagnostics`, `logTargets`, and `outputs` from the workload response.
3. Treat repeated restarts as a primary signal and review `lastState` for terminated containers.
4. Call `workloads.metrics.get` for the workload.
5. Call `workloads.logs.get` for recent logs from the relevant target.
6. If restarts are present or `lastState` shows a terminated container, call `workloads.logs.get` again with `previous=true`.
7. Call `cluster.events.list` for recent scheduling, image, mount, probe, or restart-loop events.
8. Call `vault.runtime.metadata.get` only when the symptoms suggest runtime or secret sync issues.
9. Call `workloads.list` when you need to discover dependency workloads for databases, message brokers, chain nodes, or other backing services before inspecting them directly.
10. Inspect the relevant dependency workloads when the primary workload depends on those services.

## What To Check

- `diagnostics` for readiness, probe, scheduling, image pull, or configuration failures.
- `logTargets` to choose the container or sidecar that is actually failing.
- `outputs` to confirm the workload is exposing the expected ports or service endpoints.
- restart counts and terminated `lastState` details.
- memory, CPU, and other saturation signals from `workloads.metrics.get`.
- recent current logs and previous-instance logs when the container has restarted.
- cluster events that explain scheduling, storage, image, secret, or probe failures.
- dependency workload health when the primary workload is blocked on another service.

## Common Failure Patterns

- Repeated restarts:
  - This is a primary signal that the workload is not staying healthy long enough to serve traffic.
  - Use current logs and previous logs together to separate startup failures from steady-state failures.
- `OOMKilled`:
  - This is a strong memory-pressure signal to investigate alongside restart history and memory metrics.
  - If the workload is restarting often and `OOMKilled`, the workload may be short on memory and increasing memory is a reasonable next step once the evidence supports that conclusion.
- Probe failures:
  - `diagnostics` or events show readiness or liveness failures.
  - Logs often show startup delays, bad bindings, or dependency timeouts.
- Secret or runtime config issues:
  - Events or logs show missing files, missing env vars, auth failures, or stale rendered config.
  - Only then call `vault.runtime.metadata.get` to inspect runtime metadata.
- Dependency failures:
  - The primary workload is healthy enough to start but logs repeated connection or timeout errors.
  - Inspect the referenced dependency workloads before concluding the issue is local to the primary workload.

## Limits

If MCP does not expose the workload field, dependency, or runtime detail needed to prove the cause, state that limit clearly instead of inventing shell-based steps.

## Rules

- Start with `workloads.get` before logs or metrics so the troubleshooting path is anchored to workload state.
- Prefer MCP data sources over shell commands or exec access.
- Use `workloads.logs.get` with `previous=true` when restart history or terminated `lastState` indicates the prior container instance matters.
- Call `vault.runtime.metadata.get` only for runtime or secret sync suspicion.
- Use `workloads.list` when dependency workloads must be discovered before deeper inspection.
- Inspect dependency workloads when the symptoms point to an upstream or downstream service issue.
