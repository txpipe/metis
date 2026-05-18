# Hydra Node Helm Chart

This chart deploys a configurable [Hydra](https://hydra.family/head-protocol/) node
as a Metis extension. It mirrors the conventions used by the existing Cardano,
Midnight, and Dolos charts in this repository while defaulting to an offline
head suitable for rapid experiments.

## Features

- StatefulSet with persistent storage for the Hydra persistence directory.
- Optional ConfigMaps for the offline `protocol-parameters.json` and
  `utxo.json` payloads.
- VaultStaticSecret integration for Hydra and Cardano signing key material.
  The chart never generates keys for you and does not accept direct Kubernetes
  Secret or inline signing-key sources.
- Separate Services for identity (headless) and access (ClusterIP/NodePort/LoadBalancer).
- Mandatory PodMonitor and a Metis `/opt/metis/bin/metrics.sh` script for
  Prometheus and MCP metrics collection.
- Tunable probes, resources, tolerations, and topology settings.

## Prerequisites

The chart expects you to generate and manage keys **outside** of Helm. With MCP,
use `hydra.keys.generate` to replicate `hydra-node gen-hydra-key` and write both
`hydra.sk` and `hydra.vk` to a `runtime/...` Vault path. The tool returns only
the public verification key payload for chart configuration.

Required key material:

- A Hydra signing key (`hydra.sk`) and at least one Hydra verification key file.
  Each verification key listed under `keys.hydraVerification.items` is passed to
  `--hydra-verification-key`. Include the node's own verification key together
  with any peers you intend to operate with.
- For offline heads you must also provide an `node.offlineHeadSeed`, ledger
  protocol parameters, and an initial `utxo.json`. Inline defaults are included
  to match quick experiments, but override them with your own content before
  sharing a head with peers.
- For online heads set `keys.cardano.enabled=true` and supply a Cardano signing
  key, verification key, remote node address, and a `node.hydraScriptsTxId`
  value. The chart can launch a `socat` sidecar via
  `node.cardanoSocketProxy` to project the remote Cardano socket into the pod,
  or you can disable the proxy and mount a socket path manually using
  `node.extraVolumes` / `node.extraVolumeMounts`.

Write the required signing key material into Vault ahead of time and let the
chart render VaultStaticSecret resources that sync Kubernetes Secret
destinations for the pod. If you are not using MCP key generation, the equivalent
manual workflow is:

```shell
vault kv put kv/runtime/hydra/demo/hydra-signing \
  hydra.sk=@/path/to/hydra.sk

kubectl create configmap hydra-verification \
  --from-file=hydra.vk=/path/to/hydra.vk
```

## Installing an Offline Node

```shell
helm install hydra-node ./hydra-node \
  --set keys.hydraSigning.vaultStaticSecret.path=runtime/hydra/demo/hydra-signing \
  --set keys.hydraVerification.existingConfigMap.name=hydra-verification \
  --set keys.hydraVerification.items[0].filename=hydra.vk
```

Offline mode is enabled by default (`node.offlineMode=true`) and uses the
projection of the ledger ConfigMaps mounted at `/etc/hydra`. The default
offline head seed is `0001`. Adjust `node.offlineHeadSeed`, the inline JSON
documents under `ledger.protocolParameters.data` and `ledger.initialUtxo.data`,
or reference pre-existing ConfigMaps.

## Local Testing With kind

You can run the chart end-to-end on a laptop by deploying it to a local
[kind](https://kind.sigs.k8s.io/) Kubernetes cluster. The walkthrough below
assumes that `kind`, `kubectl`, and `helm` are already installed and that you
have a Hydra key pair available on disk.

1. Create a kind cluster dedicated to the experiment:

   ```shell
   kind create cluster --name hydra
   kubectl config use-context kind-hydra
   ```

2. Prepare a namespace and populate Vault with the Hydra signing key. Also
   create at least one public verification key ConfigMap. Replace the file paths
   with your own key material (you can get test keys on the
   [Hydra demo](https://github.com/cardano-scaling/hydra/tree/master/demo)):

   ```shell
   kubectl create namespace hydra
   vault kv put kv/runtime/hydra/kind/hydra-signing \
     hydra.sk=@/path/to/hydra.sk
   kubectl -n hydra create configmap hydra-verification \
     --from-file=hydra.vk=/path/to/hydra.vk
   ```

3. (Optional) Write a values file with tweaks that make local runs easier to
   iterate on, such as disabling persistence or reducing probe delays:

   ```shell
   cat <<'EOF' > hydra-kind-values.yaml
   persistence:
     enabled: false
   livenessProbe:
     initialDelaySeconds: 10
   readinessProbe:
     initialDelaySeconds: 5
   EOF
   ```

4. Install the chart into the kind cluster, wiring the namespace, values file,
   and key references together:

   ```shell
helm install hydra ./hydra-node \
      --namespace hydra \
      --values hydra-kind-values.yaml \
      --set keys.hydraSigning.vaultStaticSecret.path=runtime/hydra/kind/hydra-signing \
      --set keys.hydraVerification.existingConfigMap.name=hydra-verification \
      --set keys.hydraVerification.items[0].filename=hydra.vk
   ```

5. Wait for the StatefulSet to settle and inspect the pod logs to confirm the
   node is running in offline mode:

   ```shell
   kubectl -n hydra get pods
   kubectl -n hydra logs statefulset/hydra-hydra-node
   ```

6. Port-forward the API service locally and issue a health check against the
   REST endpoint:

   ```shell
   kubectl -n hydra port-forward service/hydra-hydra-node 4001:4001
   curl http://127.0.0.1:4001/health
   ```

7. Tear down the release and cluster when you are done experimenting:

   ```shell
   helm -n hydra uninstall hydra
   kind delete cluster --name hydra
   ```

## Switching to Online Mode

Set `node.offlineMode=false`, enable the Cardano key bundle, and describe how to
reach your upstream Cardano node:

```yaml
node:
  offlineMode: false
  hydraScriptsTxId: abcd1234...   # tx id of the deployed Hydra scripts
  cardanoSocketProxy:
    enabled: true
    targetHost: relay.cardano.example.com
    targetPort: 3001
    targetScheme: OPENSSL         # optional, defaults to OPENSSL
    targetOptions: verify=0       # optional OpenSSL flags passed to socat
    socketPath: /ipc/node.socket  # optional, defaults to keys.cardano.socketPath

keys:
  cardano:
    enabled: true
    socketPath: /ipc/node.socket
    signing:
      vaultStaticSecret:
        path: runtime/hydra/demo/cardano-signing
      filename: cardano.sk
    verification:
      existingConfigMap:
        name: cardano-admin
      filename: cardano.vk
```

With the proxy enabled the chart injects a `socat` sidecar that listens on the
Unix socket at `/ipc/node.socket` and forwards data to the specified remote node
address. If you already mount a socket into the pod (for example via a shared
PVC), disable the proxy (`node.cardanoSocketProxy.enabled=false`) and use
`node.extraVolumes` / `node.extraVolumeMounts` instead.

## Metrics

The chart always renders a `PodMonitor` for the `monitoring` port and mounts a
Metis metrics script at `/opt/metis/bin/metrics.sh`. The script reads Hydra's
Prometheus endpoint plus selected HTTP API endpoints and returns a compact JSON
payload for MCP and dashboard consumers.

The derived payload intentionally summarizes snapshot UTxO instead of returning
full UTxO entries:

```json
{
  "type": "hydra-node",
  "mode": "offline",
  "headStatus": null,
  "headId": null,
  "hydraNodeVersion": null,
  "currentSlot": null,
  "chainSyncedStatus": null,
  "peersConnected": null,
  "pendingDeposits": null,
  "snapshotNumber": null,
  "snapshotVersion": null,
  "confirmedUtxoCount": null,
  "confirmedLovelace": null,
  "lastSeenSnapshotTag": null,
  "requestedTx": null,
  "confirmedTx": null,
  "inputs": null,
  "txConfirmationTimeMsCount": null,
  "txConfirmationTimeMsSum": null,
  "txConfirmationTimeMsAvg": null,
  "errors": []
}
```

Raw Hydra metrics used by the script include:

- `hydra_head_confirmed_tx`
- `hydra_head_inputs`
- `hydra_head_peers_connected`
- `hydra_head_requested_tx`
- `hydra_head_tx_confirmation_time_ms_*`

## MCP And Secret Custody

Signing keys must be passed as runtime Vault references, not raw Helm values and
not pre-existing Kubernetes Secrets. Use `vault.runtime.write` to stage runtime
material under `kv/runtime/...`, then configure the extension with a
`vaultStaticSecret` reference. MCP should never read or echo Hydra or Cardano
signing key values.

## Testing

Lightweight schema checks mirror the offline and online permutations through the
`ci` values files:

```shell
helm lint . -f ci/values-offline-inline.yaml
helm lint . -f ci/values-offline-existing.yaml
helm lint . -f ci/values-online-inline.yaml
helm lint . -f ci/values-vault-static-secret.yaml

helm template hydra-offline . -f ci/values-offline-inline.yaml > /tmp/hydra-offline.yaml
helm template hydra-offline-existing . -f ci/values-offline-existing.yaml > /tmp/hydra-offline-existing.yaml
helm template hydra-online . -f ci/values-online-inline.yaml > /tmp/hydra-online.yaml
helm template hydra-vault . -f ci/values-vault-static-secret.yaml > /tmp/hydra-vault.yaml
```

## Values Highlights

| Value | Description | Default |
|-------|-------------|---------|
| `node.offlineMode` | Run without Cardano L1 connectivity using pre-seeded ledger state | `true` |
| `node.offlineHeadSeed` | Hexadecimal offline head seed shared by offline participants | `0001` |
| `node.peers` | Static peer endpoints passed as `--peer` values | `[]` |
| `keys.vaultAuth.ref` | Shared VaultAuth reference used by chart-managed VaultStaticSecret resources | `control-plane/default` |
| `keys.hydraSigning.vaultStaticSecret.*` | Required VaultStaticSecret that syncs the Hydra signing key into Kubernetes | path empty |
| `keys.hydraVerification.items` | Filenames (and optional inline payloads) for verification keys | `[]` |
| `ledger.protocolParameters` | Provides `protocol-parameters.json` for offline heads | inline demo JSON |
| `ledger.initialUtxo` | Provides `utxo.json` for offline heads | inline demo JSON |
| `keys.cardano.enabled` | Switch on online mode support (requires additional settings) | `false` |
| `keys.cardano.signing.vaultStaticSecret.*` | Required VaultStaticSecret that syncs the Cardano signing key into Kubernetes when online mode is enabled | path empty |
| `node.cardanoSocketProxy.enabled` | Launch a `socat` sidecar that exposes a remote Cardano node as a Unix socket | `false` |
| `service.apiPort` | WebSocket API port exposed on the Service | `4001` |
| `service.monitoringPort` | Hydra Prometheus metrics port scraped by the mandatory PodMonitor | `6001` |
| `persistence.size` | Persistent volume claim size for the Hydra state | `5Gi` |

Consult `values.yaml` for the full matrix of options and tailoring knobs.
