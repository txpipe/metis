#!/usr/bin/env bash
set -euo pipefail

CLUSTER_NAME="${KIND_CLUSTER_NAME:-supernode}"
CONFIG_FILE=""

print_usage() {
  cat <<'EOF'
Usage: bootstrap/kind/bootstrap.sh [--config <path>] [--help]

Options:
  --config <path>  Optional Kind cluster configuration file (YAML).
  --help           Show this help message and exit.

Environment:
  KIND_CLUSTER_NAME  Overrides the default cluster name (default: metis-supernode).
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

ensure_kind() {
  if has_cmd kind; then
    return
  fi

  echo "kind not found. Attempting installation..."
  if install_with_brew kind; then
    return
  fi

  local kernel
  kernel="$(uname -s)"
  local arch
  arch="$(uname -m)"
  local version="v0.23.0"
  local url=""

  case "${kernel}" in
    Linux)
      case "${arch}" in
        x86_64|amd64) url="https://kind.sigs.k8s.io/dl/${version}/kind-linux-amd64" ;;
        arm64|aarch64) url="https://kind.sigs.k8s.io/dl/${version}/kind-linux-arm64" ;;
      esac
      ;;
    Darwin)
      case "${arch}" in
        x86_64|amd64) url="https://kind.sigs.k8s.io/dl/${version}/kind-darwin-amd64" ;;
        arm64|aarch64) url="https://kind.sigs.k8s.io/dl/${version}/kind-darwin-arm64" ;;
      esac
      ;;
  esac

  if [[ -n "${url}" ]]; then
    echo "Downloading kind ${version} from ${url}..."
    curl -Lo /tmp/kind "${url}"
    chmod +x /tmp/kind
    sudo mv /tmp/kind /usr/local/bin/kind
    return
  fi

  echo "Unable to automatically install kind. See https://kind.sigs.k8s.io/docs/user/quick-start/#installation" >&2
  exit 1
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

ensure_dependencies() {
  if ! has_cmd curl; then
    echo "error: curl is required for dependency installation. Please install curl and rerun." >&2
    exit 1
  fi
  ensure_kind
  ensure_kubectl
  ensure_helm
}

cluster_exists() {
  kind get clusters | grep -Fxq "${CLUSTER_NAME}"
}

create_cluster() {
  if cluster_exists; then
    echo "Kind cluster '${CLUSTER_NAME}' already exists. Skipping creation."
    return
  fi

  echo "Creating Kind cluster '${CLUSTER_NAME}'..."
  if [[ -n "${CONFIG_FILE}" ]]; then
    kind create cluster --name "${CLUSTER_NAME}" --config "${CONFIG_FILE}"
  else
    kind create cluster --name "${CLUSTER_NAME}"
  fi
}

configure_kubectl() {
  echo "Exporting kubeconfig for cluster '${CLUSTER_NAME}'..."
  kind export kubeconfig --name "${CLUSTER_NAME}"
  kubectl config use-context "kind-${CLUSTER_NAME}"
}

main() {
  ensure_dependencies
  create_cluster
  configure_kubectl
  echo "Kind provider bootstrap completed successfully."
}

main
