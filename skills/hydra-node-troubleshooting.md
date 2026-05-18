# Hydra Node Troubleshooting

## Goal

Diagnose Hydra node startup, networking, metrics, and head-progress issues on a Supernode cluster.

## First Checks

Use MCP:

1. `workloads.get` for the release.
2. `workloads.logs.get` for recent Hydra logs.
3. `workloads.metrics.get` for the derived Hydra metrics payload.
4. `cluster.events.list` for scheduling, mount, probe, or image errors.

## Startup Failures

Common causes:

- Missing Hydra signing key Secret or VaultStaticSecret sync.
- Missing Hydra verification key ConfigMap items.
- Offline mode missing `offlineHeadSeed`, initial UTxO, or protocol parameters.
- Online mode missing Cardano signing key, Cardano verification key, Cardano socket, or scripts transaction ID.
- PVC cannot bind because the storage class is wrong or unavailable.
- Hydra image tag and chart arguments are incompatible.

## Metrics Checks

Use `workloads.metrics.get` for the derived Metis Hydra metrics payload. The tool runs the chart-mounted `/opt/metis/bin/metrics.sh` script and returns the metrics schema with collection errors.

If raw Prometheus metric names are needed for interpretation, use the names below as reference points. Prefer `workloads.metrics.get` for normal troubleshooting rather than direct container access.

Important raw Hydra metrics:

- `hydra_head_confirmed_tx`
- `hydra_head_inputs`
- `hydra_head_peers_connected`
- `hydra_head_requested_tx`
- `hydra_head_tx_confirmation_time_ms_bucket`
- `hydra_head_tx_confirmation_time_ms_sum`
- `hydra_head_tx_confirmation_time_ms_count`

## Head Does Not Progress

First inspect `workloads.metrics.get` for `headStatus`, `lastSeenSnapshotTag`, `peersConnected`, `snapshotNumber`, and `errors`.

If deeper API state is needed, use `workloads.get` to discover the `api` output and use `workload-output-port-forward.md` to expose it locally. Then query Hydra's API through the port-forward:

```bash
curl -s http://127.0.0.1:4001/head
curl -s http://127.0.0.1:4001/snapshot
curl -s http://127.0.0.1:4001/snapshot/last-seen
```

Likely causes:

- Peers are not connected.
- Peer topology differs between participants.
- Hydra verification keys do not match expected parties.
- Protocol parameters differ across participants.
- A peer is out of sync with L1.
- A transaction is valid on one participant but invalid on another.

## Network Or Topology Mismatch

Hydra topology is static. Participants should agree on the peer set.

Look for log messages such as:

- `NetworkVersionMismatch`
- `NetworkClusterIDMismatch`
- `PeerDisconnected`
- `NetworkDisconnected`

Mirror nodes must use unique node IDs and advertise unique peer addresses while sharing the original party credentials.

## Out Of Sync

Online nodes stop accepting unsafe inputs when chain sync is too stale. Check for:

- `NodeUnsynced`
- `RejectedInputBecauseUnsynced`
- `SyncedStatusReport`

The unsynced period should be shorter than the contestation period. Mainnet contestation should generally be at least 12 hours.

## Stuck Snapshot Recovery

Use `/snapshot/last-seen` to identify whether there is an in-flight snapshot and which peers have not signed.

Sideloading the last confirmed snapshot can recover a stuck local state, but it must be coordinated by participants. See `hydra-head-operations.md` for the API procedure.

After sideloading, fix the underlying cause before resubmitting transactions.
