#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
helper="$repo_root/e2e/frontend/fixtures/developer-docs-auth-mock.mjs"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-developer-docs-runtime.XXXXXX")
issuer="http://127.0.0.1:18081"
auth_pid=""
bff_pid=""

cleanup() {
  if [ -n "$bff_pid" ]; then kill "$bff_pid" 2>/dev/null || true; wait "$bff_pid" 2>/dev/null || true; fi
  if [ -n "$auth_pid" ]; then kill "$auth_pid" 2>/dev/null || true; wait "$auth_pid" 2>/dev/null || true; fi
  rm -rf -- "$temp_dir"
}
trap cleanup EXIT HUP INT TERM

if curl --silent --fail --max-time 1 "$issuer/healthz" >/dev/null 2>&1; then
  echo "developer docs runtime proof: localhost port 18081 is already in use" >&2
  exit 1
fi
if curl --silent --fail --max-time 1 http://127.0.0.1:3000/ >/dev/null 2>&1; then
  echo "developer docs runtime proof: localhost port 3000 is already in use" >&2
  exit 1
fi

umask 077
node "$helper" generate "$temp_dir/private.pem"
node "$helper" serve "$temp_dir/private.pem" "$issuer" >"$temp_dir/auth.log" 2>&1 &
auth_pid=$!

ready=0
for _ in $(seq 1 80); do
  if curl --silent --fail --max-time 1 "$issuer/healthz" >/dev/null 2>&1; then ready=1; break; fi
  sleep 0.25
done
[ "$ready" -eq 1 ] || { sed -n '1,160p' "$temp_dir/auth.log" >&2; echo "developer docs runtime proof: auth fixture did not start" >&2; exit 1; }

EPSX_ENV=local \
API_URL="$issuer" \
OIDC_ISSUER="$issuer" \
HOST=127.0.0.1 \
PORT=3000 \
cargo run --locked -p epsx-frontend --bin bff-frontend >"$temp_dir/frontend.log" 2>&1 &
bff_pid=$!

ready=0
for _ in $(seq 1 240); do
  status=$(curl --silent --output /dev/null --write-out '%{http_code}' --max-time 1 http://127.0.0.1:3000/ || true)
  if [ "$status" = "200" ]; then ready=1; break; fi
  if ! kill -0 "$bff_pid" 2>/dev/null; then break; fi
  sleep 0.25
done
[ "$ready" -eq 1 ] || { sed -n '1,240p' "$temp_dir/frontend.log" >&2; echo "developer docs runtime proof: frontend did not start" >&2; exit 1; }

A7_DEVELOPER_DOCS_ACCESS_TOKEN=$(node "$helper" token "$temp_dir/private.pem" "$issuer") \
  bunx playwright test e2e/frontend/developer-docs-runtime.spec.ts --project=frontend --workers=1

echo "developer docs runtime proof: PASS (ephemeral localhost RS256/JWKS fixture; cleanup armed)"
