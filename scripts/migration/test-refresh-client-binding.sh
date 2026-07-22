#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-refresh-client-binding.sh"
CONTRACT="$REPO_ROOT/docs/migration/contracts/refresh-client-binding.json"
MIGRATION_DIR="$REPO_ROOT/apps/backend/migrations/core/20260723000000_bind_refresh_tokens_to_client"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/epsx-refresh-client-binding.XXXXXX")"
trap '[[ -n "${WORK_DIR:-}" && -d "$WORK_DIR" ]] && rm -rf -- "$WORK_DIR"' EXIT

die() {
  echo "refresh-client-binding-self-test: ERROR: $*" >&2
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
    tail -n 60 "$WORK_DIR/$name.out" >&2
    die "$name failed without the expected diagnostic"
  }
}

"$VERIFY" >"$WORK_DIR/evidence.out"
grep -Fq "49/49 pinned anchors" "$WORK_DIR/evidence.out" || die "baseline evidence count drifted"

set +e
"$VERIFY" --mode readiness >"$WORK_DIR/readiness.out" 2>&1
readiness_rc=$?
set -e
[[ "$readiness_rc" -eq 3 ]] || die "readiness mode must exit 3, observed $readiness_rc"
grep -Fq "PostgreSQL, A1.5 legacy enforcement, A1.6 forced-reauthentication/key lifecycle" "$WORK_DIR/readiness.out" || die "readiness STOP detail drifted"

cp "$CONTRACT" "$WORK_DIR/contract.json"
cp "$MIGRATION_DIR/up.sql" "$WORK_DIR/up.sql"
cp "$MIGRATION_DIR/down.sql" "$WORK_DIR/down.sql"

bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.productionReady = true;
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure production-overclaim "must remain partial, non-production" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

cp "$CONTRACT" "$WORK_DIR/contract.json"
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.migration.legacyPolicy = "The first caller may claim a legacy row.";
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure legacy-claim "legacy fail-closed policy drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

cp "$CONTRACT" "$WORK_DIR/contract.json"
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.evidence[0].anchors[0] = "use uuid::Uuid;";
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure anchor-tamper "evidence inventory drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

cp "$CONTRACT" "$WORK_DIR/contract.json"
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
const contract = JSON.parse(readFileSync(path, "utf8"));
contract.invariants[0].claim = "Only exact clients are accepted, except during tests.";
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json"
expect_failure invariant-tamper "runtime invariant drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

for stop_id in \
  core-migration-version-collision \
  postgres-forward-only-migration-unproved \
  a1-6-postgres-digest-replay-unproved \
  a1-6-cutover-key-lifecycle-unproved \
  production-actions-unauthorized; do
  cp "$CONTRACT" "$WORK_DIR/contract.json"
  bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const [path, stopId] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(path, "utf8"));
const stop = contract.residualStops.find(item => item.id === stopId);
if (!stop) throw new Error(`missing ${stopId}`);
stop.claim = `${stop.claim} Tampered.`;
writeFileSync(path, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$stop_id"
  expect_failure "stop-$stop_id" "residual STOP claim drifted" \
    "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"
done

cp "$CONTRACT" "$WORK_DIR/contract.json"
bun -e '
import { readFileSync, writeFileSync } from "node:fs";
const path = process.argv[1];
writeFileSync(path, readFileSync(path, "utf8").replace("epsx-admin", "epsx-api"));
' -- "$WORK_DIR/up.sql"
expect_failure migration-tamper "up migration checksum drifted" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

cp "$CONTRACT" "$WORK_DIR/contract.json"
cp "$MIGRATION_DIR/up.sql" "$WORK_DIR/up.sql"
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = readFileSync(upPath, "utf8").replaceAll("column_default IS NOT NULL", "FALSE");
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure hostile-default-drift "up migration missing.*column_default" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

cp "$CONTRACT" "$WORK_DIR/contract.json"
cp "$MIGRATION_DIR/up.sql" "$WORK_DIR/up.sql"
bun -e '
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const [contractPath, upPath] = process.argv.slice(1);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const up = readFileSync(upPath, "utf8").replace(
  "IF EXISTS (",
  "IF FALSE AND EXISTS ("
);
contract.migration.upSha256 = createHash("sha256").update(up).digest("hex");
writeFileSync(upPath, up);
writeFileSync(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
' -- "$WORK_DIR/contract.json" "$WORK_DIR/up.sql"
expect_failure hostile-constraint-drift "up migration missing.*IF EXISTS" \
  "$VERIFY" --contract "$WORK_DIR/contract.json" --up "$WORK_DIR/up.sql" --down "$WORK_DIR/down.sql"

expect_failure live-environment "DATABASE_URL must be unset" \
  env DATABASE_URL=postgres://127.0.0.1/forbidden "$VERIFY"

echo "refresh-client-binding-self-test: PASS — evidence, readiness STOP, and 13/13 tamper/environment cases"
