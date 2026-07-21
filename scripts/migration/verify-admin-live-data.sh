#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)

command -v bun >/dev/null 2>&1 || {
  echo "admin-live-data: ERROR: bun is required" >&2
  exit 1
}

exec bun "$script_dir/verify-admin-live-data.ts" --root "$repo_root" "$@"
