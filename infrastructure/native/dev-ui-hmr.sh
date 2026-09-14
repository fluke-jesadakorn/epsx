#!/bin/bash
# Development only: one Cargo workspace/cache, a separate DX bundle per UI.
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/../.." && pwd)
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin"
export CARGO_TARGET_DIR="$repo_dir/target"
export CARGO_BUILD_JOBS=2
# DX's separate client/server builders can overlap on the same shared crate.
# Share Cargo dependency artifacts, but avoid concurrent incremental graphs.
export CARGO_INCREMENTAL=0
cd "$repo_dir"
if [[ "${1:-}" == ui-worker ]]; then
    export EPSX_DEV_CHECKOUT="$repo_dir"
    exec /usr/bin/python3 "$HOME/.config/epsx/dev/tools/dev-ui-assets.py"
fi
case "${1:-}" in
    bff-frontend) app=frontend; package=epsx-frontend; binary=dx-frontend; port=3000 ;;
    bff-admin) app=admin; package=epsx-admin; binary=dx-admin; port=3001 ;;
    bff-pay) app=pay; package=epsx-pay-bff; binary=epsx-pay; port=3002 ;;
    *) echo 'Expected bff-frontend, bff-admin, bff-pay or ui-worker' >&2; exit 2 ;;
esac
config_dir="$HOME/.config/epsx/dev/config"
for env_file in "$config_dir/common.env" "$config_dir/$1.env"; do
    set -a
    source "$env_file"
    set +a
done
# DX assigns the inner SSR port and its own public bundle path.
unset DIOXUS_PUBLIC_PATH
export HOST=127.0.0.1
export ROOT_ENV_FILE=/dev/null
export EPSX_PUBLIC_DIR="$repo_dir/apps/$app/public"
export EPSX_BROWSER_RUNTIME_DIR="$repo_dir/target/epsx-service-worker"
export CARGO_TARGET_DIR="$repo_dir/target"
export CARGO_BUILD_JOBS=2
export CARGO_INCREMENTAL=0
cd "$repo_dir/apps/$app"
exec python3 "$repo_dir/infrastructure/native/dev-ui-realtime.py" "$1" \
    dx serve --fullstack true --web --hot-reload true --watch true \
    --addr 127.0.0.1 --port "$port" --open false --interactive true \
    --package "$package" --bin "$binary" --no-default-features --locked \
    --force-sequential true --debug-symbols false
