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

## Outputs

- Creates (or reuses) a Kind cluster named `supernode` by default.
- Exports kubeconfig into the default `kubectl` configuration and selects the `kind-<cluster-name>` context.
- Ensures `helm` is available for the subsequent control-plane chart installation stage.
