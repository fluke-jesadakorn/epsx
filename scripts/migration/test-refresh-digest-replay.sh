#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-refresh-digest-replay.sh"
CONTRACT="$REPO_ROOT/docs/migration/contracts/refresh-digest-replay.json"
MIGRATION_DIR="$REPO_ROOT/apps/backend/migrations/core/20260723100000_add_refresh_token_digest_replay_state"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/epsx-refresh-digest-replay.XXXXXX")"
trap '[[ -n "${WORK_DIR:-}" && -d "$WORK_DIR" ]] && rm -rf -- "$WORK_DIR"' EXIT

die() {
  echo "refresh-digest-replay-self-test: ERROR: $*" >&2
  exit 1
}

expect_failure() {
  name="$1"
  pattern="$2"
  shift 2
  if "$@" >"$WORK_DIR/$name.out" 2>&1; then
    die "$name unexpectedly passed"
  fi
  grep -Eq "$pattern" "$WORK_DIR/$name.out" || {
    tail -n 80 "$WORK_DIR/$name.out" >&2
    die "$name failed without the expected diagnostic"
  }
}

reset_fixture() {
  cp "$CONTRACT" "$WORK_DIR/contract.json"
  cp "$MIGRATION_DIR/up.sql" "$WORK_DIR/up.sql"
  cp "$MIGRATION_DIR/down.sql" "$WORK_DIR/down.sql"
}

reset_runtime_fixture() {
  reset_fixture
  EVIDENCE_ROOT="$WORK_DIR/evidence-root"
  rm -rf -- "$EVIDENCE_ROOT"
  mkdir -p "$EVIDENCE_ROOT"
  bun -e '
import { readFileSync } from "node:fs";
const contract = JSON.parse(readFileSync(process.argv[1], "utf8"));
const files = new Set(contract.evidence.map((item) => item.file));
files.add("docs/migration/A1_6_REFRESH_DIGEST_REPLAY.md");
for (const file of files) console.log(file);
' -- "$CONTRACT" | while IFS= read -r file; do
    mkdir -p "$EVIDENCE_ROOT/$(dirname "$file")"
    cp "$REPO_ROOT/$file" "$EVIDENCE_ROOT/$file"
  done
}

"$VERIFY" >"$WORK_DIR/evidence.out"
grep -Fq "7/7 column guards" "$WORK_DIR/evidence.out" || die "baseline column count drifted"
grep -Fq "8/8 constraints" "$WORK_DIR/evidence.out" || die "baseline constraint count drifted"
grep -Fq "10/10 runtime invariants" "$WORK_DIR/evidence.out" || die "baseline runtime invariant count drifted"
grep -Fq "72/72 anchors" "$WORK_DIR/evidence.out" || die "baseline evidence count drifted"

set +e
"$VERIFY" --mode readiness >"$WORK_DIR/readiness.out" 2>&1
readiness_rc=$?
set -e
[[ "$readiness_rc" -eq 3 ]] || die "readiness mode must exit 3, observed $readiness_rc"
grep -Fq "core collision, PostgreSQL/MVCC, drained cutover" "$WORK_DIR/readiness.out" || die "readiness STOP detail drifted"

for field in productionReady databaseProof runtimeProof; do
  reset_fixture
  bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const [path, field] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(path, "utf8"));
contract[field] = true;
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$field"
  expect_failure "overclaim-$field" "non-production, database-unproved, runtime-hermetic" \
    "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"
done

