# SuperNode MCP Architecture

## Purpose

This document defines the architectural role of MCP in SuperNode.

SuperNode needs a machine-facing interface for operating a running cluster in a way that is safe, explicit, and aligned with the platform's domain model. The goal of this document is to explain the architectural decisions behind that interface: what MCP is for, what boundary it defines, what kinds of operations it exposes, and which kinds of operations it intentionally excludes.

This document is not a wire-level protocol specification. It focuses on the architectural shape of MCP as part of the SuperNode platform.

## Context

SuperNode is a platform for operating supported workloads on Kubernetes through a curated extension model.

As described in `01_domain.md`, SuperNode's key domain concepts are extensions, workloads, the catalog, runtime secrets, operator secrets, and the control plane. These concepts define the product surface that SuperNode wants to expose. They are not the same thing as Helm charts, raw Kubernetes resources, Vault paths, or shell commands, even when those implementation details exist underneath.

A machine-facing operational interface is needed for several reasons:

- to allow agents to inspect and operate a SuperNode cluster
- to expose SuperNode's domain objects in a structured and machine-readable way
- to constrain automation to supported and auditable operations
- to avoid treating raw infrastructure access as the primary product interface
- to create a path toward convergence between human-facing interfaces and machine-facing interfaces

Without such an interface, automation would tend to fall back to lower-level mechanisms such as shell commands, direct Kubernetes access, raw Helm usage, or ad hoc secret handling. Those mechanisms are too implementation-specific, too difficult to secure consistently, and too far removed from SuperNode's actual domain model.

## Decision

SuperNode will expose MCP as the machine-facing operational interface for managing an existing SuperNode cluster.

This MCP interface is an architectural boundary, not just a transport endpoint. It exists to expose SuperNode concepts directly and to ensure that machine-driven operations happen through typed, validated, policy-aware actions rather than through unrestricted infrastructure access.

At a high level, the MCP architecture is defined by the following decisions:

- MCP operates an existing SuperNode cluster rather than provisioning one
- MCP runs inside the cluster and uses in-cluster backend credentials
- MCP exposes typed tools, structured resources, and guided workflows
- MCP is workload-centered and catalog-driven
- MCP treats extensions and workloads as first-class domain objects
- MCP separates generic lifecycle operations from domain-specific workflows
- MCP enforces a strict security boundary around secrets and signing material
- MCP avoids generic shell, generic execution, and raw infrastructure proxy patterns

## Architectural Role Of MCP

MCP is the machine-facing counterpart to the rest of the SuperNode operational surface.

Its role is to make the SuperNode domain available to agents and advanced clients in explicit form. This means that MCP should expose the concepts that SuperNode itself cares about: workloads, extensions, configuration schemas, status, runtime secrets, operator secrets, and supported workflows.

MCP is therefore not a generic remote administration interface. It is a product interface for operating SuperNode according to SuperNode's own model.

This distinction is important. A generic administration interface would expose the underlying implementation substrate directly and leave the client to assemble meaning from that substrate. MCP does the opposite: it exposes the meaning first, and keeps implementation details behind a controlled boundary.

## Scope Boundary

MCP is intended for operating an existing SuperNode cluster.

This includes:

- discovering cluster and platform state relevant to SuperNode
- inspecting supported extension definitions and workload state
- installing, upgrading, and deleting supported workloads
- exposing workload status and other machine-consumable operational state
- reading and writing approved runtime secrets through typed operations
- supporting domain-specific workflows for workload operation and maintenance
- providing structured progress, logging, and audit behavior for operational actions

MCP is not responsible for the full lifecycle of the infrastructure on which SuperNode runs.

It intentionally excludes responsibilities such as:

- provisioning the Kubernetes cluster itself
- bootstrapping the initial control plane
- initializing or unsealing Vault
- exposing the operational endpoint directly to the public internet
- taking custody of offline signing material
- performing private-key signing with operator-controlled cold material

The architectural boundary is that MCP operates a running SuperNode environment; it does not replace the external operator processes required to create that environment or to retain control over highly sensitive signing operations.

## In-Cluster Execution Model

MCP runs inside the SuperNode cluster.

This means that it uses in-cluster identities and credentials to interact with the platform's backing systems, including Kubernetes, workload lifecycle machinery, and secret-management infrastructure.

This decision has several consequences.

First, it allows MCP to act as a stable internal service of the platform rather than as an external client that depends on each operator's workstation environment.

Second, it makes it possible to give MCP a controlled backend identity with platform-specific permissions rather than relying on arbitrary end-user credentials or unstructured workstation state.

Third, it creates a cleaner security boundary: clients authenticate to MCP, and MCP uses its own internal credentials to perform bounded operations. Client credentials are not simply passed through to downstream systems.

The in-cluster execution model is therefore part of the architecture, not only a deployment convenience.

## Typed Operations Instead Of Generic Infrastructure Access

MCP exposes typed operations only.

Every action available through MCP should be represented as an explicit operation with a defined input shape, a defined behavioral contract, a defined authorization and approval posture, and a defined audit surface.

