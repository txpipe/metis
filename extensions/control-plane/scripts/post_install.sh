#!/usr/bin/env bash
set -euo pipefail

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    printf 'missing required command: %s\n' "$1" >&2
    exit 1
  }
}

need_cmd kubectl
need_cmd vault

VAULT_TOKEN="${VAULT_TOKEN:?set VAULT_TOKEN}"

RELEASE_NAME="${RELEASE_NAME:-control-plane}"
NAMESPACE="${NAMESPACE:-control-plane}"
LOCAL_PORT="${LOCAL_PORT:-8200}"
VAULT_AUTH_MOUNT="${VAULT_AUTH_MOUNT:-kubernetes}"
VAULT_AUTH_PATH="${VAULT_AUTH_PATH:-auth/kubernetes}"
VAULT_KV_PATH="${VAULT_KV_PATH:-kv}"
VAULT_KV_RUNTIME_PREFIX="${VAULT_KV_RUNTIME_PREFIX:-runtime}"
VAULT_KV_OPERATOR_PREFIX="${VAULT_KV_OPERATOR_PREFIX:-operator}"
VAULT_POLICY_NAME="${VAULT_POLICY_NAME:-control-plane}"
VAULT_ROLE_NAME="${VAULT_ROLE_NAME:-control-plane}"
VAULT_ROLE_SERVICE_ACCOUNT_NAME="${VAULT_ROLE_SERVICE_ACCOUNT_NAME:-vault-auth}"
VAULT_ROLE_SERVICE_ACCOUNT_NAMESPACES="${VAULT_ROLE_SERVICE_ACCOUNT_NAMESPACES:-*}"
VAULT_ROLE_TTL="${VAULT_ROLE_TTL:-1h}"
VAULT_ROLE_POLICIES="${VAULT_ROLE_POLICIES:-control-plane}"
VAULT_ROLE_AUDIENCE="${VAULT_ROLE_AUDIENCE:-}"
VAULT_DISABLE_ISS_VALIDATION="${VAULT_DISABLE_ISS_VALIDATION:-true}"
KUBERNETES_HOST="${KUBERNETES_HOST:-https://kubernetes.default.svc:443}"
VAULT_ADDR="http://127.0.0.1:${LOCAL_PORT}"

PORT_FORWARD_LOG="$(mktemp)"
PORT_FORWARD_PID=""

normalize_prefix() {
  local prefix="$1"
  prefix="${prefix#/}"
  prefix="${prefix%/}"
  printf '%s' "$prefix"
}

