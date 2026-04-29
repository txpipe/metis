# Cardano Stake Pool From Scratch

## Goal

Guide a human operator through creating a new Cardano stake pool and deploying a
Metis block producer for that pool.

This skill is network agnostic. The agent must first establish the target
network and command execution model, then adapt every `cardano-cli`, Helm,
Vault, and Kubernetes command to that network.

## Agent Role

The agent is the operator guide. It should:

- ask for missing operational inputs before generating commands
- explain where each command runs
- keep key custody boundaries explicit
- pause before irreversible ledger transactions
- use the repository charts and skills as the source of truth for Metis behavior
- guide the human through command execution when credentials or signing keys are
  not available to the agent

The agent should not ask the human for private key contents. It may ask for file
paths, network names, release names, namespaces, URLs, and confirmation that
funding or signing completed.

## Related Skills

Use these skills as supporting context:

- `kubernetes-extension-discovery.md`
- `kubernetes-storage-and-prereqs.md`
- `cardano-relay-setup.md`
- `cardano-block-producer-upgrade.md`
- `cardano-spo-maintenance-overview.md`
- `cardano-spo-kes-rotation.md`
- `cardano-spo-pool-update.md`
- `cardano-spo-pool-retirement.md`
- `cardano-block-producer-verification.md`
- `cardano-block-producer-troubleshooting.md`
- `cardano-node-metrics-access.md`
- `supernode-dashboard-port-forward.md`

After the pool is live, ongoing maintenance should move into the dedicated
maintenance skills instead of repeating those procedures here.

## Required Operator Inputs

Before producing final commands, collect:

- target network name, for example `preview`, `preprod`, or `mainnet`
- network CLI flag:
  - `--mainnet` for mainnet
  - `--testnet-magic <magic>` for testnets
- `cardano-cli` era command style, normally `conway` or `latest` for current
  CLI versions
- target chart path, normally `./extensions/cardano-node`
- relay release name and namespace
- block producer release name and namespace
- storage class discovered from the cluster, plus resource overrides if needed
- public relay IP and exposed relay port to publish in the pool registration
- optional public relay DNS name if the operator prefers DNS over raw IP
- pool metadata URL
- pool metadata fields:
  - name
  - ticker
  - description
  - homepage
- pledge in lovelace
- fixed pool cost in lovelace
- margin as a decimal, for example `0.02`
- reward account key choice
- owner stake key choice
- local key workspace path
- Vault KV path for runtime and online maintenance block producer material
- whether this is a testnet workflow or a mainnet-grade custody workflow

For common public networks, the usual network values are:

- Preview: `network=preview`, CLI flag `--testnet-magic 2`
- Preprod: `network=preprod`, CLI flag `--testnet-magic 1`
- Mainnet: `network=mainnet`, CLI flag `--mainnet`

Still ask the operator to confirm the target network. Do not silently assume
Preview just because the chart default is `preview`.

## Initial Conversation Pattern

Do not ask for every input in one message. Ask in small batches and use the
answers to build the next command set. When asking, propose a reasonable
default and ask the human to accept or override it.

Default naming convention:

- pool slug: lowercase ticker or short pool name, for example `metis-preview`
- relay namespace: `<network>-relay`
- relay release: `<network>-relay`
- block producer namespace: `<network>-bp`
- block producer release: `<network>-bp`
- key workspace: `$HOME/cardano-pools/<network>/<pool-slug>`
- Vault KV path: `cardano-node/<network>-<pool-slug>/block-producer`
- metadata file: `poolMetadata.json`
- metadata URL: a short HTTPS URL controlled by the operator

If existing releases already use different names, prefer the existing naming
over these defaults.

Start with:

1. "Which Cardano network are we targeting, and do you know the CLI network
   flag? Default for Preview is `--testnet-magic 2`; default for Preprod is
   `--testnet-magic 1`; mainnet uses `--mainnet`."
2. "Do you already have a healthy relay on this supernode for that network? If
   yes, I will use release `<network>-relay` in namespace `<network>-relay`
   unless you give me another name."
3. "Will this be a testnet-style local custody workflow or a mainnet-grade
   offline cold-key workflow?"
4. "Where should the operator key workspace live? I suggest
   `$HOME/cardano-pools/<network>/<pool-slug>`. If you choose a directory inside
   this Git repository, I will first make sure it is ignored by Git."

After prerequisites pass, ask:

1. "Which storage class should the relay and producer use? I will inspect
   `kubectl get storageclass` and suggest the default storage class if one is
   marked as default."
2. "What public relay IP and exposed port should be published in the pool
   registration? I will use that IP by default; if you prefer a DNS name, give me
   the DNS name and confirm it resolves to the relay."
3. "What pool metadata name, ticker, description, homepage, and metadata URL
   should be used? If you do not already have hosting, I will suggest a
   lightweight static HTTPS file."
4. "What pledge, fixed cost, and margin should be registered?"
5. "What Vault KV path should hold the runtime producer material? I suggest
   `cardano-node/<network>-<pool-slug>/block-producer`."

For each command block, label it with:

