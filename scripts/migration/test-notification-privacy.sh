#!/bin/sh
set -eu

# No-write policy-contract audit. This validates the reviewed notification
# retention, legal-hold, and erasure semantics without touching a database,
# network, provider, or deployment.
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
output=$(cd "$repo_root" && cargo xtask notification-privacy-audit --strict)
printf '%s\n' "$output"
printf '%s\n' "$output" | grep -Fq 'policy=pass channels=pass legal_hold=true erasure=explicit writes=0 network=0 database=0'
printf '%s\n' 'notification-privacy: PASS — policy contract is explicit and no-write'
printf '%s\n' 'notification-privacy: LIMIT — runtime purge, legal-hold, erasure, provider, staging, and production evidence remain separate gates'
