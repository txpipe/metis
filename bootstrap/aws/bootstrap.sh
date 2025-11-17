#!/usr/bin/env bash
set -euo pipefail

CLUSTER_NAME="${EKS_CLUSTER_NAME:-supernode}"
REGION="${AWS_REGION:-us-east-2}"
NODEGROUP_NAME="${EKS_NODEGROUP_NAME:-${CLUSTER_NAME}-nodes}"
NODE_TYPE="${EKS_NODE_TYPE:-t3.medium}"
NODE_COUNT="${EKS_NODE_COUNT:-2}"
NODE_MIN="${EKS_NODE_MIN:-1}"
NODE_MAX="${EKS_NODE_MAX:-3}"
STORAGE_CLASS_NAME="${EKS_STORAGE_CLASS_NAME:-gp3}"
STORAGE_CLASS_VOLUME_TYPE="${EKS_STORAGE_CLASS_VOLUME_TYPE:-gp3}"
CONFIG_FILE=""

print_usage() {
  cat <<'EOF'
Usage: bootstrap/aws/bootstrap.sh [options]

Options:
  --config <path>        Optional eksctl cluster config file (YAML).
  --cluster-name <name>  Override the EKS cluster name (default: supernode).
  --region <region>      AWS region (default: us-east-2 or AWS_REGION env).
  --nodes <count>        Desired node count when using default provisioning.
  --nodes-min <count>    Min nodes for managed nodegroup.
  --nodes-max <count>    Max nodes for managed nodegroup.
  --node-type <type>     EC2 instance type for worker nodes (default: t3.medium).
  --help                 Show this help message and exit.

Environment:
  EKS_CLUSTER_NAME, AWS_REGION, EKS_NODEGROUP_NAME, EKS_NODE_TYPE,
  EKS_NODE_COUNT, EKS_NODE_MIN, EKS_NODE_MAX, EKS_STORAGE_CLASS_NAME,
  EKS_STORAGE_CLASS_VOLUME_TYPE can be used instead of flags.

When --config is supplied, it is passed directly to eksctl and the script
assumes the cluster name in that file matches --cluster-name (if provided).
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --config)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --config requires a path argument" >&2
        exit 1
      fi
      CONFIG_FILE="$1"
      ;;
    --cluster-name)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --cluster-name requires a value" >&2
        exit 1
      fi
      CLUSTER_NAME="$1"
      NODEGROUP_NAME="${EKS_NODEGROUP_NAME:-${CLUSTER_NAME}-nodes}"
      ;;
    --region)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --region requires a value" >&2
        exit 1
      fi
      REGION="$1"
      ;;
    --nodes)
      shift
      NODE_COUNT="$1"
      ;;
    --nodes-min)
      shift
      NODE_MIN="$1"
      ;;
    --nodes-max)
      shift
      NODE_MAX="$1"
      ;;
    --node-type)
      shift
      NODE_TYPE="$1"
      ;;
    --help|-h)
      print_usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      print_usage
      exit 1
      ;;
  esac
  shift
done

if [[ -n "${CONFIG_FILE}" && ! -f "${CONFIG_FILE}" ]]; then
  echo "error: config file not found at '${CONFIG_FILE}'" >&2
  exit 1
fi

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

install_with_brew() {
  local package="$1"
  if has_cmd brew; then
    echo "Installing ${package} with Homebrew..."
    if brew install "${package}"; then
      return 0
    fi
    echo "Homebrew installation of ${package} failed." >&2
    return 1
  fi
  return 1
}

install_with_apt() {
  local package="$1"
  if has_cmd apt-get; then
    echo "Installing ${package} with apt..."
    if sudo apt-get update && sudo apt-get install -y "${package}"; then
      return 0
    fi
    echo "apt installation of ${package} failed." >&2
    return 1
  fi
  return 1
}

ensure_curl() {
  if has_cmd curl; then
    return
  fi
  echo "error: curl is required for dependency installation. Please install curl and rerun." >&2
  exit 1
}

ensure_aws_cli() {
  if has_cmd aws; then
    return
  fi

  echo "aws CLI not found. Attempting installation..."
  if install_with_brew awscli; then
    return
  fi

  if install_with_apt awscli; then
    return
  fi

  echo "Unable to automatically install aws CLI. See https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html" >&2
  exit 1
}

ensure_eksctl() {
  if has_cmd eksctl; then
    return
  fi

  echo "eksctl not found. Attempting installation..."
  if install_with_brew eksctl; then
    return
  fi

  local version
  version="$(curl --silent --location https://api.github.com/repos/eksctl-io/eksctl/releases/latest | grep '"tag_name":' | cut -d'"' -f4)"
  local arch
  arch="$(uname -m)"
  local os
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  local tarball=""

  case "${arch}" in
    x86_64|amd64) arch="amd64" ;;
    arm64|aarch64) arch="arm64" ;;
    *)
      echo "Unsupported architecture '${arch}' for eksctl auto-install." >&2
      exit 1
      ;;
  esac

  tarball="eksctl_${os}_${arch}.tar.gz"
  echo "Downloading eksctl ${version}..."
  curl --silent --location "https://github.com/eksctl-io/eksctl/releases/download/${version}/${tarball}" --output "/tmp/${tarball}"
  tar -xzf "/tmp/${tarball}" -C /tmp
  sudo mv /tmp/eksctl /usr/local/bin/eksctl
  rm "/tmp/${tarball}"
}

