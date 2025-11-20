#!/usr/bin/env bash
set -eo pipefail

print_usage() {
  cat <<'EOF'
Usage: bootstrap.sh --provider <name> --version <name> [--values <path>] [--config <path>] [--help]

Options:
  --provider <name>  Target provider to bootstrap (kind, aws, gcloud, azure).
  --config <path>    Optional provider-specific configuration file forwarded to the provider script.
  --version <name>   Version of control plane helm package.
  --values <path>    Path to values file for control plane, if any.
  --help             Show this help message and exit.

Examples:
  ./bootstrap.sh --provider kind
  ./bootstrap.sh --provider kind --config ./bootstrap/kind/config.yml
EOF
}

if [[ $# -eq 0 ]]; then
  print_usage
  exit 1
fi

PROVIDER=""
CONFIG_PATH=""
VALUES=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --provider)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --provider requires an argument" >&2
        exit 1
      fi
      PROVIDER="$1"
      ;;
    --config)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --config requires a path argument" >&2
        exit 1
      fi
      CONFIG_PATH="$1"
      ;;
    --values)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --values requires a path argument" >&2
        exit 1
      fi
      VALUES="$1"
      ;;
    --version)
      shift
      if [[ $# -eq 0 ]]; then
        echo "error: --version requires an argument" >&2
        exit 1
      fi
      VERSION="$1"
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

if [[ -z "${PROVIDER}" ]]; then
  echo "error: --provider is required" >&2
  print_usage
  exit 1
fi

PROVIDER_SCRIPT="${PROVIDER}/bootstrap.sh"

if [[ ! -x "${PROVIDER_SCRIPT}" ]]; then
  if [[ -f "${PROVIDER_SCRIPT}" ]]; then
    chmod +x "${PROVIDER_SCRIPT}"
  else
    echo "error: provider script not found for '${PROVIDER}' at ${PROVIDER_SCRIPT}" >&2
    exit 1
  fi
fi

cmd=("${PROVIDER}/bootstrap.sh")
if [[ -n "${CONFIG_PATH}" ]]; then
  cmd+=(--config "${CONFIG_PATH}")
fi

"${cmd[@]}"

# Install control plane
DEFAULT_VALUES=""
case "${PROVIDER}" in
  aws)
    if [[ -z "${VALUES}" && -f "${PROVIDER}/values.yml" ]]; then
      DEFAULT_VALUES="${PROVIDER}/values.yml"
    fi
    ;;
  kind)
    if [[ -z "${VALUES}" && -f "${PROVIDER}/values.yml" ]]; then
      DEFAULT_VALUES="${PROVIDER}/values.yml"
    fi
    ;;
esac
if [[ -z "${VALUES}" && -n "${DEFAULT_VALUES}" ]]; then
  VALUES="${DEFAULT_VALUES}"
fi

cmd=(
  helm install control-plane oci://oci.supernode.store/control-plane
  --version "${VERSION}"
  --namespace control-plane
  --create-namespace
)
if [[ -n "${VALUES:-}" ]]; then
  cmd+=( --values "$VALUES" )
fi
"${cmd[@]}"
