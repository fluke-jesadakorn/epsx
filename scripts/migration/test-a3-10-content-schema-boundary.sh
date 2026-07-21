#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a3-10-content-schema-boundary.sh"
contract="$repo_root/docs/migration/contracts/a3-10-content-schema-boundary.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-10-content-schema.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "content runtime DDL 4→0, four guarded public tables and 34 exact columns pinned" "$temp_dir/integrity.out"
grep -q "three exact immediate unique keys and the complete inbound/outbound FK boundary are pinned" "$temp_dir/integrity.out"
grep -q "all seven pg_index unique rows are constraint-bound; standalone, partial, and expression unique indexes fail closed" "$temp_dir/integrity.out"
grep -q "JSONB/UUID/timestamptz models audited; 19 runtime relations are public-qualified" "$temp_dir/integrity.out"
grep -q "cannot apply 17 NOT NULL and one UNIQUE drift item to pre-existing tables" "$temp_dir/integrity.out"
grep -q "one preserved ON DELETE CASCADE lexical finding remains explicit" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || {
  cat "$temp_dir/readiness.out" >&2
  echo "a3-10 content schema self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
}
grep -q "six residual A3.10 blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.productionReady !== false || report.readinessExit !== 3) process.exit(1);
if (report.developmentNews.evidence !== 7 || report.developmentNews.imported !== false) process.exit(1);
if (report.runtimeRust.ddlFindings !== 0 || report.runtimeRust.expectedDelta !== -4) process.exit(1);
if (report.runtimeRust.jsonbTextProjections !== 38 || report.runtimeRust.returningStars !== 0) process.exit(1);
if (Object.values(report.runtimeRust.qualifiedRelations).reduce((sum, value) => sum + value, 0) !== 19) process.exit(1);
if (report.migrationRoot.migrations !== 1 || report.migrationRoot.pinnedBytes !== 1656 || report.migrationRoot.runner !== null) process.exit(1);
if (report.migrationRoot.guardedTables !== 4 || report.migrationRoot.lexicalCascadeFindings !== 1) process.exit(1);
if (report.schema.tables.length !== 4 || report.schema.columns !== 34 || report.schema.backingIndexes !== 7) process.exit(1);
if (report.schema.inventoriedUniqueIndexes !== 7 || report.schema.standaloneUniqueIndexesAccepted !== false) process.exit(1);
if (report.schema.partialUniqueIndexesAccepted !== false || report.schema.expressionUniqueIndexesAccepted !== false) process.exit(1);
if (report.schema.freshSchemaDriftItems !== 18 || report.schema.notNullAdditions !== 17 || report.schema.uniqueAdditions !== 1) process.exit(1);
if (report.blockers.length !== 6) process.exit(1);
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
  echo "a3-10 content schema self-test: expected production-ready tamper exit 1" >&2
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
  echo "a3-10 content schema self-test: expected migration-hash tamper exit 1" >&2
  exit 1
}
grep -q "ordered migration pin drifted\|content migration bytes changed" "$temp_dir/migration-hash.out"

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
  echo "a3-10 content schema self-test: expected query-digest tamper exit 1" >&2
  exit 1
}
grep -q "compatibility query boundary or pin drifted\|compatibility query bytes changed" "$temp_dir/query-digest.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/relation-count.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.runtimeBoundary.qualifiedRelationOccurrences["public.pages"] = 7;
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/relation-count.json" >"$temp_dir/relation-count.out" 2>&1
relation_count_status=$?
set -e
[ "$relation_count_status" -eq 1 ] || {
  cat "$temp_dir/relation-count.out" >&2
  echo "a3-10 content schema self-test: expected relation-count tamper exit 1" >&2
  exit 1
}
grep -q "qualified relation occurrence contract drifted\|public.pages runtime reference count" "$temp_dir/relation-count.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/duplicate-unique.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.constraintSemantics.uniqueKeys[2] = contract.constraintSemantics.uniqueKeys[1];
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/duplicate-unique.json" >"$temp_dir/duplicate-unique.out" 2>&1
duplicate_unique_status=$?
set -e
[ "$duplicate_unique_status" -eq 1 ] || {
  cat "$temp_dir/duplicate-unique.out" >&2
  echo "a3-10 content schema self-test: expected duplicate-unique tamper exit 1" >&2
  exit 1
}
grep -q "constraint semantics drifted" "$temp_dir/duplicate-unique.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/missing-unique.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.constraintSemantics.uniqueKeys.pop();
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/missing-unique.json" >"$temp_dir/missing-unique.out" 2>&1
missing_unique_status=$?
set -e
[ "$missing_unique_status" -eq 1 ] || {
  cat "$temp_dir/missing-unique.out" >&2
  echo "a3-10 content schema self-test: expected missing-unique tamper exit 1" >&2
  exit 1
}
grep -q "constraint semantics drifted" "$temp_dir/missing-unique.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/deferrable-unique.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.constraintSemantics.uniqueConstraintPolicy.deferrable = true;
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/deferrable-unique.json" >"$temp_dir/deferrable-unique.out" 2>&1
deferrable_unique_status=$?
set -e
[ "$deferrable_unique_status" -eq 1 ] || {
  cat "$temp_dir/deferrable-unique.out" >&2
  echo "a3-10 content schema self-test: expected deferrable-unique tamper exit 1" >&2
  exit 1
}
grep -q "constraint semantics drifted" "$temp_dir/deferrable-unique.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/standalone-unique-index.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.constraintSemantics.uniqueIndexInventory.standaloneAccepted = true;
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/standalone-unique-index.json" >"$temp_dir/standalone-unique-index.out" 2>&1
standalone_unique_index_status=$?
set -e
[ "$standalone_unique_index_status" -eq 1 ] || {
  cat "$temp_dir/standalone-unique-index.out" >&2
  echo "a3-10 content schema self-test: expected standalone-unique-index tamper exit 1" >&2
  exit 1
}
grep -q "constraint semantics drifted" "$temp_dir/standalone-unique-index.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/partial-unique-index.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.constraintSemantics.uniqueIndexInventory.partialAccepted = true;
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/partial-unique-index.json" >"$temp_dir/partial-unique-index.out" 2>&1
partial_unique_index_status=$?
set -e
[ "$partial_unique_index_status" -eq 1 ] || {
  cat "$temp_dir/partial-unique-index.out" >&2
  echo "a3-10 content schema self-test: expected partial-unique-index tamper exit 1" >&2
  exit 1
}
grep -q "constraint semantics drifted" "$temp_dir/partial-unique-index.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/expression-unique-index.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.constraintSemantics.uniqueIndexInventory.expressionAccepted = true;
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/expression-unique-index.json" >"$temp_dir/expression-unique-index.out" 2>&1
expression_unique_index_status=$?
set -e
[ "$expression_unique_index_status" -eq 1 ] || {
  cat "$temp_dir/expression-unique-index.out" >&2
  echo "a3-10 content schema self-test: expected expression-unique-index tamper exit 1" >&2
  exit 1
}
grep -q "constraint semantics drifted" "$temp_dir/expression-unique-index.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/fk-direction.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.constraintSemantics.foreignKeyBoundary.inventoryDirection = "outbound-only";
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/fk-direction.json" >"$temp_dir/fk-direction.out" 2>&1
fk_direction_status=$?
set -e
[ "$fk_direction_status" -eq 1 ] || {
  cat "$temp_dir/fk-direction.out" >&2
  echo "a3-10 content schema self-test: expected FK-direction tamper exit 1" >&2
  exit 1
}
grep -q "constraint semantics drifted" "$temp_dir/fk-direction.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/cascade-classification.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.legacyCascadeFinding.classification = "aligned";
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/cascade-classification.json" >"$temp_dir/cascade-classification.out" 2>&1
cascade_classification_status=$?
set -e
[ "$cascade_classification_status" -eq 1 ] || {
  cat "$temp_dir/cascade-classification.out" >&2
  echo "a3-10 content schema self-test: expected cascade-classification tamper exit 1" >&2
  exit 1
}
grep -q "legacy cascade finding drifted" "$temp_dir/cascade-classification.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/source-commit.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.developmentNewsBoundary.sourceCommit = "0".repeat(40);
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/source-commit.json" >"$temp_dir/source-commit.out" 2>&1
source_commit_status=$?
set -e
[ "$source_commit_status" -eq 1 ] || {
  cat "$temp_dir/source-commit.out" >&2
  echo "a3-10 content schema self-test: expected source-commit tamper exit 1" >&2
  exit 1
}
grep -q "pinned development source commit drifted" "$temp_dir/source-commit.out"

