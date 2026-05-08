# SuperNode Domain Definitions

## Purpose

This document defines the major concepts used across SuperNode.

SuperNode and Metis refer to the same system. "SuperNode" is the external product name, while "Metis" is the internal technical nickname used in this repository.

The goal of this document is not to create rigid formal terminology, but to establish a shared high-level vocabulary for the platform, its interfaces, and its operational model.

## Core Concepts

### SuperNode

SuperNode is an infrastructure orchestration platform for running supported blockchain and related infrastructure workloads on Kubernetes.

It provides a common operational layer for deploying, managing, monitoring, and securing those workloads. It is designed around a curated model: operators do not manage arbitrary infrastructure through SuperNode, but supported extensions and the workloads instantiated from them.

### Kubernetes

Kubernetes is the orchestration substrate on which SuperNode runs.

SuperNode uses Kubernetes as the common execution environment for its control plane, workloads, networking, persistence, and operational automation. Kubernetes is not itself the product domain, but it is the base platform that SuperNode relies on to realize its model.

### Control Plane

The control plane is the set of shared platform services that make a SuperNode cluster operable.

It includes the common infrastructure used for observability, secret management, management interfaces, and cluster-level operational support. Other workloads may depend on the control plane being present in order to function correctly within the SuperNode environment.

### Extension

An extension is the abstract contract for a supported kind of SuperNode workload.

An extension defines its identity, configuration schema, validation rules, secret requirements, operational capabilities, status model, metrics schema, and associated MCP workflows. Its implementation may currently be backed by a Helm chart, but that is an internal detail rather than the defining interface.

An extension is the reusable type-level definition of something SuperNode knows how to install and operate. In object-oriented terms, it is closer to a class than to an instance.

#### Extension Interface

An extension should be understood primarily through its interface rather than through its packaging format.

At a minimum, an extension definition should include the following parts.

##### Identity

An extension has a stable identity.

This includes at least:
- an extension identifier
- a human-readable name
- a description of what kind of workload it represents

This identity allows the extension to be referenced consistently across the catalog, MCP tools, workflows, and user-facing interfaces.

##### Version Model

An extension defines the versions of itself that SuperNode supports.

This includes:
- supported versions
- compatibility expectations
- upgrade expectations where relevant

This allows SuperNode to reason about whether a workload can be installed, upgraded, or operated safely.

##### Configuration Schema

An extension defines the schema of the parameters required to instantiate a workload.

This schema should be represented as JSON Schema. It defines:
- which parameters exist
- which parameters are required
- default values
- validation rules
- which inputs are sensitive
- any structural constraints on valid configuration

Today these values may map internally onto Helm chart values, but the long-term interface should be the extension schema itself, not the raw chart structure.

##### Instance Identity Rules

An extension defines how workloads of that type are identified once instantiated.

This includes concepts such as:
- release name
- namespace
- naming expectations
- whether multiple workloads of the same extension may coexist

This matters because a workload is not only configuration plus execution, but also an addressable managed object inside the SuperNode domain.

##### Secret Contract

An extension defines which secrets it needs and what role those secrets play.

This includes:
- which secrets are required at runtime
- which secret references are optional
- which secret material belongs in the runtime secret space
- which secret material belongs in the operator secret space
- whether the extension accepts references, values, or both

This contract is important because secret handling is not an implementation detail. It is part of the operational semantics of the extension.

##### Dependency Contract

An extension defines the platform capabilities or other components it depends on.

This may include:
- dependency on the control plane
- dependency on shared secret-management facilities
- dependency on another workload being available
- dependency on external upstream services or chain endpoints

This allows SuperNode to reason about prerequisites, safe installation, and operational readiness.

##### Operational Interface

An extension defines the operations SuperNode can perform on workloads of that type.

This includes the generic workload lifecycle operations, such as install, inspect, upgrade, and delete, as well as any domain-specific operations that are meaningful for that extension.

This is important because an extension is not just something that can be deployed. It is something that SuperNode knows how to operate.

##### Metrics Schema