prefix_overlaps() {
  local left="$1"
  local right="$2"
  [[ "$left" == "$right" || "$left" == "$right"/* || "$right" == "$left"/* ]]
}

cleanup() {
  if [[ -n "${PORT_FORWARD_PID}" ]] && kill -0 "${PORT_FORWARD_PID}" >/dev/null 2>&1; then
    kill "${PORT_FORWARD_PID}" >/dev/null 2>&1 || true
    wait "${PORT_FORWARD_PID}" >/dev/null 2>&1 || true
  fi
  rm -f "${PORT_FORWARD_LOG}"
}
trap cleanup EXIT

VAULT_KV_RUNTIME_PREFIX="$(normalize_prefix "$VAULT_KV_RUNTIME_PREFIX")"
VAULT_KV_OPERATOR_PREFIX="$(normalize_prefix "$VAULT_KV_OPERATOR_PREFIX")"

if [[ -z "$VAULT_KV_RUNTIME_PREFIX" ]]; then
  printf 'VAULT_KV_RUNTIME_PREFIX must not be empty.\n' >&2
  exit 1
fi

if [[ -z "$VAULT_KV_OPERATOR_PREFIX" ]]; then
  printf 'VAULT_KV_OPERATOR_PREFIX must not be empty.\n' >&2
  exit 1
fi

if prefix_overlaps "$VAULT_KV_RUNTIME_PREFIX" "$VAULT_KV_OPERATOR_PREFIX"; then
  printf 'VAULT_KV_RUNTIME_PREFIX and VAULT_KV_OPERATOR_PREFIX must not overlap.\n' >&2
  exit 1
fi

printf 'Starting kubectl port-forward to service/%s-vault in namespace %s...\n' "$RELEASE_NAME" "$NAMESPACE"
kubectl -n "$NAMESPACE" port-forward "service/${RELEASE_NAME}-vault" "${LOCAL_PORT}:8200" >"${PORT_FORWARD_LOG}" 2>&1 &
PORT_FORWARD_PID="$!"

wait_for_vault() {
  local attempts=60

  while [[ "$attempts" -gt 0 ]]; do
    if VAULT_ADDR="$VAULT_ADDR" VAULT_TOKEN="$VAULT_TOKEN" vault status >/dev/null 2>&1; then
      return 0
    fi

    local status=$?
    if [[ "$status" -eq 2 ]]; then
      printf 'Vault is sealed. Run vault operator init/unseal first, then rerun this script.\n' >&2
      return 1
    fi

    attempts=$((attempts - 1))
    sleep 2
  done

  printf 'Vault did not become reachable through the local port-forward.\n' >&2
  cat "${PORT_FORWARD_LOG}" >&2 || true
  return 1
}

wait_for_vault

export VAULT_ADDR
export VAULT_TOKEN

printf 'Ensuring Kubernetes auth mount exists...\n'
if ! vault auth list -format=json | grep -q '"'"${VAULT_AUTH_MOUNT}/"'"'; then
  vault auth enable -path="${VAULT_AUTH_MOUNT}" kubernetes
fi

printf 'Ensuring KV v2 mount exists at %s...\n' "$VAULT_KV_PATH"
if ! vault secrets list -format=json | grep -q '"'"${VAULT_KV_PATH}/"'"'; then
  vault secrets enable -path="${VAULT_KV_PATH}" -version=2 kv
fi

printf 'Configuring Vault Kubernetes auth...\n'
if [[ "$VAULT_DISABLE_ISS_VALIDATION" == "true" ]]; then
  vault write "$VAULT_AUTH_PATH/config" \
    kubernetes_host="$KUBERNETES_HOST" \
    disable_iss_validation=true
else
  vault write "$VAULT_AUTH_PATH/config" \
    kubernetes_host="$KUBERNETES_HOST"
fi

printf 'Writing shared VSO policy %s...\n' "$VAULT_POLICY_NAME"
vault policy write "$VAULT_POLICY_NAME" - <<EOF
path "${VAULT_KV_PATH}/data/${VAULT_KV_RUNTIME_PREFIX}/*" {
  capabilities = ["read"]
}

path "${VAULT_KV_PATH}/metadata/${VAULT_KV_RUNTIME_PREFIX}/*" {
  capabilities = ["read", "list"]
}
EOF

printf 'Writing shared VSO role %s...\n' "$VAULT_ROLE_NAME"
role_args=(
  "bound_service_account_names=${VAULT_ROLE_SERVICE_ACCOUNT_NAME}"
  "bound_service_account_namespaces=${VAULT_ROLE_SERVICE_ACCOUNT_NAMESPACES}"
  "ttl=${VAULT_ROLE_TTL}"
  "policies=${VAULT_ROLE_POLICIES}"
)

if [[ -n "$VAULT_ROLE_AUDIENCE" ]]; then
  role_args+=("audience=${VAULT_ROLE_AUDIENCE}")
fi

vault write "$VAULT_AUTH_PATH/role/${VAULT_ROLE_NAME}" "${role_args[@]}"

cat <<EOF

Vault post-install configuration finished.

Shared workload auth can read only:
  ${VAULT_KV_PATH}/${VAULT_KV_RUNTIME_PREFIX}/...

Operator-only space remains outside shared workload auth:
  ${VAULT_KV_PATH}/${VAULT_KV_OPERATOR_PREFIX}/...

The root token can still write anywhere in the mounted KV.

Verify:
  kubectl -n ${NAMESPACE} exec -it ${RELEASE_NAME}-vault-0 -- vault auth list
  kubectl -n ${NAMESPACE} exec -it ${RELEASE_NAME}-vault-0 -- vault read ${VAULT_AUTH_PATH}/role/${VAULT_ROLE_NAME}

EOF
