# Supernode Dashboard Port Forward

## Goal

Expose the user-facing `supernode-dashboard` locally with `kubectl port-forward`. This is the preferred way for users to inspect supernode information. Grafana and Prometheus are secondary debug paths.

## Assumptions

- The cluster is reachable with `kubectl`.
- The target namespace and service or pod can be discovered.

## Preferred User-Facing Port Forward

The control-plane extension deploys a dashboard in the `control-plane` namespace:

- deployment: `supernode-dashboard`
- service: `supernode-dashboard`
- service port: `3000`

Use this first:

```bash
kubectl -n control-plane port-forward service/supernode-dashboard 3000:3000
```

If needed, you can also forward the pod directly:

```bash
kubectl -n control-plane port-forward deployment/supernode-dashboard 3000:3000
```

## Supporting Control-Plane Port Forwards

### Grafana

```bash
kubectl -n control-plane port-forward service/grafana 3000:3000
```

### Prometheus

```bash
kubectl -n control-plane port-forward pod/prometheus-prometheus-0 9090:9090
```

## Workload Dashboard Or Service Discovery

Before port-forwarding a workload-facing UI or service, identify the namespace and target object:

```bash
kubectl get svc -A
kubectl get pods -A
kubectl get endpoints -A
```

If the service name is not obvious, inspect the namespace directly:

```bash
kubectl get all -n <namespace>
```

## Service Port-Forward Pattern

Prefer service port-forward for stable user access:

```bash
kubectl -n <namespace> port-forward service/<service-name> <local-port>:<service-port>
```

## Pod Port-Forward Pattern

Use pod port-forward for one-off debugging when you know the exact pod and container port:

```bash
kubectl -n <namespace> port-forward pod/<pod-name> <local-port>:<container-port>
```

## What To Verify Before Port-Forwarding

- local port is free
- namespace is correct
- target service or pod exists
- target service port or container port is correct
- backing pod is ready enough to serve traffic

Useful checks:

```bash
kubectl get svc -n <namespace>
kubectl get pods -n <namespace>
kubectl describe svc -n <namespace> <service-name>
kubectl describe pod -n <namespace> <pod-name>
```

## Common Failure Modes

- wrong namespace
- wrong service name
- wrong local or remote port
- service exists but has no ready endpoints
- pod is not ready even though the service exists

## Best Practices

- Prefer `supernode-dashboard` as the primary user-facing dashboard.
- Prefer forwarding a service for normal operator access.
- Use pod forwarding for targeted debugging.
- Verify the target is healthy before telling the user the dashboard is broken.
- Use Grafana and Prometheus port-forwards as secondary debugging paths for observability issues.
