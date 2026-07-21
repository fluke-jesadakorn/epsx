#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a3-9-wallet-schema-boundary.sh"
contract="$repo_root/docs/migration/contracts/a3-9-wallet-schema-boundary.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-9-wallet-schema.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "wallet runtime DDL 3→0, one 775-byte migration pinned, three tables/17 columns and Rust bind models verified" "$temp_dir/integrity.out"
grep -q "no runner, baseline adoption, populated upgrade, reconciliation, concurrent startup, or live database proof ran" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || {
  cat "$temp_dir/readiness.out" >&2
  echo "a3-9 wallet schema self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
}
grep -q "six residual A3.9 blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.productionReady !== false || report.readinessExit !== 3) process.exit(1);
if (report.developmentMapping.candidateServicePresent !== false || report.developmentMapping.status !== "blocked") process.exit(1);
if (report.runtimeRust.files !== 2 || report.runtimeRust.ddlFindings !== 0 || report.runtimeRust.expectedDelta !== -3) process.exit(1);
if (report.runtimeRust.qualifiedRelationOccurrences["public.accounts"] !== 3) process.exit(1);
if (report.runtimeRust.qualifiedRelationOccurrences["public.nonces"] !== 1) process.exit(1);
if (report.runtimeRust.qualifiedRelationOccurrences["public.signed_transactions"] !== 1) process.exit(1);
if (report.runtimeRust.compatibilityQueryBytes !== 22561) process.exit(1);
if (report.migrationRoot.migrations !== 1 || report.migrationRoot.pinnedBytes !== 775 || report.migrationRoot.runner !== null) process.exit(1);
if (report.schema.tables !== 3 || report.schema.columns !== 17 || report.schema.nullableColumns !== 9 || report.schema.expectedNotNullColumns !== 8 || report.schema.pg18NotNullInventory !== true || report.schema.prePg18NoRowPath !== true || report.schema.constraints !== 3 || report.schema.indexes !== 3 || report.schema.serialSequences !== 1 || report.schema.exactDefaultDependencies !== 1 || report.schema.datetimePrecisionColumns !== 3 || report.schema.databaseDefaultCollationColumns !== 12) process.exit(1);
if (report.models.responseFields !== 4 || report.models.nullableResponseFields !== 2 || report.models.uuidFields !== 0 || report.models.boundedBindAnchors !== 7 || report.models.hermeticBinaryTests !== 4 || report.models.atomicTransactionExecutors !== 2) process.exit(1);
if (report.blockers.length !== 6) process.exit(1);
' "$temp_dir/report-one.json"

write_tamper() {
  output=$1
  expression=$2
  A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$output" A3_EXPRESSION="$expression" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
const mutate = new Function("contract", process.env.A3_EXPRESSION);
mutate(contract);
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
}

expect_integrity_failure() {
  label=$1
  tampered=$2
  pattern=$3
  set +e
  "$verify" --mode integrity --contract "$tampered" >"$temp_dir/$label.out" 2>&1
  status=$?
  set -e
  [ "$status" -eq 1 ] || {
    cat "$temp_dir/$label.out" >&2
    echo "a3-9 wallet schema self-test: expected $label tamper exit 1" >&2
    exit 1
  }
  grep -Eq "$pattern" "$temp_dir/$label.out"
}

write_tamper "$temp_dir/production-ready.json" 'contract.productionReady = true;'
expect_integrity_failure production-ready "$temp_dir/production-ready.json" "readiness sentinel changed"

write_tamper "$temp_dir/migration-hash.json" 'contract.migrationRoot.orderedMigrations[0].sha256 = "0".repeat(64);'
expect_integrity_failure migration-hash "$temp_dir/migration-hash.json" "ordered migration pin drifted|wallet migration bytes changed"

write_tamper "$temp_dir/query-digest.json" 'contract.runtimeBoundary.compatibilityQuerySha256 = "0".repeat(64);'
expect_integrity_failure query-digest "$temp_dir/query-digest.json" "compatibility query boundary or pin drifted|compatibility query bytes changed"

