# Supernode Bootstrap

This directory contains the automation for standing up the Metis Supernode
control plane on a Kubernetes cluster. The root `bootstrap.sh` script
orchestrates two steps:

1. Provision (or reuse) a Kubernetes cluster for a supported provider.
2. Install the Supernode control plane Helm chart into that cluster.

Run the script from this directory so that relative paths resolve correctly.

```bash
./bootstrap.sh --provider <provider> --version <control-plane version> [--config <file>] [--values <file>]
```

- `--provider` selects the infrastructure target.
- `--version` is the Helm chart version for `oci://oci.supernode.store/control-plane` (required).
- `--config` forwards a provider-specific configuration file to the underlying script.
- `--values` passes a Helm values file for the control plane. If omitted, provider defaults are applied when available (AWS and Kind ship with `values.yml`).

For Vault-specific control-plane examples, see `extensions/control-plane/examples/`.
Those files are not applied automatically by the bootstrap scripts; pass one
explicitly with `--values` if you want to start from a provider-tailored Vault
configuration or from one of the non-default Vault server modes such as
standalone or dev.

## Vault Handling

The shared `bootstrap.sh` flow now pre-applies the Vault Secrets Operator CRDs
before installing the control-plane chart. This avoids first-install failures
when the chart renders `VaultConnection` and `VaultAuth` resources on a fresh
cluster.

What `bootstrap.sh` does for Vault:

- provisions or reuses the Kubernetes cluster
- installs the control-plane chart with the values file you choose
- pre-applies the Vault Secrets Operator CRDs before the Helm install

What it does not do automatically:

- configure cloud-provider KMS permissions for Vault auto-unseal
- initialize Vault in standalone or HA modes
- run the local Vault post-install script that configures Kubernetes auth and
  the shared VSO role/policy

Mode-specific follow-up:

- `dev`: no init/unseal required
- `standalone`: run `vault operator init` once, then bootstrap Vault auth
- `ha`: run `vault operator init` once, then bootstrap Vault auth

Useful follow-up helpers:

- `extensions/control-plane/scripts/setup-gke-autopilot-vault.sh`
- `extensions/control-plane/scripts/post_install.sh` (requires local `vault` CLI)

## Providers

Pick the provider that matches where you want to run the Supernode cluster and
review its README for prerequisites, configuration, and outputs.

- [AWS EKS](./aws/README.md): Creates or reuses an Amazon EKS cluster and prepares persistent volumes with the EBS CSI driver.
- [Azure AKS](./azure/README.md): Spins up an Azure Kubernetes Service cluster and configures access for the control plane.
- [Google Cloud GKE](./gcloud/README.md): Provisions a Google Kubernetes Engine cluster with the necessary tooling.
- [Kind (Self-Hosted)](./kind/README.md): Creates a local multi-node cluster using Kind for development or evaluation.

Each provider script installs any required CLIs if they are missing (for
example `kubectl`, `helm`, or cloud-specific CLIs) and exports credentials into
your default kubeconfig.

## Example: Local Cluster with Kind

Kind is the fastest way to spin up a Supernode locally. Make sure you have a
container runtime (Docker Desktop, Rancher Desktop, etc.) and see the [Kind
provider README](./kind/README.md) for the full list of prerequisites.

```bash
cd bootstrap
# Optional: tweak cluster topology via ./kind/config.yml
# Replace 0.1.0 with the control-plane release you want to deploy
./bootstrap.sh \
  --provider kind \
  --version 0.1.0 \
  --config ./kind/config.yml
```

The script will:

- Ensure `kind`, `kubectl`, and `helm` are installed.
- Create (or reuse) a Kind cluster named `supernode` by default.
- Apply `bootstrap/kind/values.yml` unless you supply your own `--values` file.
- Pre-apply Vault Secrets Operator CRDs before the control-plane install.
- Install the `control-plane` release in the `control-plane` namespace using the version you specified.

Once the command completes, your local kubeconfig will be pointed at the Kind cluster and the Supernode control plane will be ready for use. Consult the provider-specific README for additional customization options or troubleshooting steps.
