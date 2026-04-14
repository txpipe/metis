# Control Plane Helm Chart

This chart deploys the observability control-plane required to operate a SuperNode.
It packages the Prometheus Operator stack and persistent monitoring backends that
the SuperNode expects to find in-cluster.

## Features

- Deploys Prometheus Operator v0.63.0 with its ClusterRoles, bindings, and headless Service.
- Ships the Prometheus Operator CRDs (Prometheus, Alertmanager, ServiceMonitor, etc.) so the stack can reconcile its custom resources.
- Provisions a Prometheus instance with 30-day retention, RBAC-secured scraping, and a 40Gi persistent volume bound with the chosen `storageClass`.
- Runs Grafana in a single-replica StatefulSet with a 5Gi persistent volume and a ConfigMap-provided `grafana.ini` tuned for UTC dashboards.
- Deploys Vault OSS together with Vault Secrets Operator for Kubernetes-native secret sync.
- Creates cluster-shared `VaultConnection` and `VaultAuth` resources in the control-plane namespace so other namespaces can reference `control-plane/default`.

## Getting Started

```shell
helm install control-plane oci://oci.supernode.store/control-plane \
  --namespace control-plane \
  --create-namespace
```

By default the chart targets the `control-plane` namespace and binds persistent
volumes with the `gp` storage class. Override these knobs to match your cluster:

```shell
helm dependency build ./control-plane
helm install control-plane ./control-plane \
  --namespace control-plane \
  --create-namespace \
  --set storageClass=standard
```

Component-level tolerations are configurable via values such as
`prometheusOperator.tolerations` or `grafana.tolerations`, which default to empty
lists when omitted.

If you override `storageClass`, also override `vault.server.dataStorage.storageClass`
so the Vault Raft PVCs land on the same storage backend.

Vault phase 1 is enabled by default. The chart deploys Vault and Vault Secrets
Operator, but the exact Vault server mode is configurable.

## Deployment Modes

The chart supports three Vault server modes:

| Mode | Values | Intended use |
|------|--------|--------------|
| HA Raft | `vault.server.ha.enabled=true`, `vault.server.ha.raft.enabled=true` | Production default |
| Standalone | `vault.server.standalone.enabled=true` | Persistent single-node installs |
| Dev | `vault.server.dev.enabled=true` | Local or disposable development only |

Exactly one of those modes must be enabled at a time. The default chart values
use HA Raft. VSO remains enabled in all three modes, including dev mode.

The control-plane chart treats Vault and VSO as cluster-wide dependencies. The
rendered `VaultAuth` in `control-plane` is intended to be referenced from other
namespaces via `vaultAuthRef: control-plane/default`.

For non-dev deployments, the default seal mode is Shamir. Configure
`vault.integration.seal.mode` for cloud-native auto unseal (`awskms`,
`gcpckms`, `azurekeyvault`, `ocikms`) or Transit-based auto unseal (`transit`)
before installation if you want Vault to start unsealed automatically.

Seal settings are ignored by Vault dev mode.

## Provider Examples

Provider-specific example values live under `examples/`:

- `examples/aws-values.yaml`
- `examples/gcp-values.yaml`
- `examples/azure-values.yaml`
- `examples/oci-values.yaml`
- `examples/standalone-values.yaml`
- `examples/dev-values.yaml`
- `examples/bare-metal-shamir-values.yaml`
- `examples/bare-metal-transit-values.yaml`

Use them as starting points rather than production-ready drop-ins. For example:

```shell
helm dependency build ./control-plane
helm install control-plane ./control-plane \
  --namespace control-plane \
  --create-namespace \
  --values ./control-plane/examples/aws-values.yaml
```

## Seal Mode Guidance

| Environment | Recommended seal mode | Notes |
|-------------|-----------------------|-------|
| AWS EKS | `awskms` | Prefer IRSA or another IAM-backed pod identity. Example: `examples/aws-values.yaml` |
| GKE | `gcpckms` | Prefer GKE Workload Identity. Example: `examples/gcp-values.yaml` |
| AKS | `azurekeyvault` | Prefer Azure Workload Identity or Managed Identity. Example: `examples/azure-values.yaml` |
| OCI OKE | `ocikms` | Prefer instance principals or workload identity. Example: `examples/oci-values.yaml` |
| Bare metal without external KMS | `shamir` | Operationally simplest fallback. Example: `examples/bare-metal-shamir-values.yaml` |
| Bare metal with trusted external Vault | `transit` | Best portable auto-unseal option outside public cloud. Example: `examples/bare-metal-transit-values.yaml` |

