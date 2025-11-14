# Midnight Helm Chart

This chart deploys a Midnight node on Kubernetes. It takes inspiration from the
[paritytech node chart](https://github.com/paritytech/helm-charts/tree/main/charts/node) but
focuses on the subset of features needed to run a Midnight as packaged in the
[midnight-node-docker](https://github.com/midnightntwrk/midnight-node-docker) project.

## Features

- StatefulSet with persistent storage for the `/node` data directory.
- Configurable Midnight chain preset, bootnodes, and CLI append arguments.
- Optional ConfigMap that ships the default `pc-chain-config.json` from the Docker reference.
- Integration points for supplying the node private key and Cardano DB sync connection string via existing secrets or inline values.
- Optional Postgres + Cardano DB sync workloads that can be hosted alongside the node.
- Separate Services for pod identity (headless) and traffic exposure (ClusterIP/NodePort/LoadBalancer).
- Readiness and liveness probes that reuse the container's `/health` endpoint.

## Getting Started

```shell
helm install midnight oci://oci.supernode.store/midnight \
  --set nodeKey.existingSecret.name=my-node-key \
  --set nodeKey.existingSecret.key=node.key
```

If you do not yet have a node key secret you can create one with:

```shell
kubectl create secret generic midnight-node-key \
  --from-file=node.key=/path/to/midnight-node.privatekey
```

## Local Testing With kind

You can exercise the chart end-to-end on a laptop by running it inside a local
[kind](https://kind.sigs.k8s.io/) Kubernetes cluster. The steps below assume
that `kind`, `kubectl`, `helm`, and `openssl` are installed.

1. Create a kind cluster (one control-plane node is enough):

   ```shell
   kind create cluster --name midnight
   kubectl config use-context kind-midnight
   ```

2. Prepare a namespace and Kubernetes secret that holds a node key:

   ```shell
   kubectl create namespace midnight
   openssl rand -hex 32 > /tmp/midnight-node.privatekey
   kubectl -n midnight create secret generic midnight-node-key \
     --from-file=node.key=/tmp/midnight-node.privatekey
   ```

3. (Optional) Write a values file with overrides that make local runs lighter:

   ```shell
   cat <<'EOF' > midnight-kind-values.yaml
   persistence:
     size: 5Gi
   service:
     type: ClusterIP
   EOF
   ```

4. Install the chart into the kind cluster:

   ```shell
   helm install midnight ./midnight \
     --namespace midnight \
     --values midnight-kind-values.yaml \
     --set nodeKey.existingSecret.name=midnight-node-key \
     --set nodeKey.existingSecret.key=node.key
   ```

5. Wait for the StatefulSet to reach the `Running` state and inspect the logs:

   ```shell
   kubectl -n midnight get pods
   kubectl -n midnight logs statefulset/midnight
   ```

6. Port-forward the RPC endpoint and issue a health check call:

   ```shell
   kubectl -n midnight port-forward service/midnight 9944:9944
   curl -H 'Content-Type: application/json' \
     -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
     http://127.0.0.1:9944
   ```

7. When you are done testing, uninstall the release and remove the local cluster:

   ```shell
   helm -n midnight uninstall midnight
   kind delete cluster --name midnight
   ```

## Testing

Lightweight schema checks run in CI with `helm lint` and
[kubeconform](https://github.com/yannh/kubeconform) across a few value files to
make sure rendered manifests stay valid. You can reproduce the same checks
locally from `extensions/midnight`:

```shell
helm lint .
helm lint . -f ci/values-inline-secrets.yaml
helm lint . -f ci/values-existing-secrets.yaml
helm lint . -f ci/values-managed-dbsync.yaml

helm template midnight . | kubeconform -strict -summary -
helm template midnight . -f ci/values-inline-secrets.yaml | kubeconform -strict -summary -
helm template midnight . -f ci/values-existing-secrets.yaml | kubeconform -strict -summary -
helm template midnight . -f ci/values-managed-dbsync.yaml | kubeconform -strict -summary -
```

## Managed DB Sync

Set `dbSync.managed.enabled=true` to have the chart deploy the Postgres and
Cardano DB sync containers. The Midnight node will automatically consume the
generated credentials and talk to the in-cluster Postgres instance. At a
minimum you must provide a fully synced Cardano node connection string via
`dbSync.managed.nodeConnection.value` (or reference an existing secret) and the
network the DB sync should join. The string is surfaced to the container
through `dbSync.managed.nodeConnection.envVar`, which defaults to
`CARDANO_NODE_SOCKET_PATH` so it can point at a mounted IPC socket.

```yaml
dbSync:
  managed:
    enabled: true
    dbSync:
      network: preview
    nodeConnection:
      value: /ipc/node.socket  # example path when mounting a shared socket
```

When you need to bridge to a remote Cardano node over TLS, turn on the optional
`dbSync.managed.nodeSocat` sidecar. It creates a Unix domain socket inside the
pod (default `/ipc/node.socket`) and forwards traffic to the configured
`targetHost`/`targetPort` via `socat` with OpenSSL. When this sidecar is active
the chart will automatically set the node connection environment variable to
the exposed socket unless you override it explicitly.

```yaml
dbSync:
  managed:
    enabled: true
    nodeSocat:
      enabled: true
      targetHost: yourdemeterapikey.cardano-preview.cnode-m1.demeter.run
      targetPort: 9443
```

The managed deployment creates two persistent volumes by default: one for the
Postgres data directory (`50Gi`) and one for the DB sync state (`10Gi`). These
values can be tuned or disabled through `dbSync.managed.postgres.persistence`
and `dbSync.managed.dbSync.persistence` respectively. Credentials are sourced
from `dbSync.managed.connection`; switch to
`dbSync.managed.connection.existingSecret` if you already maintain those in a
Kubernetes secret.

## Key Values

| Value | Description | Default |
|-------|-------------|---------|
| `image.repository` | Midnight node image repository | `midnightnetwork/midnight-node` |
| `node.cfgPreset` | Chain preset to join | `testnet-02` |
| `node.bootnodes` | List of bootnodes passed to the node | `[...]` |
| `node.appendArgs` | Additional CLI arguments passed through `APPEND_ARGS` | `[...]` |
| `nodeKey.existingSecret` | Reference to a secret that holds `NODE_KEY` | empty |
| `dbSync.existingSecret` | Reference to a secret that holds the Cardano DB sync connection string | empty |
| `dbSync.managed.enabled` | Deploy Postgres and DB sync pods alongside the node | `false` |
| `dbSync.managed.dbSync.network` | Cardano network the managed DB sync connects to | `preprod` |
| `dbSync.managed.nodeConnection` | Source of the fully synced node connection string for DB sync | empty |
| `dbSync.managed.nodeSocat.enabled` | Launch a `socat` sidecar that exposes a TLS-backed node socket to db-sync | `false` |
| `chainConfig.create` | Whether to create a ConfigMap with `pc-chain-config.json` | `true` |
| `persistence.enabled` | Provision PersistentVolumeClaims for chain data | `true` |
| `service.type` | Kubernetes Service type for exposing RPC/P2P/Metrics ports | `ClusterIP` |

Consult `values.yaml` for the complete list of tunables.

## Maintenance

- Bump `appVersion` and `image.tag` together when upgrading the Midnight node image.
- Update `chainConfig.data` if the upstream testnet config changes.
- Regenerate documentation as needed when introducing new values.
