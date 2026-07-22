#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFIER="$REPO_ROOT/scripts/migration/verify-refresh-outcome-coordination.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a1-7-self-test.XXXXXX")"
trap '[[ -n "${TMP_ROOT:-}" && -d "$TMP_ROOT" ]] && rm -rf -- "$TMP_ROOT"' EXIT

FILES=(
  apps/backend/src/web/auth/handlers.rs
  shared/rust/bff/src/refresh_outcome.rs
  apps/frontend/src/api.rs
  apps/admin/src/session_auth.rs
  apps/admin/src/session_auth_tests.rs
  shared/rust/client/src/lib.rs
  shared/rust/bff/src/browser_auth.rs
  scripts/migration/test-browser-session-coordination.js
  shared/rust/templates/src/lib.rs
  apps/frontend/src/ssr.rs
  shared/rust/dioxus_ui/src/auth/connected_wallet_dropdown.rs
  shared/rust/dioxus_ui/src/auth/wallet_button.rs
  shared/rust/dioxus_ui/src/layout/admin_shell.rs
  apps/admin/src/ssr.rs
  docs/migration/A1_4_AUTH_SESSION_GATE.md
  docs/migration/A1_5_REFRESH_CLIENT_BINDING.md
  docs/migration/PRODUCTION_READINESS_PLAN.md
  docs/migration/contracts/refresh-outcome-coordination.json
)

copy_case() {
  case_root="$1"
  for file in "${FILES[@]}"; do
    mkdir -p "$case_root/$(dirname "$file")"
    cp "$REPO_ROOT/$file" "$case_root/$file"
  done
}

mutate_after() {
  file="$1"
  needle="$2"
  from="$3"
  to="$4"
  bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const [file, needle, from, to] = process.argv.slice(1);
const source = readFileSync(file, "utf8");
const start = source.indexOf(needle);
if (start < 0) throw new Error(`needle missing: ${needle}`);
const found = source.indexOf(from, start);
if (found < 0) throw new Error(`mutation source missing after needle: ${from}`);
writeFileSync(file, source.slice(0, found) + to + source.slice(found + from.length));
' -- "$file" "$needle" "$from" "$to"
}

expect_failure() {
  id="$1"
  case_root="$TMP_ROOT/$id"
  copy_case "$case_root"
  shift
  "$@" "$case_root"
  if EPSX_A1_7_EVIDENCE_ROOT="$case_root" "$VERIFIER" >"$TMP_ROOT/$id.log" 2>&1; then
    echo "refresh-outcome-coordination-self-test: ERROR: $id was not detected" >&2
    exit 1
  fi
  echo "refresh-outcome-coordination-self-test: PASS $id"
}

case_database_claims_not_rotated() {
  root="$1"
  mutate_after "$root/apps/backend/src/web/auth/handlers.rs" \
    'Err(Web3AuthError::DatabaseError' \
    'REFRESH_OUTCOME_UNKNOWN' \
    'REFRESH_OUTCOME_NOT_ROTATED'
}

case_unknown_classifier_preserves() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/refresh_outcome.rs" \
    'pub fn classify_refresh_outcome' \
    '_ => RefreshDisposition::Clear' \
    '_ => RefreshDisposition::Preserve'
}

case_expands_preserve_statuses() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/refresh_outcome.rs" \
    'pub fn classify_refresh_outcome' \
    'StatusCode::BAD_REQUEST' \
    'StatusCode::BAD_REQUEST | StatusCode::TOO_MANY_REQUESTS'
}

case_frontend_transport_preserves() {
  root="$1"
  mutate_after "$root/apps/frontend/src/api.rs" \
    'tracing::warn!("Refresh upstream unavailable' \
    'clear_refresh_session_response' \
    'safe_error'
}

case_admin_transport_preserves() {
  root="$1"
  mutate_after "$root/apps/admin/src/session_auth.rs" \
    'tracing::warn!("Admin refresh upstream unavailable' \
    'clear_refresh_session_response' \
    'safe_error'
}

