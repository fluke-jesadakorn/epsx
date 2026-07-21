#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-offline-runtime.XXXXXX")
bff_pid=""

cleanup() {
  if [ -n "$bff_pid" ]; then kill "$bff_pid" 2>/dev/null || true; wait "$bff_pid" 2>/dev/null || true; fi
  rm -rf -- "$temp_dir"
}
trap cleanup EXIT HUP INT TERM

if curl --silent --fail --max-time 1 http://127.0.0.1:3000/ >/dev/null 2>&1; then
  echo "offline runtime proof: localhost port 3000 is already in use" >&2
  exit 1
fi

EPSX_ENV=local \
API_URL=http://127.0.0.1:9 \
OIDC_ISSUER=http://127.0.0.1:9 \
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
[ "$ready" -eq 1 ] || { sed -n '1,240p' "$temp_dir/frontend.log" >&2; echo "offline runtime proof: frontend did not start" >&2; exit 1; }

bunx playwright test e2e/frontend/offline-fresh-runtime.spec.ts --project=frontend --workers=1

echo "offline runtime proof: PASS (fresh controlled mobile+desktop navigation; public /offline cache only; cleanup armed)"