SuperNode intentionally does not treat generic shell access as part of the MCP model. It also avoids exposing unrestricted execution inside pods, generic infrastructure proxies, or raw operational escape hatches as the normal path.

This is a foundational decision. If MCP were allowed to become a thin wrapper around shell commands, raw Kubernetes actions, or unbounded backend calls, it would stop being a SuperNode interface and would instead become an arbitrary remote control mechanism. That would undermine the platform's domain model, weaken its security posture, and make reliable agent behavior much harder to achieve.

Typed operations keep the surface explicit. They allow SuperNode to define what is supported, what is safe, what is observable, and what is auditable.

## Workload-Centered And Catalog-Driven Interface

MCP is workload-centered.

The main operational object it acts on is the workload: a concrete deployed instance of an extension. This follows directly from the domain model in `01_domain.md`.

MCP is also catalog-driven.

Supported extensions are not merely implementation artifacts hidden somewhere in the system. They are part of the explicit contract of the platform. The catalog defines what kinds of workloads SuperNode supports, what their validated inputs are, what their operational expectations are, and how they fit into the platform.

This has an important architectural consequence: MCP should not expose raw packaging internals as its primary interface. Instead, it should expose the extension and workload contract in product terms.

That means:

- clients operate supported extensions, not arbitrary charts
- clients provide validated workload inputs, not unconstrained internal configuration blobs
- clients reason about workload state and supported workflows, not only raw infrastructure state

The catalog-driven model allows SuperNode to evolve internal implementation details without changing the conceptual interface it presents to agents and other machine clients.

## Generic Lifecycle Surface And Domain-Specific Workflows

MCP should keep the generic workload lifecycle surface small and stable.

There are a handful of operations that are broadly meaningful across supported workloads: installation, inspection, upgrade, deletion, status retrieval, and related operational access. These form the generic lifecycle layer.

Beyond that, some extensions have domain-specific operational workflows that are meaningful only for that extension or family of extensions. Those should exist as explicit extension-specific workflows only when they represent real domain behavior that cannot be reduced to generic lifecycle management.

This separation is important for two reasons.

First, it keeps the common operational model coherent. Most interactions with SuperNode should be understandable in terms of workloads and their lifecycle.

Second, it avoids flattening domain-specific workflows into raw infrastructure mutations. If a workflow represents a meaningful operational concept in the extension's domain, then MCP should expose it as such rather than forcing clients to reproduce it from lower-level primitives.

The result is a layered model:
- generic lifecycle operations for all workloads
- domain-specific workflows only where justified by the extension's operational semantics

## Guided Workflows

MCP includes guided workflows as part of its architecture.

These workflows exist because operating certain workloads is not only a matter of applying configuration. Some tasks require staged decisions, operator confirmations, safety checkpoints, or coordination across multiple underlying operations.

Guided workflows allow SuperNode to expose these tasks in a way that remains aligned with the product's domain model. They provide a structured operational path for tasks that are higher-level than a single lifecycle action but still belong within the supported surface of the platform.

Guided workflows are therefore not separate from the extension model. They are part of how an extension becomes operable through MCP.

## Authorization And Policy Model

MCP uses a policy-aware authorization model.

The architecture assumes that access to operations is determined by explicit policy concepts rather than by ad hoc checks embedded in arbitrary backend logic. These policy concepts include scopes, approval requirements, and audit obligations.

The long-term direction is an enforced authorization model in which machine clients act under real authenticated identities and server-side policy determines what they are allowed to do.

However, the architecture also permits an early trusted mode for environments where SuperNode is being operated through trusted cluster access rather than a full user and identity system. In that mode, policy remains part of the model even when enforcement is relaxed. The point is not to bypass the policy system, but to preserve the same architectural shape while enabling incremental delivery.

This decision matters because it avoids a common trap: shipping an initial operational surface with no real policy model and trying to retrofit one later. Instead, the policy concepts exist from the beginning, even if the enforcement story matures over time.

## Approval Model

Authorization alone is not enough for all operations.

Some actions are sensitive not because they require a different identity model, but because they require deliberate human confirmation even when the caller is otherwise authorized. This includes actions that are destructive, secret-bearing, or operationally consequential.

For that reason, MCP includes approval as a separate concept from identity and scope.

Approval should be understood as an operational safeguard layered on top of authorization. It is not a replacement for authorization, and it cannot grant actions that policy would otherwise forbid. Instead, it marks operations that should require explicit operator intent before execution.

This allows MCP to treat read-only inspection, normal operational mutations, destructive operations, sensitive secret access, and certain ledger-adjacent actions as distinct classes of action with distinct operational expectations.

## Secret Boundary

MCP enforces a strong secret boundary.

SuperNode distinguishes between runtime secrets and operator secrets. That distinction is architectural and must remain visible in the MCP model.

Runtime secrets are part of workload execution. They may be needed by running workloads and may be accessed only through typed, approval-aware operations that preserve the SuperNode secret contract.

Operator secrets are different. They belong to operator-controlled custody and are not part of ordinary workload execution. MCP must not normalize routine access to them. Any support for interacting with operator secrets should remain exceptional, tightly bounded, and clearly distinct from the normal runtime path.

