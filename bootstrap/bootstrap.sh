#!/usr/bin/env bash
set -eo pipefail

VSO_CHART_VERSION="${VSO_CHART_VERSION:-1.3.0}"

ensure_vso_crds() {
  local chart_ref="hashicorp/vault-secrets-operator"

  echo "Ensuring Vault Secrets Operator CRDs are installed..."
  helm repo add hashicorp https://helm.releases.hashicorp.com >/dev/null 2>&1 || true
  helm show crds "${chart_ref}" --version "${VSO_CHART_VERSION}" | kubectl apply -f - >/dev/null

  kubectl wait --for=condition=Established crd/vaultauths.secrets.hashicorp.com --timeout=120s >/dev/null
  kubectl wait --for=condition=Established crd/vaultconnections.secrets.hashicorp.com --timeout=120s >/dev/null
  kubectl wait --for=condition=Established crd/vaultstaticsecrets.secrets.hashicorp.com --timeout=120s >/dev/null
  kubectl wait --for=condition=Established crd/vaultauthglobals.secrets.hashicorp.com --timeout=120s >/dev/null
}

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

ensure_vso_crds

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

cat <<EOF

Control-plane install finished.

If your chosen values enable Vault:
- VSO CRDs were pre-applied by bootstrap.sh for this first install.
- Vault dev mode requires no init/unseal.
- Vault standalone or HA modes still require one-time initialization after the pods start.
- Provider-specific Vault guidance lives under bootstrap/<provider>/README.md.
- Additional control-plane Vault examples live under extensions/control-plane/examples/.

EOF
