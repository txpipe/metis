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
  --set node.network=vector-testnet \
  --set node.networkMagic=1
```

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
