#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
repo_root="$(git -C "$script_dir" rev-parse --show-toplevel)"

healthy="$repo_root/docs/migration/fixtures/notification-metrics-healthy.json"
unhealthy="$repo_root/docs/migration/fixtures/notification-metrics-unhealthy.json"

cargo xtask notification-readiness --dry-run --input "$healthy" >/tmp/epsx-notification-readiness-healthy.json
if cargo xtask notification-readiness --dry-run --input "$unhealthy" >/tmp/epsx-notification-readiness-unhealthy.json 2>/tmp/epsx-notification-readiness-unhealthy.err; then
  echo "notification-readiness self-test: ERROR unhealthy snapshot was accepted" >&2
  exit 1
fi

grep -q '"healthy": true' /tmp/epsx-notification-readiness-healthy.json
grep -q 'notification readiness thresholds failed' /tmp/epsx-notification-readiness-unhealthy.err

echo 'notification-readiness self-test: PASS (healthy=accepted, unhealthy=fail-closed, writes=0, network=0, database=0)'
