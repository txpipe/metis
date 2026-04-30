# Cardano SPO Maintenance Overview

## Goal

Guide an agent through ongoing Cardano SPO maintenance on Metis-managed
workloads after the pool and producer already exist.

## When To Use This Skill

Use this as the entry point when the operator wants to:

- rotate `KES` keys
- update stake pool registration parameters
- retire a stake pool

Then switch into the task-specific skill.

## Related Skills

- `cardano-spo-kes-rotation.md`
- `cardano-spo-pool-update.md`
- `cardano-spo-pool-retirement.md`
- `cardano-block-producer-upgrade.md`
- `cardano-block-producer-verification.md`
- `cardano-block-producer-troubleshooting.md`
- `cardano-node-metrics-access.md`
- `supernode-dashboard-port-forward.md`

## Maintenance Task Matrix

| Task | Cold keys required | On-chain tx required | Cluster/Vault change required |
| --- | --- | --- | --- |
| `KES` rotation | yes | no | yes |
| Pool update | yes | yes | maybe |
| Pool retirement | yes | yes | maybe |

Interpretation:

- `KES` rotation is an offline artifact-generation workflow followed by online
  runtime deployment.
- pool update is a hybrid workflow: online build, offline signing, online
  submit.
- pool retirement is also a hybrid workflow.

## Key Custody Boundaries

Cold or offline machine only:

- `cold.skey`
- `cold.counter`
- any signing operation involving `cold.skey`

Operator workstation only:

- transaction body creation
- metadata file creation and hashing
- Vault uploads of runtime artifacts
- Helm upgrades and Kubernetes inspection

In-cluster only:

- node socket queries when local socket access is unavailable
- pod metrics checks
- topology inspection

Do not place these in the producer-mounted Vault runtime path or Kubernetes by
default:

- `cold.skey`
- `cold.counter`
- `payment.skey`
- `stake.skey`

If the operator wants online signing keys in Vault for operator convenience,
keep them in a separate salted operator Vault path such as
`kv/operator/cardano-node/<network>-<pool-slug>-<hex-salt>/...` that is not
mounted into the producer pod by the chart. That is safer than leaving
sensitive files on an unprotected workstation filesystem, but cold keys are
still best kept on separate offline or air-gapped devices.

Required producer runtime material for the normal Vault runtime path:

- `kes.skey`
- `vrf.skey`
- `op.cert`

Optional convenience artifacts that may live in the same Vault record for agent
and operator discovery:

- `kes.vkey`
- `vrf.vkey`
- `cold.vkey`
- `payment.addr`
- `reward.addr`
- `pool.id`
- `pool.id.bech32`
- pool metadata JSON and hash
- relay publication data

Interpretation:

- the required runtime set is the only part the node needs to run
- the optional convenience set is not required by the chart, but can help future
  agents know what to do without hunting through operator workspaces
- online signing keys should remain separate from the producer runtime bundle

## Command Location Model

Every command block in the task-specific skills should be labeled as one of:

- `Local online workstation`
- `Offline machine`
- `Kubernetes pod`

Also label whether it:

- touches signing keys
- changes live cluster state
- changes live ledger state

## Dry-Run Rules

Dry runs must stop before any irreversible or live-mutating step.

Allowed in dry runs:

- reading live metrics and topology
- generating replacement local artifacts in a scratch workspace
- generating updated certificates locally
- building unsigned transaction bodies
- estimating fees and witness counts
- validating Vault path wiring and restart targets

Not allowed in dry runs unless the operator explicitly asks for a staged test:

- `vault kv put` to the live runtime path
- `helm upgrade`
- signed transaction submission
- retirement submission

If the operator wants a deeper rehearsal, use a separate scratch workspace and a
separate non-live Vault path.

## Required Operator Inputs

Collect these before producing final commands:

- target network and CLI network flag
- target release name and namespace
- chart path and chart family, for example `./extensions/cardano-node` or
  `./extensions/apex-fusion`
- local operator workspace path
- whether the flow is dry run or live run

Also collect task-specific inputs from the task skill.

## Local CLI Fallback

If `cardano-cli` is missing locally but Docker is available, use the chart's
Cardano image with a narrow workspace mount:

```bash
export POOL_WORKDIR="$HOME/cardano-pools/<network>/<pool-slug>"
export CARDANO_CLI_IMAGE="ghcr.io/blinklabs-io/cardano-node:10.5.1"
export CARDANO_CLI="docker run --rm --entrypoint cardano-cli -v ${POOL_WORKDIR}:/work -w /work ${CARDANO_CLI_IMAGE}"
${CARDANO_CLI} --version
```

Then replace `cardano-cli ...` with `${CARDANO_CLI} ...` in workstation
commands.

## Choosing A Dry-Run Target

For a dry run, select an already-running producer workload that represents the
same chart family and custody model the operator plans to maintain.

Collect:

- release name
- namespace
- network and network magic
- Vault runtime path if Vault-backed runtime artifacts are used
- pool id
- local pool and wallet workspace paths

The task-specific skills should then be executed against that selected
workload without introducing references to a particular cluster or pool.

## Confirmation Checkpoints

Pause and ask the operator to confirm before:

- any `vault kv put` to a live runtime path
- any `helm upgrade`
- any offline signing step
- any transaction submission
- any pool retirement target epoch selection
