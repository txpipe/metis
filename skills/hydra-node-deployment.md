# Hydra Node Deployment

## Goal

Deploy a Hydra node as a Metis extension using MCP catalog-driven workload lifecycle tools and runtime secret references.

## Boundary

Use MCP for Supernode infrastructure tasks: catalog discovery, Vault runtime writes, workload install, workload status, logs, and metrics.

Do not use MCP to read Hydra or Cardano signing key values. Do not ask for raw signing key values in normal chat unless the user is explicitly using `vault.runtime.write`, and never echo the values back.

## Deployment Modes

Use `mode: offline` for local experiments and kind-cluster validation. Offline mode does not connect to Cardano L1 and requires an offline head seed, initial UTxO, Hydra ledger protocol parameters, a Hydra signing key, and Hydra verification keys.

Use `mode: online` only when the operator has deliberately selected a Cardano network, supplied Cardano fuel keys, configured a Cardano backend, and understood the mainnet risks.

## Recommended Offline Workflow

1. Ask for namespace, release name, storage class, and whether persistence should stay enabled.
2. Ask for the runtime Vault path where the Hydra key pair should be staged and synced by VaultStaticSecret.
3. Prefer `hydra.keys.generate` to create a Hydra signing/verification key pair. The tool writes both `hydra.sk` and `hydra.vk` to Vault and returns only the public verification key payload.
4. If the operator already has key material, call `vault.runtime.write` with a `runtime/...` path and the expected key such as `hydra.sk`; do not echo the value.
5. Prepare public Hydra verification key entries under `hydraVerificationKeys`; these are public and may be inline ConfigMap data. When using `hydra.keys.generate`, use the returned `verificationKey.value`.
6. Use `extensions.catalog.get` for `hydra-node` and build configuration against the schema.
7. Call `workloads.install` with `dryRun: true` first.
8. Inspect `helmValues` for only safe mapped values and no raw secret values.
9. After user approval, call `workloads.install` with `dryRun: false`.
10. Use `workloads.get`, `workloads.logs.get`, and `workloads.metrics.get` to validate startup.

Minimal offline configuration shape:

```json
{
  "namespace": "hydra",
  "storageClass": "standard",
  "mode": "offline",
  "hydraSigningKey": {
    "source": "vaultStaticSecret",
    "vaultPath": "runtime/hydra/demo/hydra-signing",
    "key": "hydra.sk"
  },
  "hydraVerificationKeys": [
    {
      "filename": "hydra.vk",
      "value": "<public hydra verification key>"
    }
  ],
  "offline": {
    "headSeed": "0001"
  }
}
```

## Online Cautions

Online mode can move real funds on L1 through Hydra lifecycle transactions. Mainnet use should be treated as high risk.

Before online install, confirm:

- Cardano signing key is runtime material and has only the fuel required for Hydra lifecycle fees.
- Cardano verification key matches the signing key and peers' participant configuration.
- Hydra verification keys match every participant.
- Every participant agrees on contestation period, deposit period, protocol parameters, network, and peer topology.
- The Cardano backend is reachable and synchronized.
- Mainnet contestation period is at least 12 hours unless the user explicitly accepts the risk.

## Validation

After install, use MCP:

- `workloads.get` to inspect services, pods, PVCs, and outputs.
- `workloads.logs.get` for bounded Hydra node logs.
- `workloads.metrics.get` for the derived Metis Hydra metrics payload.
- `cluster.events.list` scoped to the workload namespace if pods, mounts, or probes are unhealthy.

Expected outputs include `api`, `ws`, `p2p`, and `monitoring`.