- where it runs: local workstation, offline machine, or Kubernetes pod
- whether it touches signing keys
- what output the human should confirm before continuing

Stop after each irreversible or externally visible step, especially metadata
publication, stake address registration, pool registration submission, Vault
runtime upload, and active forging activation.

## Required Tools On The Operator Workstation

The human operator needs a workstation that can reach the Kubernetes cluster and
the Cardano node socket path used for `cardano-cli` queries.

Required:

- `kubectl`
- `helm`
- `jq`
- `curl`
- `vault` CLI when Vault is accessed through a local port-forward

Recommended:

- `cardano-cli`
- Docker or another container runtime if `cardano-cli` is not installed locally
- `cardano-node` with matching or compatible version to the running network
- a secure directory or encrypted volume for key files
- a metadata hosting path that serves a stable JSON file over HTTPS

The agent should verify tool presence when it has shell access. Local
`cardano-cli` is convenient but not required if Docker can run the same node
image as the chart.

```bash
kubectl version --client
helm version
cardano-cli --version || true
cardano-cli conway --help || true
cardano-cli latest --help || true
docker --version || true
jq --version
curl --version
vault version
```

If the operator will run commands manually, ask them to run those checks and
paste only non-sensitive output.

## Where Things Live

### Operator Workstation Or Offline Machine

These files are generated and retained by the operator. They do not belong in
Kubernetes Secrets, ConfigMaps, Git, or Vault runtime paths:

- `payment.skey`
- `stake.skey`
- `cold.skey`
- `cold.counter`

These public or less-sensitive files may stay in the same operator workspace:

- `payment.vkey`
- `stake.vkey`
- `payment.addr`
- `stake.addr`
- `cold.vkey`
- `vrf.vkey`
- `kes.vkey`
- `pool.id`
- `pool.id.bech32`
- `protocol.json`
- `pool.cert`
- `deleg.cert`
- `stake.cert`
- transaction body and signed transaction files

For mainnet, cold-key operations should happen on an offline or air-gapped
machine. The online workstation may prepare unsigned transaction bodies and
submit signed transactions, but the cold signing key should not be exposed to an
online node or cluster.

For testnets, the operator may choose a simpler local workflow, but the agent
should still explain the mainnet boundary so the habit is clear.

### Vault Runtime And Online Maintenance Path

By default, use one Vault KV path for the block producer's online material. The
Metis chart consumes these runtime fields:

- `kes.skey`
- `vrf.skey`
- `op.cert`

It is also reasonable to keep related online maintenance artifacts in the same
Vault record so future agents and operators know where to find them:

- `kes.vkey`
- `vrf.vkey`
- `cold.vkey`
- `payment.addr`
- `reward.addr`
- `pool.id`
- `pool.id.bech32`
- pool metadata JSON and hash
- relay publication data

The operational certificate may be created as `node.cert` by standard Cardano
examples. In the Metis chart, upload it under the Vault field name `op.cert`.

Interpretation:

- `kes.skey`, `vrf.skey`, and `op.cert` are the only required runtime fields
- the other files listed here are optional convenience artifacts for future
  maintenance and agent discovery
- the convenience artifacts are helpful, but the node does not require them to
  run

Do not upload by default:

- `cold.skey`
- `cold.counter`
- `payment.skey`
- `stake.skey`

If the operator wants online signing keys in Vault at all, place them in a
separate operator-only Vault path that is not mounted into the producer pod.
For mainnet-grade workflows, keep cold and payment/stake signing keys outside
the producer runtime path and outside Kubernetes.

### Kubernetes Cluster

Kubernetes holds:

- relay and block producer Helm releases
- PVCs for node data
- a `VaultStaticSecret` created by the chart when producer mode is enabled
- the synced runtime Kubernetes Secret created by Vault Secrets Operator
- the mounted runtime material in the producer pod

The chart expects `control-plane/default` Vault auth unless explicitly
overridden.

### Public Metadata Host

Pool metadata must be available at a stable URL. The URL and the content hash are
included in the pool registration certificate. If the uploaded file changes, the
hash changes and the registration must be updated.

Lightweight default: use a static HTTPS file on an existing domain or static
hosting service. For testnets, a short GitHub Pages URL is often enough if it
stays under Cardano's metadata URL length limit and serves the raw JSON without
authentication. For longer-lived or mainnet pools, prefer an operator-controlled
domain, object-storage static website, or CDN-backed static file with a short,
stable URL.

## High-Level Workflow

1. Build the network profile.
2. Verify local tools, cluster access, control-plane, Vault, storage, and relay.
3. Install the relay if it does not already exist.
4. Install the future block producer as a non-forging, internal relay-mode node
   with producer topology so it can restore/sync while registration work
   continues.
5. Decide key custody and local workspace.
6. Generate wallet keys and addresses.
7. Fund the payment address.
8. Register the stake address if it is new.
9. Generate cold, KES, and VRF keys.
10. Query tip and genesis values, then issue the operational certificate.
11. Create, publish, and hash pool metadata.
12. Generate pool registration and owner delegation certificates.
13. Build, review, sign, and submit the registration transaction.
14. Verify the pool ID on-chain.
15. Upload required runtime material to Vault, plus optional convenience
    artifacts if the operator wants a richer maintenance record.
