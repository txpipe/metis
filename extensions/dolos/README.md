# Dolos Helm Chart

This chart deploys [Dolos](https://github.com/txpipe/dolos).

## Features

- StatefulSet with persistent storage for the Dolos data directory.
- ConfigMap based configuration with presets for Cardano `cardano-mainnet`,
  `cardano-preprod`, and `cardano-preview`.
- Opinionated Supernode deployment shape: generated ServiceAccount, generated
  ConfigMap at `/etc/config/dolos.toml`, fixed readiness probe, and fixed
  termination grace period.
- Optional override support for supplying complete custom Dolos configuration
  content.
- Tunable resources, storage, service exposure, tolerations, and topology
  constraints.
- Service exposure for `grpc`, `minibf`, `minikupo`, and `trp`.
  `ouroboros` remains internal-only.

## Configuration

The chart values are the public configuration interface. By default the chart
resolves the preset from `dolos.network` and writes it to
`/etc/config/dolos.toml`. For all built-in Cardano presets, you must provide
`config.upstreamAddress` with the address of a trusted Cardano relay. MCP does
not auto-resolve this value; use `workloads.list` to inspect installed
same-network relay workloads and derive a candidate address.

Minimal values:

```yaml
dolos:
  network: cardano-preview

persistence:
  storageClass: standard

config:
  upstreamAddress: "trusted-relay.example.org:3000"
```

To inject a completely custom configuration set `config.customConfig`:

```yaml
config:
  upstreamAddress: "trusted-relay.example.org:3000"
  customConfig: |-
    [upstream]
    peer_address = "custom-relay:3000"
    network_magic = 1234
    is_testnet = true
```

### Persistence

Persistent storage is always enabled for Dolos. Set `persistence.storageClass`
for normal Supernode installs. Supply `persistence.existingClaim` only when
reusing an existing initialized claim.

### Resources

Use `resources.requests.cpu: 500m` for all networks. For memory, set requests
and limits to the same value: `8Gi` for `cardano-mainnet`, and `4Gi` for
`cardano-preview` or `cardano-preprod`. CPU limits should usually be `4` for
mainnet and `2` for preview or preprod.
