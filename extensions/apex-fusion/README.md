# Apex Fusion Helm Chart

This chart packages a Apex fusion node and the supporting TCP proxy sidecar
used by Metis extensions.

## Features

- StatefulSet with persistent storage for the node database directory.
- Optional ConfigMap for shipping custom `config.json` / `topology.json`
  content to the container.
- Sidecar `nginx` TCP proxy that exposes the node socket on the n2c port.
- Services for headless pod identity and traffic access (n2n, n2c, metrics).
- PodMonitor opt-in for Prometheus Operator installations.
- Fully templated container resources and tolerations.

## Getting Started

Install the chart by pointing Helm at the directory:

```shell
helm install apex-fusion ./apex-fusion \
  --set node.network=vector-testnet
```

`node.networkMagic` is derived automatically for the built-in test networks.
Override it only if you need a non-standard value.

Only the `vector-testnet`, `prime-testnet` and `prime-mainnet` networks are currently exposed. Support for
`vector-mainnet` will be added in a future release.

Resources and tolerations are off by default so the manifests stay minimal.
Override them as needed:

```yaml
resources:
  limits:
    cpu: 1000m
    memory: 6Gi
  requests:
    cpu: 500m
    memory: 4Gi

tolerations:
  - key: workload
    operator: Equal
    value: vector
    effect: NoSchedule
```

Apply the overrides with `helm install apex-fusion ./apex-fusion -f my-values.yaml`.

## Block Producer Runtime Material

Block producer mode now assumes Vault and Vault Secrets Operator are already
installed by the `control-plane` extension. Instead of creating a Kubernetes
secret yourself, write the runtime block producer artifacts to Vault ahead of
time and point the chart at that Vault KV path.

The expected Vault fields are:

- `kes.skey`
- `vrf.skey`
- `op.cert`

The cold key and the operational certificate counter should stay outside the
cluster. Generate the KES key and operational certificate in your operator
workflow, then write only the runtime artifacts that the node needs into Vault.

This chart assumes the `control-plane` extension is already installed and has
bootstrapped the shared Vault auth at `control-plane/default`. Normal block
producer configuration only needs the Vault KV path and block producer values.

Example values:

```yaml
node:
  blockProducer:
    enabled: true
    debug: false
    mountPath: /block-producer
    kesKeyFile: kes.skey
    vrfKeyFile: vrf.skey
    operationalCertificateFile: op.cert
    vaultStaticSecret:
      mount: kv
      path: apex-fusion/prime-mainnet-bp/block-producer
      refreshAfter: 1m
```

Write the artifacts to Vault before installing or upgrading the chart:

```shell
vault kv put kv/apex-fusion/prime-mainnet-bp/block-producer \
  kes.skey=@kes.skey \
  vrf.skey=@vrf.skey \
  op.cert=@op.cert
```

When `node.blockProducer.enabled=true` the chart creates a namespace-local
`vault-auth` service account and a `VaultStaticSecret` that references the
shared `control-plane/default` Vault auth. Vault Secrets Operator syncs the
Kubernetes Secret that the StatefulSet mounts, and the node starts with the
following flags:

- `--shelley-kes-key`
- `--shelley-vrf-key`
- `--shelley-operational-certificate`

Set `node.blockProducer.debug=true` to keep the block producer secret and
schedule metrics wiring while suppressing those runtime flags. In that mode the
node still runs like a relay, but the pod mounts the block producer material so
the dashboard can calculate leader schedule metrics. When
`node.blockProducer.enabled=false`, `debug` has no effect.

If the Vault path is missing or incomplete, the synced Kubernetes secret will
not be ready and the block producer will not start correctly. This is expected:
the runtime material must exist in Vault first.

## Upgrade A Relay To Block Producer

Recommended sequence:

1. Upload the runtime material to Vault.
2. Upgrade the existing relay into block producer `debug` mode first.
3. Confirm the dashboard shows block producer schedule metrics.
4. Disable `debug` to let the node start with the forging keys and operational certificate.

Example Vault upload:

```shell
vault kv put kv/apex-fusion/prime-mainnet-bp/block-producer \
  kes.skey=@kes.skey \
  vrf.skey=@vrf.skey \
  op.cert=@op.cert
```

Example upgrade values for debug mode:

```yaml
node:
  blockProducer:
    enabled: true
    debug: true
    poolId: pool1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    vaultStaticSecret:
      path: apex-fusion/prime-mainnet-bp/block-producer
```

Upgrade the existing release with those values:

```shell
helm upgrade apex-fusion ./apex-fusion -f my-values.yaml
```

In this mode the StatefulSet still mounts `kes.skey`, `vrf.skey`, and `op.cert`,
and the dashboard can compute `Leader`, `Ideal`, `Luck`, and `Next block`, but
the chart does not add the runtime flags below:

- `--shelley-kes-key`
- `--shelley-vrf-key`
- `--shelley-operational-certificate`

Once the metrics look correct, switch to full producer mode by changing only:

```yaml
node:
  blockProducer:
    enabled: true
    debug: false
```

Then upgrade again:

```shell
helm upgrade apex-fusion ./apex-fusion -f my-values.yaml
```

At that point the node starts with the runtime block producer flags and behaves
as a full producer instead of a relay.

## Advanced Override

Superusers can override the shared Vault auth reference if the cluster does not
use the default `control-plane/default` resource:

```yaml
vaultAuth:
  ref: some-other-namespace/custom-auth
```

Most operators should not need to set this.
