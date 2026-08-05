#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a3-13-pay-schema-boundary.sh"
contract="$repo_root/docs/migration/contracts/a3-13-pay-schema-boundary.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-13-pay-schema.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

expect_contract_tamper() {
  name=$1
  mutation=$2
  expected=$3
  output_contract="$temp_dir/$name.json"
  output_log="$temp_dir/$name.out"

  A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$output_contract" A3_MUTATION="$mutation" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
Function("contract", process.env.A3_MUTATION)(contract);
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
  set +e
  "$verify" --mode integrity --contract "$output_contract" >"$output_log" 2>&1
  status=$?
  set -e
  [ "$status" -eq 1 ] || {
    cat "$output_log" >&2
    echo "a3-13 pay schema self-test: expected $name tamper exit 1, got $status" >&2
    exit 1
  }
  grep -Eq "$expected" "$output_log" || {
    cat "$output_log" >&2
    echo "a3-13 pay schema self-test: $name did not report expected failure" >&2
    exit 1
  }
}

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "pay runtime DDL 10→0" "$temp_dir/integrity.out"
grep -q "39 columns, five structural constraints, PG18 29-NOT-NULL/34-total constraints, zero FKs, and 11 indexes" "$temp_dir/integrity.out"
grep -q "policies, partial/expression/INCLUDE, inheritance/partition, RLS, default type collation, pg_catalog text_ops, and search_path drift are rejected" "$temp_dir/integrity.out"
grep -q "54 runtime relation references are public-qualified" "$temp_dir/integrity.out"
grep -q "unsafe financial, admin mutation, deposit-confirmation, and webhook routes remain fail-closed 404" "$temp_dir/integrity.out"
grep -q "payment database authority remains unresolved" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || {
  cat "$temp_dir/readiness.out" >&2
  echo "a3-13 pay schema self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
}
grep -q "eight residual A3.13 blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.productionReady !== false || report.readinessExit !== 3) process.exit(1);
if (report.authority.decision !== "unresolved-do-not-cut-over-or-dual-write" || report.authority.evidence !== 7) process.exit(1);
if (report.runtimeRust.files !== 12 || report.runtimeRust.ddlFindings !== 0 || report.runtimeRust.expectedDelta !== -10) process.exit(1);
if (Object.values(report.runtimeRust.qualifiedRelations).reduce((sum, value) => sum + value, 0) !== 55) process.exit(1);
if (report.runtimeRust.bindAnchors !== 11 || report.migrationRoot.statements !== 10 || report.migrationRoot.runner !== null) process.exit(1);
if (report.schema.tables.length !== 4 || report.schema.columns !== 39 || report.schema.structuralConstraints !== 5) process.exit(1);
if (report.schema.postgres18NotNullConstraints !== 29 || report.schema.postgres18CatalogConstraints !== 34) process.exit(1);
if (report.schema.primaryKeys !== 4 || report.schema.uniqueKeys !== 1 || report.schema.foreignKeys !== 0) process.exit(1);
if (report.schema.indexes !== 11 || report.schema.standaloneIndexes !== 6) process.exit(1);
if (report.schema.partialIndexes !== 0 || report.schema.expressionIndexes !== 0 || report.schema.includeIndexes !== 0) process.exit(1);
if (report.schema.policies !== 0 || report.schema.defaultTypeCollationColumns !== 28) process.exit(1);
if (report.schema.freshSchemaDriftItems !== 13 || report.failClosed.unsafeRoutesReachable !== false) process.exit(1);
if (report.isolatedPostgres18.readinessEvidence !== false || report.isolatedPostgres18.cleanupConfirmed !== true) process.exit(1);
if (report.blockers.length !== 8) process.exit(1);
' "$temp_dir/report-one.json"

expect_contract_tamper production-ready \
  'contract.productionReady = true' \
  'readiness sentinel changed'
expect_contract_tamper missing-deployment-safety \
  'delete contract.safety.deploymentAuthorized' \
  'safety boundary drifted'
expect_contract_tamper extra-production-claim \
  'contract.productionClaim = "ready"' \
  'top-level field inventory drifted'
expect_contract_tamper authority \
  'contract.authorityBoundary.decision = "epsx_pay-is-authority"' \
  'database authority STOP changed'
expect_contract_tamper authority-extra-production-write \
  'contract.authorityBoundary.productionWriteAuthority = "epsx_pay"' \
  'authority boundary field inventory drifted'
