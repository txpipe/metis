# Skills

These files are intended for agents guiding users through Metis cluster operations.

All skills assume the repository in this workspace is the source of truth for the Helm charts and expected behavior.

Use these skills as follows:

- `kubernetes-extension-discovery.md`: discover which Metis extensions are already installed and whether prerequisites like `control-plane` are missing.
- `kubernetes-storage-and-prereqs.md`: validate storage classes, PVC behavior, node readiness, and scheduling prerequisites before installs or upgrades.
- `cardano-relay-setup.md`: install and validate a Cardano relay workload first.
- `cardano-block-producer-upgrade.md`: upgrade an existing relay to block-producer mode from an existing pool, using debug mode first, with explicit producer topology guidance.
- `cardano-block-producer-verification.md`: explain what can be verified today from the dashboard and what still requires external confirmation.
- `cardano-block-producer-troubleshooting.md`: diagnose cases where a producer looks healthy locally but recent pool blocks are missing from the canonical external chain view.
- `cardano-node-metrics-access.md`: read raw node metrics and the derived Metis metrics payload directly from a running pod via `kubectl exec`.
- `supernode-dashboard-port-forward.md`: expose the user-facing `supernode-dashboard` locally with `kubectl port-forward`, with Grafana and Prometheus as supporting debug paths.

Operational assumptions:

- `control-plane` is expected to exist for normal workflows.
- Agents should still verify cluster state rather than assume it blindly.
- Cold keys stay offline.
- Only runtime block-producer material belongs in-cluster.
- Relay topology can start on `image-default`, but producer topology should be explicit, relay-only, and private.
