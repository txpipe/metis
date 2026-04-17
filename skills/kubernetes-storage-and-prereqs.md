# Kubernetes Storage And Prereqs

## Goal

Validate cluster readiness before a relay install or a producer upgrade.

## Storage Class Checks

Always inspect available storage classes before building the Helm command:

```bash
kubectl get storageclass
```

If needed, inspect a specific class:

```bash
kubectl describe storageclass <storage-class-name>
```

Do not assume names like `gp`, `standard`, or cloud-provider defaults exist in every cluster.

## PVC Checks

Before installation:

- pick the storage class deliberately
- confirm the target cluster supports dynamic provisioning for it

After installation:

```bash
kubectl get pvc -A
kubectl describe pvc -n <namespace> <pvc-name>
```

What to look for:

- PVC is `Bound`
- no repeated provisioning failures
- no access-mode mismatch
- no topology restriction preventing scheduling

## Scheduling And Node Checks

Use these when pods are pending or behaving unexpectedly:

```bash
kubectl get nodes
kubectl describe node <node-name>
kubectl get events -A --sort-by=.lastTimestamp
kubectl describe pod -n <namespace> <pod-name>
```

Check for:

- insufficient CPU or memory
- taints that require matching tolerations
- missing storage topology
- image pull issues
- namespace quota issues

## Control-Plane And Shared Infra Checks

For workflows that depend on the shared Metis base:

```bash
kubectl get pods -n control-plane
kubectl get vaultauth -n control-plane
kubectl get vaultconnection -n control-plane
```

## Best Practices

- Validate storage classes before writing the Helm install command.
- Treat PVC binding problems as first-line install failures, not chart bugs.
- Validate node readiness and scheduling constraints before changing chart values blindly.
- Prefer concrete cluster evidence from `kubectl describe` and `kubectl get events` over assumptions.
