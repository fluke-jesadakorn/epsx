#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a3-12-indexer-schema-boundary.sh"
contract="$repo_root/docs/migration/contracts/a3-12-indexer-schema-boundary.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-12-indexer-schema.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "indexer runtime DDL 5→0; three guarded public tables, 27 exact columns and chain-scoped transaction PK pinned" "$temp_dir/integrity.out"
grep -q "31 exact constraints and 10 exact btree indexes fail closed" "$temp_dir/integrity.out"
grep -q "autonomous provider, placeholder sync and fabricated ingestion are absent" "$temp_dir/integrity.out"
grep -q "all four surviving runtime relations are public-qualified; only health remains reachable" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || { cat "$temp_dir/readiness.out" >&2; exit 1; }
grep -q "ten residual A3.12 blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const r = await Bun.file(process.argv[1]).json();
if (r.productionReady !== false || r.readinessExit !== 3) process.exit(1);
if (r.provenance.standaloneSourceIndexer !== false) process.exit(1);
if (r.runtimeRust.ddlFindings !== 0 || r.runtimeRust.expectedDelta !== -5 || r.runtimeRust.fakeSyncAvailable !== false) process.exit(1);
if (Object.values(r.runtimeRust.qualifiedRelations).reduce((a,b) => a+b, 0) !== 4) process.exit(1);
if (r.migrationRoot.migrations !== 1 || r.migrationRoot.pinnedBytes !== 4822 || r.migrationRoot.guardedTables !== 3 || r.migrationRoot.guardedIndexes !== 5) process.exit(1);
if (r.schema.tables !== 3 || r.schema.columns !== 27 || r.schema.structuralConstraints !== 7 || r.schema.checkConstraints !== 24 || r.schema.indexes !== 10) process.exit(1);
if (JSON.stringify(r.schema.transactionPrimaryKey) !== JSON.stringify(["chain_id", "hash"])) process.exit(1);
if (r.blockers.length !== 10) process.exit(1);
' "$temp_dir/report-one.json"

tamper() {
  name=$1
  expression=$2
  expected=$3
  out="$temp_dir/$name.json"
  A3_IN="$contract" A3_OUT="$out" A3_EXPR="$expression" bun -e '
    const value = await Bun.file(process.env.A3_IN).json();
    Function("value", process.env.A3_EXPR)(value);
    await Bun.write(process.env.A3_OUT, `${JSON.stringify(value, null, 2)}\n`);
  '
  set +e
  "$verify" --mode integrity --contract "$out" >"$temp_dir/$name.out" 2>&1
  status=$?
  set -e
  [ "$status" -eq 1 ] || { cat "$temp_dir/$name.out" >&2; echo "tamper $name unexpectedly passed" >&2; exit 1; }
  grep -Eq "$expected" "$temp_dir/$name.out" || { cat "$temp_dir/$name.out" >&2; exit 1; }
}

tamper production-ready 'value.productionReady = true' 'readiness sentinel changed'
tamper source-commit 'value.provenance.sourceCommit = "0".repeat(40)' 'development provenance drifted'
tamper source-path 'value.provenance.absentPaths[0] = "apps/backend"' 'absent development paths drifted'
tamper runtime-blob 'value.provenance.removedRuntimeSnapshot.blob = "0".repeat(40)' 'removed runtime snapshot pin drifted'
tamper query-digest 'value.runtimeBoundary.compatibilityQuerySha256 = "0".repeat(64)' 'compatibility query bytes changed'
tamper query-bytes 'value.runtimeBoundary.compatibilityQueryBytes -= 1' 'compatibility query bytes changed'
tamper structural-array-type 'value.runtimeBoundary.structuralKeyArrayTextCastOccurrences = 1' 'structural key-array type contract drifted'
tamper relation-count 'value.runtimeBoundary.qualifiedRelationOccurrences["public.blocks"] = 1' 'public.blocks runtime occurrence count'
tamper fake-sync-policy 'value.runtimeBoundary.forbiddenRuntimeAnchors.pop()' 'unsafe runtime anchor returned|runtime boundary|drifted'
tamper migration-hash 'value.migrationRoot.orderedMigrations[0].sha256 = "0".repeat(64)' 'ordered migration pin drifted'
tamper migration-guard 'value.migrationRoot.orderedMigrations[0].guards.pop()' 'migration root boundary drifted|migration guard|drifted'
tamper global-hash-pk 'value.schemaContract.globalTransactionHashPrimaryKeyAccepted = true' 'schema fail-closed policy drifted'
tamper structural-fk 'value.schemaContract.structuralConstraints.pop()' 'schema structural constraint descriptors drifted'
tamper column-substitution 'value.schemaContract.tables.blocks[0] = "chain_ix:varchar(10):required"' 'schema column descriptors drifted'
tamper structural-substitution 'value.schemaContract.structuralConstraints[0] = "blocks_pkey:pk(chain_ix,number)"' 'schema structural constraint descriptors drifted'
tamper weak-check-substitution 'value.schemaContract.checkConstraints[1] = "blocks_number_check:blocks:check(number<=0)"' 'schema check constraint descriptors drifted'
tamper pg18-status-shape 'value.schemaContract.checkConstraints[15] = "transactions_status_check:transactions:check(statusisnullorstatus=any(array[0,1]))"' 'schema check constraint descriptors drifted'
tamper index-substitution 'value.schemaContract.indexes[0] = "blocks_pkey:blocks:unique-primary:btree(chain_ix,number)"' 'schema index descriptors drifted'
tamper pg18-index-quote 'value.schemaContract.indexes[2] = value.schemaContract.indexes[2].replace("\"timestamp\"", "timestampxx")' 'schema index descriptors drifted'
tamper check-count 'value.schemaContract.checkConstraints.pop()' 'schema check constraint descriptors drifted'
tamper index-inventory 'value.schemaContract.indexes.pop()' 'schema index descriptors drifted'
tamper inheritance 'value.schemaContract.inheritanceAccepted = true' 'schema fail-closed policy drifted'
tamper rls 'value.schemaContract.rowLevelSecurityAccepted = true' 'schema fail-closed policy drifted'
tamper opclass 'value.schemaContract.nonCatalogOperatorClassesAccepted = true' 'schema fail-closed policy drifted'
tamper collation 'value.schemaContract.indexCollationDriftAccepted = true' 'schema fail-closed policy drifted'
tamper partial-index 'value.schemaContract.partialExpressionOrIncludedIndexesAccepted = true' 'schema fail-closed policy drifted'
tamper blockers 'value.blockers.pop()' 'exact ten residual blockers are required'

echo "a3-12-indexer-schema-boundary self-test: PASS"