This distinction prevents the machine interface from collapsing the difference between "what a workload must consume to run" and "what an operator must control to preserve safe custody."

## Ledger Boundary

MCP may assist with some ledger-adjacent tasks, but it does not own operator signing authority.

This means MCP can participate in preparing information, coordinating workflows, or handling certain bounded submission steps where appropriate, but it does not take custody of offline signing keys and does not replace the operator's responsibility for signing-sensitive actions.

This is an important boundary for SuperNode's security model. Even where a workflow touches ledger operations, the machine interface must not erase the distinction between cluster-side automation and operator-controlled signing authority.

## Transport Boundary

The initial MCP transport should remain operationally narrow and security-conscious.

The architectural intent is that MCP be reachable through trusted cluster access rather than being exposed immediately as a public internet service. This keeps the initial deployment shape aligned with the rest of the in-cluster execution model and reduces the amount of external security infrastructure required for an early version.

The transport layer is therefore not the main architectural feature of MCP. It is a controlled delivery mechanism for the machine-facing interface. The important architectural decision is that transport state must not be treated as authorization state, and that network reachability must not be allowed to redefine the security boundary of the system.

## Long-Running Operations

MCP must treat long-running operations as first-class.

Many meaningful SuperNode actions are not instantaneous. Workload installation, upgrade, deletion, and other operational tasks often take time, involve waiting on cluster state, and require structured reporting back to the client.

For that reason, progress reporting, cancellation semantics, and final structured outcomes are architectural requirements rather than incidental details.

This matters especially for agent-driven usage. A machine client must be able to understand whether an action is still in progress, whether it is safe to stop waiting, whether cancellation is possible, and what final state was reached.

## Logging And Audit

MCP must provide both operational logging and structured audit behavior.

Operational logging is needed for debugging and platform observability. Audit behavior is needed because MCP is an action surface for sensitive operations, not only a read-only query interface.

Audit is especially important for mutations, destructive actions, secret access, and other high-consequence operations. The architecture therefore treats audit as part of the action model itself rather than as an optional implementation detail.

This requirement also reinforces the typed-operation decision. A system can only audit operations reliably if those operations are defined as explicit, bounded actions with well-understood semantics.

## Consequences

This architecture has several positive consequences.

It gives SuperNode a machine interface that is aligned with its own domain model rather than with the accidental structure of its underlying implementation. It makes automation safer by constraining it to supported operations. It improves auditability and security by making policy, approvals, and secret boundaries explicit. It also creates a clearer long-term path for integrating agents into normal SuperNode operations.

The architecture also introduces costs and constraints.

It requires more up-front design than exposing raw infrastructure primitives. It requires maintaining explicit schemas, policies, and workflow definitions. It intentionally limits flexibility in cases where a generic shell or infrastructure proxy would be quicker to implement. It also demands discipline in keeping the public machine interface stable even as internal implementation details evolve.

These tradeoffs are acceptable because the primary goal of MCP is not unrestricted power. The goal is safe, explicit, domain-aligned operability.

## Rejected Alternatives

Several alternatives are intentionally rejected by this architecture.

### Generic Shell Access

Exposing shell commands directly through MCP would be fast but would turn the interface into a remote execution surface rather than a SuperNode product interface. It would weaken safety, make audit behavior less reliable, and encourage clients to depend on implementation details.

### Generic Pod Execution

Allowing unrestricted execution inside arbitrary pods would bypass the workload and extension model. It would make the platform much harder to secure and would undermine the distinction between supported operations and arbitrary cluster access.

### Raw Kubernetes Or Helm Interface

Exposing raw Kubernetes resources or raw Helm actions as the primary interface would force machine clients to reconstruct SuperNode meaning from low-level implementation details. That is the opposite of the intended architecture.

### Secret Access As A Generic Capability

Treating secret access as a flat or generic capability would erase the distinction between runtime secrets and operator secrets. SuperNode needs that distinction to remain part of the machine-facing model.

### External MCP As The Primary Model

Running MCP primarily as an external service using workstation or user-local credentials would make the interface more dependent on operator environment and less aligned with the platform's own internal security boundary.

## Relationship To The Domain Model

This ADR builds directly on `01_domain.md`.

In particular:

- MCP exposes extensions and workloads as first-class operational objects
- MCP relies on the catalog as the source of supported extension definitions
- MCP preserves the distinction between runtime secrets and operator secrets
- MCP depends on the control plane and the Kubernetes substrate but should not expose them as the primary product abstraction

MCP should therefore be understood as an expression of the SuperNode domain model in machine-operable form.

## Notes

This document defines the architectural shape of MCP, not its final detailed protocol surface.

As SuperNode evolves, the detailed schemas, workflow definitions, policy metadata, and backend mechanics may change. The architectural commitments in this ADR should remain stable:

- MCP is a machine-facing SuperNode interface
- it is typed rather than generic
- it is workload-centered and catalog-driven
- it preserves explicit security and custody boundaries
- it operates on an existing SuperNode cluster through controlled backend access
