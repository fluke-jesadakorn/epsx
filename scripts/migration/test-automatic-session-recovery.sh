#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFIER="$REPO_ROOT/scripts/migration/verify-automatic-session-recovery.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a1-8-self-test.XXXXXX")"
trap '[[ -n "${TMP_ROOT:-}" && -d "$TMP_ROOT" ]] && rm -rf -- "$TMP_ROOT"' EXIT

FILES=(
  shared/rust/bff/src/session.rs
  shared/rust/bff/src/browser_auth.rs
  apps/frontend/src/auth.rs
  apps/admin/src/auth.rs
  apps/frontend/src/ssr.rs
  apps/admin/src/ssr.rs
  apps/frontend/src/main.rs
  apps/admin/src/main.rs
  scripts/migration/test-browser-session-coordination.js
  docs/migration/A1_8_AUTOMATIC_SESSION_RECOVERY.md
  docs/migration/A1_4_AUTH_SESSION_GATE.md
  docs/migration/PRODUCTION_READINESS_PLAN.md
  docs/migration/contracts/automatic-session-recovery.json
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
  if EPSX_A1_8_EVIDENCE_ROOT="$case_root" EPSX_A1_8_SELF_TEST_STATIC=1 "$VERIFIER" >"$TMP_ROOT/$id.log" 2>&1; then
    echo "automatic-session-recovery-self-test: ERROR: $id was not detected" >&2
    exit 1
  fi
  echo "automatic-session-recovery-self-test: PASS $id"
}

case_verifier_outage_permits_recovery() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/session.rs" \
    'pub const fn permits_refresh_recovery' \
    'Self::MissingOrRejected' \
    'Self::VerifierUnavailable'
}

case_unknown_key_becomes_rejection() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/session.rs" \
    'pub const fn is_verifier_unavailable' \
    '| Self::UnknownKeyId' \
    ''
}

case_unredacts_access_outcome_debug() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/session.rs" \
    'impl fmt::Debug for AccessVerification' \
    'Self::Verified { user, .. }' \
    'Self::Verified { token, user }'
  mutate_after "$root/shared/rust/bff/src/session.rs" \
    'impl fmt::Debug for AccessVerification' \
    '.field("token", &"[REDACTED]")' \
    '.field("token", token)'
}

case_frontend_drops_refresh_cookie_gate() {
  root="$1"
  mutate_after "$root/apps/frontend/src/ssr.rs" \
    'let refresh_cookie_present = auth::refresh_token(&headers, state.cookie_environment)' \
    'is_some()' \
    'false'
}

case_admin_drops_refresh_cookie_gate() {
  root="$1"
  mutate_after "$root/apps/admin/src/ssr.rs" \
    'let recover_session = access_verification.permits_refresh_recovery()' \
    '&& auth::refresh_token(&headers, state.cookie_environment).is_some();' \
    '&& true;'
}

case_frontend_recovers_offline() {
  root="$1"
  mutate_after "$root/apps/frontend/src/ssr.rs" \
    'let recover_session = access_verification.permits_refresh_recovery()' \
    '&& path != "/offline"' \
    '&& true'
}

case_resets_recovery_promise() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'function epsxRecoverSession()' \
    'return epsxRecoverPromise;' \
    'epsxRecoverPromise = null; return epsxRecoverPromise;'
}

case_bypasses_hardened_refresh() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'function epsxRecoverSession()' \
    'epsxRefreshSession().then(function(session)' \
    'Promise.resolve().then(function(session)'
}

case_adds_direct_recovery_fetch() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'function epsxRecoverSession()' \
    'if (epsxRecoverPromise)' \
    "fetch('/api/v1/auth/refresh'); if (epsxRecoverPromise)"
}

case_siwe_bypasses_session_lock() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'function epsxSiweLogin' \
    'return epsxWithSessionMutation(function() {' \
    'return (function() {'
}

case_frontend_cache_ignores_recovery() {
  root="$1"
  mutate_after "$root/apps/frontend/src/ssr.rs" \
    'fn apply_ssr_cache_policy' \
    'is_authenticated || recover_session' \
    'is_authenticated'
}

case_admin_drops_authorization_vary() {
  root="$1"
  mutate_after "$root/apps/admin/src/ssr.rs" \
    'fn private_admin_html_response' \
    'Cookie, Authorization' \
    'Cookie'
}

case_changes_fixed_bootstrap() {
  root="$1"
  mutate_after "$root/shared/rust/bff/src/browser_auth.rs" \
    'pub fn browser_session_recovery_script' \
    'window.epsxAuth.recover()' \
    'window.epsxAuth.refresh()'
}

case_reorders_evidence_inventory() {
  root="$1"
  bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const file = process.argv[1];
const contract = JSON.parse(readFileSync(file, "utf8"));
[contract.evidence[0], contract.evidence[1]] = [contract.evidence[1], contract.evidence[0]];
writeFileSync(file, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$root/docs/migration/contracts/automatic-session-recovery.json"
}

case_comment_spoofs_typed_adapter() {
  root="$1"
  mutate_after "$root/apps/frontend/src/auth.rs" \
    'pub async fn access_verification' \
    '.verify_optional_access_token(access_token(headers, environment))' \
    '.verify_optional_access_token_bypassed(access_token(headers, environment))'
  printf '\n// .verify_optional_access_token(access_token(headers, environment))\n' >> "$root/apps/frontend/src/auth.rs"
}

"$VERIFIER" >/dev/null
echo "automatic-session-recovery-self-test: PASS baseline"

if EPSX_A1_8_EVIDENCE_ROOT="$REPO_ROOT/." EPSX_A1_8_SELF_TEST_STATIC=1 \
  "$VERIFIER" >"$TMP_ROOT/same-root-alias.log" 2>&1; then
  echo "automatic-session-recovery-self-test: ERROR: same-root-alias bypass was not rejected" >&2
  exit 1
fi
echo "automatic-session-recovery-self-test: PASS same-root-alias-rejected"

expect_failure verifier-outage-permits-recovery case_verifier_outage_permits_recovery
expect_failure unknown-key-becomes-rejection case_unknown_key_becomes_rejection
expect_failure unredacts-access-outcome-debug case_unredacts_access_outcome_debug
expect_failure frontend-drops-refresh-cookie-gate case_frontend_drops_refresh_cookie_gate
expect_failure admin-drops-refresh-cookie-gate case_admin_drops_refresh_cookie_gate
expect_failure frontend-recovers-offline case_frontend_recovers_offline
expect_failure resets-recovery-promise case_resets_recovery_promise
expect_failure bypasses-hardened-refresh case_bypasses_hardened_refresh
expect_failure adds-direct-recovery-fetch case_adds_direct_recovery_fetch
expect_failure siwe-bypasses-session-lock case_siwe_bypasses_session_lock
expect_failure frontend-cache-ignores-recovery case_frontend_cache_ignores_recovery
expect_failure admin-drops-authorization-vary case_admin_drops_authorization_vary
expect_failure changes-fixed-bootstrap case_changes_fixed_bootstrap
expect_failure reorders-evidence-inventory case_reorders_evidence_inventory
expect_failure comment-spoofs-typed-adapter case_comment_spoofs_typed_adapter

echo "automatic-session-recovery-self-test: PASS 16/16 tamper/environment cases"
