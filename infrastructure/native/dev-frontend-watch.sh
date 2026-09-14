#!/bin/bash
# Run the dev tunnel's frontend directly from this checkout, preserving dev auth.
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/../.." && pwd)
config_dir="$HOME/.config/epsx/dev/config"
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin"
cd "$repo_dir"
if [[ "${1:-}" != "--run" ]]; then
    exec cargo watch --why --watch apps/frontend --watch shared/rust \
        --ignore '*/public/dist/*' \
        --shell 'bash infrastructure/native/dev-frontend-watch.sh --run'
fi
for env_file in "$config_dir/common.env" "$config_dir/bff-frontend.env"; do
    set -a
    source "$env_file"
    set +a
done
export HOST=127.0.0.1
export ROOT_ENV_FILE=/dev/null
export EPSX_PUBLIC_DIR="$repo_dir/apps/frontend/public"
export EPSX_BROWSER_RUNTIME_DIR="$repo_dir/target/epsx-service-worker"
cargo xtask browser-runtime build
dx build --fullstack true --package epsx-frontend --bin dx-frontend --web --no-default-features --locked --force-sequential true
# A separate DX build may replace its output while this server is still running.
# Serve a completed snapshot so SSR, WASM and worker assets always share a build.
snapshot_dir=$(mktemp -d "$repo_dir/target/dev-frontend.XXXXXX")
trap 'rm -rf -- "$snapshot_dir"' EXIT
cp "$repo_dir/target/dx/dx-frontend/debug/web/server" "$snapshot_dir/bff-frontend"
cp -R "$repo_dir/target/dx/dx-frontend/debug/web/public" "$snapshot_dir/public"
cp -R "$repo_dir/apps/frontend/public" "$snapshot_dir/static"
cp -R "$repo_dir/target/epsx-service-worker" "$snapshot_dir/runtime"
export DIOXUS_PUBLIC_PATH="$snapshot_dir/public"
export EPSX_PUBLIC_DIR="$snapshot_dir/static"
export EPSX_BROWSER_RUNTIME_DIR="$snapshot_dir/runtime"
"$snapshot_dir/bff-frontend" &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; exit 0' INT TERM
wait "$server_pid"
