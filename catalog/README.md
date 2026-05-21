# MCP Catalogs

This directory contains the catalog documents consumed by the Supernode MCP server.

- `extension-catalog.json`: installable extension contracts and chart references.
- `skill-catalog.manifest.json`: editable metadata for operational skill guides.
- `skill-catalog.json`: generated, tracked skill catalog payload with embedded markdown content.

Run this after editing `skill-catalog.manifest.json` or `../skills/*.md`:

```sh
node catalog/scripts/generate-skill-catalog.mjs
```

`skill-catalog.json` is committed so MCP builds and catalog consumers have a ready-to-use bundled skill catalog. Regenerate it locally after editing skill metadata or markdown, then commit the updated artifact with the source changes.

## Extension Catalog Contract

`extension-catalog.json` uses this top-level shape:

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

## Skill Catalog Contract

`skill-catalog.json` uses this top-level shape:

```json
{
  "schemaVersion": "supernode.skillCatalog/v1",
  "skills": []
}
```

Each skill entry describes one operational guide:

- `id`: canonical URI-safe skill ID used by `supernode://skills/{skillId}`.
- `title` and `description`: human-readable metadata for agents and users.
- `tags`: discovery labels.
- `extensions`: related extension IDs, when applicable.
- `tools`: MCP tools used by the guide.
- `content`: markdown guide content embedded from `../skills/*.md`.

Edit `skill-catalog.manifest.json` rather than `skill-catalog.json` directly. The manifest stores the same metadata plus `contentPath`; the generator embeds the referenced markdown into the publishable JSON document. Because `skill-catalog.json` is generated and tracked, changes to skill metadata or markdown should include the regenerated catalog artifact.

## Trusted Sources

By default, MCP only trusts official Supernode OCI references:

- Catalog artifacts must be fetched from `oci.supernode.store`.
- Extension charts must be under `oci://oci.supernode.store/extensions/{extensionId}`.

This is intentional. Extension catalogs influence what MCP can install and execute for metrics collection. Skill catalogs are prompt-bearing operational guidance and can influence agent behavior. Treat both as supply-chain inputs, not cosmetic documents.

For local development only, MCP can be started with:

```text
MCP_EXTENSION_CATALOG_ALLOW_UNTRUSTED=true
MCP_SKILL_CATALOG_ALLOW_UNTRUSTED=true
```

Do not enable these in production.

## Publishing

The official catalogs should be published as standalone OCI artifacts containing a single JSON layer. Use the publish script rather than invoking `oras` directly; it regenerates `skill-catalog.json` immediately before pushing so the OCI artifact matches the current manifest and markdown.

Recommended extension catalog layer media type:

```text
application/vnd.supernode.extension-catalog.v1+json
```

Recommended skill catalog layer media type:

```text
application/vnd.supernode.skill-catalog.v1+json
```

Publish both catalogs with:

```sh
CATALOG_TAG=0.1.0 catalog/scripts/publish-catalogs.sh
```

The script runs the equivalent of:

```sh
node catalog/scripts/generate-skill-catalog.mjs

oras push \
  oci.supernode.store/extension-catalog:${CATALOG_TAG} \
  catalog/extension-catalog.json:application/vnd.supernode.extension-catalog.v1+json

oras push \
  oci.supernode.store/skill-catalog:${CATALOG_TAG} \
  catalog/skill-catalog.json:application/vnd.supernode.skill-catalog.v1+json
```

Production deployments should prefer digest-pinned catalog references when practical:

```text
oci://oci.supernode.store/extension-catalog@sha256:<digest>
oci://oci.supernode.store/skill-catalog@sha256:<digest>
```

Tag references are supported and convenient for development or release channels, but digest references are safer because they are immutable.

## Validation

MCP validates extension catalogs at load time:

- `schemaVersion` must be `supernode.extensionCatalog/v1`.
- extension IDs must be unique and non-empty.
- `defaultVersion` must appear in `versions`.
- `configuration` and `metrics` must be JSON objects.
- dependency IDs must exist in the same catalog.
- chart references must be OCI references.
- by default, chart references must point to `oci://oci.supernode.store/extensions/{extensionId}`.

MCP validates skill catalogs at load time:

- `schemaVersion` must be `supernode.skillCatalog/v1`.
- skill IDs must be unique, non-empty, and URI-safe.
- `title`, `description`, and `content` must be non-empty.

Longer term, official catalog artifacts should also be signed and verified before MCP accepts them.
