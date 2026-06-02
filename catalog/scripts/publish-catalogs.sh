#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REGISTRY="${REGISTRY:-oci.supernode.store}"
: "${CATALOG_TAG:?CATALOG_TAG must be set}"

node "${ROOT_DIR}/catalog/scripts/generate-skill-catalog.mjs"

oras push --disable-path-validation \
  "${REGISTRY}/extension-catalog:${CATALOG_TAG}" \
  "${ROOT_DIR}/catalog/extension-catalog.json:application/vnd.supernode.extension-catalog.v1+json"

oras push --disable-path-validation \
  "${REGISTRY}/skill-catalog:${CATALOG_TAG}" \
  "${ROOT_DIR}/catalog/skill-catalog.json:application/vnd.supernode.skill-catalog.v1+json"
