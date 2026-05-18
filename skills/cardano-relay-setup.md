# Cardano Relay Setup

## Goal

Install and validate a Cardano relay workload through the Supernode MCP catalog workflow first. Relay-first is the base path before any block-producer activation.

## Assumptions

- The Supernode MCP server is available and can reach Kubernetes.
- `control-plane` is expected to exist, but should still be verified when diagnosing problems.
- The supported relay install path in MCP is the `cardano-node-relay` catalog entry.

## Required Inputs

- target namespace
- target release name
- target network
- storage class, if the default is not correct for the cluster
- any required tolerations or resource overrides

## Best Practices To Follow

- Start as a relay first.
- Keep system time accurate on every node involved.
- Open only the ports actually required by the workload and topology.
- Use hardened SSH and avoid password-based logins on underlying hosts.
- Validate storage classes before installation.
- Use MCP catalog validation and typed workload tools instead of raw Helm values.

## Relay Topology Guidance

Relay topology is operationally important and should be explicit once a producer
is attached to it.

Recommended stance for Metis:

- if the relay has no producer attached yet, `node.topology.mode=image-default` is a reasonable starting point
- once the relay is paired with a producer, the relay should also use explicit topology
- the relay should keep a private `localRoots` path to its producer and public network roots at the same time

For a relay:

- if paired to a producer, `localRoots` should contain the producer path
- `publicRoots` should contain the trusted public relay set used for network connectivity
- `useLedgerAfterSlot` should remain `0`
- the relay can and should face the broader Cardano network

For a producer later:

- do not use `image-default`
- connect only to your own relays or relays you explicitly trust
- keep `publicRoots` empty
- disable ledger peers with `useLedgerAfterSlot: -1`

## Preflight Checks

### Discover Installed Extensions

Use MCP resources and tools first:

- read `supernode://status`
- read `supernode://extensions/catalog`
- read `supernode://extensions/catalog/cardano-node-relay`
- call `extensions.catalog.get` with `extensionId=cardano-node-relay` when you need the structured schema
- call `workloads.list` to inspect existing installed releases

### Check Storage Classes

Use `cluster.storage_classes.list`.

Pick a storage class that the cluster actually supports. Do not invent or assume a storage class name.

### Check Cluster Scheduling Basics

Use `cluster.events.list` for recent scheduling or provisioning problems. Fall back to direct `kubectl` inspection only when MCP data is not enough for diagnosis.

## Install Pattern

Use `workloads.install` with `extensionId=cardano-node-relay`.

Start with a dry run so the MCP server validates the configuration schema and shows the exact Helm plan it would apply.

Minimal configuration shape:

```json
{
  "extensionId": "cardano-node-relay",
  "releaseName": "<release>",
  "namespace": "<namespace>",
  "dryRun": true,
  "configuration": {
    "network": "preview",
    "namespace": "<namespace>",
    "storageClass": "<storage-class>"
  }
}
```

Safe optional fields exposed by the catalog include:

- `topology`
- `exposeLoadBalancer`
- `imageTag`
- `resources`
- `pvcSize`

Do not pass raw Helm values. The MCP workflow rejects them intentionally.

If you want the simple default relay networking path, leave topology at the catalog default `image-default` until a producer is attached.

### Relay Topology After A Producer Exists

Once a producer is paired with the relay, prefer explicit topology instead of
`image-default`.

Recommended shape:

```yaml
node:
  topology:
    mode: custom
    localRoots:
      - accessPoints:
          - address: <producer-service-dns>
            port: 3000
        advertise: false
        valency: 1
    publicRoots:
      - accessPoints:
          - address: <public-relay-a>
            port: <public-relay-port>
          - address: <public-relay-b>
            port: <public-relay-port>
        advertise: true
        valency: 1
    useLedgerAfterSlot: 0
```

Operational intent:

- private producer path through `localRoots`
- public network path through `publicRoots`
- explicit relay behavior instead of image defaults

When moving an existing relay off `image-default`, do not drop the public
connectivity it already has. Capture the current bootstrap peers or current
public peer set and carry them into `publicRoots`, then add the producer as the
private path in `localRoots`.

## Post-Install Validation

Use MCP workload inspection first:

- `workloads.get` for the installed release
- `workloads.logs.get` for bounded relay pod logs
- `workloads.metrics.get` for typed relay metrics

Healthy relay expectations:

- PVCs are `Bound`
- StatefulSet pod is `Running`
- readiness is stable
- node metrics are exposed
- sync is progressing
- transaction processing is not stuck at zero for long periods on a healthy synced node

If you need lower-level Kubernetes details after the MCP view, then inspect the namespace directly with `kubectl`.

Topology interpretation:

- a healthy relay can start on `image-default` before a producer is attached
- once a producer exists, relay topology should become explicit
- relay `localRoots` should include the producer path
- relay `publicRoots` should remain populated for wider network connectivity
- do not copy relay networking assumptions onto the producer; the producer should be explicit and private

If the dashboard is available, validate:

- sync-related metrics
- node/resource metrics
- connectivity metrics

## Common Failure Modes

- wrong storage class
- PVC pending forever
- pod unschedulable because of node taints or resource requests
- wrong network magic
- unsupported network value for the catalog entry
- missing tolerations in constrained clusters
- raw Helm values supplied instead of extension configuration

## Escalation Rule

Do not continue to block-producer steps until the relay installation is healthy and stable.
