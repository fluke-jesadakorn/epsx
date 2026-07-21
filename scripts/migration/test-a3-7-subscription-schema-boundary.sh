#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a3-7-subscription-schema-boundary.sh"
contract="$repo_root/docs/migration/contracts/a3-7-subscription-schema-boundary.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-7-subscription-schema.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "subscription runtime DDL 2→0, one 844-byte migration pinned, two 10-column tables and 20 Rust response fields verified" "$temp_dir/integrity.out"
grep -q "no runner, baseline adoption, populated upgrade, reconciliation, concurrent startup, or live database proof ran" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || {
  cat "$temp_dir/readiness.out" >&2
  echo "a3-7 subscription schema self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
}
grep -q "six residual A3.7 blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.productionReady !== false || report.readinessExit !== 3) process.exit(1);
if (report.developmentMapping.candidateServicePresent !== false || report.developmentMapping.status !== "blocked") process.exit(1);
if (report.runtimeRust.files !== 2 || report.runtimeRust.ddlFindings !== 0 || report.runtimeRust.expectedDelta !== -2) process.exit(1);
if (report.runtimeRust.qualifiedRelationOccurrences["public.subscription_plans"] !== 3) process.exit(1);
if (report.runtimeRust.qualifiedRelationOccurrences["public.subscriptions"] !== 4) process.exit(1);
if (report.models.responseFields !== 20 || report.models.nullableResponseFields !== 11 || report.models.requestFields !== 11) process.exit(1);
if (report.models.uuidPathExtractors !== 3 || report.models.queryAsOccurrences.SubscriptionPlan !== 3 || report.models.queryAsOccurrences.Subscription !== 4) process.exit(1);
if (report.migrationRoot.migrations !== 1 || report.migrationRoot.pinnedBytes !== 844 || report.migrationRoot.runner !== null) process.exit(1);
if (report.requiredTables.length !== 2 || report.requiredTables.some((table) => table.columns !== 10) || report.blockers.length !== 6) process.exit(1);
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
    echo "a3-7 subscription schema self-test: expected $label tamper exit 1" >&2
    exit 1
  }
  grep -Eq "$pattern" "$temp_dir/$label.out"
}

write_tamper "$temp_dir/production-ready.json" 'contract.productionReady = true;'
expect_integrity_failure production-ready "$temp_dir/production-ready.json" "readiness sentinel changed"

write_tamper "$temp_dir/migration-hash.json" 'contract.migrationRoot.orderedMigrations[0].sha256 = "0".repeat(64);'
expect_integrity_failure migration-hash "$temp_dir/migration-hash.json" "ordered migration pin drifted|subscription migration bytes changed"

write_tamper "$temp_dir/query-digest.json" 'contract.runtimeBoundary.compatibilityQuerySha256 = "0".repeat(64);'
expect_integrity_failure query-digest "$temp_dir/query-digest.json" "compatibility query boundary or pin drifted|compatibility query bytes changed"

write_tamper "$temp_dir/default-anchor.json" 'contract.runtimeBoundary.queryRequiredAnchors[2] = "missing required default comparison";'
expect_integrity_failure default-anchor "$temp_dir/default-anchor.json" "missing compatibility-query anchor"

write_tamper "$temp_dir/constraint-anchor.json" 'contract.runtimeBoundary.queryRequiredAnchors[5] = "missing required primary key constraint";'
expect_integrity_failure constraint-anchor "$temp_dir/constraint-anchor.json" "missing compatibility-query anchor"

write_tamper "$temp_dir/index-anchor.json" 'contract.runtimeBoundary.queryRequiredAnchors[8] = "missing required valid index";'
expect_integrity_failure index-anchor "$temp_dir/index-anchor.json" "missing compatibility-query anchor"

write_tamper "$temp_dir/column-default.json" 'contract.requiredTables[0].columns[0].sqlAnchor = "id UUID PRIMARY KEY,";'
expect_integrity_failure column-default "$temp_dir/column-default.json" "subscription_plans column definitions drifted"

write_tamper "$temp_dir/qualified-count.json" 'contract.runtimeBoundary.qualifiedRelationOccurrences["public.subscriptions"] = 3;'
expect_integrity_failure qualified-count "$temp_dir/qualified-count.json" "qualified relation counts drifted|public.subscriptions references"

write_tamper "$temp_dir/development-hash.json" 'contract.developmentBaseline.evidence[0].bytesSha256 = "0".repeat(64);'
expect_integrity_failure development-hash "$temp_dir/development-hash.json" "development baseline evidence drifted|development evidence bytes changed"

write_tamper "$temp_dir/blocker-status.json" 'contract.blockers[0].status = "aligned";'
expect_integrity_failure blocker-status "$temp_dir/blocker-status.json" "B01: residual blocker drifted"

write_tamper "$temp_dir/response-uuid-type.json" 'contract.modelBoundary.responseModels[0].fields[0].rustType = "String";'
expect_integrity_failure response-uuid-type "$temp_dir/response-uuid-type.json" "response model/schema mapping drifted"

write_tamper "$temp_dir/nullable-response-type.json" 'contract.modelBoundary.responseModels[0].fields[8].rustType = "bool";'
expect_integrity_failure nullable-response-type "$temp_dir/nullable-response-type.json" "response model/schema mapping drifted"

write_tamper "$temp_dir/request-uuid-type.json" 'contract.modelBoundary.requestModels[1].fields[0].rustType = "String";'
expect_integrity_failure request-uuid-type "$temp_dir/request-uuid-type.json" "request model mapping drifted"

write_tamper "$temp_dir/model-source-digest.json" 'contract.modelBoundary.sha256 = "0".repeat(64);'
expect_integrity_failure model-source-digest "$temp_dir/model-source-digest.json" "model source boundary or pin drifted|model source slice bytes changed"

write_tamper "$temp_dir/path-uuid-count.json" 'contract.modelBoundary.pathUuidExtractorOccurrences = 2;'
expect_integrity_failure path-uuid-count "$temp_dir/path-uuid-count.json" "model coverage counts drifted|UUID path extractor occurrence count drifted"

write_tamper "$temp_dir/query-model-count.json" 'contract.modelBoundary.queryAsOccurrences.Subscription = 3;'
expect_integrity_failure query-model-count "$temp_dir/query-model-count.json" "query model occurrences drifted|query_as response model occurrence count drifted"

set +e
EPSX_ENV=production "$verify" --mode integrity >"$temp_dir/production-env.out" 2>&1
production_env_status=$?
SUBSCRIPTION_DATABASE_URL=postgres://example.invalid/epsx "$verify" --mode integrity >"$temp_dir/database-env.out" 2>&1
database_env_status=$?
set -e
[ "$production_env_status" -eq 1 ] || {
  cat "$temp_dir/production-env.out" >&2
  echo "a3-7 subscription schema self-test: expected production-env exit 1" >&2
  exit 1
}
[ "$database_env_status" -eq 1 ] || {
  cat "$temp_dir/database-env.out" >&2
  echo "a3-7 subscription schema self-test: expected database-env exit 1" >&2
  exit 1
}
grep -q "production-looking environment" "$temp_dir/production-env.out"
grep -q "never contacts a database" "$temp_dir/database-env.out"

echo "a3-7 subscription schema self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, readiness/migration/query/default/constraint/index/qualified/development/blocker/model-uuid/model-nullability/model-digest/path/query-model/prod/db tamper=1)"
