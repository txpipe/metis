# Control Plane Helm Chart

This chart deploys the observability control-plane required to operate a SuperNode.
It packages the Prometheus Operator stack, persistent monitoring backends, and the
node-level collectors that the SuperNode expects to find in-cluster.

## Features

- Deploys Prometheus Operator v0.63.0 with its ClusterRoles, bindings, and headless Service.
- Ships the Prometheus Operator CRDs (Prometheus, Alertmanager, ServiceMonitor, etc.) so the stack can reconcile its custom resources.
- Provisions a Prometheus instance with 30-day retention, RBAC-secured scraping, and a 40Gi persistent volume bound with the chosen `storageClass`.
- Runs Grafana in a single-replica StatefulSet with a 5Gi persistent volume and a ConfigMap-provided `grafana.ini` tuned for UTC dashboards.
- Installs kube-state-metrics behind kube-rbac-proxy sidecars together with a headless Service and ServiceMonitor for TLS-protected scraping.
- Deploys node-exporter as a DaemonSet with kube-rbac-proxy, host PID/network access, and a matching ServiceMonitor so every node exposes metrics on port `9100`.

## Getting Started

```shell
helm install control-plane oci://oci.supernode.store/control-plane \
  --namespace control-plane \
  --create-namespace
```

By default the chart targets the `control-plane` namespace and binds persistent
volumes with the `gp` storage class. Override these knobs to match your cluster:

```shell
helm install control-plane ./control-plane \
  --namespace control-plane \
  --create-namespace \
  --set storageClass=standard
```

Component-level tolerations are configurable via values such as
`prometheusOperator.tolerations` or `grafana.tolerations`, which default to empty
lists when omitted.

> **Note:** The bundled `Prometheus` resource references an `alertmanager` Service
> in the same namespace. Provide an Alertmanager deployment (either manually or
> through a future chart addition) if you plan to send alerts.

## Accessing the Stack

- Port-forward Grafana to inspect dashboards: `kubectl -n control-plane port-forward service/grafana 3000:3000`
- Port-forward Prometheus to verify scrape targets: `kubectl -n control-plane port-forward pod/prometheus-prometheus-0 9090:9090`
- Check that the collectors are healthy: `kubectl -n control-plane get pods`

Grafana keeps the upstream defaults for credentials (`admin`/`admin`). Make sure
to rotate them or layer your preferred authentication in production.

#### Manual steps

In order to use Grafana to visualize Prometheus metrics you will need to [add
the corresponding datasource](https://grafana.com/docs/grafana/latest/datasources/prometheus/configure/).
The URL is `http://prometheus-operated:9090`.

## Local Testing With kind

You can exercise the chart end-to-end inside a local
[kind](https://kind.sigs.k8s.io/) cluster. Install `kind`, `kubectl`, `helm`,
and (optionally) `kubeconform` first.

1. Create and target a cluster:

   ```shell
   kind create cluster --name supernode
   kubectl config use-context kind-supernode
   ```

2. (Optional) Point the chart at the default kind storage class:

   ```shell
   cat <<'EOF' > control-plane-kind-values.yaml
   storageClass: standard
   EOF
   ```

3. Install the chart:

   ```shell
   helm install control-plane ./control-plane \
     --namespace control-plane \
     --create-namespace \
     --values control-plane-kind-values.yaml
   ```

4. Wait for the workloads to settle and inspect logs:

   ```shell
   kubectl -n control-plane get pods
   kubectl -n control-plane logs deployment/prometheus-operator
   ```

5. When finished, clean up:

   ```shell
   helm -n control-plane uninstall control-plane
   kind delete cluster --name control-plane
   ```

## Testing

Run the same checks used in CI from `extensions/control-plane`:

```shell
helm lint .
helm template control-plane . | kubeconform -strict -summary -
```

## Key Values

| Value | Description | Default |
|-------|-------------|---------|
| `namespace` | Namespace that hosts all control-plane resources | `control-plane` |
| `storageClass` | Storage class used by Prometheus and Grafana persistent volumes | `standard` |
| `prometheusOperator.tolerations` | Tolerations applied to the Prometheus Operator deployment | `[]` |
| `grafana.tolerations` | Tolerations applied to the Grafana StatefulSet | `[]` |
| `kubeStateMetrics.tolerations` | Tolerations applied to the kube-state-metrics deployment | `[]` |
| `nodeExporter.tolerations` | Tolerations applied to the node-exporter DaemonSet | `[]` |
| `prometheus.tolerations` | Tolerations applied to the Prometheus CRD | `[]` |

Consult `values.yaml` for the authoritative list.

## Maintenance

- Keep the Prometheus Operator, kube-state-metrics, and node-exporter image tags in sync with upstream security releases.
- Update the CRDs under `crds/` whenever the operator version is bumped.
- Review the bundled Grafana configuration and dashboards as team requirements evolve.
- Double-check tolerations and node selectors when adjusting SuperNode scheduling policies so the control-plane keeps landing on the intended nodes.
