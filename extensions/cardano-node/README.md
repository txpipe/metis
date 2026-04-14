# Cardano Node Helm Chart

This chart packages a Cardano node and the supporting TCP proxy sidecar used by
Metis extensions.

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

Example values:

```yaml
vaultAuth:
  ref: control-plane/default

node:
  blockProducer:
    enabled: true
    mountPath: /block-producer
    kesKeyFile: kes.skey
    vrfKeyFile: vrf.skey
    operationalCertificateFile: op.cert
    vaultStaticSecret:
      mount: kv
      path: cardano-node/mainnet-bp/block-producer
      refreshAfter: 1m
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

If the Vault path is missing or incomplete, the synced Kubernetes secret will
not be ready and the block producer will not start correctly. This is expected:
the runtime material must exist in Vault first.
