# Cardano SPO Maintenance Overview

## Goal

Route Cardano SPO maintenance work through MCP-supported operations only.

## Supported MCP Actions

- inspect workloads with `workloads.list` and `workloads.get`
- inspect logs with `workloads.logs.get`
- inspect metrics with `workloads.metrics.get`
- inspect events with `cluster.events.list`
- inspect runtime secret metadata with `vault.runtime.metadata.get`
- update runtime secret records with `vault.runtime.write` or `vault.runtime.patch`
- apply workload changes with `workloads.upgrade`

## Not Supported By MCP

- Cardano transaction construction
- Cardano transaction signing
- stake pool registration updates
- stake pool retirement certificate generation
- KES key generation
- operational certificate issuance
- local port-forward sessions

If the operator requests one of these, say MCP does not currently expose that operation and stop for operator direction.

## Rules

- Start live-changing operations with `dryRun=true` where the tool supports it.
- Do not ask users to paste signing keys or runtime secret values in chat.
- Do not use non-MCP commands.