A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$temp_dir/source-blob.json" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
contract.developmentNewsBoundary.evidence[0].blob = "0".repeat(40);
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/source-blob.json" >"$temp_dir/source-blob.out" 2>&1
source_blob_status=$?
set -e
[ "$source_blob_status" -eq 1 ] || {
  cat "$temp_dir/source-blob.out" >&2
  echo "a3-10 content schema self-test: expected source-blob tamper exit 1" >&2
  exit 1
}
grep -q "news-table: pinned blob drifted" "$temp_dir/source-blob.out"

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
  echo "a3-10 content schema self-test: expected blocker-status tamper exit 1" >&2
  exit 1
}
grep -q "B01: residual blocker drifted" "$temp_dir/blocker-status.out"

set +e
EPSX_ENV=production "$verify" --mode integrity >"$temp_dir/production-env.out" 2>&1
production_env_status=$?
CONTENT_DATABASE_URL=postgres://example.invalid/epsx "$verify" --mode integrity >"$temp_dir/database-env.out" 2>&1
database_env_status=$?
set -e
[ "$production_env_status" -eq 1 ] || {
  cat "$temp_dir/production-env.out" >&2
  echo "a3-10 content schema self-test: expected production-env exit 1" >&2
  exit 1
}
[ "$database_env_status" -eq 1 ] || {
  cat "$temp_dir/database-env.out" >&2
  echo "a3-10 content schema self-test: expected database-env exit 1" >&2
  exit 1
}
grep -q "production-looking environment" "$temp_dir/production-env.out"
grep -q "never contacts a database" "$temp_dir/database-env.out"

echo "a3-10 content schema self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, readiness/migration/query/relation/unique/standalone-index/partial-index/expression-index/fk/cascade/source-commit/source-blob/blocker/prod/db tamper=1)"