write_tamper "$temp_dir/constraint-inventory.json" 'contract.runtimeBoundary.adversarialGuards.constraintInventoryJoin = "JOIN pg_catalog.pg_index AS constraint_index";'
expect_integrity_failure constraint-inventory "$temp_dir/constraint-inventory.json" "adversarial query guards drifted"

write_tamper "$temp_dir/double-nextval.json" 'contract.runtimeBoundary.adversarialGuards.allowedSerialDefaultExpressions[0] = "nextval twice";'
expect_integrity_failure double-nextval "$temp_dir/double-nextval.json" "adversarial query guards drifted"

write_tamper "$temp_dir/inheritance-inventory.json" 'contract.runtimeBoundary.adversarialGuards.inheritanceInventory = "missing pg_inherits";'
expect_integrity_failure inheritance-inventory "$temp_dir/inheritance-inventory.json" "adversarial query guards drifted"

write_tamper "$temp_dir/relation-not-exists.json" 'contract.runtimeBoundary.adversarialGuards.relationNotExistsStructure = "duplicate opener allowed";'
expect_integrity_failure relation-not-exists "$temp_dir/relation-not-exists.json" "adversarial query guards drifted"

write_tamper "$temp_dir/not-null-exposure.json" 'contract.runtimeBoundary.adversarialGuards.notNullCatalogExposure = "version blind";'
expect_integrity_failure not-null-exposure "$temp_dir/not-null-exposure.json" "adversarial query guards drifted"

write_tamper "$temp_dir/not-null-enforcement.json" 'contract.runtimeBoundary.adversarialGuards.notNullEnforcement = "convalidated only";'
expect_integrity_failure not-null-enforcement "$temp_dir/not-null-enforcement.json" "adversarial query guards drifted"

write_tamper "$temp_dir/pre-pg18-path.json" 'contract.runtimeBoundary.adversarialGuards.prePg18NoRowPath = "false";'
expect_integrity_failure pre-pg18-path "$temp_dir/pre-pg18-path.json" "adversarial query guards drifted"

write_tamper "$temp_dir/nullable-not-null.json" 'contract.runtimeBoundary.adversarialGuards.nullableColumnRule = "nullable constraints allowed";'
expect_integrity_failure nullable-not-null "$temp_dir/nullable-not-null.json" "adversarial query guards drifted"

write_tamper "$temp_dir/default-anchor.json" 'contract.runtimeBoundary.queryRequiredAnchors[1] = "missing identity guard";'
expect_integrity_failure default-anchor "$temp_dir/default-anchor.json" "missing compatibility-query anchor"

write_tamper "$temp_dir/constraint-anchor.json" 'contract.runtimeBoundary.queryRequiredAnchors[4] = "missing primary key guard";'
expect_integrity_failure constraint-anchor "$temp_dir/constraint-anchor.json" "missing compatibility-query anchor"

write_tamper "$temp_dir/index-anchor.json" 'contract.runtimeBoundary.queryRequiredAnchors[7] = "missing predicate guard";'
expect_integrity_failure index-anchor "$temp_dir/index-anchor.json" "missing compatibility-query anchor"

write_tamper "$temp_dir/qualified-count.json" 'contract.runtimeBoundary.qualifiedRelationOccurrences["public.accounts"] = 2;'
expect_integrity_failure qualified-count "$temp_dir/qualified-count.json" "qualified relation counts drifted|public.accounts references"

write_tamper "$temp_dir/nullable-role.json" 'contract.modelBoundary.responseModels[0].fields[3].rustType = "String";'
expect_integrity_failure nullable-role "$temp_dir/nullable-role.json" "account response model drifted"

write_tamper "$temp_dir/scalar-type.json" 'contract.modelBoundary.databaseScalarTypes[0].rustType = "u64";'
expect_integrity_failure scalar-type "$temp_dir/scalar-type.json" "database scalar types drifted"

write_tamper "$temp_dir/model-digest.json" 'contract.modelBoundary.accountSlice.sha256 = "0".repeat(64);'
expect_integrity_failure model-digest "$temp_dir/model-digest.json" "account model source slice bytes changed"