case_removes_web_lock() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'function epsxWithSessionMutation' \
    'navigator.locks.request' \
    'navigator.unlocked.request'
}

case_unlocked_refresh_fallback() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'if (requireCrossTabLock)' \
    "return Promise.reject(new Error('Session refresh requires cross-tab coordination'));" \
    'return operation();'
}

case_adds_refresh_retry() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'async function epsxRefreshOnce' \
    "response = await fetch('/api/v1/auth/refresh'" \
    "await fetch('/api/v1/auth/refresh', { method: 'POST' }); response = await fetch('/api/v1/auth/refresh'"
}

case_broadcasts_identity() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'function epsxPublishSessionEvent' \
    'var event = { version: 1, type: type };' \
    "var event = { version: 1, type: type, wallet: 'dynamic' };"
}

case_redirects_without_clear() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'function epsxLogoutSession' \
    "!== 'cleared'" \
    "=== 'cleared'"
}

case_follows_auth_redirects() {
  root="$1"
  mutate_after "$root/shared/rust/client/src/lib.rs" \
    'pub fn new(config: ClientConfig)' \
    '.redirect(reqwest::redirect::Policy::none())' \
    '.redirect(reqwest::redirect::Policy::limited(10))'
}

case_accepts_duplicate_outcome_headers() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/refresh_outcome.rs" \
    'fn exact_refresh_outcome' \
    'values.next().is_none().then_some(outcome)' \
    'Some(outcome)'
}

case_false_clear_attestation() {
  root="$1"
  mutate_after "$root/apps/frontend/src/api.rs" \
    'fn clear_refresh_session_response' \
    'Err(error) => error' \
    'Err(error) => refresh_response(error, RefreshDisposition::Clear)'
}

case_reorders_evidence_inventory() {
  root="$1"
  bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const file = process.argv[1];
const contract = JSON.parse(readFileSync(file, "utf8"));
[contract.evidence[0], contract.evidence[1]] = [contract.evidence[1], contract.evidence[0]];
writeFileSync(file, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$root/docs/migration/contracts/refresh-outcome-coordination.json"
}

case_comments_out_denial_delegation() {
  root="$1"
  mutate_after "$root/apps/admin/src/ssr.rs" \
    'fn admin_denial_runtime_script' \
    'await window.epsxAuth.logout(target);' \
    '// await window.epsxAuth.logout(target);'
}

case_comment_spoofs_lock() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'function epsxWithSessionMutation' \
    'navigator.locks.request' \
    'navigator.unlocked.request'
  printf '\n// navigator.locks.request(epsxSessionLockName, { mode: '\''exclusive'\'' }, operation)\n' >> "$root/shared/rust/bff/src/browser_auth.rs"
}

"$VERIFIER" >/dev/null
echo "refresh-outcome-coordination-self-test: PASS baseline"

expect_failure database-claims-not-rotated case_database_claims_not_rotated
expect_failure unknown-classifier-preserves case_unknown_classifier_preserves
expect_failure expands-preserve-statuses case_expands_preserve_statuses
expect_failure frontend-transport-preserves case_frontend_transport_preserves
expect_failure admin-transport-preserves case_admin_transport_preserves
expect_failure removes-web-lock case_removes_web_lock
expect_failure unlocked-refresh-fallback case_unlocked_refresh_fallback
expect_failure adds-refresh-retry case_adds_refresh_retry
expect_failure broadcasts-identity case_broadcasts_identity
expect_failure redirects-without-clear case_redirects_without_clear
expect_failure follows-auth-redirects case_follows_auth_redirects
expect_failure accepts-duplicate-outcome-headers case_accepts_duplicate_outcome_headers
expect_failure false-clear-attestation case_false_clear_attestation
expect_failure reorders-evidence-inventory case_reorders_evidence_inventory
expect_failure comments-out-denial-delegation case_comments_out_denial_delegation
expect_failure comment-spoofs-lock case_comment_spoofs_lock

echo "refresh-outcome-coordination-self-test: PASS 16/16 tamper cases"
