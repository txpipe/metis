# Google Cloud GKE Bootstrap

The Google Cloud provider provisions a [Google Kubernetes Engine (GKE)](https://cloud.google.com/kubernetes-engine) cluster and prepares local tooling so the Metis control plane can be installed immediately after.

```bash
./bootstrap.sh --provider gcloud --version <control-plane version>
```

By default the script creates an **Autopilot** cluster, but you can switch to a **Standard** (managed node) cluster by passing `--mode standard`.

## Prerequisites

- A Google Cloud project with billing enabled and the [Kubernetes Engine API](https://console.cloud.google.com/marketplace/product/google/container.googleapis.com) available.
- Permissions to create GKE clusters, networks (if needed), and service accounts in the target project. The minimal suggested roles are:
  - `roles/container.admin`
  - `roles/compute.networkAdmin` (if you need to create or modify networks/subnets)
  - `roles/iam.serviceAccountUser` (when using service accounts for node pools)
- Ability to install CLI tools into `/usr/local/bin`.

The script ensures (and installs when missing):

- [`gcloud` (Google Cloud SDK)](https://cloud.google.com/sdk/docs/install)
- [`kubectl`](https://kubernetes.io/docs/tasks/tools/)
- [`helm`](https://helm.sh/docs/intro/install/)
- [`gke-gcloud-auth-plugin`](https://cloud.google.com/blog/products/containers-kubernetes/kubectl-auth-changes-in-gke) for modern kubectl authentication

## Credential Setup

Before running the bootstrap you must authenticate with Google Cloud and select the project. Two common approaches are shown below.

### Using a User Account

```bash
gcloud auth login                      # Launches a browser for OAuth login
gcloud auth application-default login  # (Optional) Provides ADC credentials for other tools
gcloud config set project <PROJECT_ID>
gcloud auth list                        # Verify there is an ACTIVE account
```

### Using a Service Account

```bash
PROJECT_ID=<PROJECT_ID>
SA_NAME=metis-bootstrap

gcloud iam service-accounts create "${SA_NAME}" \
  --project "${PROJECT_ID}" \
  --display-name "Metis Bootstrap"

gcloud projects add-iam-policy-binding "${PROJECT_ID}" \
  --member "serviceAccount:${SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role roles/container.admin

gcloud projects add-iam-policy-binding "${PROJECT_ID}" \
  --member "serviceAccount:${SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role roles/compute.networkAdmin

gcloud iam service-accounts keys create gcloud-key.json \
  --iam-account "${SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com"

gcloud auth activate-service-account \
  "${SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com" \
  --key-file gcloud-key.json

gcloud config set project "${PROJECT_ID}"
```

Remove `gcloud-key.json` after use or store it securely. You can avoid creating keys by using [workload identity federation](https://cloud.google.com/iam/docs/workload-identity-federation) if your environment supports it.

Validate credentials before proceeding:

```bash
gcloud auth list --filter="status:ACTIVE"
gcloud config get-value project
```

## Configuration

### Cluster Modes

- **Autopilot** (default): Serverless cluster; Google manages nodes, scaling, and security. Requires only a region.
- **Standard**: You manage the node pool. Can be zonal (via `--zone`) or regional (via `--region`).

### Defaults

- Cluster name: `supernode` (`GKE_CLUSTER_NAME`, `--cluster-name`)
- Project ID: from `GCP_PROJECT`, `GOOGLE_CLOUD_PROJECT`, or current gcloud config (`--project`)
- Region: `us-central1` (`GKE_REGION`, `--region`)
- Zone (standard only): unset (`GKE_ZONE`, `--zone`)
- Cluster mode: `autopilot` (`GKE_MODE`, `--mode`)
- Machine type (standard): `e2-standard-4` (`GKE_MACHINE_TYPE`, `--machine-type`)
- Node count (standard): `3` (`GKE_NODE_COUNT`, `--node-count`)
- Release channel (standard): `regular` (`GKE_RELEASE_CHANNEL`, `--release-channel`)
- Network / subnetwork: inherited from project defaults unless overridden with `GKE_NETWORK`, `GKE_SUBNETWORK`, `--network`, or `--subnetwork`

### Flags & Environment Variables

| Flag / Env Var            | Description                                                       |
|---------------------------|-------------------------------------------------------------------|
| `--cluster-name <name>`   | Cluster name (`GKE_CLUSTER_NAME`).                                |
| `--project <id>`          | Google Cloud project (`GCP_PROJECT`, `GOOGLE_CLOUD_PROJECT`).     |
| `--region <name>`         | Region for the cluster (`GKE_REGION`). Required for Autopilot.    |
| `--zone <name>`           | Zone for Standard clusters (`GKE_ZONE`). Overrides `--region`.    |
| `--mode {autopilot|standard}` | Cluster provisioning mode (`GKE_MODE`).                        |
| `--machine-type <type>`   | Machine type for Standard clusters (`GKE_MACHINE_TYPE`).          |
| `--node-count <count>`    | Node count for Standard clusters (`GKE_NODE_COUNT`).              |
| `--release-channel <c>`   | Release channel for Standard clusters (`GKE_RELEASE_CHANNEL`).    |
| `--network <name>`        | Use a custom VPC (`GKE_NETWORK`).                                 |
| `--subnetwork <name>`     | Use a specific subnetwork (`GKE_SUBNETWORK`).                     |
| `--config <path>`         | Reserved for future advanced scenarios; currently ignored.        |

Add `--mode standard` when you need direct control over node pools (for example, to attach GPUs or tune autoscaling).

## What the Script Does

1. Installs or verifies `gcloud`, `kubectl`, `helm`, and the GKE auth plugin.
2. Confirms you have an active Google Cloud identity and the selected project.
3. Enables the `container.googleapis.com` and `compute.googleapis.com` APIs.
4. Creates (or reuses) the target Autopilot or Standard cluster.
5. Fetches credentials into your kubeconfig so subsequent `helm install` commands target the new cluster.

The script is idempotent: if the cluster already exists it is reused and the kubeconfig is updated.

## Useful Links

- [Autopilot overview](https://cloud.google.com/kubernetes-engine/docs/concepts/autopilot-overview)
- [Standard cluster options](https://cloud.google.com/kubernetes-engine/docs/concepts/types-of-clusters#standard-clusters)
- [GKE networking](https://cloud.google.com/kubernetes-engine/docs/how-to/alias-ips)
- [Role reference for GKE](https://cloud.google.com/kubernetes-engine/docs/how-to/iam)
