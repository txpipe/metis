# Cardano Block Producer Troubleshooting

## Goal

Help an agent debug cases where a block producer looks healthy locally but recent pool blocks are not appearing on an external canonical chain view.

## Common Symptom Pattern

This skill applies when most or all of the following are true:

- producer pod is running
- `forging enabled` is true
- KES and op-cert metrics look healthy
- schedule metrics are present
- peer counts are non-zero
- the relay is healthy
- but an external canonical chain source shows no recent blocks for the pool

This should be interpreted as a contradiction between:

- local producer signals
- external canonical-chain observation

## What Local Metrics Mean

The current dashboard can already show local node counters:

- `forgedCount`
- `adoptedCount`

These come from raw node metrics:

- `cardano_node_metrics_Forge_forged_*`
- `cardano_node_metrics_Forge_adopted_*`

Important limitation:

- these are local node counters
- they do not prove that a block stayed on the canonical chain

They are useful for debugging, not for final production confirmation.

## Initial Hypotheses To Challenge

When operators suspect this issue, they often blame:

- no peers
- broken producer topology
- epoch boundary problems

Those are reasonable hypotheses, but they should be checked rather than assumed.

## Debugging Flow

### 1. Check Producer Runtime

Confirm the producer is actually running in producer mode:

- `CARDANO_BLOCK_PRODUCER=true`
- runtime forging args are present
- `forging enabled` is true

Useful checks:

```bash
kubectl -n <namespace> exec <pod> -c <container> -- sh -lc 'env | grep -E "^(CARDANO|METIS)_"'
kubectl -n <namespace> exec <pod> -c <container> -- sh -lc 'curl -s --fail http://127.0.0.1:<metricsPort>/metrics || wget -qO- http://127.0.0.1:<metricsPort>/metrics'
```

### 2. Check KES And OP Cert State

Confirm:

- `KES current / remaining`
- `KES expiration`
- `OP Cert disk | chain`

If these are unhealthy, fix them before chasing peers or topology.

### 3. Check Current Epoch Schedule

Confirm:

- `Leader`
- `Ideal`
- `Luck`
- `Next Block in`

If these exist and look reasonable, the epoch boundary is probably not the main issue.

### 4. Check Local Forging Counters

Inspect:

- `Forge_node_is_leader_*`
- `Forge_forged_*`
- `Forge_adopted_*`

Interpretation:

- if these are all zero, the producer may not be reaching actual leadership wins
- if they are increasing, the producer is doing local work even if external sources show no recent pool blocks

### 5. Check Producer Topology

Inspect the active `topology.json` inside the producer pod.

Recommended producer pattern:

- explicit topology
- relay-only `localRoots`
- empty `publicRoots`
- `useLedgerAfterSlot = -1`

Avoid:

- image-default topology on producers
- mixed local roots where one target is not operator-controlled
- public relay roots on producers

### 6. Check Actual Connections, Not Just Counts

Peer counts alone are not enough. Inspect the actual active sockets.

Useful checks:

```bash
kubectl -n <namespace> exec <pod> -c <container> -- sh -lc 'ss -tnp 2>/dev/null || netstat -tnp 2>/dev/null || true'
```

Confirm the producer is connected to the intended relay path.

### 7. Check Relay Health

Confirm the relay:

- is in sync
- has explicit topology when paired with a producer
- includes the producer in `localRoots`
- has healthy external peers
- keeps trusted public relay targets in `publicRoots`
- is actually receiving the producer connection

If the relay is unhealthy, the producer may still look locally healthy while blocks fail to propagate well.

Recommended relay pattern during producer troubleshooting:

- `localRoots`: producer only
- `publicRoots`: trusted public relays
- `useLedgerAfterSlot = 0`

### 8. Compare Against External Canonical Chain View

Use an external source that is trusted for canonical-chain observation.

If there is an external blockfrost source to check, one can do:

```bash
curl '$EXTERNAL_BLOCKFROST/pools/<pool-id>'
curl '$EXTERNAL_BLOCKFROST/pools/<pool-id>/blocks?order=desc&count=5'
```

Important limitation:

- this debugging flow still depends on an external source to confirm canonical pool blocks
- until native block outcome tracking exists, external confirmation is required

## How To Interpret Contradictions

### Case A. No peers, no forge activity

Likely causes:

- broken topology
- wrong relay target
- network/firewall issue

### Case B. Peers exist, schedule exists, forging enabled is true, but local forge counters stay zero

Likely causes:

- no winning slots in the observed window
- wrong pool identity or pool registration mismatch

### Case C. Peers exist, schedule exists, local forged/adopted counters increase, but external chain shows no recent pool blocks

This is the most subtle and important case.

Likely interpretation:

- the producer is forging or locally adopting blocks
- but those blocks are not surviving onto the canonical public chain

Probable causes to investigate next:

- propagation path / topology design
- producer favoring the wrong local root
- relay path not being the intended one
- relay still using image-default topology instead of explicit producer + public roots
- non-canonical outcomes such as ghosted/stolen behavior

Useful topology diagnostic:

- if the producer is forging and locally adopting blocks but those blocks do not
  survive onto the canonical chain, temporarily set the producer
  `useLedgerAfterSlot` back to `0`
- this allows the producer to discover more relays and peers than the strict
  steady-state private topology
- if that improves the outcome, the likely issue is relay topology or relay
  propagation quality rather than KES, op-cert, or basic forging state

Without native block outcome tracking, this cannot be classified exactly from local metrics alone.

## Recommended Fix Pattern

When topology is suspicious, simplify it aggressively.

Preferred producer topology:

- connect only to operator-controlled relay services
- remove extra custom local roots unless they are truly part of the intended architecture
- keep `publicRoots` empty
- keep `useLedgerAfterSlot = -1`

If using Metis managed topology, prefer `relay-service` mode.

Bootstrap exception:

- if the node is not yet acting as a true producer and is only being bootstrapped
  or synced before cutover, temporarily using `useLedgerAfterSlot = 0` can help
  it discover more peers and sync faster
- once bootstrap is complete, return the steady-state producer topology to
  `useLedgerAfterSlot = -1`

Preferred relay topology:

- explicit topology, not image-default, once a producer is attached
- producer in `localRoots`
- relay keeps a stable private path back to the producer in `localRoots`
- trusted public relays in `publicRoots`
- `useLedgerAfterSlot = 0`

## What This Skill Does Not Replace

This skill does not replace true local block outcome verification.

It helps agents distinguish between:

- basic runtime failure
- topology/connectivity issues
- local forge/adopt signals without canonical-chain confirmation

Until native block outcome tracking exists, final production confirmation still depends on an external source.
