# Metis

Metis is an infrastructure orchestration platform (SuperNode) tailored for Cardano Stake Pool Operators (SPOs) to deploy, manage, and monitor Partnerchain nodes and services. It leverages Kubernetes for portability, automation, and extensibility.

For an in-depth overview of the architectural decisions behind Metis, see the design document in the [ADR directory](adr/00_design.md).

## Components

Metis is organized into several key components, each with its own responsibilities:

- **adr**: Architectural Decision Records and design documentation.
- [**backend**](backend/README.md): Rust-based management API server powering the Metis Management service.
- [**cli**](cli/README.md): Rust-based command-line interface for interacting with the Metis platform.
- [**extensions**](extensions/README.md): Helm charts packaging Partnerchain integrations as Kubernetes extensions.
- [**frontends/dashboard**](frontends/dashboard/README.md): React application built with TanStack Start, deployed in the cluster for managing Helm charts, installing workloads, and monitoring deployments.
- [**frontends/catalog**](frontends/catalog/README.md): React application built with TanStack Start, serving as the marketing landing page and workload catalog.
- [**operator**](operator/README.md): Kubernetes operator for automating custom resource management and lifecycle of SuperNode components.

See each component's README for detailed setup, development, and deployment instructions.
