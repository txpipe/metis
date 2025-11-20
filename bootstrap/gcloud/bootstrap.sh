#!/usr/bin/env bash
set -euo pipefail

CLUSTER_NAME="${GKE_CLUSTER_NAME:-supernode}"
PROJECT_ID="${GCP_PROJECT:-${GOOGLE_CLOUD_PROJECT:-}}"
REGION="${GKE_REGION:-us-central1}"
ZONE="${GKE_ZONE:-}"
MODE="${GKE_MODE:-autopilot}"
MACHINE_TYPE="${GKE_MACHINE_TYPE:-e2-standard-4}"
NODE_COUNT="${GKE_NODE_COUNT:-3}"
RELEASE_CHANNEL="${GKE_RELEASE_CHANNEL:-regular}"
NETWORK="${GKE_NETWORK:-}"
SUBNETWORK="${GKE_SUBNETWORK:-}"
CONFIG_FILE=""

print_usage() {
  cat <<'EOF'
Usage: bootstrap/gcloud/bootstrap.sh [options]

Options:
  --cluster-name <name>   Override the GKE cluster name (default: supernode).
  --project <id>          Google Cloud project ID (defaults to GCP_PROJECT or gcloud config).
  --region <name>         Region for the cluster (default: us-central1).
  --zone <name>           Zone for standard clusters (overrides --region when provided).
  --mode <type>           Cluster mode: autopilot (default) or standard.
  --machine-type <type>   Machine type for standard clusters (default: e2-standard-4).
  --node-count <count>    Node count for standard clusters (default: 3).
  --release-channel <c>   Release channel for standard clusters (default: regular).
  --network <name>        VPC network to use (optional).
  --subnetwork <name>     Subnetwork to use (optional).
  --config <path>         Reserved for future use; currently ignored (file must exist if provided).
  --help                  Show this help message and exit.

Environment:
  GKE_CLUSTER_NAME, GCP_PROJECT, GOOGLE_CLOUD_PROJECT, GKE_REGION,
  GKE_ZONE, GKE_MODE, GKE_MACHINE_TYPE, GKE_NODE_COUNT,
  GKE_RELEASE_CHANNEL, GKE_NETWORK, GKE_SUBNETWORK can be used instead of flags.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cluster-name)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --cluster-name requires a value" >&2
        exit 1
      fi
      CLUSTER_NAME="$1"
      ;;
    --project)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --project requires a value" >&2
        exit 1
      fi
      PROJECT_ID="$1"
      ;;
    --region)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --region requires a value" >&2
        exit 1
      fi
      REGION="$1"
      ;;
    --zone)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --zone requires a value" >&2
        exit 1
      fi
      ZONE="$1"
      ;;
    --mode)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --mode requires a value" >&2
        exit 1
      fi
      MODE="$1"
      ;;
    --machine-type)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --machine-type requires a value" >&2
        exit 1
      fi
      MACHINE_TYPE="$1"
      ;;
    --node-count)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --node-count requires a value" >&2
        exit 1
      fi
      NODE_COUNT="$1"
      ;;
    --release-channel)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --release-channel requires a value" >&2
        exit 1
      fi
      RELEASE_CHANNEL="$1"
      ;;
    --network)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --network requires a value" >&2
        exit 1
      fi
      NETWORK="$1"
      ;;
    --subnetwork)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --subnetwork requires a value" >&2
        exit 1
      fi
      SUBNETWORK="$1"
      ;;
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

if [[ -n "${CONFIG_FILE}" ]]; then
  if [[ ! -f "${CONFIG_FILE}" ]]; then
    echo "error: config file not found at '${CONFIG_FILE}'" >&2
    exit 1
  fi
  echo "warning: --config is currently ignored by the gcloud provider." >&2
fi

MODE="$(echo "${MODE}" | tr '[:upper:]' '[:lower:]')"
case "${MODE}" in
  autopilot|standard)
    ;;
  *)
    echo "error: unsupported mode '${MODE}'. Expected 'autopilot' or 'standard'." >&2
    exit 1
    ;;
esac

if [[ "${MODE}" == "autopilot" && -z "${REGION}" ]]; then
  echo "error: --region (or GKE_REGION) is required for autopilot clusters." >&2
  exit 1
fi

if [[ -n "${ZONE}" && "${MODE}" == "autopilot" ]]; then
  echo "warning: --zone is ignored for autopilot clusters; using region '${REGION}'." >&2
  ZONE=""
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

ensure_gcloud() {
  if has_cmd gcloud; then
    return
  fi

  echo "gcloud CLI not found. Attempting installation..."
  if install_with_brew google-cloud-sdk; then
    return
  fi

  if install_with_apt google-cloud-sdk; then
    return
  fi

  echo "Unable to automatically install the Google Cloud SDK. See https://cloud.google.com/sdk/docs/install for manual steps." >&2
  exit 1
}

