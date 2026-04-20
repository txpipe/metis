# Cardano Node Helm Chart

This chart packages a Cardano node and the supporting TCP proxy sidecar used by
Metis extensions.

## Features

- StatefulSet with persistent storage for the node database directory.
- First-class managed `topology.json` support for relay-service and custom
  producer connectivity.
- Optional ConfigMap for shipping custom configuration files to the container.
- Sidecar `nginx` TCP proxy that exposes the node socket on the n2c port.
- Services for headless pod identity and traffic access (n2n, n2c, metrics).
- PodMonitor opt-in for Prometheus Operator installations.
- Fully templated container resources and tolerations.

## Getting Started

Install the chart by pointing Helm at the directory:

```shell
helm install cardano-node ./cardano-node \
  --set node.network=preprod \
  --set node.networkMagic=1
```

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
    value: cardano
    effect: NoSchedule
```

Apply the overrides with `helm install cardano-node ./cardano-node -f my-values.yaml`.

## Service Ports

`service.n2nPort`, `service.n2cPort`, and `service.metricsPort` control the
Service-facing ports only. Internal container ports stay on the fixed defaults
unless you explicitly override them with `node.n2nPort`, `node.metricsPort`, or
`proxy.n2cPort`.

Example using custom Service ports while keeping the default internal ports:

```yaml
service:
  n2nPort: 4000
  n2cPort: 4307
  metricsPort: 42798
```

If you also need to change the internal proxy listener, set `proxy.n2cPort`.
The default generated nginx config follows that value automatically.

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
  topology:
    mode: relay-service
    relayTargets:
      - releaseName: preview-relay
        namespace: preview-relay
        chart: cardano-node
  blockProducer:
    enabled: true
    debug: false
    poolId: pool1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    vaultStaticSecret:
      path: cardano-node/mainnet-bp/block-producer
```

Write the artifacts to Vault before installing or upgrading the chart:

```shell
vault kv put kv/cardano-node/mainnet-bp/block-producer \
  kes.skey=@kes.skey \
  vrf.skey=@vrf.skey \
  op.cert=@op.cert
```

When `node.blockProducer.enabled=true` the chart creates a namespace-local
`vault-auth` service account and a `VaultStaticSecret` that references the
shared `control-plane/default` Vault auth. Vault Secrets Operator syncs the
Kubernetes Secret that the StatefulSet mounts, and Cardano node starts with the
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

## Producer Topology

For block producers, prefer explicit relay connectivity instead of relying on
the image-default public topology.

Supported modes:

- `image-default`: use the image-provided `topology.json`
- `relay-service`: generate `topology.json` from one or more relay releases
- `custom`: supply explicit `localRoots` and `publicRoots`

If you leave `node.topology.mode` at `image-default`, the chart does not mount a
managed `topology.json` and the workload keeps using the image default exactly
as before.

Recommended producer example using a relay release:

```yaml
node:
  topology:
    mode: relay-service
    relayTargets:
      - releaseName: preview-relay
        namespace: preview-relay
        chart: cardano-node
```

Custom topology example:

```yaml
node:
  topology:
    mode: custom
    localRoots:
      - accessPoints:
          - address: 54.209.123.202
            port: 6000
        advertise: false
        valency: 1
    publicRoots: []
    useLedgerAfterSlot: 0
```

When `node.blockProducer.enabled=true`, the chart requires `node.topology.mode`
to be `relay-service` or `custom`.

## Upgrade A Relay To Block Producer

Recommended sequence:

1. Upload the runtime material to Vault.
2. Upgrade the existing relay into block producer `debug` mode first.
3. Confirm the dashboard shows block producer schedule metrics.
4. Disable `debug` to let the node start with the forging keys and operational certificate.

Example Vault upload:

```shell
vault kv put kv/cardano-node/mainnet-bp/block-producer \
  kes.skey=@kes.skey \
  vrf.skey=@vrf.skey \
  op.cert=@op.cert
```

Example upgrade values for debug mode:

```yaml
node:
  topology:
    mode: relay-service
    relayTargets:
      - releaseName: preview-relay
        namespace: preview-relay
        chart: cardano-node
  blockProducer:
    enabled: true
    debug: true
    poolId: pool1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    vaultStaticSecret:
      path: cardano-node/mainnet-bp/block-producer
```

Upgrade the existing release with those values:

```shell
helm upgrade cardano-node ./cardano-node -f my-values.yaml
```

In this mode the StatefulSet still mounts `kes.skey`, `vrf.skey`, and `op.cert`,
and the dashboard can compute `Leader`, `Ideal`, `Luck`, and `Next Block in`, but
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
helm upgrade cardano-node ./cardano-node -f my-values.yaml
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
