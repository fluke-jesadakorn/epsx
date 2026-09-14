#!/bin/bash
set -euo pipefail
umask 077
binary=${1:?binary required}
config_dir=${2:?absolute configuration directory required}
case "$config_dir" in /*) ;; *) echo 'Configuration path must be absolute' >&2; exit 1;; esac
release_dir=$(cd "$(dirname "$0")/.." && pwd -P)
case "$binary" in epsx|bff-frontend|bff-admin|bff-pay|wallet|pay-service|subscription|notification|analytics|migrate) ;; *) echo 'Unknown service' >&2; exit 1;; esac
# Keep secrets outside the versioned release. No checkout dotenv discovery.
for env_file in "$config_dir/common.env" "$config_dir/$binary.env"; do
    test -f "$env_file" || { echo "Missing configuration file: $env_file" >&2; exit 1; }
    if [ "$(stat -f '%Lp' "$env_file")" != 600 ]; then echo "Configuration must have mode 600: $env_file" >&2; exit 1; fi
    set -a
    . "$env_file"
    set +a
done
if [ -n "${RSA_PRIVATE_KEY_FILE:-}" ]; then
    test -f "$RSA_PRIVATE_KEY_FILE"
    test "$(stat -f '%Lp' "$RSA_PRIVATE_KEY_FILE")" = 600 || { echo 'Private signing key must have mode 600' >&2; exit 1; }
    export RSA_PRIVATE_KEY="$(cat "$RSA_PRIVATE_KEY_FILE")"
fi
if [ -n "${RSA_PUBLIC_KEY_FILE:-}" ]; then
    test -f "$RSA_PUBLIC_KEY_FILE"
    export RSA_PUBLIC_KEY="$(cat "$RSA_PUBLIC_KEY_FILE")"
fi
export HOST=127.0.0.1
export EPSX_BROWSER_RUNTIME_DIR="$release_dir/runtime"
export EPSX_MIGRATIONS_DIR="$release_dir/migrations"
export ROOT_ENV_FILE=/dev/null
case "$binary" in
    bff-frontend) export EPSX_PUBLIC_DIR="$release_dir/public/frontend";;
    bff-admin) export EPSX_PUBLIC_DIR="$release_dir/public/admin";;
    bff-pay) export EPSX_PUBLIC_DIR="$release_dir/public/pay";;
esac
case "$binary" in
    bff-frontend) export DIOXUS_PUBLIC_PATH="$release_dir/fullstack/frontend/public";;
    bff-admin) export DIOXUS_PUBLIC_PATH="$release_dir/fullstack/admin/public";;
    bff-pay) export DIOXUS_PUBLIC_PATH="$release_dir/fullstack/pay/public";;
esac
cd "$release_dir"
shift 2
exec "$release_dir/bin/$binary" "$@"
