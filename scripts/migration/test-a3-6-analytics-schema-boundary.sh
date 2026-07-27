#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a3-6-analytics-schema-boundary.sh"
contract="$repo_root/docs/migration/contracts/a3-6-analytics-schema-boundary.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-6-analytics-schema.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "analytics runtime DDL 1→0, baseline plus additive subject migration pinned, seven-column compatibility boundary verified" "$temp_dir/integrity.out"
grep -q "no runner, baseline adoption, populated upgrade, reconciliation, concurrent startup, or live database proof ran" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || {
  cat "$temp_dir/readiness.out" >&2
  echo "a3-6 analytics schema self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
}
grep -q "six residual A3.6 blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.productionReady !== false || report.readinessExit !== 3) process.exit(1);
if (report.runtimeRust.ddlFindings !== 0 || report.runtimeRust.expectedDelta !== -1) process.exit(1);
if (report.runtimeRust.qualifiedEventsRelation !== "public.events" || report.runtimeRust.qualifiedEventsSqlOccurrences !== 5) process.exit(1);
if (report.migrationRoot.migrations !== 2 || report.migrationRoot.pinnedBytes !== 675 || report.migrationRoot.runner !== null) process.exit(1);
if (report.requiredColumns.length !== 6 || report.blockers.length !== 6) process.exit(1);
' "$temp_dir/report-one.json"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/production-ready.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.productionReady = true;
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/production-ready.json" >"$temp_dir/production-ready.out" 2>&1
production_ready_status=$?
set -e
[ "$production_ready_status" -eq 1 ] || {
  cat "$temp_dir/production-ready.out" >&2
  echo "a3-6 analytics schema self-test: expected production-ready tamper exit 1" >&2
  exit 1
}
grep -q "readiness sentinel changed" "$temp_dir/production-ready.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/migration-hash.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.migrationRoot.orderedMigrations[0].sha256 = "0".repeat(64);
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/migration-hash.json" >"$temp_dir/migration-hash.out" 2>&1
migration_hash_status=$?
set -e
[ "$migration_hash_status" -eq 1 ] || {
  cat "$temp_dir/migration-hash.out" >&2
  echo "a3-6 analytics schema self-test: expected migration-hash tamper exit 1" >&2
  exit 1
}
grep -q "ordered migration pin drifted\|analytics migration bytes changed" "$temp_dir/migration-hash.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/blocker-status.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.blockers[0].status = "aligned";
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/blocker-status.json" >"$temp_dir/blocker-status.out" 2>&1
blocker_status=$?
set -e
[ "$blocker_status" -eq 1 ] || {
  cat "$temp_dir/blocker-status.out" >&2
  echo "a3-6 analytics schema self-test: expected blocker-status tamper exit 1" >&2
  exit 1
}
grep -q "B01: residual blocker drifted" "$temp_dir/blocker-status.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/query-anchor.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.runtimeBoundary.queryRequiredAnchors[0] = "missing read-only anchor";
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/query-anchor.json" >"$temp_dir/query-anchor.out" 2>&1
query_anchor_status=$?
set -e
[ "$query_anchor_status" -eq 1 ] || {
  cat "$temp_dir/query-anchor.out" >&2
  echo "a3-6 analytics schema self-test: expected query-anchor tamper exit 1" >&2
  exit 1
}
grep -q "missing compatibility-query anchor" "$temp_dir/query-anchor.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/query-digest.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.runtimeBoundary.compatibilityQuerySha256 = "0".repeat(64);
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/query-digest.json" >"$temp_dir/query-digest.out" 2>&1
query_digest_status=$?
set -e
[ "$query_digest_status" -eq 1 ] || {
  cat "$temp_dir/query-digest.out" >&2
  echo "a3-6 analytics schema self-test: expected query-digest tamper exit 1" >&2
  exit 1
}
grep -q "runtime boundary anchors or query pin drifted\|compatibility query bytes changed" "$temp_dir/query-digest.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/qualified-events-count.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.runtimeBoundary.qualifiedEventsSqlOccurrences = 6;
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/qualified-events-count.json" >"$temp_dir/qualified-events-count.out" 2>&1
qualified_events_count_status=$?
set -e
[ "$qualified_events_count_status" -eq 1 ] || {
  cat "$temp_dir/qualified-events-count.out" >&2
  echo "a3-6 analytics schema self-test: expected qualified-events-count tamper exit 1" >&2
  exit 1
}
grep -q "runtime boundary anchors or query pin drifted\|public.events relation references" "$temp_dir/qualified-events-count.out"

set +e
EPSX_ENV=production "$verify" --mode integrity >"$temp_dir/production-env.out" 2>&1
production_env_status=$?
ANALYTICS_DATABASE_URL=postgres://example.invalid/epsx "$verify" --mode integrity >"$temp_dir/database-env.out" 2>&1
database_env_status=$?
set -e
[ "$production_env_status" -eq 1 ] || {
  cat "$temp_dir/production-env.out" >&2
  echo "a3-6 analytics schema self-test: expected production-env exit 1" >&2
  exit 1
}
[ "$database_env_status" -eq 1 ] || {
  cat "$temp_dir/database-env.out" >&2
  echo "a3-6 analytics schema self-test: expected database-env exit 1" >&2
  exit 1
}
grep -q "production-looking environment" "$temp_dir/production-env.out"
grep -q "never contacts a database" "$temp_dir/database-env.out"

echo "a3-6 analytics schema self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, readiness/migration/query/qualified-relation/blocker/anchor/prod/db tamper=1)"