16. Upgrade the Metis producer into debug mode.
17. Validate dashboard, metrics, topology, KES, op-cert, and schedule.
18. Switch the producer from debug mode to active forging.
19. Verify forging state and external chain visibility after expected slots.

## Step 1: Build The Network Profile

Ask:

- Which network are we targeting?
- Is this a public Cardano network or a custom/devnet network?
- What CLI network flag should be used?
- What chart `node.network` value should be used?
- What chart `node.networkMagic` value should be used, if any?

Use a small shell variable block for human-run commands:

```bash
export CARDANO_NETWORK_NAME="<network>"
export CARDANO_CLI_NETWORK="--testnet-magic <magic>"
export CARDANO_CLI_ERA="conway"
export CARDANO_NODE_SOCKET_PATH="<path-to-node.socket>"
```

For mainnet:

```bash
export CARDANO_NETWORK_NAME="mainnet"
export CARDANO_CLI_NETWORK="--mainnet"
export CARDANO_CLI_ERA="conway"
export CARDANO_NODE_SOCKET_PATH="<path-to-node.socket>"
```

If the installed CLI prefers `latest` instead of `conway`, set:

```bash
export CARDANO_CLI_ERA="latest"
```

For Helm values, keep a separate values file instead of relying on many
`--set` flags once producer mode is involved.

## Step 2: Verify Metis And Cluster Prerequisites

Run from the operator workstation:

```bash
helm list -A -o json
kubectl get ns
kubectl get pods -A
kubectl get storageclass
kubectl get storageclass -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.metadata.annotations.storageclass\.kubernetes\.io/is-default-class}{"\n"}{end}'
kubectl get vaultauth -n control-plane
kubectl get vaultconnection -n control-plane
```

Confirm:

- `control-plane` exists
- `control-plane/default` Vault auth exists, unless using a custom Vault auth
- storage is available; suggest the class marked default, or the storage class
  already used by healthy Cardano workloads on the same cluster
- the target relay release exists or will be installed first
- relay pod is healthy and synced enough for transaction submission and producer
  connectivity

If no healthy relay exists, stop and use `cardano-relay-setup.md` first.
When installing a new relay for this workflow, propose the default release and
namespace names from this skill and pass the selected storage class explicitly
through relay values or `--set persistence.storageClass=<selected-storage-class>`.

Do not rely on implicit default resources for Cardano nodes. Set explicit
resource requests and limits as a normal Kubernetes practice so scheduling and
eviction behavior are intentional. Preview restore/sync can exceed a 2Gi memory
request. A practical starting point is:

```yaml
resources:
  requests:
    cpu: 1000m
    memory: 6Gi
  limits:
    memory: 6Gi
```

On GKE Autopilot specifically, a pod may stay pending while the autoscaler adds
capacity, and restore-heavy Cardano pods can be evicted if requests are too
low. In that case, wait and watch events before lowering the request. Lower
requests can make scheduling faster but also increase eviction risk during
restore.

After installing a relay, record the public endpoint:

```bash
kubectl get svc -n <relay-namespace> <relay-service> -o wide
```

For `LoadBalancer` relays, use the external IP and the exposed node-to-node
port in pool registration unless the operator provides a DNS name.

## Step 2A: Start The Future Producer Early

If the pool is being created from scratch and the producer release does not
exist yet, install it before ledger registration so chain restore starts early.
At this stage it should not be a block producer yet:

- `node.blockProducer.enabled=false`
- service remains internal `ClusterIP`
- topology is explicit and producer-like
- `localRoots` points only at the operator relay
- the producer keeps a private local connection to that relay through `localRoots`
- `publicRoots` is empty
- `useLedgerAfterSlot` is `0`

Example values:

```yaml
resources:
  requests:
    cpu: 1000m
    memory: 6Gi

persistence:
  storageClass: <selected-storage-class>

node:
  network: <network>
  networkMagic: <magic-if-testnet>
  topology:
    mode: relay-service
    useLedgerAfterSlot: -1
    relayTargets:
      - releaseName: <relay-release>
        namespace: <relay-namespace>
        chart: cardano-node
```

Install it:

```bash
helm install <producer-release> ./extensions/cardano-node \
  --namespace <producer-namespace> \
  --create-namespace \
  -f <producer-values.yaml>
```

Validate the rendered or active topology before continuing:

```bash
kubectl get configmap -n <producer-namespace> <producer-release>-cardano-node-topology \
  -o jsonpath='{.data.topology\.json}'
```

Expect:

- `localRoots` contains the relay service DNS
- `publicRoots` is `[]`
- `useLedgerAfterSlot` is `0`

Both relay and producer may spend time restoring their databases via Mithril
before node sockets and metrics are available. Kubernetes `Running` means the
pod is alive; it does not mean the Cardano node is synced yet.

## Step 3: Decide Command Locations

Use this command-location model:

- Local online workstation:
  - Kubernetes discovery
  - Helm install/upgrade
  - Vault port-forward and runtime upload
  - key generation for testnets when the operator accepts that custody
    model
  - transaction build and submit
