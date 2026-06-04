# Apex Fusion Block Producer Helm Chart

This chart deploys a private Apex Fusion block producer with optional managed relays in the same Helm release. It is intentionally opinionated for Supernode: persistent storage is always enabled, the n2c proxy is chart-owned, and producer runtime material is sourced only through VaultStaticSecret.

## Install

```shell
helm install vector-producer ./apex-fusion-block-producer \
  --set node.network=vector-testnet \
  --set persistence.storageClass=standard \
  --set relayPersistence.storageClass=standard \
  --set blockProducer.vaultStaticSecret.path=runtime/apex-fusion/vector-producer/block-producer
```

By default `relays.count=1`, which creates one managed relay and points the producer topology at it. If you set `relays.count=0`, provide `relays.trusted` explicitly. MCP does not auto-resolve trusted relays; use `workloads.list` to inspect candidate `apex-fusion-relay` workloads.

The block producer Service is always kept private as `ClusterIP`. The configurable `service.*` values apply to the managed relay Services created by this chart. Set `service.type=LoadBalancer` only when those managed relays need external node-to-node connectivity.

Set `blockProducer.debug=true` to mount the producer material and expose metrics wiring without passing forging flags to the node.

Use `values.schema.json` as the public configuration contract for Helm, MCP, and LLM clients.
