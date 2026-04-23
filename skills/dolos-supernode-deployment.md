# Dolos Supernode Deployment

## Goal

Deploy Dolos on a supernode cluster so it can act as an internal chain data source for operators and validation workflows.

## Assumptions

- `control-plane` is already installed and functional.
- The agent has `kubectl` and `helm` access to the target cluster.
- The Dolos chart in this repository is the deployment source of truth.

## Inputs The Agent Must Ask For

Always ask the user to confirm these values, even when proposing defaults:

- target network
- target namespace and release name
- storage class
- persistent volume size
- display name
- whether to use the default upstream relay or override it

Recommended defaults to propose:

- display name:
  - `Dolos Prime Testnet` for `prime-testnet`
  - `Dolos Prime Mainnet` for `prime-mainnet`
  - `Dolos <Network>` for other networks
- volume size:
  - `50Gi` for `prime-testnet`
  - `50Gi` for `prime-mainnet`
  - `20Gi` only for lightweight/test usage

## Preflight Checks

### 1. Check Installed Extensions

Use the extension discovery skill if needed.

At minimum:

```bash
helm list -A -o json
kubectl get ns
kubectl get pods -A
```

### 2. Check Storage Classes

Always ask the user to choose the storage class after checking what is actually available:

```bash
kubectl get storageclass
```

Do not assume a default storage class name.

### 3. Check For An Existing Relay For The Same Network

If a relay already exists on the supernode for the target network, propose using its internal service DNS as the Dolos upstream.

Useful checks:

```bash
helm list -A -o json
kubectl get svc -A
```

Common internal relay address patterns:

- Apex Fusion relay service:
  - `<release>-apex-fusion.<namespace>.svc.cluster.local:6000`
- Cardano Node relay service:
  - `<release>-cardano-node.<namespace>.svc.cluster.local:3000`

If a matching relay is already installed, propose that internal URL first.

## Recommended Deployment Flow

1. Ask the user which network Dolos should follow.
2. Check available storage classes.
3. Ask the user to choose the storage class, proposing the most appropriate one found in the cluster.
4. Ask the user to choose the PVC size, proposing a reasonable default.
5. Ask the user to choose the display name, proposing a reasonable default.
6. Check whether a relay already exists for that network.
7. If a relay exists, propose its internal service URL as the upstream.
8. If no internal relay exists, ask the user to confirm the external or custom upstream.
9. Build the Helm values.
10. Install or upgrade the Dolos release.

## Prime Testnet

For `prime-testnet`, the Dolos chart now supports a built-in preset with:

- bundled genesis files
- network magic `3311`
- default upstream relay `relay-0.prime.testnet.apexfusion.org:5521`
- `bootstrap relay`

Minimal values example:

```yaml
displayName: "Dolos Prime Testnet"

dolos:
  network: prime-testnet

persistence:
  storageClass: "<chosen-storage-class>"
  size: 50Gi
```

If an internal relay already exists, propose overriding the upstream with that service instead.

Example using an internal Apex Fusion relay:

```yaml
displayName: "Dolos Prime Testnet"

dolos:
  network: prime-testnet

config:
  upstreamAddress: "prime-testnet-relay-apex-fusion.prime-testnet-relay.svc.cluster.local:6000"

persistence:
  storageClass: "<chosen-storage-class>"
  size: 100Gi
```

## Prime Mainnet

For `prime-mainnet`, the Dolos chart now supports a built-in preset with:

- bundled genesis files
- network magic `764824073`
- default upstream relay `relay-g1.prime.mainnet.apexfusion.org:5521`
- `bootstrap relay`

Minimal values example:

```yaml
displayName: "Dolos Prime Mainnet"

dolos:
  network: prime-mainnet

persistence:
  storageClass: "<chosen-storage-class>"
  size: 50Gi
```

If an internal relay already exists, propose overriding the upstream with that service instead.

## Install Pattern

```bash
helm install <release> ./extensions/dolos \
  --namespace <namespace> \
  --create-namespace \
  -f my-values.yaml
```

For upgrades:

```bash
helm upgrade <release> ./extensions/dolos \
  --namespace <namespace> \
  -f my-values.yaml
```

## Validation Checklist

### Kubernetes Checks

```bash
kubectl get pods -n <namespace>
kubectl get pvc -n <namespace>
kubectl get svc -n <namespace>
kubectl describe pod -n <namespace> <pod-name>
```

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

```bash
kubectl get svc -n <namespace> <service-name> -o yaml
```

### Basic Functional Check

If `minibf` is exposed through the service, port-forward it locally and verify it responds.

Example:

```bash
kubectl -n <namespace> port-forward service/<service-name> 3001:3001
```

Then query it from another shell.

## Best Practices

- Always ask the user to choose storage class, size, and display name.
- Always propose a reasonable default rather than silently choosing one.
- Prefer an internal relay URL when a matching relay already exists in the cluster.
- Keep Dolos close to the relay path that the operator already trusts.
- Use the built-in `prime-testnet` preset instead of a custom config unless there is a concrete reason not to.
