# Kubernetes Extension Discovery

## Goal

Determine which Metis extensions are already installed in the cluster and whether required prerequisites are present before attempting a relay install or a block-producer upgrade.

## Assumptions

- The operator has `kubectl` and `helm` access to the target cluster.
- This repository contains the expected Metis extension set.

## Current Metis Extensions In This Repo

- `control-plane`
- `cardano-node`
- `apex-fusion`
- `hydra-node`
- `dolos`
- `midnight`

## Primary Discovery Commands

Use Helm releases as the main source of truth:

```bash
helm list -A -o json
```

Useful supporting checks:

```bash
kubectl get ns
kubectl get pods -A
kubectl get statefulsets -A
kubectl get services -A
kubectl get vaultauth -A
kubectl get vaultstaticsecret -A
kubectl get vaultconnection -A
```

## What To Look For

### Control Plane Presence

At minimum, confirm all of the following are true:

- a Helm release named `control-plane` exists
- namespace `control-plane` exists
- the shared Vault resources exist in `control-plane`
- Prometheus/Grafana/Vault-related pods are healthy enough for workload charts to depend on them

Useful checks:

```bash
helm list -A -o json | jq '.[] | {name, namespace, chart}'
kubectl get ns control-plane
kubectl get vaultauth -n control-plane
kubectl get vaultconnection -n control-plane
kubectl get pods -n control-plane
```

For normal workload behavior, `control-plane/default` should be present as the shared Vault auth reference.

### Existing Workload Presence

Before installing a relay or upgrading to producer mode, identify:

- which release name already exists
- which namespace it runs in
- which chart it is using
- whether it is already a relay or block producer

Start with:

```bash
helm list -A -o json | jq '.[] | {name, namespace, chart, app_version}'
kubectl get statefulsets -A
kubectl get pods -A
```

For a specific workload namespace:

```bash
kubectl get all -n <namespace>
kubectl get pvc -n <namespace>
kubectl get vaultstaticsecret -n <namespace>
kubectl get secret -n <namespace>
```

## When To Stop And Surface A Prerequisite Gap

Stop the workflow and tell the user clearly if any of the following are missing:

- `control-plane` Helm release is absent
- `control-plane/default` Vault auth is absent
- target namespace is wrong or missing
- workload release is not the one the user thinks it is
- PVCs or pods show that the current installation is already unhealthy

Do not continue into block-producer steps if control-plane or storage prerequisites are broken.

## Best Practices

- Prefer `helm list -A -o json` over guessing from pod names alone.
- Use namespace-scoped checks after identifying the release.
- Confirm existing workload state before proposing `helm upgrade`.
- Treat missing `control-plane` as a hard prerequisite failure for block-producer workflows.