The chart intentionally does not hard-code provider credential delivery. Vault's
upstream chart already exposes the hooks you need for that under
`vault.server.extraEnvironmentVars`, `vault.server.extraSecretEnvironmentVars`,
`vault.server.volumes`, and `vault.server.volumeMounts`. The cloud examples are
written around identity-based auth because that is the cleanest multi-provider
operating model.

### Standalone And Dev Notes

- `examples/standalone-values.yaml` keeps Vault persistent but single-node.
- `examples/dev-values.yaml` runs Vault in dev mode with VSO still installed.
- Dev mode is not initialized or unsealed. It starts ready immediately and loses
  all state on restart.
- In dev mode, run `scripts/post_install.sh` with the configured dev root token.
  With the example values that token is `root`.

### Bare Metal Notes

- `shamir` remains the baseline-safe mode when you do not have an external KMS
  or HSM.
- `transit` is the best OSS-compatible auto-unseal option for bare metal, but it
  requires a separate trusted Vault deployment and secure handling of the transit
  token.
- The `examples/bare-metal-transit-values.yaml` file shows one way to mount a CA
  certificate and transit token into the Vault server pods.

### Cloud Identity Notes

- AWS: prefer IRSA so the Vault pods can call KMS without static credentials.
- GCP: prefer Workload Identity so the Vault pods can call Cloud KMS without a
  mounted service-account key.
- Azure: prefer Workload Identity or Managed Identity so the Vault pods do not
  need a static client secret.
- OCI: prefer instance principals or workload identity. If you must use API key
  auth, extend the example by adding the required OCI credential material via
  `vault.server.extraSecretEnvironmentVars` and mounted files.

## Bootstrapping Vault

After the chart is installed, complete Vault bootstrap before expecting the
`VaultAuth` resource to work:

1. Start from the bootstrap flow that matches your Vault mode.

For HA Raft:

```shell
kubectl -n control-plane exec -it control-plane-vault-0 -- vault status
kubectl -n control-plane exec -it control-plane-vault-0 -- vault operator init
kubectl -n control-plane exec -it control-plane-vault-0 -- vault operator unseal
kubectl -n control-plane exec -it control-plane-vault-1 -- vault operator unseal
kubectl -n control-plane exec -it control-plane-vault-2 -- vault operator unseal
```

For standalone:

```shell
kubectl -n control-plane exec -it control-plane-vault-0 -- vault status
kubectl -n control-plane exec -it control-plane-vault-0 -- vault operator init
kubectl -n control-plane exec -it control-plane-vault-0 -- vault operator unseal
```

For dev mode, skip initialization and unseal entirely.

2. Run the local post-install helper from `extensions/control-plane`. It
   requires the `vault` CLI to be installed on your machine, assumes
   `vault operator init` has already been run for standalone or HA modes,
   port-forwards to `service/control-plane-vault`, and configures the shared
   Kubernetes auth mount, KV v2 mount, policy, and role using the local
   `vault` CLI.

```shell
VAULT_TOKEN=<vault-admin-token> ./scripts/post_install.sh
```

In dev mode with the example values, use `root` unless you changed
`vault.server.dev.devRootToken`.

3. Confirm the auth mount and role exist.

```shell
kubectl -n control-plane exec -it control-plane-vault-0 -- \
  vault auth list

kubectl -n control-plane exec -it control-plane-vault-0 -- \
  vault read auth/kubernetes/role/control-plane
```

The `post_install.sh` script is safe to rerun when you intentionally want to
reconcile the Vault auth mount, policy, role, or KV mount.

The default post-install policy is intentionally broad enough for cluster-wide VSO
usage: it grants read/list access across the configured KV v2 mount. Workload
charts that create `VaultStaticSecret` resources are expected to create a local
`vault-auth` service account in their own namespace and reference the shared
auth as `control-plane/default`.

> **Note:** The bundled `Prometheus` resource references an `alertmanager` Service
> in the same namespace. Provide an Alertmanager deployment (either manually or
> through a future chart addition) if you plan to send alerts.

## Accessing the Stack