- Offline/cold machine:
  - mainnet cold key generation
  - mainnet operational certificate issuance involving `cold.skey` and
    `cold.counter`
  - mainnet transaction signing operations involving `cold.skey`
  - KES rotation operations involving `cold.counter`
- Inside cluster/pod:
  - node socket queries when the local workstation does not have socket access
  - metrics inspection
  - topology inspection

If `cardano-cli` is not installed locally but Docker is available, use the same
Cardano node image as the chart and mount only the operator workspace:

```bash
export CARDANO_CLI_IMAGE="ghcr.io/blinklabs-io/cardano-node:10.5.1"
export CARDANO_CLI="docker run --rm --entrypoint cardano-cli -v ${POOL_WORKDIR}:/work -w /work ${CARDANO_CLI_IMAGE}"
${CARDANO_CLI} --version
```

Then replace `cardano-cli ...` in local commands with `${CARDANO_CLI} ...`.
This keeps keys local while avoiding a host-level CLI install. Do not mount the
whole repository if a narrower key workspace mount is enough.

For commands that need `cardano-cli` and the node socket, choose one of two
patterns.

Pattern A, local CLI talks to a local socket:

```bash
cardano-cli query tip ${CARDANO_CLI_NETWORK}
```

Pattern B, run the query inside the Cardano pod:

```bash
kubectl -n <namespace> exec <pod-name> -c cardano-node -- sh -lc \
  'cardano-cli query tip <network-flag>'
```

Use Pattern B only for queries or non-sensitive operations. Do not copy cold keys
into the pod.

## Step 4: Create A Key Workspace

Ask the operator for a directory path outside the Git repository.

Recommended shape:

```bash
export POOL_WORKDIR="$HOME/cardano-pools/<network>/<pool-name>"
mkdir -p "$POOL_WORKDIR"/{keys,tx,metadata}
chmod 700 "$POOL_WORKDIR" "$POOL_WORKDIR"/keys
cd "$POOL_WORKDIR"
```

If the operator deliberately chooses a path inside the Git repository, require a
Git ignore rule before generating keys. For example, if using a repo-local
`temp/` workspace:

```bash
printf '\n/temp/\n' >> .gitignore
git status --short
```

Confirm the key workspace does not appear as untracked Git content after
creation.

If the agent has shell access on the operator workstation, it may run directory
creation commands after confirming the path. Otherwise, provide the commands for
the human to run.

## Step 5: Generate Wallet Keys And Addresses

Run in the operator key workspace.

With modern `cardano-cli`, several commands are era-scoped. Prefer
`${CARDANO_CLI_ERA}` for `stake-address`, `stake-pool`, and transaction
commands. For current CLI builds, `CARDANO_CLI_ERA=conway` is usually right.

Payment keys:

```bash
cardano-cli address key-gen \
  --verification-key-file keys/payment.vkey \
  --signing-key-file keys/payment.skey
```

Stake keys:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-address key-gen \
  --verification-key-file keys/stake.vkey \
  --signing-key-file keys/stake.skey
```

Stake address:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-address build \
  --stake-verification-key-file keys/stake.vkey \
  ${CARDANO_CLI_NETWORK} \
  --out-file keys/stake.addr
```

Payment address linked to the stake key:

```bash
cardano-cli ${CARDANO_CLI_ERA} address build \
  --payment-verification-key-file keys/payment.vkey \
  --stake-verification-key-file keys/stake.vkey \
  ${CARDANO_CLI_NETWORK} \
  --out-file keys/payment.addr
```

Ask the human to fund `payment.addr` with enough ada or test ada for:

- stake address deposit
- stake pool deposit
- transaction fees
- pledge amount if the owner stake is pledging from this wallet

For public Cardano testnets, recommend the official testnet faucet as the
default funding path. Tell the human to request funds for `payment.addr` on the
selected testnet, then come back when the faucet transaction is visible. For
mainnet, funding must come from the operator's normal wallet/custody process.

Then verify funding:

```bash
cardano-cli query utxo \
  --address "$(cat keys/payment.addr)" \
  ${CARDANO_CLI_NETWORK}
```

## Step 6: Register The Stake Address

If this stake address is new, query protocol parameters first:

```bash
cardano-cli query protocol-parameters \
  ${CARDANO_CLI_NETWORK} \
  --out-file tx/protocol.json

export STAKE_ADDRESS_DEPOSIT="$(jq -r '.stakeAddressDeposit' tx/protocol.json)"
```

Create the certificate:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-address registration-certificate \
  --stake-verification-key-file keys/stake.vkey \
  --key-reg-deposit-amt "${STAKE_ADDRESS_DEPOSIT}" \
  --out-file tx/stake.cert
```

Some older CLI builds expose this command at `cardano-cli stake-address` and do
not accept `--key-reg-deposit-amt`. The agent should inspect the installed
command and adapt before telling the human to run it:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-address registration-certificate --help
```

