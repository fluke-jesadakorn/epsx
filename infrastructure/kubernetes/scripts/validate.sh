#!/bin/bash
# validate.sh [env]
# Validates k3s deployment health for one or all environments.
# Usage:
#   ./validate.sh          # validate all envs
#   ./validate.sh prod     # validate prod only

set -euo pipefail

ENV_FILTER="${1:-all}"
PASS=0
FAIL=0

check() {
  local name="$1"
  local url="$2"
  local expected="${3:-200}"

  code=$(curl -sf -o /dev/null -w "%{http_code}" --max-time 10 "$url" 2>/dev/null || echo "000")
  if [[ "$code" == "$expected" || ("$expected" == "200" && "$code" =~ ^2) ]]; then
    echo "  ✓ $name ($code)"
    ((PASS++)) || true
  else
    echo "  ✗ $name — expected $expected, got $code"
    ((FAIL++)) || true
  fi
}

check_pods() {
  local ns="$1"
  echo "  Pods in $ns:"
  kubectl get pods -n "$ns" --no-headers 2>/dev/null | while read -r line; do
    echo "    $line"
  done
}

validate_env() {
  local env="$1"
  local ns="epsx-${env}"

  echo ""
  echo "=== $env (namespace: $ns) ==="

  # Pod health
  check_pods "$ns"

  # Service endpoints
  case "$env" in
    prod)
      check "prod backend health"  "https://api.epsx.io/health"
      check "prod frontend"        "https://epsx.io"
      check "prod admin"           "https://admin.epsx.io" "307"
      check "MinIO health"         "https://minio.epsx.io/minio/health/live"
      check "CDN (MinIO S3)"       "https://cdn.epsx.io/minio/health/live"
      ;;
    staging)
      check "staging backend"      "https://staging-api.epsx.io/health"
      check "staging frontend"     "https://staging.epsx.io"
      check "staging admin"        "https://staging-admin.epsx.io" "307"
      ;;
    dev)
      check "dev backend"          "https://dev-api.epsx.io/health"
      check "dev frontend"         "https://dev.epsx.io"
      check "dev admin"            "https://dev-admin.epsx.io" "307"
      ;;
  esac
}

echo "EPSX k3s Validation"
echo "==================="

if [[ "$ENV_FILTER" == "all" ]]; then
  validate_env dev
  validate_env staging
  validate_env prod
else
  validate_env "$ENV_FILTER"
fi

echo ""
echo "=== Summary: ${PASS} passed, ${FAIL} failed ==="
[[ "$FAIL" -eq 0 ]]
