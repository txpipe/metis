# Cardano Node Metrics Access

## Goal

Read node metrics programmatically from a running workload using `kubectl exec`, and distinguish between raw node `/metrics` data and the derived Metis JSON payload.

## Assumptions

- The workload pod is running.
- The operator knows or can discover the namespace.
- The main node container name is known or can be discovered.

## Known Metrics Ports

Current defaults in this repo:

- `cardano-node`: `12798`
- `apex-fusion`: `12789`

If unsure, inspect the chart values or service definition before querying.

## Pod And Container Discovery

Identify the pod:

```bash
kubectl get pods -n <namespace>
```

Identify the container names if needed:

```bash
kubectl get pod -n <namespace> <pod-name> -o jsonpath='{.spec.containers[*].name}'
```

## Raw Node Metrics Access

This is the raw Prometheus-style metrics endpoint exposed locally inside the pod.

Pattern:

```bash
kubectl -n <namespace> exec <pod-name> -c <container-name> -- sh -lc \
  'curl -s --fail http://127.0.0.1:<metricsPort>/metrics || wget -qO- http://127.0.0.1:<metricsPort>/metrics'
```

Example for `cardano-node`:

```bash
kubectl -n <namespace> exec <pod-name> -c cardano-node -- sh -lc \
  'curl -s --fail http://127.0.0.1:12798/metrics || wget -qO- http://127.0.0.1:12798/metrics'
```

Example for `apex-fusion`:

```bash
kubectl -n <namespace> exec <pod-name> -c apex-fusion -- sh -lc \
  'curl -s --fail http://127.0.0.1:12789/metrics || wget -qO- http://127.0.0.1:12789/metrics'
```

## Common Raw Metrics Checks

Examples of useful targeted checks:

```bash
kubectl -n <namespace> exec <pod-name> -c <container-name> -- sh -lc \
  'curl -s --fail http://127.0.0.1:<metricsPort>/metrics || wget -qO- http://127.0.0.1:<metricsPort>/metrics' \
  | rg '^cardano_node_metrics_epoch_int '
```

```bash
kubectl -n <namespace> exec <pod-name> -c <container-name> -- sh -lc \
  'curl -s --fail http://127.0.0.1:<metricsPort>/metrics || wget -qO- http://127.0.0.1:<metricsPort>/metrics' \
  | rg 'currentKESPeriod|remainingKESPeriods|forging_enabled'
```

Other useful keys to inspect:

- `cardano_node_metrics_blockNum_int`
- `cardano_node_metrics_slotNum_int`
- `cardano_node_metrics_slotInEpoch_int`
- `cardano_node_metrics_txsProcessedNum_*`
- `cardano_node_metrics_txsInMempool_int`
- connection manager metrics
- block propagation metrics
- RTS memory / GC metrics

## Derived Metis Metrics Payload

The repo also provides a structured JSON payload through `/opt/metis/bin/metrics.sh`.

Run it with:

```bash
kubectl -n <namespace> exec <pod-name> -c <container-name> -- bash -lc '/opt/metis/bin/metrics.sh'
```

This derived payload includes:

- sync and resource metrics
- derived epoch fields
- block-producer schedule metrics
- KES metrics and op-cert metrics when available

## How To Debug Using Both Sources

Use this rule:

- if raw `/metrics` is wrong, the problem is below the dashboard and below the Metis derivation layer
- if raw `/metrics` is correct but `metrics.sh` is wrong, the problem is in the derivation script
- if `metrics.sh` is correct but the dashboard is wrong, the problem is in dashboard parsing or rendering

## Important Producer-Specific Notes

- In debug producer mode, some producer metrics do not come from raw node `/metrics`.
- KES metrics in debug mode come from `cardano-cli query kes-period-info`.
- `OP Cert disk | chain` also comes from `cardano-cli query kes-period-info`.
- Leadership schedule metrics may need initial cache warm-up before they appear.

## Best Practices

- Start with raw `/metrics` when debugging node behavior.
- Use the structured JSON payload when debugging what the dashboard should render.
- Always confirm the namespace, pod, and container before drawing conclusions.
