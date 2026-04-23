# Dolos Helm Chart

This chart deploys [Dolos](https://github.com/txpipe/dolos).

## Features

- StatefulSet with persistent storage for the Dolos data directory.
- ConfigMap based configuration with presets for Cardano `cardano-mainnet`,
  `cardano-preprod`, `cardano-preview`, `prime-testnet`, and `prime-mainnet`.
- Override support for supplying custom Dolos configuration content or an
  existing ConfigMap.
- Tunable resources, tolerations, topology constraints, and probe configuration.
- Service exposure for `grpc`, `minibf`, `minikupo`, and `trp`.
  `ouroboros` remains internal-only.

## Configuration

By default the chart resolves the preset from `dolos.network` and writes it to
`/etc/config/dolos.toml`. For the existing Cardano presets, you still need to
provide `config.upstreamAddress` with the address of a trusted Cardano relay.
Adjust as needed:

```yaml
dolos:
  network: cardano-mainnet
```

Override the trusted upstream relay if needed:

```yaml
config:
  upstreamAddress: "trusted-relay.example.org:3000"
```

To inject a completely custom configuration set `config.customConfig`:

```yaml
config:
  preset: ""
  customConfig: |-
    [upstream]
    peer_address = "custom-relay:3000"
    network_magic = 1234
    is_testnet = true
```

If you already manage configuration elsewhere, disable generation and reference
your own ConfigMap:

```yaml
config:
  create: false
  existingConfigMap: my-dolos-config
```

### Persistence

Persistent volume claims are provisioned unless `persistence.enabled` is set to
`false`. Supply `persistence.existingClaim` to reuse an existing claim.