Build the transaction using `transaction build` so the CLI calculates fees and
change:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction build \
  --tx-in <tx-hash>#<tx-ix> \
  --change-address "$(cat keys/payment.addr)" \
  ${CARDANO_CLI_NETWORK} \
  --certificate-file tx/stake.cert \
  --witness-override 2 \
  --out-file tx/stake-registration.txbody
```

Sign:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction sign \
  --tx-body-file tx/stake-registration.txbody \
  --signing-key-file keys/payment.skey \
  --signing-key-file keys/stake.skey \
  ${CARDANO_CLI_NETWORK} \
  --out-file tx/stake-registration.signed
```

Submit:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction submit \
  ${CARDANO_CLI_NETWORK} \
  --tx-file tx/stake-registration.signed
```

Pause and confirm the transaction was accepted before continuing.

## Step 7: Generate Pool Keys

Run in the key workspace. For mainnet, run cold key generation on the offline
machine.

Cold keys and counter:

```bash
cardano-cli node key-gen \
  --cold-verification-key-file keys/cold.vkey \
  --cold-signing-key-file keys/cold.skey \
  --operational-certificate-issue-counter keys/cold.counter
```

KES keys:

```bash
cardano-cli node key-gen-KES \
  --verification-key-file keys/kes.vkey \
  --signing-key-file keys/kes.skey
```

VRF keys:

```bash
cardano-cli node key-gen-VRF \
  --verification-key-file keys/vrf.vkey \
  --signing-key-file keys/vrf.skey
chmod 400 keys/vrf.skey
```

Generate pool IDs:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-pool id \
  --cold-verification-key-file keys/cold.vkey \
  --output-hex \
  --out-file keys/pool.id

cardano-cli ${CARDANO_CLI_ERA} stake-pool id \
  --cold-verification-key-file keys/cold.vkey \
  --output-bech32 \
  --out-file keys/pool.id.bech32
```

Use the Bech32 pool ID for Metis dashboard configuration unless another
consumer explicitly requires hex.

## Step 8: Issue The Operational Certificate

The operational certificate binds the cold key authority to the current KES key.

You need:

- current slot
- `slotsPerKESPeriod` from the network Shelley genesis
- latest `cold.counter`
- `kes.vkey`
- `cold.skey`

If the Shelley genesis is available locally:

```bash
export SHELLEY_GENESIS="<path-to-shelley-genesis.json>"
export SLOTS_PER_KES_PERIOD="$(jq -r '.slotsPerKESPeriod' "$SHELLEY_GENESIS")"
export CURRENT_SLOT="$(cardano-cli query tip ${CARDANO_CLI_NETWORK} | jq -r '.slot')"
export START_KES_PERIOD="$((CURRENT_SLOT / SLOTS_PER_KES_PERIOD))"
```

Issue the op cert:

```bash
cardano-cli node issue-op-cert \
  --kes-verification-key-file keys/kes.vkey \
  --cold-signing-key-file keys/cold.skey \
  --operational-certificate-issue-counter keys/cold.counter \
  --kes-period "${START_KES_PERIOD}" \
  --out-file keys/op.cert
```

For mainnet, perform the `issue-op-cert` command in the offline custody
environment, then transfer only `op.cert` and the hot keys required for runtime
according to the operator's approved process.

## Step 9: Create And Publish Pool Metadata

Create metadata locally:

```bash
cat > metadata/poolMetadata.json <<'EOF'
{
  "name": "<pool name>",
  "description": "<pool description>",
  "ticker": "<TICKER>",
  "homepage": "<https://pool-homepage.example>"
}
EOF
```

Rules to communicate:

- ticker is 3 to 5 uppercase letters or digits
- description must stay short enough for Cardano metadata limits
- metadata URL must be stable
- metadata URL must be short enough for Cardano pool registration limits
- avoid redirects when possible
- the exact bytes served at the URL must match the locally hashed file

Lightweight hosting options to suggest:

- existing operator website, for example
  `https://<domain>/cardano/<ticker>/poolMetadata.json`
- short GitHub Pages URL for testnets, if the final URL is short enough
- small static object-storage bucket with HTTPS and a short custom domain

Avoid GitHub raw or gist URLs when they become too long or redirect-heavy.

Hash local metadata:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-pool metadata-hash \
  --pool-metadata-file metadata/poolMetadata.json \
  > metadata/poolMetadataHash.txt
```

After the human publishes the metadata URL, verify the remote hash:

```bash
curl -fsSL "<metadata-url>" -o metadata/poolMetadata.remote.json

cardano-cli ${CARDANO_CLI_ERA} stake-pool metadata-hash \
  --pool-metadata-file metadata/poolMetadata.remote.json

