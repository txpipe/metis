# Cardano Block Producer Verification

## Goal

Explain what can be verified today from the Metis dashboard and cluster state, and what still requires external confirmation.

## What Can Be Verified Today

The current dashboard and metrics pipeline can verify block-producer readiness and several critical runtime signals.

### Producer Schedule Metrics

- `Leader`
- `Ideal`
- `Luck`
- `Next Block in`

These come from `cardano-cli` schedule and stake data, with background caching for the leadership schedule.

### KES And Operational Certificate Metrics

- `KES current / remaining`
- `KES expiration`
- `OP Cert disk | chain`

In debug mode these come from `cardano-cli query kes-period-info` against the mounted `op.cert`.

### Cluster And Secret Readiness

- VaultStaticSecret is present
- synced Kubernetes Secret exists
- runtime material is mounted into the pod
- producer pod is healthy enough to expose metrics

## What Cannot Be Verified Natively Yet

The current dashboard does not yet provide authoritative end-to-end production outcome confirmation.

Not yet covered natively:

- adopted/confirmed/lost semantics
- blocklog-style production outcome tracking
- final proof that the node actually minted and the network accepted the block

## Current Operational Rule

For now, actual successful block production is still confirmed using a third-party process outside the dashboard.

Agents should communicate this clearly and avoid overstating what the current metrics prove.

## Verification Checklist

### Before Cutover

- relay is healthy
- debug-mode producer metrics are visible
- KES and op-cert metrics look sane
- `Next Block in` shows enough time for a safe switch

### Immediately After Cutover

- producer pod restarted correctly
- forging enabled is true
- `OP Cert disk | chain` remains aligned
- KES metrics still render

### After The First Expected Leadership Slot

- confirm schedule still looks sane
- confirm no obvious producer-side errors
- use the external confirmation process to verify actual production success

## Best Practices

- separate readiness from actual production success
- keep external confirmation in place until native outcome verification exists
- use debug mode to validate everything possible before enabling forging