reset_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.runtimeHermeticProof = false;
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure runtime-hermetic-underclaim "runtime-hermetic" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.runtimeInvariants[0].claim = "Opaque credentials may use process randomness of any length.";
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure runtime-invariant-tamper "runtime invariant drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.migration.legacyPolicy = "The first caller may assign the client, family, digest, and key version.";
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure legacy-claim "legacy fail-closed policy drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.migration.storagePolicy = "Version 1 dual-writes raw bearers for compatibility.";
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure dual-write-policy "digest-only storage policy drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.evidence[2].anchors[0] = "Raw UUID";
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure semantic-anchor "evidence inventory drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.invariants[2].claim = "Storage version 1 is temporarily accepted.";
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure invariant-tamper "schema invariant drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.residualStops.find(item => item.id === "core-migration-version-collision").claim += " Executable anyway.";
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure stop-tamper "residual STOP claim drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
writeFileSync(path, readFileSync(path, "utf8").replace("BYTEA", "TEXT"));
' -- "$WORK_DIR/up.sql"
expect_failure up-checksum "up migration checksum drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
writeFileSync(path, readFileSync(path, "utf8").replace("forward-only", "reversible"));
' -- "$WORK_DIR/down.sql"
expect_failure down-checksum "down migration checksum drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = `${readFileSync(upPath, "utf8")}\nUPDATE public.openid_refresh_tokens SET storage_version = 2;\n`;
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure destructive-sql "destructive or data-mutating SQL" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = readFileSync(upPath, "utf8").replace("storage_version = 2", "storage_version IN (1, 2)");
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure storage-v1 "storage_version = 2|version 1 must not be admitted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = readFileSync(upPath, "utf8").replace("observed_default IS NOT NULL", "FALSE");
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure default-guard "observed_default IS NOT NULL" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = readFileSync(upPath, "utf8").replace("OCTET_LENGTH(token_digest) = 32", "OCTET_LENGTH(token_digest) > 0");
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure digest-width "OCTET_LENGTH\(token_digest\) = 32" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = readFileSync(upPath, "utf8").replace("[A-Za-z0-9_-]", "[A-Za-z0-9._-]");
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure key-id-charset "digest_key_id" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = readFileSync(upPath, "utf8").replace("AND relation.relname = \u0027openid_refresh_tokens_digest_lookup_uq\u0027", "AND FALSE AND relation.relname = \u0027openid_refresh_tokens_digest_lookup_uq\u0027");
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure catalog-adoption "pre-existing digest index must be refused|catalog-refusal" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = readFileSync(upPath, "utf8").replace("is_revoked IS FALSE", "is_revoked IS TRUE");
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure terminal-active "terminal-state active shape drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_fixture
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = readFileSync(upPath, "utf8").replace("AND client_id IS NOT NULL", "AND client_id IS NULL");
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure version-two-binding "storage-version-2 binding shape drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_runtime_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const source = readFileSync(path, "utf8").replace(
  "let mut secret = [0_u8; TOKEN_SECRET_BYTES];",
  "let mut secret = [0_u8; 16];"
);
writeFileSync(path, source);
' -- "$EVIDENCE_ROOT/shared/rust/epsx-identity-shared/src/refresh_token_digest.rs"
expect_failure runtime-short-rng "rt1 issuance must use exactly" \
  "$VERIFY" --evidence-root "$EVIDENCE_ROOT" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_runtime_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const source = readFileSync(path, "utf8").replace(
  "OsRng.fill_bytes(&mut secret);",
  "secret.fill(0); // OsRng.fill_bytes(&mut secret);"
);
writeFileSync(path, source);
' -- "$EVIDENCE_ROOT/shared/rust/epsx-identity-shared/src/refresh_token_digest.rs"
expect_failure runtime-comment-spoof "rt1 issuance must use exactly" \
  "$VERIFY" --evidence-root "$EVIDENCE_ROOT" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_runtime_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const source = readFileSync(path, "utf8").replace(
  "Self::from_json(&active_key_id, &encoded_keys_json)",
  "let _unsafe_fallback = \"unwrap_or_default\"; Self::from_json(&active_key_id, &encoded_keys_json)"
);
writeFileSync(path, source);
' -- "$EVIDENCE_ROOT/shared/rust/epsx-identity-shared/src/refresh_token_digest.rs"
expect_failure runtime-key-fallback "no generated or default fallback" \
  "$VERIFY" --evidence-root "$EVIDENCE_ROOT" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_runtime_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const source = readFileSync(path, "utf8").replace(
  "openid_refresh_tokens::token_id.eq(&storage_id)",
  "openid_refresh_tokens::token_id.eq(refresh_token.credential().expose())"
);
writeFileSync(path, source);
' -- "$EVIDENCE_ROOT/shared/rust/epsx-identity-shared/src/token_service.rs"
expect_failure runtime-raw-storage "initial refresh insert must" \
  "$VERIFY" --evidence-root "$EVIDENCE_ROOT" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_runtime_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const source = readFileSync(path, "utf8").replace(
  "SELECT clock_timestamp() AS observed_at",
  "SELECT CURRENT_TIMESTAMP AS observed_at"
);
writeFileSync(path, source);
' -- "$EVIDENCE_ROOT/shared/rust/epsx-identity-shared/src/token_service.rs"
expect_failure runtime-process-clock "clock_timestamp|missing evidence anchor" \
  "$VERIFY" --evidence-root "$EVIDENCE_ROOT" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

reset_runtime_fixture
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const source = readFileSync(path, "utf8").replace(
  "#[derive(Clone, Serialize, Deserialize, ToSchema)]\npub struct OpenIDTokenResponse",
  "#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]\npub struct OpenIDTokenResponse"
);
writeFileSync(path, source);
' -- "$EVIDENCE_ROOT/shared/rust/epsx-identity-shared/src/token_service.rs"
expect_failure runtime-bearer-debug "must not derive Debug|missing evidence anchor" \
  "$VERIFY" --evidence-root "$EVIDENCE_ROOT" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

expect_failure live-database "DATABASE_URL must be unset" \
  env DATABASE_URL=postgres://127.0.0.1/forbidden "$VERIFY"
expect_failure live-active-key "REFRESH_TOKEN_HMAC_ACTIVE_KID must be unset" \
  env REFRESH_TOKEN_HMAC_ACTIVE_KID=forbidden "$VERIFY"
expect_failure live-keyring "REFRESH_TOKEN_HMAC_KEYS_JSON must be unset" \
  env REFRESH_TOKEN_HMAC_KEYS_JSON=forbidden "$VERIFY"

echo "refresh-digest-replay-self-test: PASS — readiness STOP and 29/29 tamper/environment cases"
