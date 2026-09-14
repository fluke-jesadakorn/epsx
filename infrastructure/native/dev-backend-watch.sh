#!/bin/bash
# Run the backend checkout behind the existing dev tunnel and persistent keys.
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/../.." && pwd)
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin"
export CARGO_TARGET_DIR="$repo_dir/target"
export CARGO_BUILD_JOBS=2
cd "$repo_dir"
if [[ "${1:-}" != --run ]]; then
    exec cargo watch --why --no-vcs-ignores --no-dot-ignores --skip-local-deps \
        --watch apps/backend/src --watch apps/backend/Cargo.toml --watch shared/rust \
        --watch Cargo.toml --watch Cargo.lock \
        --watch infrastructure/native/dev-backend-watch.sh \
        --ignore 'shared/rust/dioxus_ui/**' --ignore 'shared/rust/templates/**' \
        --shell 'bash infrastructure/native/dev-backend-watch.sh --run'
fi
config_dir="$HOME/.config/epsx/dev/config"
for env_file in "$config_dir/common.env" "$config_dir/epsx.env"; do
    set -a
    source "$env_file"
    set +a
done
export HOST=127.0.0.1
export ROOT_ENV_FILE=/dev/null
if [[ -n "${RSA_PRIVATE_KEY_FILE:-}" ]]; then
    export RSA_PRIVATE_KEY="$(<"$RSA_PRIVATE_KEY_FILE")"
fi
if [[ -n "${RSA_PUBLIC_KEY_FILE:-}" ]]; then
    export RSA_PUBLIC_KEY="$(<"$RSA_PUBLIC_KEY_FILE")"
fi
exec cargo run --locked -p epsx --bin epsx