ensure_gke_auth_plugin() {
  if has_cmd gke-gcloud-auth-plugin; then
    return
  fi

  if has_cmd gcloud; then
    echo "Installing gke-gcloud-auth-plugin..."
    if gcloud components install gke-gcloud-auth-plugin -q; then
      return
    fi
  fi

  echo "warning: gke-gcloud-auth-plugin is not installed. Kubernetes authentication may fail. See https://cloud.google.com/blog/products/containers-kubernetes/kubectl-auth-changes-in-gke" >&2
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

resolve_project() {
  if [[ -n "${PROJECT_ID}" ]]; then
    return
  fi

  PROJECT_ID="$(gcloud config get-value project 2>/dev/null || true)"
  PROJECT_ID="${PROJECT_ID//[$'\r\n']}"
  if [[ "${PROJECT_ID}" == "(unset)" ]]; then
    PROJECT_ID=""
  fi

  if [[ -z "${PROJECT_ID}" ]]; then
    echo "error: Google Cloud project not set. Use --project or export GCP_PROJECT / GOOGLE_CLOUD_PROJECT." >&2
    exit 1
  fi
}

ensure_active_account() {
  local active_account
  active_account="$(gcloud auth list --filter=status:ACTIVE --format='value(account)' 2>/dev/null || true)"
  if [[ -z "${active_account}" ]]; then
    echo "error: no active gcloud account found. Run 'gcloud auth login' or provide a service account using 'gcloud auth activate-service-account'." >&2
    exit 1
  fi
}

configure_project() {
  local current_project
  current_project="$(gcloud config get-value project 2>/dev/null || true)"
  current_project="${current_project//[$'\r\n']}"
  if [[ "${current_project}" != "${PROJECT_ID}" ]]; then
    echo "Setting gcloud project to '${PROJECT_ID}'..."
    gcloud config set project "${PROJECT_ID}" >/dev/null
  fi
}

enable_required_apis() {
  echo "Enabling required Google Cloud APIs..."
  gcloud services enable container.googleapis.com compute.googleapis.com \
    --project "${PROJECT_ID}" --quiet
}

location_args=()
location_desc=""
if [[ "${MODE}" == "autopilot" ]]; then
  location_args=(--region "${REGION}")
  location_desc="region '${REGION}'"
else
  if [[ -n "${ZONE}" ]]; then
    location_args=(--zone "${ZONE}")
    location_desc="zone '${ZONE}'"
  else
    location_args=(--region "${REGION}")
    location_desc="region '${REGION}'"
  fi
fi

cluster_exists() {
  if gcloud container clusters describe "${CLUSTER_NAME}" "${location_args[@]}" \
    --project "${PROJECT_ID}" --format='value(name)' >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

create_cluster() {
  if cluster_exists; then
    echo "GKE cluster '${CLUSTER_NAME}' already exists in ${location_desc}. Skipping creation."
    return
  fi

  if [[ "${MODE}" == "autopilot" ]]; then
    echo "Creating GKE Autopilot cluster '${CLUSTER_NAME}' in ${location_desc}..."
    local args=(
      container clusters create-auto "${CLUSTER_NAME}"
      "${location_args[@]}"
      --project "${PROJECT_ID}"
      --quiet
    )
    if [[ -n "${NETWORK}" ]]; then
      args+=(--network "${NETWORK}")
    fi
    if [[ -n "${SUBNETWORK}" ]]; then
      args+=(--subnetwork "${SUBNETWORK}")
    fi
    gcloud "${args[@]}"
    return
  fi

  echo "Creating standard GKE cluster '${CLUSTER_NAME}' in ${location_desc}..."
  local args=(
    container clusters create "${CLUSTER_NAME}"
    "${location_args[@]}"
    --project "${PROJECT_ID}"
    --machine-type "${MACHINE_TYPE}"
    --num-nodes "${NODE_COUNT}"
    --release-channel "${RELEASE_CHANNEL}"
    --enable-ip-alias
    --quiet
  )
  if [[ -n "${NETWORK}" ]]; then
    args+=(--network "${NETWORK}")
  fi
  if [[ -n "${SUBNETWORK}" ]]; then
    args+=(--subnetwork "${SUBNETWORK}")
  fi
  gcloud "${args[@]}"
}

configure_kubectl() {
  echo "Fetching cluster credentials for '${CLUSTER_NAME}'..."
  gcloud container clusters get-credentials "${CLUSTER_NAME}" "${location_args[@]}" \
    --project "${PROJECT_ID}" --quiet
}

main() {
  ensure_curl
  ensure_gcloud
  ensure_gke_auth_plugin
  ensure_kubectl
  ensure_helm
  ensure_active_account
  resolve_project
  configure_project
  enable_required_apis
  create_cluster
  configure_kubectl
  echo "Google Cloud GKE bootstrap completed successfully."
}

main
