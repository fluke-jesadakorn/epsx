#!/bin/bash
# Validates env vars against manifest before deployment
# Usage: validate-env.sh <environment>
# Exit 0 = all checks pass, Exit 1 = errors found
# Compatible with bash 3.2+ (no associative arrays)

set -euo pipefail

TARGET_ENV="${1:-}"
if [[ -z "$TARGET_ENV" ]]; then
  echo "Usage: $0 <env>  (dev|staging|prod)" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=../env-manifest.sh
source "$SCRIPT_DIR/../env-manifest.sh"

errors=0

# Check required vars are non-empty
check_required() {
  local service="$1"; shift
  for var in "$@"; do
    eval "val=\${${var}:-}"
    if [[ -z "$val" ]]; then
      echo "  MISSING: $var (required for $service)"
      errors=$((errors + 1))
    fi
  done
}

# Check format of set vars against regex rules
check_format() {
  local i=0
  while [[ $i -lt ${#FORMAT_VARS[@]} ]]; do
    local var="${FORMAT_VARS[$i]}"
    local regex="${FORMAT_REGEX[$i]}"
    eval "val=\${${var}:-}"
    if [[ -n "$val" ]] && ! echo "$val" | grep -qE "$regex"; then
      echo "  BAD FORMAT: $var (expected: $regex)"
      errors=$((errors + 1))
    fi
    i=$((i + 1))
  done
}

echo "=== Validating $TARGET_ENV environment ==="
echo ""

echo "[postgres]"
check_required postgres $REQUIRED_POSTGRES

echo "[redis]"
check_required redis $REQUIRED_REDIS

echo "[minio]"
check_required minio $REQUIRED_MINIO

echo "[backend]"
check_required backend $REQUIRED_BACKEND

echo "[frontend]"
check_required frontend $REQUIRED_FRONTEND

echo ""
echo "[format checks]"
check_format

echo ""
if [[ $errors -gt 0 ]]; then
  echo "FAILED: $errors error(s) found"
  exit 1
fi
echo "PASSED: All env vars present and valid"