expect_contract_tamper source-commit \
  'contract.authorityBoundary.developmentSourceCommit = "0".repeat(40)' \
  'pinned development source commit drifted'
expect_contract_tamper source-blob \
  'contract.authorityBoundary.developmentEvidence[0].blob = "0".repeat(40)' \
  'immutable development evidence drifted|pinned source blob drifted'
expect_contract_tamper removed-source-blob \
  'contract.runtimeBoundary.removedRuntimeSnapshot.blob = "0".repeat(40)' \
  'removed runtime snapshot drifted|removed runtime snapshot blob drifted'
expect_contract_tamper runtime-extra-provider-authority \
  'contract.runtimeBoundary.providerAccessAuthorized = true' \
  'runtime boundary field inventory drifted'
expect_contract_tamper handler-source-blob \
  'contract.runtimeBoundary.handlerQualificationSource.files[0].blob = "0".repeat(40)' \
  'qualification-only handler pins drifted|pinned handler blob drifted'
expect_contract_tamper migration-digest \
  'contract.migrationRoot.orderedMigrations[0].sha256 = "0".repeat(64)' \
  'migration metadata drifted|pay migration bytes changed'
expect_contract_tamper migration-root-extra-production-runner \
  'contract.migrationRoot.productionRunner = "enabled"' \
  'migration root field inventory drifted'
expect_contract_tamper query-digest \
  'contract.runtimeBoundary.compatibilityQuerySha256 = "0".repeat(64)' \
  'compatibility function boundary drifted|compatibility query bytes changed'
expect_contract_tamper query-mutation-policy \
  'contract.runtimeBoundary.queryRequiredAnchors[0] = "UPDATE public.pay_intents"' \
  'compatibility-query anchors drifted|mutation or command token'
expect_contract_tamper hostile-search-path \
  'const index = contract.runtimeBoundary.queryRequiredAnchors.findIndex((value) => value.includes("public.pay_intents")); contract.runtimeBoundary.queryRequiredAnchors[index] = "to_regclass(\"pay_intents\")"' \
  'compatibility-query anchors drifted|missing compatibility-query anchor'
expect_contract_tamper unqualified-runtime-relation \
  'contract.runtimeBoundary.qualifiedRelationOccurrences["public.pay_intents"] = 27' \
  'qualified relation occurrence contract drifted|runtime relation count'
expect_contract_tamper bind-contract \
  'contract.runtimeBoundary.bindAnchors[0] = ".bind(state.chain_id)"' \
  'SQLx bind anchors drifted|missing SQLx bind anchor'
expect_contract_tamper fake-rust-model-field \
  'contract.rustModelContract.requiredNonOptionalDefaultColumns[0] = "Fake.field"' \
  'Rust model contract drifted'
expect_contract_tamper fake-not-null-column \
  'contract.freshSchemaDrift.notNullAdditions[0] = "fake.column"' \
  'fresh-schema drift boundary drifted'
expect_contract_tamper table-extra-production-status \
  'contract.requiredTables[0].status = "production-ready"' \
  'pay_intents required table field inventory drifted'
expect_contract_tamper column-extra-production-canonical \
  'contract.requiredTables[0].columns[0].productionCanonical = true' \
  'pay_intents column descriptor 1 field inventory drifted'
expect_contract_tamper column-type \
  'contract.requiredTables[0].columns[1].sqlAnchor = "chain_id VARCHAR(10) NOT NULL,"' \
  'migration anchor must occur exactly once|column definitions drifted'
expect_contract_tamper column-nullability \
  'contract.requiredTables[0].columns[6].sqlAnchor = "status VARCHAR(30) DEFAULT '\''pending'\'',"' \
  'migration anchor must occur exactly once|column definitions drifted'
expect_contract_tamper column-default \
  'contract.requiredTables[2].columns[3].sqlAnchor = "max_uses INTEGER NOT NULL DEFAULT 2,"' \
  'migration anchor must occur exactly once|column definitions drifted'
expect_contract_tamper primary-key \
  'contract.constraintAndIndexSemantics.primaryKeys.pop()' \
  'constraint and index semantics drifted'
expect_contract_tamper unique-key \
  'contract.constraintAndIndexSemantics.uniqueKeys = []' \
  'constraint and index semantics drifted'
