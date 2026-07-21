#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

command -v bun >/dev/null 2>&1 || {
  echo "a3-1-history-preflight: ERROR: bun is required" >&2
  exit 64
}

exec bun "$script_dir/a3-1-history-preflight.ts" "$@"
