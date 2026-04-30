# Apex Fusion Helm Chart

This chart packages a Apex fusion node and the supporting TCP proxy sidecar
used by Metis extensions.

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
  metricsPort: 42789
```

If you also need to change the internal proxy listener, set `proxy.n2cPort`.
The default generated nginx config follows that value automatically.

## Block Producer Runtime Material

Block producer mode now assumes Vault and Vault Secrets Operator are already
installed by the `control-plane` extension. Instead of creating a Kubernetes
secret yourself, write the runtime block producer artifacts to Vault ahead of
time and point the chart at that Vault KV path.

The required Vault fields are:

- `kes.skey`
- `vrf.skey`
- `op.cert`

The cold key and the operational certificate counter should stay outside the
cluster. Generate the KES key and operational certificate in your operator
workflow, then write only the runtime artifacts that the node needs into Vault.

You may also keep optional public or reference artifacts in the same Vault
record for future operator and agent discovery, for example:

- `kes.vkey`
- `vrf.vkey`
- `cold.vkey`
- `payment.addr`
- `reward.addr`
- `pool.id`
- `pool.id.bech32`
- pool metadata JSON and hash
- relay publication data

Interpretation:

- `kes.skey`, `vrf.skey`, and `op.cert` are the only fields the node needs to run
- the optional convenience artifacts are not required by the chart
- do not place online signing keys in this producer-mounted Vault path
- if you want operator-only material in Vault, use a salted `kv/operator/...`
  path that is not mounted into the producer pod

This chart assumes the `control-plane` extension is already installed and has
bootstrapped the shared Vault auth at `control-plane/default`. Normal block
producer configuration only needs the Vault KV path and block producer values.
That shared auth can read `kv/runtime/...` but cannot read `kv/operator/...`.

Example values:

```yaml
node:
  topology:
    mode: relay-service
    relayTargets:
      - releaseName: prime-testnet-relay
        namespace: prime-testnet-relay
        chart: apex-fusion
  blockProducer:
    enabled: true
    debug: false
    poolId: pool1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    vaultStaticSecret:
      path: runtime/apex-fusion/prime-mainnet-bp/block-producer
```

Write the artifacts to Vault before installing or upgrading the chart:

If Vault is only reachable inside the cluster, use the local `vault` CLI with a
port-forward first:

```shell
kubectl get svc -n control-plane
kubectl -n control-plane port-forward service/control-plane-vault 8200:8200
export VAULT_ADDR=http://localhost:8200
```

Then log in locally using your normal operator auth flow and upload the runtime
artifacts from local files:

```shell
vault kv put kv/runtime/apex-fusion/prime-mainnet-bp/block-producer \
  kes.skey=@/absolute/path/to/kes.skey \
  vrf.skey=@/absolute/path/to/vrf.skey \
  op.cert=@/absolute/path/to/op.cert
```

Only `kes.skey`, `vrf.skey`, and `op.cert` are required in this Vault path.
Optional public or reference artifacts may live beside them, but cold keys,
the operational certificate counter, and online signing keys should stay out of
the producer-mounted path.

If you want semi-cold operator storage in Vault, use a separate salted
operator path, for example
`kv/operator/apex-fusion/prime-mainnet-mypool-7f3c9d2a8e4b1f6c/...`. That is
safer than leaving sensitive files on an unprotected workstation filesystem,
but the ideal custody model for cold keys is still separate offline or
air-gapped devices.

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
      - releaseName: prime-testnet-relay
        namespace: prime-testnet-relay
        chart: apex-fusion
```

This renders a `topology.json` that points the producer at the relay service DNS
inside the cluster.

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
    useLedgerAfterSlot: -1
```

When `node.blockProducer.enabled=true`, the chart requires `node.topology.mode`
to be `relay-service` or `custom`.

Bootstrap note:

- for a future producer that is still being synced before it enters private
  steady-state producer operation, temporarily using `useLedgerAfterSlot: 0`
  can help it discover more peers and sync faster
- once the node is operating as a real private producer behind relays, prefer
  `useLedgerAfterSlot: -1`

## Upgrade A Relay To Block Producer

Recommended sequence:

1. Upload the runtime material to Vault.
2. Upgrade the existing relay into block producer `debug` mode first.
3. Confirm the dashboard shows block producer schedule metrics.
4. Disable `debug` to let the node start with the forging keys and operational certificate.

Example Vault upload:

```shell
vault kv put kv/runtime/apex-fusion/prime-mainnet-bp/block-producer \
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
      - releaseName: prime-testnet-relay
        namespace: prime-testnet-relay
        chart: apex-fusion
  blockProducer:
    enabled: true
    debug: true
    poolId: pool1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    vaultStaticSecret:
      path: runtime/apex-fusion/prime-mainnet-bp/block-producer
```

Upgrade the existing release with those values:

```shell
helm upgrade apex-fusion ./apex-fusion -f my-values.yaml
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