- Port-forward Grafana to inspect dashboards: `kubectl -n control-plane port-forward service/grafana 3000:3000`
- Port-forward Prometheus to verify scrape targets: `kubectl -n control-plane port-forward pod/prometheus-prometheus-0 9090:9090`
- Check that the collectors are healthy: `kubectl -n control-plane get pods`

Grafana credentials remain a manual concern in this chart. By default Grafana
uses the upstream defaults (`admin`/`admin`) unless you change them through your
own Grafana configuration or operational process.

#### Manual steps

In order to use Grafana to visualize Prometheus metrics you will need to [add
the corresponding datasource](https://grafana.com/docs/grafana/latest/datasources/prometheus/configure/).
The URL is `http://prometheus-operated:9090`.

## Local Testing With kind

You can exercise the chart end-to-end inside a local
[kind](https://kind.sigs.k8s.io/) cluster. Install `kind`, `kubectl`, `helm`,
and (optionally) `kubeconform` first.

### Quickstart: Kind + Vault Dev Mode

This is the shortest local path if you want Vault and VSO running on Kind
without going through init/unseal:

```shell
kind create cluster --name supernode
kubectl config use-context kind-supernode

helm dependency build ./control-plane
helm install control-plane ./control-plane \
  --namespace control-plane \
  --create-namespace \
  --values ./control-plane/examples/dev-values.yaml
```

Because `examples/dev-values.yaml` keeps VSO enabled, you can immediately run the
post-install helper using the dev root token from that example:

```shell
VAULT_TOKEN=root ./scripts/post_install.sh
```

After that, Vault and VSO are ready for local experimentation without an
init/unseal flow.

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
    helm dependency build ./control-plane
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
helm dependency build .
helm lint .
helm template control-plane . | kubeconform -strict -skip PodMonitor,ServiceMonitor,Prometheus,VaultConnection,VaultAuth -summary -
helm template control-plane . -f examples/standalone-values.yaml | kubeconform -strict -skip PodMonitor,ServiceMonitor,Prometheus,VaultConnection,VaultAuth,VaultStaticSecret -summary -
helm template control-plane . -f examples/dev-values.yaml | kubeconform -strict -skip PodMonitor,ServiceMonitor,Prometheus,VaultConnection,VaultAuth,VaultStaticSecret -summary -
helm template control-plane . -f examples/aws-values.yaml | kubeconform -strict -skip PodMonitor,ServiceMonitor,Prometheus,VaultConnection,VaultAuth,VaultStaticSecret -summary -
```

## Key Values

| Value | Description | Default |
|-------|-------------|---------|
| `namespace` | Namespace that hosts all control-plane resources | `control-plane` |
| `storageClass` | Storage class used by Prometheus and Grafana persistent volumes | `standard` |
| `vault.server.dataStorage.storageClass` | Storage class used by Vault Raft PVCs | `standard` |
| `vault.server.dev.enabled` | Runs Vault in disposable dev mode | `false` |
| `vault.server.standalone.enabled` | Runs Vault as a persistent single-node deployment | `false` |
| `vault.server.ha.enabled` | Runs Vault in HA mode | `true` |
| `vault.integration.seal.mode` | Vault seal mode (`shamir`, `awskms`, `gcpckms`, `azurekeyvault`, `ocikms`, `transit`) | `shamir` |
| `vault.integration.auth.role` | Vault Kubernetes auth role bound to the control-plane auth service account | `control-plane` |
| `vault.integration.auth.allowedNamespaces` | Kubernetes namespaces allowed to reference the shared `VaultAuth` | `['*']` |
| `prometheusOperator.tolerations` | Tolerations applied to the Prometheus Operator deployment | `[]` |
| `grafana.tolerations` | Tolerations applied to the Grafana StatefulSet | `[]` |
| `prometheus.tolerations` | Tolerations applied to the Prometheus CRD | `[]` |

Consult `values.yaml` for the authoritative list.

## Maintenance

- Update the CRDs under `crds/` whenever the operator version is bumped.
- Run `helm dependency build extensions/control-plane` whenever the pinned Vault or Vault Secrets Operator versions change.
- Review the bundled Grafana configuration and dashboards as team requirements evolve.
- Double-check tolerations and node selectors when adjusting SuperNode scheduling policies so the control-plane keeps landing on the intended nodes.