cat metadata/poolMetadataHash.txt
```

Do not continue until the remote and local hashes match.

## Step 10: Generate Pool Registration Certificates

Ask the operator to confirm:

- pledge lovelace
- pool cost lovelace
- pool margin decimal
- reward account verification key
- owner stake verification key
- public relay IP and exposed node-to-node port
- optional public relay DNS name, if the operator prefers DNS
- metadata URL and hash

The relay registered on-chain must be reachable by the Cardano network. Do not
use the Kubernetes ClusterIP service DNS in pool registration metadata.

Discover the Kubernetes service shape:

```bash
kubectl get svc -n <relay-namespace>
kubectl describe svc -n <relay-namespace> <relay-service>
```

Then ask the operator to confirm the actual public endpoint:

- default: use the public relay IP and exposed node-to-node port
- DNS override: use a DNS name only if the operator confirms it resolves to the
  relay public IP and will remain stable
- port: use the externally exposed relay node-to-node port, not necessarily the
  pod's internal port

For a public IPv4 endpoint, use `--pool-relay-ipv4`. For a DNS endpoint, use
`--single-host-pool-relay`. For IPv6, use the matching Cardano CLI relay flag.

Query protocol parameters and inspect minimums:

```bash
cardano-cli query protocol-parameters \
  ${CARDANO_CLI_NETWORK} \
  --out-file tx/protocol.json

jq '{stakePoolDeposit, stakeAddressDeposit, minPoolCost}' tx/protocol.json
```

Generate the pool registration certificate:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-pool registration-certificate \
  --cold-verification-key-file keys/cold.vkey \
  --vrf-verification-key-file keys/vrf.vkey \
  --pool-pledge <pledge-lovelace> \
  --pool-cost <cost-lovelace> \
  --pool-margin <margin-decimal> \
  --pool-reward-account-verification-key-file keys/stake.vkey \
  --pool-owner-stake-verification-key-file keys/stake.vkey \
  ${CARDANO_CLI_NETWORK} \
  --pool-relay-ipv4 <relay-public-ipv4> \
  --pool-relay-port <relay-port> \
  --metadata-url <metadata-url> \
  --metadata-hash "$(cat metadata/poolMetadataHash.txt)" \
  --out-file tx/pool.cert
```

If the operator chooses DNS instead of a raw IP, replace the relay endpoint
lines with:

```bash
  --single-host-pool-relay <relay-dns-name> \
  --pool-relay-port <relay-port> \
```

If the pool has multiple public relays or owners, add all required relay and
owner flags. Do not invent owner keys; ask the human which owner stake keys are
authoritative.

Create the pledge delegation certificate. Current CLI builds use the pool ID
explicitly:

```bash
cardano-cli ${CARDANO_CLI_ERA} stake-address stake-delegation-certificate \
  --stake-verification-key-file keys/stake.vkey \
  --stake-pool-id "$(cat keys/pool.id.bech32)" \
  --out-file tx/deleg.cert
```

Some older CLI builds also support:

```bash
cardano-cli stake-address delegation-certificate \
  --stake-verification-key-file keys/stake.vkey \
  --cold-verification-key-file keys/cold.vkey \
  --out-file tx/deleg.cert
```

Prefer the current `stake-delegation-certificate` form when available.

## Step 11: Build, Sign, And Submit Pool Registration

Collect UTXOs:

```bash
cardano-cli query utxo \
  --address "$(cat keys/payment.addr)" \
  ${CARDANO_CLI_NETWORK}
```

Build with automatic fee and change calculation:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction build \
  --tx-in <tx-hash>#<tx-ix> \
  --change-address "$(cat keys/payment.addr)" \
  ${CARDANO_CLI_NETWORK} \
  --certificate-file tx/pool.cert \
  --certificate-file tx/deleg.cert \
  --witness-override 3 \
  --out-file tx/pool-registration.txbody
```

Before signing, ask the human to confirm:

- the target network is correct
- the relay DNS/IP and port are correct
- the metadata URL and hash are correct
- pledge, cost, and margin are correct
- the transaction includes the expected certificates
- the cold key custody model is acceptable for this network

Sign:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction sign \
  --tx-body-file tx/pool-registration.txbody \
  --signing-key-file keys/payment.skey \
  --signing-key-file keys/stake.skey \
  --signing-key-file keys/cold.skey \
  ${CARDANO_CLI_NETWORK} \
  --out-file tx/pool-registration.signed
```

For mainnet, signing may be split between online and offline machines. Keep
`cold.skey` offline and use the operator's approved transaction signing flow.

Submit:

```bash
cardano-cli ${CARDANO_CLI_ERA} transaction submit \
  ${CARDANO_CLI_NETWORK} \
  --tx-file tx/pool-registration.signed
```

## Step 12: Verify Pool Registration

Get the pool ID if not already generated:

```bash
cat keys/pool.id
cat keys/pool.id.bech32
```

Check the pool from the node:

```bash
cardano-cli query stake-snapshot \
  --stake-pool-id "$(cat keys/pool.id)" \
  ${CARDANO_CLI_NETWORK}
```

Interpret the snapshot before expecting leadership:

- if `stakeMark` is non-zero but `stakeSet` is still `0`, the pool registration
  and delegation are visible, but that stake is not active for leadership yet
- in that state, it is normal to have no assigned slots in the current epoch
- do not tell the operator to expect scheduled slots until the first epoch where
  `stakeSet` for the pool becomes non-zero

For a newly registered and delegated pool, this usually means waiting through
the current epoch and checking again in the next eligible epoch. The important
signal is not just that the pool exists, but that `stakeSet` has become
non-zero for that pool.

