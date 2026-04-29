# Cardano SPO Pool Update

## Goal

Update an existing on-chain stake pool registration safely, using an offline
signing boundary when cold keys are not available to the agent.

## When To Use This Skill

Use this when the operator wants to change any registered pool parameter,
including:

- metadata URL or metadata hash
- relay DNS, IP, or port
- pledge
- fixed cost
- margin
- reward account
- owner set
- VRF verification key

Treat those as one pool-update workflow, not separate task families.

## Related Skills

- `cardano-spo-maintenance-overview.md`
- `cardano-spo-kes-rotation.md`
- `cardano-stake-pool-from-scratch.md`
- `cardano-block-producer-verification.md`

## Required Operator Inputs

- target network and CLI network flag
- `cardano-cli` era command style
- local operator workspace path
- payment address and signing workflow for fees
- existing pool `cold.vkey`
- current `vrf.vkey`, unless intentionally rotating it
- reward account `stake.vkey`
- pool owner stake key set
- current or new metadata URL
- current or new relay publication data
- current or new pledge, cost, and margin
- whether this is dry run or live run

If the owner set changes, also collect:

- which owners are being added or removed
- the stake verification key for each owner
- whether each owner also needs a new delegation certificate

## Key Custody Boundaries

Offline machine:

- `cold.skey`
- any signing step using `cold.skey`

Operator workstation:

- metadata file creation and hashing
- pool registration certificate generation
- delegation certificate generation from verification keys
- unsigned transaction body build
- signed transaction submission

The payment and stake signing keys may also remain offline in stricter custody
models. If so, the operator should sign the final transaction entirely offline
and return only the signed transaction for submission.

## Preflight Checks

Local online workstation:

```bash
kubectl get pods -n <producer-namespace>
helm list -A
```

Also collect the current published configuration before changing it.

Confirm:

- the intended changes are complete and explicit
- metadata hosting is ready if the metadata URL or JSON changes
- fee-paying wallet has funds
- all owner and reward account changes are intentional

## Step 1: Prepare Updated Metadata If Needed

Local online workstation:

```bash
mkdir -p metadata tx
cat > metadata/poolMetadata.json <<'EOF'
{
  "name": "<pool name>",
  "description": "<pool description>",
  "ticker": "<TICKER>",
  "homepage": "<https://pool-homepage.example>"
}
EOF
```

Rules:

- ticker should remain within the network's current limits
- metadata URL must remain short enough for registration constraints
- if the metadata file changes, publish it before building the final update cert

Calculate the metadata hash:

```bash
cardano-cli stake-pool metadata-hash \
  --pool-metadata-file metadata/poolMetadata.json > metadata/poolMetadataHash.txt
```

If the metadata URL is live, confirm the remote hash matches the local hash.

## Step 2: Decide Whether VRF Changes Are In Scope

If the operator is rotating VRF material as part of the update, use the new
`vrf.vkey` in the pool registration certificate and separately plan the runtime
change for `vrf.skey`.

If the runtime `vrf.skey` changes, coordinate the live deployment step with the
producer workload after the on-chain update is ready.

## Step 3: Build The Updated Pool Registration Certificate

Local online workstation:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-pool registration-certificate \
  --cold-verification-key-file keys/cold.vkey \
  --vrf-verification-key-file keys/vrf.vkey \
  --pool-pledge <pledge-lovelace> \
  --pool-cost <fixed-cost-lovelace> \
  --pool-margin <margin-decimal> \
  --pool-reward-account-verification-key-file keys/reward-stake.vkey \
  --pool-owner-stake-verification-key-file keys/owner1-stake.vkey \
  --pool-owner-stake-verification-key-file keys/owner2-stake.vkey \
  <network-flag> \
  --single-host-pool-relay <relay1.example.com> \
  --pool-relay-port <relay-port> \
  --metadata-url <metadata-url> \
  --metadata-hash "$(cat metadata/poolMetadataHash.txt)" \
  --out-file tx/pool-update.cert
```

Adapt the relay flags to the actual publication form:

- repeated `--single-host-pool-relay` plus `--pool-relay-port`
- repeated `--pool-relay-ipv4` plus `--pool-relay-port`
- `--multi-host-pool-relay` for SRV

If owners changed, include the full intended owner set, not just the delta.

## Step 4: Build Delegation Certificates If Owners Changed

For each owner stake key that must delegate to the pool:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-address stake-delegation-certificate \
  --stake-verification-key-file keys/<owner-stake>.vkey \
  --cold-verification-key-file keys/cold.vkey \
  --out-file tx/<owner>.deleg.cert
```

If the reward account changed but owner set did not, no new owner delegation
certificate is needed unless delegation itself also changes.

## Step 5: Build The Unsigned Transaction

Local online workstation:

Use the same UTxO discovery pattern as the pool-creation skill, then build an
unsigned transaction body that includes:

- updated pool registration certificate
- any required owner delegation certificates

Example shape:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction build \
  ${TX_IN_ARGS} \
  --change-address "$(cat keys/payment.addr)" \
  <network-flag> \
  --certificate-file tx/pool-update.cert \
  --certificate-file tx/owner1.deleg.cert \
  --out-file tx/pool-update.raw
```

Use the correct witness count for the final signing set.

Typical signers may include:

- payment signing key for fees
- `cold.skey`
- owner `stake.skey` files for any new or changed delegation witness requirements

## Step 6: Dry Run Stop Point

If this is a dry run, stop after:

- metadata hash is generated and validated
- updated pool registration certificate exists
- any required delegation certificates exist
- unsigned transaction body exists
- fee estimate and witness expectations are understood

If `transaction build` fails because the local CLI era and the connected node
era do not line up, switch the dry run to one of these fallback patterns:

- use the era command that matches the node's current era
- if balanced build still fails, use `transaction build-raw` with a real input,
  a temporary placeholder fee, and a short validity window so the unsigned body
  can still be reviewed

Record the exact era behavior observed in the target environment so the live run
can use the matching path.

Do not:

- sign the transaction
- submit the transaction
- update live Vault data unless VRF runtime rotation is explicitly part of a
  staged test

## Step 7: Offline Signing

Offline machine:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction sign \
  --tx-body-file tx/pool-update.raw \
  --signing-key-file keys/payment.skey \
  --signing-key-file keys/cold.skey \
  --signing-key-file keys/owner1-stake.skey \
  <network-flag> \
  --out-file tx/pool-update.signed
```

Only include the signing keys actually required by the final certificate set and
fee-paying wallet.

## Step 8: Submit And Verify

Local online workstation:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction submit \
  --tx-file tx/pool-update.signed \
  <network-flag>
```

Then verify through the normal external or chain-query process used by the
operator.

## Optional Step 9: Apply Matching Runtime Changes

If the update changed live producer runtime material, especially VRF signing
material, coordinate the runtime side after the on-chain step is ready.

For a Vault-managed producer, that means updating the runtime path with the new
artifact set and verifying the pod restart.

## Common Failure Cases

- metadata URL too long: shorten and rebuild the certificate
- remote metadata hash mismatch: republish and verify again
- owner set mismatch: rebuild the registration certificate with the full correct
  owner list
- missing witness: rebuild or resign with the correct signing set
- VRF update split-brain: do not leave on-chain `vrf.vkey` and runtime
  `vrf.skey` out of sync
