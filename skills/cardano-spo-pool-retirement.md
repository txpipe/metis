# Cardano SPO Pool Retirement

## Goal

Retire an existing stake pool deliberately and safely, using an offline signing
boundary for the retirement certificate transaction.

## When To Use This Skill

Use this when the operator wants to:

- retire a pool permanently
- stop advertising the pool on-chain in a future epoch
- rehearse the retirement flow with a dry run before any live submission

## Related Skills

- `cardano-spo-maintenance-overview.md`
- `cardano-spo-pool-update.md`
- `cardano-block-producer-verification.md`

## Required Operator Inputs

- target network and CLI network flag
- `cardano-cli` era command style
- local operator workspace path
- `cold.vkey`
- target retirement epoch
- fee-paying wallet and signing workflow
- whether this is dry run or live run

## Key Custody Boundaries

Offline machine:

- `cold.skey`
- any final transaction signing that uses `cold.skey`

Operator workstation:

- generate retirement certificate
- build unsigned transaction body
- submit the signed transaction

## Preflight Checks

Local online workstation:

```bash
kubectl get pods -n <producer-namespace>
helm list -A
```

Confirm:

- the correct pool is being retired
- the target epoch is intentional
- the fee-paying wallet has funds
- the operator understands retirement is not a trivial rollback

Reversing retirement means a later re-registration transaction, not an undo.

## Step 1: Generate The Retirement Certificate

Local online workstation:

```bash
mkdir -p tx
cardano-cli ${CARDANO_CLI_ERA} stake-pool deregistration-certificate \
  --cold-verification-key-file keys/cold.vkey \
  --epoch <retirement-epoch> \
  --out-file tx/pool-retire.cert
```

## Step 2: Build The Unsigned Retirement Transaction

Local online workstation:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction build \
  ${TX_IN_ARGS} \
  --change-address "$(cat keys/payment.addr)" \
  <network-flag> \
  --certificate-file tx/pool-retire.cert \
  --out-file tx/pool-retire.raw
```

Typical signing set:

- payment signing key for fees
- `cold.skey`

## Step 3: Dry Run Stop Point

If this is a dry run, stop after:

- retirement certificate exists
- unsigned transaction body exists
- target epoch is confirmed
- witness expectations are confirmed

If balanced `transaction build` fails because the selected era does not match
the connected node behavior, use the same fallback strategy as the pool-update
skill: first try the matching node era, then fall back to `transaction build-raw`
for rehearsal only.

Do not sign or submit during a dry run.

## Step 4: Offline Signing

Offline machine:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction sign \
  --tx-body-file tx/pool-retire.raw \
  --signing-key-file keys/payment.skey \
  --signing-key-file keys/cold.skey \
  <network-flag> \
  --out-file tx/pool-retire.signed
```

## Step 5: Submit And Verify

Local online workstation:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction submit \
  --tx-file tx/pool-retire.signed \
  <network-flag>
```

Verify with the operator's normal chain view or query path that the retirement
was accepted for the intended epoch.

## Operational Follow-Up

After submission, coordinate the runtime plan explicitly.

Questions to answer:

- should the producer continue running until the retirement epoch passes?
- should the relay remain available for historical or network purposes?
- should the Vault runtime path be preserved, archived, or removed later?

Do not destroy runtime material or remove the workload just because the
retirement certificate was built.

## Common Failure Cases

- wrong retirement epoch selected: rebuild before signing
- wrong pool selected: verify `cold.vkey` before submission
- missing payment funds for fees: fund before rebuilding or signing
