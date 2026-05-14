# Dolos Supernode Deployment

## Goal

Deploy Dolos on a supernode cluster through the Supernode MCP catalog workflow so it can act as an internal chain data source for operators and validation workflows.

## Assumptions

- `control-plane` is already installed and functional.
- The MCP server can reach Kubernetes and expose the workload lifecycle tools.
- The supported MCP deployment path currently covers Cardano Dolos networks only.

## Inputs The Agent Must Ask For

Always ask the user to confirm these values, even when proposing defaults:

- target network
- target namespace and release name
- storage class
- persistent volume size
- whether to use the default upstream relay or override it

Recommended defaults to propose:

- volume size:
  - `300Gi` for `cardano-mainnet`
  - `50Gi` for `cardano-preprod`
  - `50Gi` for `cardano-preview`

Supported networks in MCP:

- `cardano-mainnet`
- `cardano-preprod`
- `cardano-preview`

## Preflight Checks

### 1. Check Installed Extensions

Use the extension discovery skill if needed.

At minimum, inspect:

- `supernode://status`
- `supernode://extensions/catalog`
- `supernode://extensions/catalog/dolos`
- `workloads.list`

### 2. Check Storage Classes

Always ask the user to choose the storage class after checking what is actually available:

Use `cluster.storage_classes.list`.

Do not assume a default storage class name. Prefer a class the cluster marks as default when appropriate, but still confirm the choice.

### 3. Check For An Existing Relay For The Same Network

If a relay already exists on the supernode for the target network, propose using its internal service DNS as the Dolos upstream.

Use `workloads.list` and existing release metadata first.

For the MCP install path, Dolos will automatically try to resolve a same-network Cardano relay when `upstreamAddress` is not provided.

Resolution behavior:

- same network match only
- prefer a relay in the same namespace
- if still ambiguous, prefer deterministic name ordering

Common internal relay address pattern for the supported Cardano relay workflow:

- Cardano Node relay service:
  - `<release>-cardano-node.<namespace>.svc.cluster.local:3000`

If a matching relay is already installed, propose that internal URL first. If the user provides `upstreamAddress` explicitly, that wins over auto-discovery.

## Recommended Deployment Flow

1. Ask the user which network Dolos should follow.
2. Check available storage classes.
3. Ask the user to choose the storage class, proposing the most appropriate one found in the cluster.
4. Ask the user to choose the PVC size, proposing a reasonable default.
5. Check whether a relay already exists for that network.
6. If a relay exists, propose its internal service URL as the upstream.
7. If no internal relay exists, ask the user to confirm the external or custom upstream.
8. Read `supernode://extensions/catalog/dolos` so the current MCP schema and defaults drive the request.
9. Call `workloads.install` with `dryRun: true` first.
10. Review the resolved configuration, including the chosen or discovered upstream and available storage classes.
11. Call `workloads.install` with `dryRun: false` only after the plan is accepted.

## Install Pattern

Use `workloads.install` with `extensionId=dolos`.

Minimal dry-run example:

```json
{
  "extensionId": "dolos",
  "releaseName": "dolos-preview",
  "namespace": "cardano-preview",
  "dryRun": true,
  "configuration": {
    "network": "cardano-preview",
    "namespace": "cardano-preview",
    "storageClass": "<chosen-storage-class>"
  }
}
```

Useful optional fields exposed by the catalog:

- `upstreamAddress`
- `imageTag`
- `resources`
- `pvcSize`
- `exposeLoadBalancer`

Do not use raw Helm values through MCP. The typed extension configuration is the supported interface.

Prime presets still exist in the chart, but they are not part of the supported MCP Dolos workflow right now.

## Validation Checklist

### Kubernetes Checks

Use MCP workload inspection first:

- `workloads.get`
- `workloads.logs.get`
- `workloads.metrics.get`

Confirm:

- bootstrap init container completed successfully
- StatefulSet pod is `Running`
- PVC is `Bound`
- services exist for:
  - `grpc`
  - `minibf`
  - `minikupo`
  - `trp`

### Service Exposure Checks

Confirm the service ports are present:

- `grpc`
- `minibf`
- `minikupo`
- `trp`

### Basic Functional Check

Use `workloads.metrics.get` as the first functional check. The Dolos metrics payload should expose:

- `blockHeight`
- `epoch`
- `slotNum`

If you need deeper verification, port-forward `minibf` locally and query it directly.

Example:

```bash
kubectl -n <namespace> port-forward service/<service-name> 3001:3001
```

Then query it from another shell.

## Best Practices

- Always ask the user to choose storage class and confirm the proposed volume size.
- Always propose a reasonable default rather than silently choosing one.
- Prefer an internal relay URL when a matching relay already exists in the cluster.
- Keep Dolos close to the relay path that the operator already trusts.
- Treat `upstreamAddress` as required unless MCP can resolve a same-network internal relay automatically.
- Treat bootstrap as always enabled in the supported MCP deployment flow.
