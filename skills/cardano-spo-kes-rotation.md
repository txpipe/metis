# Cardano SPO KES Rotation

## Goal

Rotate the producer `KES` keys, issue a new operational certificate, deploy the
new runtime artifacts to Vault, and verify the producer recovers cleanly.

## When To Use This Skill

Use this when:

- KES remaining periods are getting low
- KES expiration is approaching
- the operator wants to rehearse the flow with a dry run

## Related Skills

- `cardano-spo-maintenance-overview.md`
- `cardano-block-producer-verification.md`
- `cardano-node-metrics-access.md`
- `cardano-block-producer-troubleshooting.md`

## Required Operator Inputs

- target network and CLI network flag
- chart path
- release name and namespace
- local operator workspace path
- Vault KV path for runtime material
- Shelley genesis path, or a way to read `slotsPerKESPeriod`
- whether this is dry run or live run

The Vault runtime path only requires:

- `kes.skey`
- `vrf.skey`
- `op.cert`

It may also include optional public or reference artifacts for operator and
agent convenience, but should not include online signing keys. If the operator
also keeps semi-cold material in Vault, use a separate salted
`kv/operator/cardano-node/<network>-<pool-slug>-<hex-salt>/...` path.

For live runs the operator must also provide the offline step that uses:

- latest `cold.counter`
- `cold.skey`

## Key Custody Boundaries

Offline machine only:

- `cold.skey`
- `cold.counter`
- `cardano-cli node issue-op-cert`

Online workstation:

- generate fresh `kes.vkey` and `kes.skey`
- query current slot
- upload `kes.skey` and `op.cert` to Vault
- roll or observe the producer restart

In-cluster:

- metrics checks
- topology checks
- secret and `VaultStaticSecret` checks

## Preflight Checks

Local online workstation:

```bash
helm list -A
kubectl get pods -n <namespace>
kubectl get vaultstaticsecret -n <namespace>
kubectl get secret -n <namespace>
```

Confirm:

- producer pod is healthy
- `VaultStaticSecret` is healthy
- synced Kubernetes secret exists
- runtime path is the expected one

If the producer is already running, fetch metrics first.

Kubernetes pod:

```bash
kubectl exec -n <namespace> <pod> -c <container> -- sh -lc 'curl -s --fail http://127.0.0.1:<metrics-port>/metrics'
```

Confirm these before rotating:

- `cardano_node_metrics_currentKESPeriod_int`
- `cardano_node_metrics_remainingKESPeriods_int`
- `cardano_node_metrics_operationalCertificateStartKESPeriod_int`
- `cardano_node_metrics_operationalCertificateExpiryKESPeriod_int`
- `cardano_node_metrics_forging_enabled`

## Step 1: Create A Scratch Workspace

Local online workstation:

```bash
export POOL_WORKDIR="$HOME/cardano-pools/<network>/<pool-slug>"
mkdir -p "$POOL_WORKDIR"/{keys-next,tx,tmp}
chmod 700 "$POOL_WORKDIR" "$POOL_WORKDIR"/keys-next
```

If using a dry run, keep all replacement artifacts under `keys-next/` and do not
overwrite the current live files.

## Step 2: Determine The New Start KES Period

Local online workstation:

```bash
export SHELLEY_GENESIS="<path-to-shelley-genesis.json>"
export SLOTS_PER_KES_PERIOD="$(jq -r '.slotsPerKESPeriod' "$SHELLEY_GENESIS")"
export CURRENT_SLOT="$(kubectl exec -n <namespace> <pod> -c <container> -- sh -lc 'cardano-cli query tip <network-flag> | jq -r .slot')"
export START_KES_PERIOD="$((CURRENT_SLOT / SLOTS_PER_KES_PERIOD))"
```

Confirm `START_KES_PERIOD` is greater than or equal to the current on-chain start
period.

## Step 3: Generate Fresh KES Keys

Local online workstation:

```bash
cardano-cli node key-gen-KES \
  --verification-key-file keys-next/kes.vkey \
  --signing-key-file keys-next/kes.skey
chmod 400 keys-next/kes.skey
```

If using the Docker fallback, replace `cardano-cli` with `${CARDANO_CLI}`.

## Step 4: Issue The New Operational Certificate

Offline machine:

```bash
cardano-cli node issue-op-cert \
  --kes-verification-key-file keys-next/kes.vkey \
  --cold-signing-key-file keys/cold.skey \
  --operational-certificate-issue-counter keys/cold.counter \
  --kes-period "${START_KES_PERIOD}" \
  --out-file keys-next/op.cert
```

Rules:

- use the latest `cold.counter`
- do not re-use the previous `kes.vkey`
- transfer back only the new `op.cert` and the new runtime `kes.skey`

## Step 5: Dry Run Stop Point

If this is a dry run, stop after the new `kes.skey`, `kes.vkey`, and `op.cert`
exist and the `START_KES_PERIOD` math is confirmed.

Dry-run validation checklist:

- current producer metrics were captured
- scratch artifacts were created under `keys-next/`
- `START_KES_PERIOD` was derived successfully
- live Vault path was confirmed
- `VaultStaticSecret` restart target was confirmed

Do not perform `vault kv put` in a dry run.

The agent should record the target workload values it discovered during
preflight and keep them in the operator notes for the live run.

## Step 6: Upload The New Runtime Artifacts

Local online workstation:

```bash
kubectl -n control-plane port-forward service/control-plane-vault 8200:8200
export VAULT_ADDR=http://localhost:8200
vault login
vault kv put kv/runtime/<workload>/<network>-<pool-slug>/block-producer \
  kes.skey=@keys-next/kes.skey \
  vrf.skey=@keys/vrf.skey \
  op.cert=@keys-next/op.cert
```

Notes:

- only `kes.skey`, `vrf.skey`, and `op.cert` are required for the live runtime
- keep `vrf.skey` unchanged unless the operator is also rotating VRF material
- the live producer should restart automatically if the `VaultStaticSecret`
  already has `rolloutRestartTargets`
- if the target environment does not use Vault-backed runtime artifacts, adapt
  this step to the operator's approved runtime artifact delivery path
- if the Vault record also carries optional convenience artifacts, preserve them
  during updates instead of accidentally dropping them

## Step 7: Confirm Restart And Secret Sync

Local online workstation:

```bash
kubectl get vaultstaticsecret -n <namespace>
kubectl get pods -n <namespace>
kubectl describe pod -n <namespace> <pod>
```

Confirm:

- `VaultStaticSecret` is healthy
- synced Kubernetes secret refreshed
- producer pod restarted successfully

## Step 8: Post-Rotation Verification

Kubernetes pod:

```bash
kubectl exec -n <namespace> <pod> -c <container> -- sh -lc 'curl -s --fail http://127.0.0.1:<metrics-port>/metrics'
```

Confirm:

- `cardano_node_metrics_currentKESPeriod_int` is present
- `cardano_node_metrics_remainingKESPeriods_int` increased relative to the old
  near-expiry state
- `cardano_node_metrics_operationalCertificateStartKESPeriod_int` reflects the
  new period
- `cardano_node_metrics_forging_enabled` is still `1`
- `OP Cert disk | chain` remains aligned in the dashboard if available

## Common Failure Cases

- wrong `cold.counter` used: regenerate offline with the latest counter
- wrong `START_KES_PERIOD`: reissue `op.cert` with corrected period
- Vault path typo: `VaultStaticSecret` will fail to sync or the secret will be
  incomplete
- producer did not restart: inspect `rolloutRestartTargets` and restart the
  StatefulSet manually only after confirming the secret content is correct
