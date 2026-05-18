# Hydra Head Operations

## Goal

Guide an operator or agent through direct Hydra HTTP and WebSocket API interactions after a Hydra workload is deployed.

## Boundary

Hydra lifecycle operations are not MCP dynamic tools. Use MCP to discover workload outputs and health. Interact with the Hydra API directly through the approved port-forward workflow.

Never read signing key values through MCP. Transactions that require wallet signatures should be signed outside MCP by the operator's wallet or CLI workflow.

## Access The API

1. Call `workloads.get` for the Hydra workload.
2. Find the `api` or `ws` output.
3. Use `workload-output-port-forward.md` to create a local port-forward to the API service.
4. Default local API address is usually `http://127.0.0.1:4001` and WebSocket address is `ws://127.0.0.1:4001`.

## Read Head State

```bash
curl -s http://127.0.0.1:4001/head
curl -s http://127.0.0.1:4001/snapshot
curl -s http://127.0.0.1:4001/snapshot/utxo
curl -s http://127.0.0.1:4001/snapshot/last-seen
curl -s http://127.0.0.1:4001/commits
```

Use `snapshot-utxo=no` on WebSocket clients when full UTxO replay would be noisy.

## Open A Head

Use the WebSocket API:

```bash
printf '{"tag":"Init"}\n' | websocat 'ws://127.0.0.1:4001?history=no'
```

Offline heads open immediately according to local configuration. Online heads drive L1 transactions and require a working Cardano backend and fuel.

## Submit An L2 Transaction

HTTP API:

```bash
curl -s -X POST http://127.0.0.1:4001/transaction --data @signed-l2-tx.json
```

WebSocket API:

```bash
jq -c '{tag:"NewTx", transaction:.}' signed-l2-tx.json | websocat 'ws://127.0.0.1:4001?history=no'
```

The transaction must be valid against the current Hydra ledger state. A node may observe `TxValid` before the transaction is included in a confirmed snapshot.

## Draft A Deposit

For online heads, draft a deposit transaction with `/commit`:

```bash
curl -s -X POST http://127.0.0.1:4001/commit --data @commit-request.json > deposit-tx.json
```

The returned transaction must be signed and submitted to L1 outside MCP. After it appears on-chain, the Hydra node should emit deposit-related outputs and eventually make funds available on L2.

## Recover A Pending Deposit

List pending deposits:

```bash
curl -s http://127.0.0.1:4001/commits
```

Recover one by transaction ID:

```bash
curl -s -X DELETE http://127.0.0.1:4001/commits/<tx-id>
```

## Decommit

Build and sign a transaction that spends UTxO from `/snapshot/utxo`, then submit it:

```bash
curl -s -X POST http://127.0.0.1:4001/decommit --data @signed-decommit-tx.json
```

The decommit will become available on L1 after consensus and L1 processing.

## Close, Contest, Fanout

Use WebSocket client inputs:

```bash
printf '{"tag":"Close"}\n' | websocat 'ws://127.0.0.1:4001?history=no'
printf '{"tag":"Contest"}\n' | websocat 'ws://127.0.0.1:4001?history=no'
printf '{"tag":"Fanout"}\n' | websocat 'ws://127.0.0.1:4001?history=no'
```

Use `Close` deliberately. The Hydra API is unauthenticated by default, so exposing it broadly can let anyone close an open head.

## Sideload Snapshot

Sideloading is a recovery action for stuck heads and must be coordinated by participants.

```bash
curl -s http://127.0.0.1:4001/snapshot > snapshot.json
curl -s -X POST http://127.0.0.1:4001/snapshot --data @snapshot.json
```

After sideloading, pending transactions are pruned and may need to be resubmitted.
