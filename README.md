# Metis

Metis is the Supernode platform for running and operating Cardano Partnerchain infrastructure on Kubernetes. It packages workloads as Helm-based extensions, exposes cluster operations through a Model Context Protocol (MCP) server, and provides catalogs and operational skills that agents can use to install, inspect, and maintain Supernode workloads.

The current repository is organized around these concerns:

- Kubernetes bootstrap automation for local and cloud clusters.
- Helm charts for the Supernode control plane and installable workloads.
- A Rust MCP server that exposes typed tools, resources, prompts, policy checks, and audit events.
- Extension and skill catalogs used by MCP clients and agents.
- Frontend applications for the Supernode dashboard and public catalog.
- Tx3 transaction specifications for Partner Chain operations.

## Repository Layout

- [**bootstrap**](bootstrap/README.md): Scripts for creating or reusing Kubernetes clusters and installing the Supernode control plane. Supported providers include AWS EKS, Azure AKS, Google Cloud GKE, and local Kind clusters.
- [**catalog**](catalog/README.md): Extension and skill catalog documents consumed by the MCP server, plus scripts for generating and publishing catalog artifacts.
- [**extensions**](extensions/README.md): Helm charts for the control plane and workload extensions, including Cardano, Apex Fusion, Dolos, Hydra, and Midnight charts.
- **mcp-server**: Rust MCP server named `supernode-mcp`. It loads extension and skill catalogs, talks to Kubernetes and Vault, manages Helm releases, exposes workload tools, and enforces advisory policy/audit behavior.
- [**skills**](skills/README.md): MCP-only operational guides for agents. Skills describe safe workflows for installing, upgrading, troubleshooting, and maintaining Supernode workloads through MCP tools.
- [**frontends/dashboard**](frontends/dashboard/README.md): TanStack Start dashboard for operating a Supernode cluster.
- [**frontends/catalog**](frontends/catalog/README.md): TanStack Start catalog UI for browsing Supernode workloads and related metadata.
- [**tx3**](tx3/README.md): Tx3 transaction documentation for Partner Chain governance and validator-management operations.

## Quick Start

### Bootstrap a Local Supernode

Kind is the fastest path for local development or evaluation. The bootstrap script provisions or reuses a Kubernetes cluster, applies provider defaults, pre-applies Vault Secrets Operator CRDs, and installs the control-plane Helm chart.

```bash
cd bootstrap
./bootstrap.sh \
  --provider kind \
  --version 0.1.0 \
  --config ./kind/config.yml
```

See [bootstrap/README.md](bootstrap/README.md) and the provider-specific READMEs for prerequisites, cloud-provider details, and Vault follow-up steps.

### Work on the MCP Server

The MCP server is a Rust service under `mcp-server`.

```bash
cd mcp-server
cargo test
cargo clippy -- -D warnings
cargo run
```

Common environment variables include:

- `MCP_BIND_ADDR`: HTTP bind address, defaulting to `0.0.0.0:8443`.
- `MCP_AUTH_MODE`: authentication mode, defaulting to `trusted`.
- `MCP_SESSION_STORE`: `memory` or `sqlite`.
- `MCP_EXTENSION_CATALOG_SOURCE`: `bundled` or `oci`.
- `MCP_SKILL_CATALOG_SOURCE`: `bundled` or `oci`.

The Docker image builds the Rust binary and includes Helm so the server can manage releases from inside the runtime container.

### Work on Frontends

Each frontend is managed independently with `pnpm` from its own directory.

```bash
cd frontends/dashboard
pnpm install
pnpm dev
pnpm test
pnpm typecheck
```

```bash
cd frontends/catalog
pnpm install
pnpm dev
pnpm test
pnpm typecheck
```

Both applications use TanStack Start, TanStack Router, React, Vite, Vitest, and Tailwind CSS.

### Work on Extensions

Extension charts live under `extensions/*`. The CI workflow runs Helm linting and schema validation across the supported chart/value combinations.

For a local spot check, run Helm commands from the `extensions` directory, for example:

```bash
cd extensions
helm lint ./control-plane
helm template control-plane ./control-plane
```

Some charts have dependencies or CI-specific values files. Check the chart README and `.github/workflows/check_extensions.yml` when matching CI behavior locally.

### Update Catalogs and Skills

MCP consumes two catalog artifacts from `catalog`:

- `extension-catalog.json`: installable extension contracts and Helm chart references.
- `skill-catalog.json`: generated skill catalog with embedded markdown from `skills/*.md`.

After editing `catalog/skill-catalog.manifest.json` or any skill markdown, regenerate the tracked skill catalog:

```bash
node catalog/scripts/generate-skill-catalog.mjs
```

Catalogs are treated as supply-chain inputs. By default, MCP trusts official Supernode OCI references from `oci.supernode.store`; untrusted catalog sources are intended for local development only.

## CI Checks

The repository has separate GitHub workflows for the major areas:

- MCP server checks generate the skill catalog and run `cargo clippy -- -D warnings`.
- MCP server tests run the Rust test suite.
- MCP server builds produce the runtime image.
- Frontend workflows check and build the dashboard and catalog apps.
- Extension checks lint and render Helm charts, then validate the generated manifests.

Use the component-specific commands above when reproducing failures locally.

## Operational Model

Metis is designed around a Supernode control plane installed into Kubernetes. Workloads are installed as Helm chart extensions, described in catalogs, and operated through MCP tools rather than ad hoc shell commands. Skills provide agent-facing runbooks that use those tools for common flows such as workload discovery, Cardano relay setup, block-producer maintenance, Hydra operations, and Dolos deployment.
