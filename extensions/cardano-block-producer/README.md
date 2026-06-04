# Cardano Block Producer Helm Chart

This chart deploys one private Cardano block producer and, optionally, managed
relays in the same Helm release.

## Configuration

Minimal values with one managed relay:

```yaml
node:
  network: preview

persistence:
  storageClass: standard

relayPersistence:
  storageClass: standard

blockProducer:
  poolId: pool1...
  vaultStaticSecret:
    path: runtime/cardano-block-producer/my-pool/block-producer

relays:
  count: 1
```

If `relays.count` is greater than zero, the chart creates managed relay
StatefulSets and points the producer `topology.json` at those relay Services.

The block producer Service is always kept private as `ClusterIP`. The
configurable `service.*` values apply to the managed relay Services created by
this chart. Set `service.type: LoadBalancer` only when those managed relays
need external node-to-node connectivity.

If `relays.count` is `0`, provide at least one trusted relay:

```yaml
relays:
  count: 0
  trusted:
    - address: relay-preview.cardano.svc.cluster.local
      port: 3000
```

MCP does not auto-resolve trusted relays. If the address is unknown, run
`workloads.list` and inspect same-network `cardano-relay` workloads.

## Debug Mode

Set `blockProducer.debug: true` to mount producer runtime material and expose
schedule metrics without passing forging flags to `cardano-node`. Set it back to
`false` to run as an active block producer.

## Runtime Material

The Vault path referenced by `blockProducer.vaultStaticSecret.path` must contain:

- `kes.skey`
- `vrf.skey`
- `op.cert`

Cold keys and operational certificate counters must stay outside the
producer-mounted runtime path.
