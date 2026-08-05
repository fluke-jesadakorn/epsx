#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

cargo xtask env validate
echo "Starting Automatic Migration for All Databases"
exec cargo run -p epsx --features cli-tools --bin migrate -- up
