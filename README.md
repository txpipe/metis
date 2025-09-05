# Metis

Metis is an infrastructure orchestration platform (SuperNode) tailored for Cardano Stake Pool Operators (SPOs) to deploy, manage, and monitor Partnerchain nodes and services. It leverages Kubernetes for portability, automation, and extensibility.

For an in-depth overview of the architectural decisions behind Metis, see the design document in the [ADR directory](adr/00_design.md).

## Components

Metis is organized into several key components, each with its own responsibilities:

- **adr**: Architectural Decision Records and design documentation.
- **backend**: Rust-based management API server powering the Metis Management service.
- **cli**: Rust-based command-line interface for interacting with the Metis platform.
- **extensions**: Helm charts packaging Partnerchain integrations as Kubernetes extensions.
- **frontend**: React/Vite web application providing the Metis Management user interface.
- **operator**: Kubernetes operator for automating custom resource management and lifecycle of SuperNode components.

See each component's README for detailed setup, development, and deployment instructions.