ensure_kubectl() {
  if has_cmd kubectl; then
    return
  fi

  echo "kubectl not found. Attempting installation..."
  if install_with_brew kubectl; then
    return
  fi

  if install_with_apt kubectl; then
    return
  fi

  local version
  version="$(curl -L -s https://dl.k8s.io/release/stable.txt)"
  local kernel
  kernel="$(uname -s | tr '[:upper:]' '[:lower:]')"
  local arch
  arch="$(uname -m)"

  case "${arch}" in
    x86_64|amd64) arch="amd64" ;;
    arm64|aarch64) arch="arm64" ;;
    *)
      echo "Unsupported architecture '${arch}' for kubectl auto-install." >&2
      exit 1
      ;;
  esac

  echo "Downloading kubectl ${version}..."
  curl -LO "https://dl.k8s.io/release/${version}/bin/${kernel}/${arch}/kubectl"
  chmod +x kubectl
  sudo mv kubectl /usr/local/bin/kubectl
}

ensure_helm() {
  if has_cmd helm; then
    return
  fi

  echo "helm not found. Attempting installation..."
  if install_with_brew helm; then
    return
  fi

  if install_with_apt helm; then
    return
  fi

  if curl -fsSL https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash; then
    return
  fi

  echo "Unable to automatically install helm. See https://helm.sh/docs/intro/install/" >&2
  exit 1
}

verify_aws_credentials() {
  if ! aws sts get-caller-identity >/dev/null 2>&1; then
    echo "error: AWS credentials not configured or insufficient permissions. Configure credentials and retry." >&2
    exit 1
  fi
}

cluster_exists() {
  if ! has_cmd eksctl; then
    return 1
  fi
  if ! eksctl get cluster --name "${CLUSTER_NAME}" --region "${REGION}" >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

create_cluster() {
  if cluster_exists; then
    echo "EKS cluster '${CLUSTER_NAME}' already exists in region '${REGION}'. Skipping creation."
    return
  fi

  if [[ -n "${CONFIG_FILE}" ]]; then
    echo "Creating EKS cluster using config '${CONFIG_FILE}'..."
    eksctl create cluster -f "${CONFIG_FILE}"
    return
  fi

  echo "Creating EKS cluster '${CLUSTER_NAME}' in region '${REGION}'..."
  eksctl create cluster \
    --name "${CLUSTER_NAME}" \
    --region "${REGION}" \
    --managed \
    --nodegroup-name "${NODEGROUP_NAME}" \
    --node-type "${NODE_TYPE}" \
    --nodes "${NODE_COUNT}" \
    --nodes-min "${NODE_MIN}" \
    --nodes-max "${NODE_MAX}"
}

configure_kubectl() {
  echo "Updating kubeconfig for cluster '${CLUSTER_NAME}' in region '${REGION}'..."
  aws eks update-kubeconfig --name "${CLUSTER_NAME}" --region "${REGION}"
}

ensure_ebs_csi_driver() {
  local addon_name="aws-ebs-csi-driver"
  echo "Ensuring ${addon_name} add-on is installed..."

  if aws eks describe-addon \
    --cluster-name "${CLUSTER_NAME}" \
    --addon-name "${addon_name}" \
    --region "${REGION}" >/dev/null 2>&1; then
    echo "${addon_name} add-on already present."
    return
  fi

  echo "Associating IAM OIDC provider..."
  eksctl utils associate-iam-oidc-provider \
    --cluster "${CLUSTER_NAME}" \
    --region "${REGION}" \
    --approve

  echo "Creating or updating IAM service account for the EBS CSI driver..."
  local role_name="eksctl-${CLUSTER_NAME}-addon-ebs-csi-controller-sa"
  eksctl create iamserviceaccount \
    --name ebs-csi-controller-sa \
    --namespace kube-system \
    --role-name "${role_name}" \
    --cluster "${CLUSTER_NAME}" \
    --region "${REGION}" \
    --attach-policy-arn arn:aws:iam::aws:policy/service-role/AmazonEBSCSIDriverPolicy \
    --approve \
    --override-existing-serviceaccounts


  local role_arn=""
  echo "Waiting for IAM role '${role_name}' to be ready..."
  aws iam wait role-exists --role-name "${role_name}"
  role_arn="$(aws iam get-role \
    --role-name "${role_name}" \
    --query 'Role.Arn' \
    --output text 2>/dev/null || true)"

  if [[ -z "${role_arn}" || "${role_arn}" == "None" ]]; then
    echo "error: unable to resolve IAM role ARN for '${role_name}'. Verify IAM permissions and rerun." >&2
    exit 1
  fi

  echo "Installing ${addon_name}..."
  eksctl create addon \
    --name "${addon_name}" \
    --cluster "${CLUSTER_NAME}" \
    --region "${REGION}" \
    --service-account-role-arn "${role_arn}" \
    --force

  aws eks wait addon-active \
    --cluster-name "${CLUSTER_NAME}" \
    --addon-name "${addon_name}" \
    --region "${REGION}"

  echo "${addon_name} add-on installation completed."
}

ensure_storage_class() {
  echo "Ensuring StorageClass '${STORAGE_CLASS_NAME}' exists..."
  if kubectl get storageclass "${STORAGE_CLASS_NAME}" >/dev/null 2>&1; then
    echo "StorageClass '${STORAGE_CLASS_NAME}' already present."
    return
  fi

  cat <<EOF | kubectl apply -f -
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: ${STORAGE_CLASS_NAME}
provisioner: ebs.csi.aws.com
parameters:
  type: ${STORAGE_CLASS_VOLUME_TYPE}
  encrypted: "true"
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
EOF
  echo "StorageClass '${STORAGE_CLASS_NAME}' created."
}

main() {
  ensure_curl
  ensure_aws_cli
  ensure_eksctl
  ensure_kubectl
  ensure_helm
  verify_aws_credentials
  create_cluster
  configure_kubectl
  ensure_ebs_csi_driver
  ensure_storage_class
  echo "AWS EKS bootstrap completed successfully."
}

main
