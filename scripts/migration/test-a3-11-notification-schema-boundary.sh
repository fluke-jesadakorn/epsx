#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a3-11-notification-schema-boundary.sh"
contract="$repo_root/docs/migration/contracts/a3-11-notification-schema-boundary.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-11-notification-schema.XXXXXX")
fixture="$temp_dir/worktree"
fixture_registered=false

cleanup() {
  if [ "$fixture_registered" = true ]; then
    git -C "$repo_root" worktree remove --force "$fixture" >/dev/null 2>&1 || true
  fi
  rm -rf -- "$temp_dir"
}
trap cleanup EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "runtime DDL 4→0 and startup seed calls 2→0" "$temp_dir/integrity.out"
grep -q "26 exact columns, 14 NOT NULL columns, three exact keys, and five exact indexes" "$temp_dir/integrity.out"
grep -q "PostgreSQL 18 keys require connoinherit=true" "$temp_dir/integrity.out"
grep -q "query/up/down evidence is independently hard-pinned" "$temp_dir/integrity.out"
grep -q "complete inbound/outbound FK and CHECK inventories are empty" "$temp_dir/integrity.out"
grep -q "19 runtime relations are public-qualified" "$temp_dir/integrity.out"
grep -q "seven blockers remain" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || {
  cat "$temp_dir/readiness.out" >&2
  echo "a3-11 notification schema self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
}
grep -q "seven residual blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.productionReady !== false || report.readinessExit !== 3) process.exit(1);
if (report.evidencePinning.authority !== "contract-plus-independent-verifier-constants" || report.evidencePinning.catalogIdentifiers !== 11 || report.evidencePinning.exactDownBody !== true) process.exit(1);
if (report.source.developmentCommit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" || report.source.servicePresent !== false) process.exit(1);
if (report.runtime.ddlBefore !== 4 || report.runtime.ddlAfter !== 0 || report.runtime.seedCallsBefore !== 2 || report.runtime.seedCallsAfter !== 0) process.exit(1);
if (Object.values(report.runtime.qualifiedRelations).reduce((sum, count) => sum + count, 0) !== 19) process.exit(1);
if (report.migration.migrations !== 1 || report.migration.historyStatus !== "blocked-preexisting-unsafe-history" || report.migration.runnerPrintSchemaMissing.join(",") !== "templates") process.exit(1);
if (report.schema.columns !== 26 || report.schema.notNull !== 14 || report.schema.primaryKeys !== 2 || report.schema.uniqueKeys !== 1) process.exit(1);
if (report.schema.foreignKeys !== 0 || report.schema.checks !== 0 || report.schema.indexes !== 5 || report.blockers.length !== 7) process.exit(1);
if (report.schema.keyConstraintPolicy.noInherit !== true || report.schema.keyConstraintPolicy.pg18Period !== false) process.exit(1);
' "$temp_dir/report-one.json"

assert_contract_tamper() {
  name=$1
  mutation=$2
  expected=$3
  output_contract="$temp_dir/$name.json"
  A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$output_contract" A3_MUTATION="$mutation" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
new Function("contract", process.env.A3_MUTATION)(contract);
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
  set +e
  "$verify" --mode integrity --contract "$output_contract" >"$temp_dir/$name.out" 2>&1
  status=$?
  set -e
  [ "$status" -eq 1 ] || {
    cat "$temp_dir/$name.out" >&2
    echo "a3-11 notification schema self-test: expected $name tamper exit 1, got $status" >&2
    exit 1
  }
  grep -q "$expected" "$temp_dir/$name.out"
}

assert_contract_tamper readiness-claim 'contract.productionReady = true' "readiness sentinel changed"
assert_contract_tamper query-digest 'contract.runtimeBoundary.compatibilityQuerySha256 = "0".repeat(64)' "compatibility query bytes changed"
assert_contract_tamper independent-query-pin 'contract.evidencePinning.compatibilityQuery.sha256 = "0".repeat(64)' "independent evidence pinning drifted"
assert_contract_tamper source-blob 'contract.sourceProvenance.development.evidence[0].blob = "0".repeat(40)' "pinned blob drifted"
assert_contract_tamper column-default 'contract.requiredTables[1].columns[8].sqlAnchor = "tampered status default"' "notifications columns drifted"
assert_contract_tamper primary-key 'contract.constraintSemantics.primaryKeys.pop()' "primary keys drifted"
assert_contract_tamper unique-key 'contract.constraintSemantics.uniqueKeys = []' "unique keys drifted"
assert_contract_tamper key-no-inherit 'contract.constraintSemantics.keyConstraintPolicy.noInherit = false' "key constraint policy drifted"
assert_contract_tamper index-opclass 'contract.constraintSemantics.indexes[3].opclasses[1] = "timestamp_ops"' "index inventory drifted"
assert_contract_tamper index-collation 'contract.constraintSemantics.indexes[0].collation = "database-default"' "index inventory drifted"
assert_contract_tamper foreign-key 'contract.constraintSemantics.foreignKeys.push("notifications.template_id->templates.id")' "foreign keys drifted"
assert_contract_tamper check-constraint 'contract.constraintSemantics.checkConstraints.push("notifications_status_check")' "check constraints drifted"
assert_contract_tamper inheritance 'contract.constraintSemantics.relationPolicy.inheritanceAccepted = true' "relation policy drifted"
assert_contract_tamper row-security 'contract.constraintSemantics.relationPolicy.rowLevelSecurity = true' "relation policy drifted"
assert_contract_tamper partial-index 'contract.constraintSemantics.indexPolicy.partialAccepted = true' "index policy drifted"
assert_contract_tamper sample-restoration 'contract.runtimeBoundary.startupSeedCallsAfter = 1' "startup seed/error boundary drifted"
assert_contract_tamper mutation-authority 'contract.safety.dataMutationAuthorized = true' "safety flags must remain false"
assert_contract_tamper blocker-claim 'contract.residualBlockers[0].status = "complete"' "B01: residual blocker drifted"
assert_contract_tamper traversal 'contract.migrationRoot.orderedMigrations[0].up.path = "../outside/up.sql"' "ordered migration path drifted"

git -C "$repo_root" worktree add --detach "$fixture" HEAD >/dev/null
fixture_registered=true
mkdir -p "$fixture/services/notification/src"
mkdir -p "$fixture/apps/backend/migrations/notifications/20260722040000_create_notification_service_tables"
mkdir -p "$fixture/docs/migration/contracts"
mkdir -p "$fixture/scripts/migration"
cp "$repo_root/services/notification/src/lib.rs" "$fixture/services/notification/src/lib.rs"
cp "$repo_root/services/notification/src/main.rs" "$fixture/services/notification/src/main.rs"
cp "$repo_root/apps/backend/migrations/notifications/20260722040000_create_notification_service_tables/up.sql" "$fixture/apps/backend/migrations/notifications/20260722040000_create_notification_service_tables/up.sql"
cp "$repo_root/apps/backend/migrations/notifications/20260722040000_create_notification_service_tables/down.sql" "$fixture/apps/backend/migrations/notifications/20260722040000_create_notification_service_tables/down.sql"
cp "$contract" "$fixture/docs/migration/contracts/a3-11-notification-schema-boundary.json"
cp "$verify" "$fixture/scripts/migration/verify-a3-11-notification-schema-boundary.sh"
chmod +x "$fixture/scripts/migration/verify-a3-11-notification-schema-boundary.sh"
fixture_verify="$fixture/scripts/migration/verify-a3-11-notification-schema-boundary.sh"
fixture_contract="$fixture/docs/migration/contracts/a3-11-notification-schema-boundary.json"
fixture_up="$fixture/apps/backend/migrations/notifications/20260722040000_create_notification_service_tables/up.sql"
fixture_down="$fixture/apps/backend/migrations/notifications/20260722040000_create_notification_service_tables/down.sql"
fixture_lib="$fixture/services/notification/src/lib.rs"
fixture_main="$fixture/services/notification/src/main.rs"

"$fixture_verify" --mode integrity >/dev/null

restore_fixture() {
  cp "$repo_root/services/notification/src/lib.rs" "$fixture_lib"
  cp "$repo_root/services/notification/src/main.rs" "$fixture_main"
  cp "$repo_root/apps/backend/migrations/notifications/20260722040000_create_notification_service_tables/up.sql" "$fixture_up"
  cp "$repo_root/apps/backend/migrations/notifications/20260722040000_create_notification_service_tables/down.sql" "$fixture_down"
  cp "$contract" "$fixture_contract"
}

assert_fixture_failure() {
  name=$1
  expected=$2
  set +e
  "$fixture_verify" --mode integrity >"$temp_dir/fixture-$name.out" 2>&1
  status=$?
  set -e
  [ "$status" -eq 1 ] || {
    cat "$temp_dir/fixture-$name.out" >&2
    echo "a3-11 notification schema self-test: expected fixture $name exit 1, got $status" >&2
    exit 1
  }
  grep -q "$expected" "$temp_dir/fixture-$name.out"
  restore_fixture
}

A3_FILE="$fixture_up" A3_CONTRACT="$fixture_contract" bun -e '
const file = process.env.A3_FILE;
const contractPath = process.env.A3_CONTRACT;
const sql = `${await Bun.file(file).text()}\nDELETE FROM public.notifications;\n`;
await Bun.write(file, sql);
const contract = await Bun.file(contractPath).json();
contract.migrationRoot.orderedMigrations[0].up.bytes = Buffer.byteLength(sql);
contract.migrationRoot.orderedMigrations[0].up.sha256 = new Bun.CryptoHasher("sha256").update(sql).digest("hex");
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
'
assert_fixture_failure mutation-token "up migration contains a forbidden mutation/control token"

A3_FILE="$fixture_down" A3_CONTRACT="$fixture_contract" bun -e '
const file = process.env.A3_FILE;
const contractPath = process.env.A3_CONTRACT;
const sql = `${await Bun.file(file).text()}\nDROP TABLE public.templates;\n`;
await Bun.write(file, sql);
const contract = await Bun.file(contractPath).json();
contract.migrationRoot.orderedMigrations[0].down.bytes = Buffer.byteLength(sql);
contract.migrationRoot.orderedMigrations[0].down.sha256 = new Bun.CryptoHasher("sha256").update(sql).digest("hex");
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
'
assert_fixture_failure destructive-down "down refusal contains destructive or data-mutating SQL"

A3_FILE="$fixture_down" A3_CONTRACT="$fixture_contract" bun -e '
const file = process.env.A3_FILE;
const contractPath = process.env.A3_CONTRACT;
const original = await Bun.file(file).text();
const sql = original.replace("\n$forward_only$;\n", "\n$forward_onlX$;\n");
if (sql === original || Buffer.byteLength(sql) !== Buffer.byteLength(original)) process.exit(1);
await Bun.write(file, sql);
const contract = await Bun.file(contractPath).json();
contract.migrationRoot.orderedMigrations[0].down.bytes = Buffer.byteLength(sql);
contract.migrationRoot.orderedMigrations[0].down.sha256 = new Bun.CryptoHasher("sha256").update(sql).digest("hex");
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
'
assert_fixture_failure same-length-down-delimiter "down migration exact body or dollar-quote delimiters drifted"

A3_FILE="$fixture_lib" A3_CONTRACT="$fixture_contract" bun -e '
const file = process.env.A3_FILE;
const contractPath = process.env.A3_CONTRACT;
let source = await Bun.file(file).text();
source = source.replace("AND constraint_record.connoinherit", "AND NOT constraint_record.connoinherit");
await Bun.write(file, source);
const match = source.match(/const NOTIFICATION_SCHEMA_COMPATIBILITY_QUERY: &str = r#"([\s\S]*?)"#;/);
if (!match) process.exit(1);
const contract = await Bun.file(contractPath).json();
contract.runtimeBoundary.compatibilityQueryBytes = Buffer.byteLength(match[1]);
contract.runtimeBoundary.compatibilityQuerySha256 = new Bun.CryptoHasher("sha256").update(match[1]).digest("hex");
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
'
assert_fixture_failure pg18-key-connoinherit "key constraints must require connoinherit=true for PostgreSQL 18 fresh-schema keys"

A3_FILE="$fixture_lib" A3_CONTRACT="$fixture_contract" bun -e '
const file = process.env.A3_FILE;
const contractPath = process.env.A3_CONTRACT;
let source = await Bun.file(file).text();
const original = source;
source = source.replace("pg_catalog.pg_namespace", "pg_catalog.pg_namespacX");
if (source === original || Buffer.byteLength(source) !== Buffer.byteLength(original)) process.exit(1);
await Bun.write(file, source);
const match = source.match(/const NOTIFICATION_SCHEMA_COMPATIBILITY_QUERY: &str = r#"([\s\S]*?)"#;/);
if (!match) process.exit(1);
const contract = await Bun.file(contractPath).json();
contract.runtimeBoundary.compatibilityQueryBytes = Buffer.byteLength(match[1]);
contract.runtimeBoundary.compatibilityQuerySha256 = new Bun.CryptoHasher("sha256").update(match[1]).digest("hex");
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
'
assert_fixture_failure same-length-query-catalog "compatibility query catalog identifier inventory drifted"

A3_FILE="$fixture_lib" A3_CONTRACT="$fixture_contract" bun -e '
const file = process.env.A3_FILE;
const contractPath = process.env.A3_CONTRACT;
let source = await Bun.file(file).text();
source = source.replace("\nSELECT\n    to_regclass", ",\nmutation_probe AS (DELETE FROM public.templates RETURNING *)\nSELECT\n    to_regclass");
await Bun.write(file, source);
const match = source.match(/const NOTIFICATION_SCHEMA_COMPATIBILITY_QUERY: &str = r#"([\s\S]*?)"#;/);
if (!match) process.exit(1);
const contract = await Bun.file(contractPath).json();
contract.runtimeBoundary.compatibilityQueryBytes = Buffer.byteLength(match[1]);
contract.runtimeBoundary.compatibilityQuerySha256 = new Bun.CryptoHasher("sha256").update(match[1]).digest("hex");
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
'
assert_fixture_failure query-mutation "compatibility query contains a mutation token"

A3_FILE="$fixture_main" bun -e '
const file = process.env.A3_FILE;
const source = await Bun.file(file).text();
await Bun.write(file, `${source}\nasync fn seed_sample_notifications() {}\n`);
'
assert_fixture_failure restored-sample-seed "runtime forbidden anchor remains: seed_sample_notifications"

assert_refused_env() {
  env_name=$1
  env_value=$2
  output=$3
  set +e
  env "$env_name=$env_value" "$verify" --mode integrity >"$output" 2>&1
  status=$?
  set -e
  [ "$status" -eq 1 ] || {
    cat "$output" >&2
    echo "a3-11 notification schema self-test: expected $env_name refusal exit 1, got $status" >&2
    exit 1
  }
  grep -q "$env_name\|production-looking" "$output"
}

assert_refused_env EPSX_ENV production "$temp_dir/production-env.out"
assert_refused_env NOTIFICATIONS_DATABASE_URL postgresql://local.invalid/db "$temp_dir/database-env.out"
assert_refused_env REDIS_URL redis://local.invalid/0 "$temp_dir/redis-env.out"
assert_refused_env SMTP_HOST smtp.invalid "$temp_dir/smtp-env.out"
assert_refused_env HTTPS_PROXY http://proxy.invalid "$temp_dir/network-env.out"

echo "a3-11 notification schema self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, readiness/source/query/column/default/key/index/opclass/collation/fk/check/inheritance/rls/sample/mutation/blocker/traversal tamper=1, actual up/down/query/sample/PG18-connoinherit mutations plus same-length query-catalog/down-delimiter corruptions=1, prod/db/redis/smtp/network env refusal=1)"
