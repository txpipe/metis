# Skills

These files are intended for agents guiding users through Metis cluster operations.

All skills assume the repository in this workspace is the source of truth for the Helm charts and expected behavior.

Use these skills as follows:

- `kubernetes-extension-discovery.md`: discover which Metis extensions are already installed and whether prerequisites like `control-plane` are missing.
- `kubernetes-storage-and-prereqs.md`: validate storage classes, PVC behavior, node readiness, and scheduling prerequisites before installs or upgrades.
- `cardano-relay-setup.md`: install and validate a Cardano relay workload first.
- `cardano-stake-pool-from-scratch.md`: guide a human operator through creating a new Cardano stake pool on any supported network, including key custody, metadata, pool registration, Vault runtime upload, and debug-first producer activation.
- `cardano-block-producer-upgrade.md`: upgrade an existing relay to block-producer mode from an existing pool, using debug mode first, with explicit producer topology guidance.
- `cardano-spo-maintenance-overview.md`: choose the right ongoing SPO maintenance workflow, understand custody boundaries, and apply dry-run rules before touching live Vault or the live ledger.
- `cardano-spo-kes-rotation.md`: rotate `KES` keys and issue a new `op.cert`, with explicit offline custody boundaries and Vault-backed producer rollout guidance.
- `cardano-spo-pool-update.md`: update an existing stake pool registration, including metadata, relays, pledge, cost, margin, reward account, owner set, and optional VRF changes.
- `cardano-spo-pool-retirement.md`: retire a stake pool with a deliberate epoch choice and offline signing flow.
- `cardano-block-producer-verification.md`: explain what can be verified today from the dashboard and what still requires external confirmation.
- `cardano-block-producer-troubleshooting.md`: diagnose cases where a producer looks healthy locally but recent pool blocks are missing from the canonical external chain view.
- `dolos-supernode-deployment.md`: deploy Dolos on the supernode cluster, including storage-class selection, size/display-name prompts, and internal relay upstream selection.
- `cardano-node-metrics-access.md`: read raw node metrics and the derived Metis metrics payload directly from a running pod via `kubectl exec`.
- `supernode-dashboard-port-forward.md`: expose the user-facing `supernode-dashboard` locally with `kubectl port-forward`, with Grafana and Prometheus as supporting debug paths.

Operational assumptions:

- `control-plane` is expected to exist for normal workflows.
- Agents should still verify cluster state rather than assume it blindly.
- Cold keys stay offline.
- Runtime block-producer material belongs in-cluster through Vault; related online maintenance artifacts can live in the same Vault record, but cold/payment/stake signing keys should stay outside by default.
- The required producer runtime set is only `kes.skey`, `vrf.skey`, and `op.cert`; additional public/reference artifacts in the same Vault record are optional convenience for operators and agents.
- If an operator wants payment or stake signing keys in Vault, use a separate operator-only path rather than the producer-mounted `VaultStaticSecret` path.
- New pool creation must start from an explicit network profile; do not silently assume Preview, Preprod, or mainnet.
- If `cardano-cli` is missing locally, use the chart's Cardano node image through Docker with a narrow key-workspace mount.
- Dry runs should stop before live `vault kv put`, `helm upgrade`, or transaction submission unless the operator explicitly asks for a staged mutation test.
- Set explicit Cardano-node resource requests and limits as a normal Kubernetes practice.
- On GKE Autopilot specifically, watch for memory evictions and autoscaler delays during Mithril restore/sync.
- Relay topology can start on `image-default`, but once a producer is attached the relay should also be explicit: producer in `localRoots`, public relays in `publicRoots`. Producer topology should remain explicit, relay-only, and private.