To check how many slots the pool is assigned in the first eligible epoch, run
the leadership schedule query from the block producer pod in debug mode, or
from any environment that has the node socket, Shelley genesis file, and VRF
signing key available. Use `--next` before that eligible epoch starts, and use
`--current` once the pool is already inside that epoch:

```bash
kubectl -n <producer-namespace> exec <producer-pod> -c cardano-node -- bash -lc '
  export CARDANO_NODE_SOCKET_PATH=/ipc/node.socket
  cardano-cli query leadership-schedule \
    ${CARDANO_CLI_NETWORK} \
    --genesis /opt/cardano/config/${CARDANO_NETWORK}/shelley-genesis.json \
    --stake-pool-id <pool-id-bech32> \
    --vrf-signing-key-file /block-producer/vrf.skey \
    --next \
    --out-file /tmp/leadership-schedule-next.json
  jq "length" /tmp/leadership-schedule-next.json
'
```

If the result is `0`, that can still be normal for a small or newly delegated
testnet pool. If `stakeSet` is non-zero and the pool still has no assigned
slots, wait for the next schedule window before assuming something is wrong.

Also check a suitable public explorer for the selected Cardano network when one
exists. Cexplorer is a common choice for Cardano networks and has network-specific
views for mainnet and public testnets. Use the pool ID or ticker to confirm that
the pool exists after registration, and later use the same explorer as one
external signal that the pool is producing blocks.

Registration confirmation is separate from block production success. Do not
claim that the pool is producing blocks just because registration is visible.

## Step 13: Upload Runtime Material To Vault

The Vault upload runs from the operator workstation.

If Vault is reachable only in-cluster, port-forward it:

```bash
kubectl -n control-plane port-forward service/control-plane-vault 8200:8200
export VAULT_ADDR=http://localhost:8200
vault status
```

At this point, the agent should keep the port-forward running and ask the human
operator to authenticate locally using their normal Vault login flow. For
example:

```bash
export VAULT_ADDR=http://localhost:8200
vault login
```

If the operator uses a token directly, they can run `vault login <token>`
instead. The agent should not ask the human to paste secrets into the chat.
`vault token lookup` is optional as a quick local sanity check, but it is not
required for the workflow.
Once the human confirms local login succeeded, upload runtime material plus the
default optional convenience artifacts to the same Vault record:

```bash
vault kv put kv/<vault-kv-path> \
  kes.skey=@keys/kes.skey \
  kes.vkey=@keys/kes.vkey \
  vrf.skey=@keys/vrf.skey \
  vrf.vkey=@keys/vrf.vkey \
  op.cert=@keys/op.cert \
  cold.vkey=@keys/cold.vkey \
  pool.id=@keys/pool.id \
  pool.id.bech32=@keys/pool.id.bech32
```

Ask the operator to confirm the Vault path. The chart value
`node.blockProducer.vaultStaticSecret.path` should omit the `kv/` mount prefix
when the mount is configured as `kv`.

The chart only requires `kes.skey`, `vrf.skey`, and `op.cert`, but keeping the
related online artifacts beside them gives future maintenance flows a single
place to look. Do not add `cold.skey`, `cold.counter`, `payment.skey`, or
`stake.skey` to the producer-mounted runtime path. If the operator wants online
signing keys in Vault at all, use a separate operator-only path.

## Step 14: Deploy Producer In Debug Mode

Create a values file for the block producer. Example shape:

```yaml
resources:
  requests:
    cpu: 1000m
    memory: 6Gi

persistence:
  storageClass: <selected-storage-class>

node:
  network: <network>
  networkMagic: <magic-if-testnet>
  topology:
    mode: relay-service
    useLedgerAfterSlot: -1
    relayTargets:
      - releaseName: <relay-release>
        namespace: <relay-namespace>
        chart: cardano-node
  blockProducer:
    enabled: true
    debug: true
    poolId: <pool-id-bech32>
    vaultStaticSecret:
      path: <vault-kv-path>
```

For mainnet, omit `node.networkMagic` if the chart/image expects no testnet
magic override. For custom networks, use the values required by that network's
configuration.

Bootstrap note:

- if this future producer is still in its initial non-forging bootstrap phase and
  needs to sync faster, temporarily using `useLedgerAfterSlot: 0` can help it
  discover more peers
- after bootstrap is complete and the node is used as a real private producer
  behind relays, set `useLedgerAfterSlot: -1`

Install a new producer release:

```bash
helm install <bp-release> ./extensions/cardano-node \
  --namespace <bp-namespace> \
  --create-namespace \
  -f <bp-values.yaml>
```

Or upgrade an existing non-producing release:

```bash
helm upgrade <bp-release> ./extensions/cardano-node \
  --namespace <bp-namespace> \
  -f <bp-values.yaml>
```

Debug mode mounts runtime material and enables derived producer metrics without
adding the forging runtime flags.

## Step 15: Validate Debug Mode

Run:

```bash
kubectl get pods -n <bp-namespace>
kubectl get vaultstaticsecret -n <bp-namespace>
kubectl get secret -n <bp-namespace>
kubectl describe pod -n <bp-namespace> <bp-pod>
```