An extension defines the metrics that workloads of that type expose.

This schema should describe the available metrics in a structured form so that SuperNode can consume them consistently. The metrics schema should make clear:
- which metrics exist
- what they mean
- which are required versus optional
- which are raw measurements versus derived signals

This allows dashboards, automation, and MCP tools to treat metrics as part of the extension contract rather than as ad hoc output.

##### Status Schema

An extension defines how SuperNode determines the health and operational state of a workload of that type.

This should include the status signals, conditions, or rules by which a workload can be interpreted as healthy, unhealthy, degraded, starting, or otherwise operationally significant.

The status schema is distinct from the metrics schema. Metrics describe measurable signals; the status schema describes how SuperNode interprets the workload's operational condition.

This is important because SuperNode must be able to determine whether a workload is healthy in a way that is defined by the extension itself, not by generic Kubernetes state alone.

##### Workflow Surface

An extension defines the domain-specific workflows associated with operating that kind of workload through MCP.

These workflows represent higher-level operational tasks that go beyond the generic lifecycle surface. They are part of the extension definition because SuperNode is not only a deployment system, but also an operational system.

##### Outputs And Exposed Interfaces

An extension defines what a workload of that type exposes once instantiated.

This may include:
- internal services
- external endpoints
- protocol interfaces
- management interfaces
- operator-visible outputs

These exposed interfaces are part of the meaning of the extension because they define what the instantiated workload contributes to the SuperNode environment.

### Workload

A workload is a concrete deployed instance of an extension.

A workload is produced by selecting an extension version, supplying validated inputs, assigning an operational identity such as namespace and release name, and materializing the result in the cluster as running resources with observable state.

A workload is therefore not just "some pods running in Kubernetes". It is a specific operational object in the SuperNode domain: a named instance of a known extension, with concrete configuration, real runtime state, visible health, exposed metrics, and a lifecycle that SuperNode can manage.

### Catalog

The catalog is the curated set of extensions that SuperNode supports.

It is the source of truth for what kinds of workloads can be instantiated through the platform. More importantly, it defines the contract surface exposed to users and to MCP clients: which extensions exist, which versions are supported, what inputs they require, and which operational workflows they participate in.

The catalog is therefore not just a list of available packages. It is the registry of supported extension definitions.

## Security Concepts

### Runtime Secret

A runtime secret is secret material stored in the runtime Vault space and intended to be consumed by running workloads.

These are secrets that must be available to the workload in order for it to run correctly. They are part of the runtime contract of an instantiated workload.

### Operator Secret

An operator secret is secret material stored in the operator Vault space and intended for operator-controlled access rather than normal workload consumption.

These secrets may still be relevant to operating a workload, but they are not part of the ordinary runtime surface. Their handling follows stricter custody and access expectations.

## Interfaces

### MCP

The Model Context Protocol interface is the structured machine interface for operating SuperNode.

It exposes SuperNode concepts in typed form so that an agent or advanced client can inspect the system, reason about supported extensions, and operate workloads through well-defined operations rather than arbitrary shell access.

MCP is important to the domain model because it pushes SuperNode toward explicit contracts: extension definitions, workload identity, configuration schema, status interpretation, and operational workflows all need to be representable in a machine-readable way.

## Relationship Summary

The main relationships between concepts are:

- SuperNode is the overall platform.
- Kubernetes is the execution substrate SuperNode runs on.
- The control plane provides shared cluster capabilities required by the platform.
- The catalog contains the supported extension definitions.
- An extension is an abstract supported workload type.
- A workload is a concrete instance of an extension.
- Runtime secrets support workload execution.
- Operator secrets support operator-controlled workflows outside the normal runtime path.
- MCP is the machine-facing interface through which these concepts are exposed and operated.

## Notes

This document should evolve as SuperNode's extension model becomes more explicit.

In particular, the long-term direction is for the extension contract to become a first-class platform concept independent from Helm chart internals. That means the schemas, status model, metrics model, secret contract, and workflow surface should gradually become explicit product-level definitions rather than implicit behavior derived from chart structure.
