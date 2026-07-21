#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-a3-3-runtime-ddl-triage.ts"
CONTRACT="$REPO_ROOT/docs/migration/contracts/a3-3-runtime-ddl-triage.json"
UPSTREAM="$REPO_ROOT/docs/migration/contracts/migration-safety.json"
TEST_DIR="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-3-runtime-ddl-triage.XXXXXX")"
trap 'rm -rf -- "$TEST_DIR"' EXIT

command -v bun >/dev/null 2>&1 || {
  echo "a3-3-runtime-ddl-triage-self-test: ERROR: bun is required" >&2
  exit 1
}

expect_failure() {
  local label="$1"
  shift
  if "$@" >"$TEST_DIR/$label.stdout" 2>"$TEST_DIR/$label.stderr"; then
    echo "a3-3-runtime-ddl-triage-self-test: ERROR: $label unexpectedly passed" >&2
    exit 1
  fi
  if ! grep -q "a3-3-runtime-ddl-triage: ERROR:" "$TEST_DIR/$label.stderr"; then
    echo "a3-3-runtime-ddl-triage-self-test: ERROR: $label did not fail closed" >&2
    exit 1
  fi
}

set +e
bun "$VERIFY" --readiness >"$TEST_DIR/readiness.stdout" 2>"$TEST_DIR/readiness.stderr"
readiness_status=$?
set -e
if [ "$readiness_status" -ne 2 ] || ! grep -q "STOP — productionReady=false" "$TEST_DIR/readiness.stderr"; then
  echo "a3-3-runtime-ddl-triage-self-test: ERROR: readiness did not exit 2 with STOP" >&2
  exit 1
fi

bun "$VERIFY" >/dev/null
bun "$VERIFY" --json >"$TEST_DIR/report-1.json"
bun "$VERIFY" --json >"$TEST_DIR/report-2.json"
cmp -s "$TEST_DIR/report-1.json" "$TEST_DIR/report-2.json" || {
  echo "a3-3-runtime-ddl-triage-self-test: ERROR: JSON report is nondeterministic" >&2
  exit 1
}
bun -e '
  import { readFileSync } from "node:fs";
  const report = JSON.parse(readFileSync(process.argv[1], "utf8"));
  const expected = {
    trackedRustFiles: 1124,
    findings: 28,
    reviewedExceptions: 6,
    actionable: 22,
    sha256: "eb754053bb0c2b7ff05babcb695c00eb0ca86653b4bd9199e183b4f2a69ee1c4",
  };
  if (JSON.stringify(report.scanner) !== JSON.stringify(expected)) process.exit(1);
' "$TEST_DIR/report-1.json" || {
  echo "a3-3-runtime-ddl-triage-self-test: ERROR: refreshed scanner inventory is not exact" >&2
  exit 1
}

cp "$CONTRACT" "$TEST_DIR/production-ready.json"
bun -e '
  import { readFileSync, writeFileSync } from "node:fs";
  const path = process.argv[1];
  const value = JSON.parse(readFileSync(path, "utf8"));
  value.productionReady = true;
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
' "$TEST_DIR/production-ready.json"
expect_failure production-ready bun "$VERIFY" --contract "$TEST_DIR/production-ready.json"

cp "$CONTRACT" "$TEST_DIR/actionable-status.json"
bun -e '
  import { readFileSync, writeFileSync } from "node:fs";
  const path = process.argv[1];
  const value = JSON.parse(readFileSync(path, "utf8"));
  value.findings.find((item) => item.classification === "actionable").status = "partial";
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
' "$TEST_DIR/actionable-status.json"
expect_failure actionable-status bun "$VERIFY" --contract "$TEST_DIR/actionable-status.json"

cp "$CONTRACT" "$TEST_DIR/exception-map.json"
bun -e '
  import { readFileSync, writeFileSync } from "node:fs";
  const path = process.argv[1];
  const value = JSON.parse(readFileSync(path, "utf8"));
  value.findings.find((item) => item.classification === "reviewed-exception").reviewedExceptionId = "exception.tampered";
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
' "$TEST_DIR/exception-map.json"
expect_failure exception-map bun "$VERIFY" --contract "$TEST_DIR/exception-map.json"

cp "$CONTRACT" "$TEST_DIR/group-count.json"
bun -e '
  import { readFileSync, writeFileSync } from "node:fs";
  const path = process.argv[1];
  const value = JSON.parse(readFileSync(path, "utf8"));
  value.expectedGroups.ddlKind["CREATE TABLE"] += 1;
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
' "$TEST_DIR/group-count.json"
expect_failure group-count bun "$VERIFY" --contract "$TEST_DIR/group-count.json"

cp "$UPSTREAM" "$TEST_DIR/migration-safety-tampered.json"
bun -e '
  import { appendFileSync } from "node:fs";
  appendFileSync(process.argv[1], "\n");
' "$TEST_DIR/migration-safety-tampered.json"
expect_failure upstream-checksum bun "$VERIFY" --upstream "$TEST_DIR/migration-safety-tampered.json"

ln -s "$CONTRACT" "$TEST_DIR/contract-link.json"
expect_failure symlink-contract bun "$VERIFY" --contract "$TEST_DIR/contract-link.json"

echo "a3-3-runtime-ddl-triage-self-test: PASS — exact 28/6/22 scanner inventory, deterministic report, readiness STOP, and 6 fail-closed tamper/path cases"
