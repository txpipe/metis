# Kind (Self-Hosted) Bootstrap

This provider spins up a local Kubernetes cluster using [Kind](https://kind.sigs.k8s.io/) and prepares `kubectl` to access it. Run it via:

```bash
./bootstrap.sh --provider kind
```

## Prerequisites

- macOS or Linux host with container runtime (Docker Desktop, Rancher Desktop, or similar).
- Ability to install CLI binaries into `/usr/local/bin`.

The script will attempt to install the following tools if they are missing:

- [`kind`](https://kind.sigs.k8s.io/docs/user/quick-start/#installation)
- [`kubectl`](https://kubernetes.io/docs/tasks/tools/)
- [`helm`](https://helm.sh/docs/intro/install/)

## Configuration

- `KIND_CLUSTER_NAME` (env var): Overrides the default cluster name (`supernode`).
- `--config <path>`: Optional Kind cluster config YAML (see [Kind config docs](https://kind.sigs.k8s.io/docs/user/configuration/)).

Example custom config file reference:

```bash
./bootstrap.sh --provider kind --config ./bootstrap/kind/config.yml
```

If no config file is provided, Kind's defaults are used.

### Default control-plane values

- The bootstrap process uses `bootstrap/kind/values.yml` when no other values file is supplied.
- These defaults switch the Grafana and supernode dashboard services to `NodePort` so they are reachable via the `kind` extra port mappings.
- Provide `--values <path>` to override or extend the defaults.

### Example: Kind With Vault Dev Mode

If you want the shortest local Vault setup, point bootstrap at the control-plane
dev-mode example values:

```bash
./bootstrap.sh \
  --provider kind \
  --version 0.1.0 \
  --values ../extensions/control-plane/examples/dev-values.yaml
```

That keeps Vault Secrets Operator enabled, but runs Vault in disposable dev mode
so there is no init/unseal step. If you want to configure the shared VSO auth
resources afterward, run the local post-install script with the dev root token
from that example (`root` by default). That shared auth is read-only and scoped
to `kv/runtime/...`.

The shared `bootstrap.sh` flow also pre-applies the Vault Secrets Operator CRDs
before installing the control-plane chart, so the dev-mode example works on a
fresh Kind cluster without a second install pass.

If you want to configure the shared VSO auth resources after the install, use:

```bash
VAULT_TOKEN=root ../extensions/control-plane/scripts/post_install.sh
```

## Local Control Plane With Local MCP Image

Use this flow when testing local MCP server changes in Kind before publishing an
image.

Create or reuse the Kind cluster:

```bash
./bootstrap/kind/bootstrap.sh --config ./bootstrap/kind/config.yml
```

Build and load the local MCP image into Kind:

```bash
docker build -t metis-supernode-mcp:dev ./mcp-server
kind load docker-image metis-supernode-mcp:dev --name "${KIND_CLUSTER_NAME:-supernode}"
```

Install the local control-plane chart with the local MCP image and dev Vault:

```bash
helm dependency build ./extensions/control-plane
helm upgrade --install control-plane ./extensions/control-plane \
  --namespace control-plane \
  --create-namespace \
  --values ./bootstrap/kind/values.yml \
  --values ./extensions/control-plane/examples/dev-values.yaml \
  --set supernodeMcp.image.repository=metis-supernode-mcp \
  --set supernodeMcp.image.tag=dev \
  --set supernodeMcp.image.pullPolicy=Never
```

The control-plane chart enables a PVC-backed SQLite MCP session store by
default. This lets existing MCP sessions survive a pod restart, as long as the
client keeps using the same `Mcp-Session-Id` and the session has not expired.

Configure Vault for local MCP runtime-secret operations:

```bash
VAULT_TOKEN=root ./extensions/control-plane/scripts/post_install.sh
```

Forward the MCP service and check health:

```bash
kubectl -n control-plane rollout status deployment/supernode-mcp
kubectl -n control-plane port-forward service/supernode-mcp 8082:8443
curl http://127.0.0.1:8082/healthz
```

Live `workloads.install` calls are enabled through MCP. Use `dryRun: true` to
inspect the generated Helm values, then call the same tool with `dryRun: false`
to apply the install.

When rebuilding the MCP image, load it again and restart the deployment:

```bash
docker build -t metis-supernode-mcp:dev ./mcp-server
kind load docker-image metis-supernode-mcp:dev --name "${KIND_CLUSTER_NAME:-supernode}"
kubectl -n control-plane rollout restart deployment/supernode-mcp
```

MCP clients should reinitialize only when the server returns `404 Session not
found`, which means the session was deleted, expired, or the PVC was replaced.

## Outputs

- Creates (or reuses) a Kind cluster named `supernode` by default.
- Exports kubeconfig into the default `kubectl` configuration and selects the `kind-<cluster-name>` context.
- Ensures `helm` is available for the subsequent control-plane chart installation stage.
