# AWS EKS Bootstrap

The AWS provider bootstraps an [Amazon EKS](https://aws.amazon.com/eks/) cluster and prepares `kubectl` and `helm` for the control-plane installation phase.

```bash
./bootstrap.sh --provider aws
```

## Prerequisites

- AWS account with permissions to create EKS clusters, VPC resources, IAM roles, and node groups.
- AWS credentials configured locally (`aws configure`, environment variables, or AWS SSO). Test via:
  ```bash
  aws sts get-caller-identity
  ```
- Ability to install CLI tools into `/usr/local/bin`.

The script auto-installs (if missing):

- [`aws` CLI](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html)
- [`eksctl`](https://eksctl.io/getting-started/)
- [`kubectl`](https://kubernetes.io/docs/tasks/tools/)
- [`helm`](https://helm.sh/docs/intro/install/)

The bootstrap also enables the [Amazon EBS CSI driver add-on](https://docs.aws.amazon.com/eks/latest/userguide/ebs-csi.html) and creates a `StorageClass` backed by EBS (`gp3`) so persistent volume claims can be provisioned automatically.

## Configuration

### Defaults

- Cluster name: `supernode` (override with `EKS_CLUSTER_NAME` or `--cluster-name`)
- Region: `us-west-2` (override with `AWS_REGION` or `--region`)
- Nodegroup: `${cluster}-nodes`
- Instance type: `t3.medium`
- Scaling: desired `2`, min `1`, max `3`

### Flags & Environment Variables

| Flag / Env Var            | Description                                                |
|---------------------------|------------------------------------------------------------|
| `--config <file>`         | eksctl cluster config YAML. Passed directly to eksctl.     |
| `--cluster-name <name>`   | Cluster name (`EKS_CLUSTER_NAME`).                          |
| `--region <region>`       | AWS region (`AWS_REGION`).                                  |
| `--nodes <count>`         | Desired nodes when using defaults (`EKS_NODE_COUNT`).       |
| `--nodes-min <count>`     | Min nodes (`EKS_NODE_MIN`).                                 |
| `--nodes-max <count>`     | Max nodes (`EKS_NODE_MAX`).                                 |
| `--node-type <instance>`  | Worker node instance (`EKS_NODE_TYPE`).                     |

When supplying `--config`, ensure the cluster name within the YAML matches `--cluster-name` (or `EKS_CLUSTER_NAME`) so the script can detect existing clusters and reuse them.

### Persistent Volume Configuration

- The script installs the `aws-ebs-csi-driver` add-on and creates a StorageClass named `gp3` using the EBS CSI provisioner.
- Override the StorageClass name via `EKS_STORAGE_CLASS_NAME`.
- Override the EBS volume type (`gp3` by default) via `EKS_STORAGE_CLASS_VOLUME_TYPE`.
- A default Helm values file is provided at `bootstrap/aws/values.yml` setting `storageClass: gp3`. The root `bootstrap.sh` script automatically uses it for AWS unless you pass `--values` to supply your own overrides.

## Outputs

- Creates (or reuses) an EKS cluster.
- Updates local kubeconfig via `aws eks update-kubeconfig`.
- Ensures `helm` is available for subsequent OCI control-plane installation.

## Useful Links

- [EKS IAM policies](https://docs.aws.amazon.com/eks/latest/userguide/service_IAM_role.html)
- [eksctl cluster config reference](https://eksctl.io/usage/schema/)
- [EKS best practices](https://aws.github.io/aws-eks-best-practices/)
