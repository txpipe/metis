# Cardano Node Helm Chart

This chart packages a Cardano node and the supporting TCP proxy sidecar used by
Metis extensions. It distills the Terraform bootstrap module into a reusable
Helm deployment that follows the same conventions as the Midnight extension
chart in this repository while omitting the Demeter, Vector, and Prime specific
logic.

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
