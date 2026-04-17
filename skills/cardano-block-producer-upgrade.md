# Cardano Block Producer Upgrade

## Goal

Upgrade an existing healthy relay to block-producer mode for an existing pool, using a debug-first rollout.

## Assumptions

- The relay is already installed and healthy.
- The pool already exists.
- The operator controls the pool's runtime artifacts outside the cluster.
- `control-plane` and the shared Vault integration are already working.

## Block Producer Modes

- `enabled=false, debug=false`: relay
- `enabled=false, debug=true`: relay
- `enabled=true, debug=true`: relay runtime with block-producer metrics and secret mount
- `enabled=true, debug=false`: full block producer

`debug=true` only changes behavior when `enabled=true`.

## Required Inputs

- target release name
- target namespace
- chart path
- pool id
- Vault KV path for runtime material
- chart/network values already used by the relay

## Runtime Material

Only runtime block-producer material belongs in-cluster:

- `kes.skey`
- `vrf.skey`
- `op.cert`

Best practices:

- cold keys stay offline
- operational certificate counter handling stays outside the cluster
- use offline workflow for sensitive operations
- never put cold keys on online nodes

## Preflight Checks

Before touching the release:

```bash
helm list -A -o json
kubectl get pods -n <namespace>
kubectl get pvc -n <namespace>
kubectl get vaultauth -n control-plane
kubectl get vaultstaticsecret -n <namespace>
```

Confirm:

- the relay is healthy
- the namespace is correct
- the chart is the expected one
- control-plane shared Vault resources exist

## Upload Runtime Material To Vault

Use the chart-specific path pattern you manage operationally.

Example shape:

```bash
vault kv put kv/<path> \
  kes.skey=@kes.skey \
  vrf.skey=@vrf.skey \
  op.cert=@op.cert
```

## Debug-First Upgrade

Prepare values like:

```yaml
node:
  blockProducer:
    enabled: true
    debug: true
    poolId: pool1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    vaultStaticSecret:
      path: <vault-kv-path>
```

Apply with:

```bash
helm upgrade <release> <chart-path> -n <namespace> -f my-values.yaml
```

In debug mode:

- runtime BP secret is mounted
- BP metrics are enabled
- node still behaves like a relay
- forging runtime flags are not added

## Debug Validation Checklist

### Kubernetes Checks

```bash
kubectl get pods -n <namespace>
kubectl get vaultstaticsecret -n <namespace>
kubectl get secret -n <namespace>
kubectl describe pod -n <namespace> <pod-name>
```

Confirm:

- pod restarted successfully
- VaultStaticSecret is healthy
- synced Kubernetes secret exists
- runtime material is mounted into the pod

### Dashboard Checks

Confirm these producer metrics are visible:

- `Leader`
- `Ideal`
- `Luck`
- `Next Block in`
- `KES current / remaining`
- `KES expiration`
- `OP Cert disk | chain`

### Important Notes Learned From Implementation

- `prime-testnet` must use network magic `3311`
- Apex Fusion derives built-in testnet magic automatically for supported networks
- `leadership-schedule` may be slow on first run
- schedule metrics use cached/background refresh, so the first refresh may need warm-up time
- in debug mode, KES metrics come from `cardano-cli query kes-period-info`, not from node metrics
- `OP Cert disk | chain` also comes from `cardano-cli query kes-period-info`

## Safe Switchover Procedure

Do not switch into active forging near an expected block.

Use `Next Block in` to choose a safe cutover window.

Recommended sequence:

1. Run in debug mode first.
2. Confirm producer metrics are healthy.
3. Wait for a comfortable window before the next scheduled slot.
4. Shut down or retire the currently active producer according to your operational process.
5. Switch the current release from debug mode to full producer mode.

## Final Producer Activation

Change only:

```yaml
node:
  blockProducer:
    enabled: true
    debug: false
```

Upgrade again:

```bash
helm upgrade <release> <chart-path> -n <namespace> -f my-values.yaml
```

This activates the runtime flags:

- `--shelley-kes-key`
- `--shelley-vrf-key`
- `--shelley-operational-certificate`

## Post-Cutover Checks

Validate:

- pod restarted cleanly
- forging is enabled
- `OP Cert disk | chain` is aligned
- KES metrics still look healthy
- schedule metrics still render

## Rollback Guidance

If the active producer rollout looks suspicious:

1. revert to:

```yaml
node:
  blockProducer:
    enabled: true
    debug: true
```

2. upgrade again
3. return to a non-forging but observable state

## Best Practices

- cold keys remain offline
- only runtime material enters the cluster
- keep system time accurate
- avoid switching close to an assigned slot
- do not treat dashboard readiness as proof of produced blocks; external confirmation is still required for now
