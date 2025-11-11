# Hydra Node Helm Chart

This chart deploys a configurable [Hydra](https://hydra.family/head-protocol/) node
as a Metis extension. It mirrors the conventions used by the existing Cardano,
Midnight, and Dolos charts in this repository while defaulting to an offline
head suitable for rapid experiments.

## Features

- StatefulSet with persistent storage for the Hydra persistence directory.
- Optional ConfigMaps for the offline `protocol-parameters.json` and
  `utxo.json` payloads.
- Secret/ConfigMap integration points for supplying Hydra and Cardano key
  material. The chart never generates keys for you.
- Separate Services for identity (headless) and access (ClusterIP/NodePort/LoadBalancer).
- Tunable probes, resources, tolerations, and topology settings.

## Prerequisites

The chart expects you to generate and manage keys **outside** of Helm:

- A Hydra signing key (`hydra.sk`) and at least one Hydra verification key file.
  Each verification key listed under `keys.hydraVerification.items` is passed to
  `--hydra-verification-key`. Include the node's own verification key together
  with any peers you intend to operate with.
- For offline heads you must also provide the ledger genesis snapshots:
  `protocol-parameters.json` and an initial `utxo.json`. Inline defaults are
  included to match the Hydra quick-start docs, but override them with your own
  content before going live.
- For online heads set `keys.cardano.enabled=true` and supply a Cardano signing
  key, verification key, remote node address, and a `node.hydraScriptsTxId`
  value. The chart can launch a `socat` sidecar via
  `node.cardanoSocketProxy` to project the remote Cardano socket into the pod,
  or you can disable the proxy and mount a socket path manually using
  `node.extraVolumes` / `node.extraVolumeMounts`.

Create the required Kubernetes objects ahead of time or let the chart create
them from inline values:

```shell
kubectl create secret generic hydra-signing \
  --from-file=hydra.sk=/path/to/hydra.sk

kubectl create configmap hydra-verification \
  --from-file=hydra.vk=/path/to/hydra.vk
```

## Installing an Offline Node

```shell
helm install hydra-node ./hydra-node \
  --set keys.hydraSigning.existingSecret.name=hydra-signing \
  --set keys.hydraSigning.existingSecret.key=hydra.sk \
  --set keys.hydraVerification.existingConfigMap.name=hydra-verification \
  --set keys.hydraVerification.items[0].filename=hydra.vk
```

Offline mode is enabled by default (`node.offlineMode=true`) and uses the
projection of the ledger ConfigMaps mounted at `/etc/hydra`. Adjust the
inline JSON documents under `ledger.protocolParameters.data` and
`ledger.initialUtxo.data` or reference pre-existing ConfigMaps.

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

2. Prepare a namespace and populate it with the Hydra signing key and at least
   one verification key. Replace the file paths with your own key material (you
   can get test keys on the [Hydra demo](https://github.com/cardano-scaling/hydra/tree/master/demo)):

   ```shell
   kubectl create namespace hydra
   kubectl -n hydra create secret generic hydra-signing \
     --from-file=hydra.sk=/path/to/hydra.sk
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
     --set keys.hydraSigning.existingSecret.name=hydra-signing \
     --set keys.hydraSigning.existingSecret.key=hydra.sk \
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
      existingSecret:
        name: cardano-admin
        key: cardano.sk
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

## Testing

Lightweight schema checks mirror the offline and online permutations through the
`ci` values files:

```shell
helm lint . -f ci/values-offline-inline.yaml
helm lint . -f ci/values-offline-existing.yaml
helm lint . -f ci/values-online-inline.yaml

helm template hydra-offline . -f ci/values-offline-inline.yaml > /tmp/hydra-offline.yaml
helm template hydra-offline-existing . -f ci/values-offline-existing.yaml > /tmp/hydra-offline-existing.yaml
helm template hydra-online . -f ci/values-online-inline.yaml > /tmp/hydra-online.yaml
```

## Values Highlights

| Value | Description | Default |
|-------|-------------|---------|
| `node.offlineMode` | Run `hydra-node offline` with pre-seeded ledger state | `true` |
| `keys.hydraSigning.*` | Source of the Hydra signing key (`hydra.sk`) | empty |
| `keys.hydraVerification.items` | Filenames (and optional inline payloads) for verification keys | `[]` |
| `ledger.protocolParameters` | Provides `protocol-parameters.json` for offline heads | inline demo JSON |
| `ledger.initialUtxo` | Provides `utxo.json` for offline heads | inline demo JSON |
| `keys.cardano.enabled` | Switch on online mode support (requires additional settings) | `false` |
| `node.cardanoSocketProxy.enabled` | Launch a `socat` sidecar that exposes a remote Cardano node as a Unix socket | `false` |
| `service.apiPort` | WebSocket API port exposed on the Service | `4001` |
| `persistence.size` | Persistent volume claim size for the Hydra state | `5Gi` |

Consult `values.yaml` for the full matrix of options and tailoring knobs.