expect_contract_tamper foreign-key \
  'contract.constraintAndIndexSemantics.foreignKeys.push("pay_links.intent_id->pay_intents.id")' \
  'constraint and index semantics drifted'
expect_contract_tamper index-count \
  'contract.constraintAndIndexSemantics.totalIndexes = 10' \
  'constraint and index semantics drifted'
expect_contract_tamper pg18-not-null-count \
  'contract.constraintAndIndexSemantics.postgres18NotNullConstraints = 28' \
  'constraint and index semantics drifted'
expect_contract_tamper policy-count \
  'contract.constraintAndIndexSemantics.policies = 1' \
  'constraint and index semantics drifted'
expect_contract_tamper default-type-collation \
  'contract.constraintAndIndexSemantics.defaultTypeCollationColumns = 27' \
  'constraint and index semantics drifted'
expect_contract_tamper partial-index \
  'contract.constraintAndIndexSemantics.partialIndexes = 1' \
  'constraint and index semantics drifted'
expect_contract_tamper expression-index \
  'contract.constraintAndIndexSemantics.expressionIndexes = 1' \
  'constraint and index semantics drifted'
expect_contract_tamper include-index \
  'contract.constraintAndIndexSemantics.includeIndexes = 1' \
  'constraint and index semantics drifted'
expect_contract_tamper inheritance \
  'contract.constraintAndIndexSemantics.inheritedOrPartitionedRelations = 1' \
  'constraint and index semantics drifted'
expect_contract_tamper rls \
  'contract.constraintAndIndexSemantics.rowLevelSecurityRelations = 1' \
  'constraint and index semantics drifted'
expect_contract_tamper opclass \
  'contract.constraintAndIndexSemantics.opclass = "varchar_ops"' \
  'constraint and index semantics drifted'
expect_contract_tamper opclass-namespace \
  'contract.constraintAndIndexSemantics.opclassNamespace = "public"' \
  'constraint and index semantics drifted'
expect_contract_tamper collation \
  'contract.constraintAndIndexSemantics.collationPolicy = "any"' \
  'constraint and index semantics drifted'
expect_contract_tamper unsafe-route-reachability \
  'contract.failClosedBoundary.unsafeRoutesReachable = true' \
  'fail-closed boundary drifted|fail-closed policy drifted'
expect_contract_tamper blocker-status \
  'contract.blockers[0].status = "aligned"' \
  'residual blocker inventory drifted'
expect_contract_tamper blocker-summary \
  'contract.blockers[0].summary = "Changed blocker text"' \
  'residual blocker inventory drifted'
expect_contract_tamper isolated-cleanup-claim \
  'contract.isolatedPostgres18Evidence.cleanupConfirmed = false' \
  'isolated PostgreSQL 18 evidence drifted'

set +e
EPSX_ENV=production "$verify" --mode integrity >"$temp_dir/production-env.out" 2>&1
production_env_status=$?
PAY_DATABASE_URL=postgres://example.invalid/epsx "$verify" --mode integrity >"$temp_dir/database-env.out" 2>&1
database_env_status=$?
RPC_URL=https://example.invalid "$verify" --mode integrity >"$temp_dir/chain-env.out" 2>&1
chain_env_status=$?
set -e
[ "$production_env_status" -eq 1 ] || {
  cat "$temp_dir/production-env.out" >&2
  echo "a3-13 pay schema self-test: expected production-env exit 1" >&2
  exit 1
}
[ "$database_env_status" -eq 1 ] || {
  cat "$temp_dir/database-env.out" >&2
  echo "a3-13 pay schema self-test: expected database-env exit 1" >&2
  exit 1
}
[ "$chain_env_status" -eq 1 ] || {
  cat "$temp_dir/chain-env.out" >&2
  echo "a3-13 pay schema self-test: expected chain-env exit 1" >&2
  exit 1
}
grep -q "production-looking environment" "$temp_dir/production-env.out"
grep -q "never contacts a database or chain" "$temp_dir/database-env.out"
grep -q "never contacts a database or chain" "$temp_dir/chain-env.out"

echo "a3-13 pay schema self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, safety/top-level/nested-field-inventory/authority/source/migration/query/search_path/relation/bind/model/drift/constraint/index/policy/inheritance/RLS/opclass/collation/fail-closed/blocker/isolated-evidence/prod/db/chain tamper=1)"
