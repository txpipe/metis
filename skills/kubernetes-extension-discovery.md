# Kubernetes Extension Discovery

## Goal

Discover Metis extensions and installed workloads using MCP tools only.

## Workflow

1. Call `supernode.status.get`.
2. Call `extensions.catalog.list`.
3. Call `workloads.list` with `includeControlPlane=true` when control-plane visibility matters.
4. For a candidate workload, call `workloads.get` with its namespace and name.
5. For a candidate extension, call `extensions.catalog.get` with the extension ID.

## What To Identify

- release name
- namespace
- extension or chart ID
- workload status
- outputs exposed by the workload
- whether it is a relay or block-producer workload

## Current Preferred Extension IDs

- `cardano-relay`
- `cardano-block-producer`
- `apex-fusion-relay`
- `apex-fusion-block-producer`
- `dolos`
- `hydra-node`

## Rules

- Use `workloads.list` and `workloads.get`; do not infer state from names alone.
- For new installs, prefer the split relay and block-producer extension IDs.
- Do not use non-MCP cluster commands from this skill.
