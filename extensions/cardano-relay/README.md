# Cardano Relay Helm Chart

This chart deploys a Supernode-opinionated Cardano relay node.

## Configuration

The chart values are the public configuration interface. Minimal values:

```yaml
node:
  network: preview

persistence:
  storageClass: standard
```

The chart always creates a ServiceAccount, always enables persistent storage,
uses a fixed image pull policy, and includes an nginx node-to-client proxy.

## Resources

Use `resources.requests.cpu: 500m` for all networks. For memory, set requests
and limits to the same value: `8Gi` for `mainnet`, and `4Gi` for `preview` or
`preprod`. CPU limits should usually be `4` for mainnet and `2` for preview or
preprod.

## Topology

Leave `node.topology.mode: image-default` for normal public relay behavior. Use
`relay-service` or `custom` only when you need explicit in-cluster or fully
custom topology roots.
