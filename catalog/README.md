# Extension Catalog

This directory contains the Supernode MCP extension catalog.

`extension-catalog.json` is the catalog document consumed by the MCP server. It is also the payload that should be published as the official OCI catalog artifact.

## Contract

The catalog document uses this top-level shape:

```json
{
  "schemaVersion": "supernode.extensionCatalog/v1",
  "extensions": []
}
```

Each extension entry describes the MCP-facing extension contract:

- `id`: canonical extension ID. This must match the Helm chart name.
- `name` and `description`: human-readable product metadata for agents and users.
- `versions` and `defaultVersion`: supported chart versions and the default version MCP installs.
- `configuration`: JSON Schema for accepted workload configuration.
- `secrets`: runtime secret metadata exposed to agents without returning secret values.
- `dependencies`: other extension IDs this extension depends on.
- `metrics`: JSON Schema for metrics returned by `workloads.metrics.get`.
- `metricsCollection`: pod exec metadata used by MCP to collect metrics.
- `outputs`: user-facing endpoints provided by workloads of this extension.
- `chart`: OCI Helm chart reference used by MCP install and upgrade operations.

## Trusted Sources

By default, MCP only trusts official Supernode OCI references:

- Catalog artifacts must be fetched from `oci.supernode.store`.
- Extension charts must be under `oci://oci.supernode.store/extensions/{extensionId}`.

This is intentional. The catalog influences what agents recommend and what MCP can install or execute for metrics collection. Treat it as a supply-chain input, not a cosmetic document.

For local development only, MCP can be started with:

```text
MCP_EXTENSION_CATALOG_ALLOW_UNTRUSTED=true
```

Do not enable this in production. An untrusted catalog can point MCP at untrusted charts, mislead agents through descriptions and schemas, or define unsafe metrics collection metadata.

## Publishing

The official catalog should be published as a standalone OCI artifact containing `extension-catalog.json` as a JSON layer.

Recommended layer media type:

```text
application/vnd.supernode.extension-catalog.v1+json
```

Example using `oras`:

```sh
oras push \
  oci.supernode.store/extension-catalog:0.1.0 \
  extension-catalog.json:application/vnd.supernode.extension-catalog.v1+json
```

Production deployments should prefer digest-pinned catalog references when practical:

```text
oci://oci.supernode.store/extension-catalog@sha256:<digest>
```

Tag references are supported and convenient for development or release channels, but digest references are safer because they are immutable.

## Validation

MCP validates the catalog at load time:

- `schemaVersion` must be `supernode.extensionCatalog/v1`.
- extension IDs must be unique and non-empty.
- `defaultVersion` must appear in `versions`.
- `configuration` and `metrics` must be JSON objects.
- dependency IDs must exist in the same catalog.
- chart references must be OCI references.
- by default, chart references must point to `oci://oci.supernode.store/extensions/{extensionId}`.

Longer term, official catalog artifacts should also be signed and verified before MCP accepts them.