Check derived metrics:

```bash
kubectl -n <bp-namespace> exec <bp-pod> -c cardano-node -- bash -lc \
  '/opt/metis/bin/metrics.sh'
```

Confirm:

- VaultStaticSecret is healthy
- runtime material is mounted
- producer topology points only to trusted relays
- `publicRoots` is empty for the producer
- `useLedgerAfterSlot` is `0` for the producer
- dashboard shows KES and op-cert data
- leadership schedule metrics appear after any cache warm-up

If the relay is now paired with the producer, move relay topology to explicit
mode too:

- producer in relay `localRoots`
- relay keeps a private local connection back to the producer
- trusted public relays in relay `publicRoots`
- if the relay was previously using `image-default`, capture its current public
  bootstrap peers or public peer set and carry that connectivity into
  `publicRoots` instead of dropping it
- relay `useLedgerAfterSlot: 0`

## Step 16: Activate Forging

Do not activate near an expected leadership slot. Use dashboard schedule
information when available.

Change only:

```yaml
node:
  blockProducer:
    enabled: true
    debug: false
```

Upgrade:

```bash
helm upgrade <bp-release> ./extensions/cardano-node \
  --namespace <bp-namespace> \
  -f <bp-values.yaml>
```

This starts the node with:

- `--shelley-kes-key`
- `--shelley-vrf-key`
- `--shelley-operational-certificate`

## Step 17: Post-Activation Verification

Check:

```bash
kubectl get pods -n <bp-namespace>
kubectl -n <bp-namespace> exec <bp-pod> -c cardano-node -- bash -lc \
  '/opt/metis/bin/metrics.sh'
```

Confirm:

- pod restarted cleanly
- forging is enabled
- peer counts are non-zero
- active sockets point to intended relay paths
- KES current and remaining periods look sane
- op-cert disk and chain values are aligned
- leadership schedule still renders
- external chain source confirms produced blocks after expected slots
- public Cardano explorer, for example Cexplorer for the selected network,
  shows pool blocks after the pool has assigned leadership and enough time has
  passed for indexing

Use `cardano-block-producer-troubleshooting.md` if local metrics look healthy
but external canonical-chain sources do not show pool blocks after expected
leadership slots.

## Stop Conditions

Stop and ask for human confirmation when:

- the target network is ambiguous
- the network magic is unknown
- the operator selected a repo-local key workspace that is not ignored by Git
- the metadata hash does not match the hosted file
- the relay DNS/IP or port is not confirmed
- the transaction includes unexpected certificates
- funds are insufficient for deposits, fees, and pledge
- the operator asks to use mainnet but has no offline cold-key custody plan
- Vault path is missing or already contains unrelated material
- the producer topology includes public roots
- the relay is unhealthy
- Cardano pods are repeatedly evicted for memory during restore or sync
- the dashboard cannot show KES or op-cert state in debug mode

## Common Debugging Checks

Tip:

```bash
cardano-cli query tip ${CARDANO_CLI_NETWORK}
```

Protocol parameters:

```bash
cardano-cli query protocol-parameters ${CARDANO_CLI_NETWORK}
```

UTXO:

```bash
cardano-cli query utxo \
  --address "$(cat keys/payment.addr)" \
  ${CARDANO_CLI_NETWORK}
```

Pool registration:

```bash
cardano-cli query stake-snapshot \
  --stake-pool-id "$(cat keys/pool.id)" \
  ${CARDANO_CLI_NETWORK}
```

Producer topology inside pod:

```bash
kubectl -n <bp-namespace> exec <bp-pod> -c cardano-node -- sh -lc \
  'cat /opt/cardano/config/${CARDANO_NETWORK}/topology.json'
```

Runtime flags:

```bash
kubectl -n <bp-namespace> get pod <bp-pod> -o jsonpath='{.spec.containers[?(@.name=="cardano-node")].args}'
```

Connections:

```bash
kubectl -n <bp-namespace> exec <bp-pod> -c cardano-node -- sh -lc \
  'ss -tnp 2>/dev/null || netstat -tnp 2>/dev/null || true'
```

## Best Practices

- build commands from a confirmed network profile
- keep all signing keys out of Git
- keep cold keys and counters out of Vault and Kubernetes
- upload only runtime producer material to Vault
- use `transaction build` unless there is a specific reason to manually
  calculate fees
- use Docker-based `cardano-cli` with a narrow workspace mount when the host CLI
  is missing
- validate metadata with `stake-pool metadata-hash` before asking the operator
  to host it; ticker length and JSON shape failures should be caught locally
- install the future producer early in non-forging mode so it can restore and
  sync while pool registration work continues
- set explicit Cardano-node resource requests and limits as a normal Kubernetes
  practice
- on GKE Autopilot, watch events for memory evictions and autoscaler delays
  during Mithril restore or first sync
- keep producer topology explicit and private
- keep relay topology explicit once paired with a producer
- run producer debug mode before active forging
- distinguish pool registration, producer readiness, and successful canonical
  block production
