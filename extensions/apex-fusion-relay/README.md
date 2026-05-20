# Apex Fusion Relay Helm Chart

This chart deploys an opinionated Apex Fusion relay node for Supernode. It keeps the node, persistent chain data, metrics script, and n2c proxy wiring fixed while exposing only the values operators normally need.

## Install

```shell
helm install vector-relay ./apex-fusion-relay \
  --set node.network=vector-testnet \
  --set persistence.storageClass=standard
```

Supported networks are `vector-testnet`, `prime-testnet`, and `prime-mainnet`. The chart derives testnet magic for `vector-testnet` and `prime-testnet`; override `node.networkMagic` only for non-standard deployments.

Use `values.schema.json` as the public configuration contract for Helm, MCP, and LLM clients.
