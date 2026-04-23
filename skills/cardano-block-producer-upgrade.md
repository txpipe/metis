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

## Producer Topology Rules

For a real Cardano block producer, topology is part of the security model, not
just a connectivity convenience.

Recommended producer behavior:

- the producer should only connect to its own relays, or relays you explicitly trust
- `localRoots` should contain only those relays
- `publicRoots` should be empty
- `useLedgerAfterSlot` should be `-1` so ledger peers stay disabled on the producer
- do not rely on image-default topology for a producer

Operational interpretation for Metis:

- `relay-service` is the preferred managed mode when the relays are other Metis releases
- `custom` is appropriate when you need to set explicit `localRoots`
- `image-default` is acceptable for relays, but not recommended for producers

This matches both current Cardano documentation and the typical
`guild-operators` block producer pattern.

## Relay Topology Rules

Once a producer is paired with a relay, the relay should also move to explicit
topology.

Recommended relay behavior:

- `localRoots` should include the producer path
- `publicRoots` should remain populated with trusted public relay targets
- `useLedgerAfterSlot` should remain `0`
- do not leave the relay on image-default topology once you are debugging real producer propagation issues

Operational interpretation for Metis:

- if the producer is explicit and private, the relay should be explicit too
- the relay should trust and maintain a private path to the producer
- the relay should still maintain public network connectivity through `publicRoots`

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
- the relay topology is known and intentional
- the namespace is correct
- the chart is the expected one
- control-plane shared Vault resources exist

## Upload Runtime Material To Vault

Use the chart-specific path pattern you manage operationally.

### Local Vault Upload Workflow

If Vault is only reachable inside the cluster, use the local `vault` CLI with a
`kubectl port-forward`.

1. Make sure the `vault` CLI is installed on your machine.
2. Discover the Vault service if needed:

```bash
kubectl get svc -n control-plane
```

3. Port-forward the Vault HTTP port locally. In this cluster the service is
   `control-plane-vault`:

```bash
kubectl -n control-plane port-forward service/control-plane-vault 8200:8200
```

4. Point the local CLI at the forwarded address:

```bash
export VAULT_ADDR=http://localhost:8200
```

5. Log in locally using your normal operator authentication flow.
6. Upload only the runtime block-producer material from local files.

Rules:

- upload only `kes.skey`, `vrf.skey`, and `op.cert`
- keep cold keys offline
- keep the operational certificate counter outside the cluster
- prefer absolute local file paths to avoid ambiguity

Example shape:

```bash
vault kv put kv/<path> \
  kes.skey=@/absolute/path/to/kes.skey \
  vrf.skey=@/absolute/path/to/vrf.skey \
  op.cert=@/absolute/path/to/op.cert
```

## Debug-First Upgrade

Prepare values like:

```yaml
node:
  topology:
    mode: relay-service
    useLedgerAfterSlot: -1
    relayTargets:
      - releaseName: <relay-release>
        namespace: <relay-namespace>
        chart: <relay-chart-name>
  blockProducer:
    enabled: true
    debug: true
    poolId: pool1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    vaultStaticSecret:
      path: <vault-kv-path>
```

If you need a fully explicit topology instead of generated relay service
targets, use `custom` and keep the producer private:

```yaml
node:
  topology:
    mode: custom
    localRoots:
      - accessPoints:
          - address: relay-a.<namespace>.svc.cluster.local
            port: 3000
          - address: relay-b.<namespace>.svc.cluster.local
            port: 3000
        advertise: false
        trustable: true
        hotValency: 2
    publicRoots: []
    useLedgerAfterSlot: -1
```

Notes for the custom form:

- only put your relays in `localRoots`
- set `hotValency` to the number of relay connections you want the producer to keep active
- omit public internet peers from producer topology entirely
- if you operate raw Cardano config outside these charts, keep `PeerSharing=false` on the producer

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

Also confirm:

- peer counts are non-zero
- the producer topology points only at the intended relay or custom local root
- producer `publicRoots` are empty
- producer `useLedgerAfterSlot` is `-1`
- if Dolos is available for the same network, use it as the preferred external chain view during later validation

### Important Notes Learned From Implementation

- `prime-testnet` must use network magic `3311`
- Apex Fusion derives built-in testnet magic automatically for supported networks
- block producers should not rely on image-default public topology; set `node.topology.mode` explicitly
- `relay-service` is the easiest managed mode for developers
- `custom` can be used for explicit local roots, but producers should point only at their relays
- for producers, `publicRoots` should be empty and `useLedgerAfterSlot` should be `-1`
- using `image-default` for a relay is fine as a starting point, but producers should always be explicit
- guild-operators follows the same producer pattern: relay-only `localRoots`, empty `publicRoots`, ledger peers disabled
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
3. Confirm the relay has explicit topology: producer in `localRoots`, public relays in `publicRoots`.
3. Wait for a comfortable window before the next scheduled slot.
4. Shut down or retire the currently active producer according to your operational process.
5. Switch the current release from debug mode to full producer mode.
6. Confirm the producer still has peers and `forging enabled` becomes true.

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
- peer counts are still non-zero
- `OP Cert disk | chain` is aligned
- KES metrics still look healthy
- schedule metrics still render
- if Dolos is deployed for this network, use its minibf endpoint as the preferred external source when checking whether canonical pool blocks appear

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
- keep producer topology private and minimal
- do not treat dashboard readiness as proof of produced blocks; external confirmation is still required for now
