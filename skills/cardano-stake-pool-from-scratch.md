# Cardano Stake Pool From Scratch

## Goal

Guide only the MCP-supported Metis deployment portion of a new Cardano stake pool.

## Boundary

MCP does not currently expose Cardano ledger transaction, key-generation, metadata publishing, or offline signing tools. If the operator asks the agent to perform those actions, state that MCP does not support them and stop for operator direction.

This skill begins once the operator can provide the pool ID and the approved runtime material path, or can provide runtime material through MCP `vault.runtime.write`.

## Required Inputs

- Cardano network
- relay release name and namespace
- producer release name and namespace
- storage class selected from MCP
- pool ID
- runtime Vault path for `kes.skey`, `vrf.skey`, and `op.cert`
- managed relay count or explicit trusted relay targets

## Workflow

1. Call `supernode.status.get`.
2. Call `extensions.catalog.get` for `cardano-relay` and `cardano-block-producer`.
3. Call `cluster.storage_classes.list` and ask the operator to choose one.
4. Call `workloads.list` to inspect existing same-network Cardano workloads.
5. If no relay exists, install `cardano-relay` through `workloads.install` with `dryRun=true`, then live after approval.
6. Validate the relay with `workloads.get`, `workloads.logs.get`, and `workloads.metrics.get`.
7. Call `vault.runtime.metadata.get` for the producer runtime path.
8. If runtime material must be written, use `vault.runtime.write`; do not ask for secret values in chat.
9. Install `cardano-block-producer` in debug mode through `workloads.install` with `dryRun=true`.
10. Review the dry-run result with the operator.
11. Install live only after approval.
12. Validate debug mode with `workloads.get`, `workloads.logs.get`, `workloads.metrics.get`, and `cluster.events.list`.
13. Activate forging with `workloads.upgrade` by changing `blockProducer.debug=false`, first with `dryRun=true`, then live after approval.

## Rules

- Use `cardano-relay` and `cardano-block-producer` for new installs.
- Use direct chart values from `extensions.catalog.get`.
- Do not perform Cardano ledger operations from this skill.
- Do not use non-MCP commands.
