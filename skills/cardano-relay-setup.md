# Cardano Relay Setup

## Goal

Install and validate a Cardano relay workload first. Relay-first is the base path before any block-producer activation.

## Assumptions

- The cluster is reachable with `kubectl` and `helm`.
- `control-plane` is expected to exist, but should still be verified when diagnosing problems.
- The operator has selected the target chart:
  - `extensions/cardano-node`
  - `extensions/apex-fusion`

## Required Inputs

- target namespace
- target release name
- target chart path
- target network
- storage class, if the default is not correct for the cluster
- any required tolerations or resource overrides

## Best Practices To Follow

- Start as a relay first.
- Keep system time accurate on every node involved.
- Open only the ports actually required by the workload and topology.
- Use hardened SSH and avoid password-based logins on underlying hosts.
- Validate storage classes before installation.

## Relay Topology Guidance

Relay topology is operationally important, but it is less restrictive than block
producer topology.

Recommended stance for Metis:

- using `node.topology.mode=image-default` for a relay is a reasonable starting point
- move to an explicit relay topology later only if you want tighter peer control or propagation tuning
- the block producer is the node that must have explicit private topology

For a relay:

- `image-default` is acceptable initially
- explicit topology is optional tuning
- the relay can face the broader Cardano network

For a producer later:

- do not use `image-default`
- connect only to your own relays or relays you explicitly trust
- keep `publicRoots` empty
- disable ledger peers with `useLedgerAfterSlot: -1`

## Preflight Checks

### Discover Installed Extensions

```bash
helm list -A -o json
kubectl get ns
```

### Check Storage Classes

```bash
kubectl get storageclass
```

If storage classes are unclear, inspect the likely candidate before install:

```bash
kubectl describe storageclass <storage-class-name>
```

### Check Cluster Scheduling Basics

```bash
kubectl get nodes
kubectl get events -A --sort-by=.lastTimestamp
```

## Install Pattern

### Cardano Node

For non-mainnet networks, set the network magic explicitly when needed.

Example pattern:

```bash
helm install <release> ./extensions/cardano-node \
  --namespace <namespace> \
  --create-namespace \
  --set node.network=<network> \
  --set node.networkMagic=<magic> \
  --set persistence.storageClass=<storage-class>
```

If you want the simple default relay networking path, leave topology at the
chart default `image-default`.

### Apex Fusion

The chart derives built-in testnet network magic automatically for supported networks.

Known built-in values:

- `vector-testnet` -> `1`
- `prime-testnet` -> `3311`

Example pattern:

```bash
helm install <release> ./extensions/apex-fusion \
  --namespace <namespace> \
  --create-namespace \
  --set node.network=<network> \
  --set persistence.storageClass=<storage-class>
```

Only override `node.networkMagic` when the network is non-standard or the chart default does not apply.

## Post-Install Validation

Check the namespace:

```bash
kubectl get all -n <namespace>
kubectl get pvc -n <namespace>
kubectl get pods -n <namespace>
```

Healthy relay expectations:

- PVCs are `Bound`
- StatefulSet pod is `Running`
- readiness is stable
- node metrics are exposed
- sync is progressing
- transaction processing is not stuck at zero for long periods on a healthy synced node

Topology interpretation:

- a healthy relay can run on `image-default`
- relay topology can be tightened later for peer diversity or propagation tuning
- do not copy relay networking assumptions onto the producer; the producer should be explicit and private

If the dashboard is available, validate:

- sync-related metrics
- node/resource metrics
- connectivity metrics

## Common Failure Modes

- wrong storage class
- PVC pending forever
- pod unschedulable because of node taints or resource requests
- wrong network magic
- unsupported network value for the selected chart
- missing tolerations in constrained clusters

## Escalation Rule

Do not continue to block-producer steps until the relay installation is healthy and stable.
