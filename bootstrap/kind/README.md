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

- `KIND_CLUSTER_NAME` (env var): Overrides the default cluster name (`metis-supernode`).
- `--config <path>`: Optional Kind cluster config YAML (see [Kind config docs](https://kind.sigs.k8s.io/docs/user/configuration/)).

Example custom config file reference:

```bash
./bootstrap.sh --provider kind --config ./bootstrap/kind/config.yml
```

If no config file is provided, Kind's defaults are used.

## Outputs

- Creates (or reuses) a Kind cluster named `metis-supernode` by default.
- Exports kubeconfig into the default `kubectl` configuration and selects the `kind-<cluster-name>` context.
- Ensures `helm` is available for the subsequent control-plane chart installation stage.