write_tamper "$temp_dir/column-nullability.json" 'contract.requiredTables[0].columns[3].nullable = false;'
expect_integrity_failure column-nullability "$temp_dir/column-nullability.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/column-name.json" 'contract.requiredTables[0].columns[0].name = "wallet";'
expect_integrity_failure column-name "$temp_dir/column-name.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/column-type.json" 'contract.requiredTables[1].columns[2].databaseType = "integer";'
expect_integrity_failure column-type "$temp_dir/column-type.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/column-default.json" 'contract.requiredTables[0].columns[3].default = null;'
expect_integrity_failure column-default "$temp_dir/column-default.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/datetime-precision.json" 'contract.requiredTables[0].columns[5].datetimePrecision = 3;'
expect_integrity_failure datetime-precision "$temp_dir/datetime-precision.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/collation.json" 'contract.requiredTables[0].columns[0].collation = null;'
expect_integrity_failure collation "$temp_dir/collation.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/operator-class.json" 'contract.requiredTables[0].onlyIndex.operatorClasses[0] = "varchar_ops";'
expect_integrity_failure operator-class "$temp_dir/operator-class.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/operator-namespace.json" 'contract.requiredTables[0].onlyIndex.operatorClassNamespaces[0] = "public";'
expect_integrity_failure operator-namespace "$temp_dir/operator-namespace.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/pk-deferrable.json" 'contract.requiredTables[0].primaryKeyConstraint.deferrable = true;'
expect_integrity_failure pk-deferrable "$temp_dir/pk-deferrable.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/sequence-max.json" 'contract.requiredTables[2].serialSequence.max = 9223372036854775807;'
expect_integrity_failure sequence-max "$temp_dir/sequence-max.json" "required wallet table/column/constraint/index contract drifted|serial sequence drifted"

write_tamper "$temp_dir/sequence-dependency.json" 'contract.requiredTables[2].serialSequence.defaultDependency = "name-only";'
expect_integrity_failure sequence-dependency "$temp_dir/sequence-dependency.json" "required wallet table/column/constraint/index contract drifted"

write_tamper "$temp_dir/send-operation-digest.json" 'contract.modelBoundary.sendOperationSlice.sha256 = "0".repeat(64);'
expect_integrity_failure send-operation-digest "$temp_dir/send-operation-digest.json" "send operation source slice bytes changed"

write_tamper "$temp_dir/helper-digest.json" 'contract.modelBoundary.bindHelperSlice.sha256 = "0".repeat(64);'
expect_integrity_failure helper-digest "$temp_dir/helper-digest.json" "bind helpers source slice bytes changed"

write_tamper "$temp_dir/transaction-begin.json" 'contract.modelBoundary.transactionBoundary.beginAnchor = ".begin_unchecked()";'
expect_integrity_failure transaction-begin "$temp_dir/transaction-begin.json" "transaction boundary drifted"

write_tamper "$temp_dir/development-commit.json" 'contract.developmentBaseline.targetCommit = "0".repeat(40);'
expect_integrity_failure development-commit "$temp_dir/development-commit.json" "development baseline drifted|immutable development commit"

write_tamper "$temp_dir/blocker-status.json" 'contract.blockers[0].status = "aligned";'
expect_integrity_failure blocker-status "$temp_dir/blocker-status.json" "B01: residual blocker drifted"

set +e
EPSX_ENV=production "$verify" --mode integrity >"$temp_dir/production-env.out" 2>&1
production_env_status=$?
WALLET_DATABASE_URL=postgres://example.invalid/epsx "$verify" --mode integrity >"$temp_dir/database-env.out" 2>&1
database_env_status=$?
set -e
[ "$production_env_status" -eq 1 ] || {
  cat "$temp_dir/production-env.out" >&2
  echo "a3-9 wallet schema self-test: expected production-env exit 1" >&2
  exit 1
}
[ "$database_env_status" -eq 1 ] || {
  cat "$temp_dir/database-env.out" >&2
  echo "a3-9 wallet schema self-test: expected database-env exit 1" >&2
  exit 1
}
grep -q "production-looking environment" "$temp_dir/production-env.out"
grep -q "never contacts a database" "$temp_dir/database-env.out"

echo "a3-9 wallet schema self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, pg18-not-null/constraint-inventory/double-nextval/inheritance/relation-not-exists/query/schema/model/transaction/prod/db tamper=1)"
