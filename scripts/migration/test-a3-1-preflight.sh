#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_root="$(git -C "$script_dir" rev-parse --show-toplevel)"
repo_root="$(cd "$repo_root" && pwd -P)"
history_tool="$script_dir/a3-1-history-preflight.sh"
database_tool="$script_dir/a3-1-database-preflight.sh"
contract="$repo_root/docs/migration/contracts/a3-1-history-preflight.json"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-1-self-test.XXXXXX")"
trap 'rm -rf -- "$test_root"' EXIT

pass_count=0
pass() {
  pass_count=$((pass_count + 1))
  printf 'ok %d - %s\n' "$pass_count" "$1"
}

expect_status() {
  expected="$1"
  shift
  set +e
  "$@"
  actual="$?"
  set -e
  if [ "$actual" -ne "$expected" ]; then
    echo "expected status $expected, observed $actual: $*" >&2
    exit 1
  fi
}

# A fake psql is placed first in PATH for every static invocation. If the static
# classifier ever starts it, the marker makes the test fail.
mkdir "$test_root/static-bin"
cat > "$test_root/static-bin/psql" <<EOF
#!/usr/bin/env bash
touch "$test_root/STATIC_PSQL_WAS_CALLED"
exit 99
EOF
chmod 0755 "$test_root/static-bin/psql"

set +e
PATH="$test_root/static-bin:$PATH" "$history_tool" --repo "$repo_root" > "$test_root/root.json"
root_status="$?"
set -e
[ "$root_status" -eq 2 ] || { echo "current static status must be STOP (2), observed $root_status" >&2; exit 1; }
jq -e '.status == "stop" and .productionReady == false and any(.stopReasons[]; .code == "duplicate_normalized_version" and .domain == "core")' "$test_root/root.json" >/dev/null
[ ! -e "$test_root/STATIC_PSQL_WAS_CALLED" ] || { echo "static classifier attempted a database connection" >&2; exit 1; }
pass "repo-root static run fails closed on the current core collision without psql"

set +e
(
  cd /tmp
  PATH="$test_root/static-bin:$PATH" "$history_tool" --repo "$repo_root" > "$test_root/tmp.json"
)
tmp_status="$?"
set -e
[ "$tmp_status" -eq 2 ] || { echo "/tmp static status must be STOP (2), observed $tmp_status" >&2; exit 1; }
cmp "$test_root/root.json" "$test_root/tmp.json"
[ ! -e "$test_root/STATIC_PSQL_WAS_CALLED" ] || { echo "static classifier attempted a database connection from /tmp" >&2; exit 1; }
pass "repo-root and /tmp static reports are byte-for-byte deterministic"

jq '(.trustedSourceFiles[0].sha256) = "0000000000000000000000000000000000000000000000000000000000000000"' \
  "$contract" > "$test_root/drift-contract.json"
expect_status 2 "$history_tool" --repo "$repo_root" --contract "$test_root/drift-contract.json" --output "$test_root/drift.json"
jq -e 'any(.stopReasons[]; .code == "trusted_source_hash_mismatch")' "$test_root/drift.json" >/dev/null
pass "checksum-contract drift is a machine-readable stop"

inside_static_output="$repo_root/docs/migration/a3-1-unsafe-static-output-$PPID.json"
[ ! -e "$inside_static_output" ] || { echo "unexpected pre-existing test output: $inside_static_output" >&2; exit 1; }
expect_status 64 "$history_tool" --repo "$repo_root" --output "$inside_static_output" >/dev/null 2>&1
[ ! -e "$inside_static_output" ]
pass "static output refuses every repository path"

cat > "$test_root/mock-psql" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail

: "${A3_1_MOCK_DIR:?}"
: "${A3_1_MOCK_MODE:?}"
sql="$(cat)"

case " $* " in
  *' -X '*) ;;
  *) echo "missing -X" >&2; exit 91 ;;
esac
case " $* " in
  *' ON_ERROR_STOP=1 '*) ;;
  *) echo "missing ON_ERROR_STOP=1" >&2; exit 92 ;;
esac
case "${PGOPTIONS:-}" in
  *'default_transaction_read_only=on'*) ;;
  *) echo "missing default_transaction_read_only" >&2; exit 93 ;;
esac
printf '%s\n' "$sql" | grep -q '^BEGIN READ ONLY;$' || { echo "missing BEGIN READ ONLY" >&2; exit 94; }
printf '%s\n' "$sql" | grep -q '^ROLLBACK;$' || { echo "missing ROLLBACK" >&2; exit 95; }
if printf '%s\n' "$sql" | grep -Eiq '^[[:space:]]*(CREATE|ALTER|DROP|TRUNCATE|INSERT|UPDATE|DELETE|GRANT|REVOKE)([[:space:]]|$)'; then
  echo "mutation statement observed" >&2
  exit 96
fi

call_number=1
if [ -f "$A3_1_MOCK_DIR/calls" ]; then
  call_number=$(( $(wc -l < "$A3_1_MOCK_DIR/calls") + 1 ))
fi
printf 'call\n' >> "$A3_1_MOCK_DIR/calls"
printf '%s\n' "$sql" > "$A3_1_MOCK_DIR/$call_number.sql"

if printf '%s\n' "$sql" | grep -q 'current_database()'; then
  if [ "$A3_1_MOCK_MODE" = "multiple" ]; then
    printf '%s\n' '{"identity":{"database":"fixture","user":"reader","transactionReadOnly":"on"},"migrationTables":[{"schema":"public","name":"__diesel_schema_migrations"},{"schema":"shadow","name":"__diesel_schema_migrations"}]}'
  else
    printf '%s\n' '{"identity":{"database":"fixture","user":"reader","transactionReadOnly":"on"},"migrationTables":[{"schema":"public","name":"__diesel_schema_migrations"}]}'
  fi
else
  printf '%s\n' '{"migrationTable":{"schema":"public","name":"__diesel_schema_migrations"},"migrationHistory":[{"version":"00000000000001","run_on":"2026-01-01 00:00:00"}],"relations":[],"columns":[],"constraints":[],"indexes":[],"triggers":[],"functions":[],"partitions":[],"views":[]}'
fi
MOCK
chmod 0755 "$test_root/mock-psql"

run_database_tool() {
  mode="$1"
  output="$2"
  mock_dir="$3"
  mkdir "$mock_dir"
  A3_1_TEST_MODE=1 \
  A3_1_PSQL_BIN="$test_root/mock-psql" \
  A3_1_MOCK_MODE="$mode" \
  A3_1_MOCK_DIR="$mock_dir" \
  A3_1_CORE_DATABASE_URL='postgresql://fixture.invalid/core' \
  A3_1_ANALYTICS_DATABASE_URL='postgresql://fixture.invalid/analytics' \
  A3_1_NOTIFICATIONS_DATABASE_URL='postgresql://fixture.invalid/notifications' \
  A3_1_PAYMENTS_DATABASE_URL='postgresql://fixture.invalid/payments' \
    "$database_tool" --output-dir "$output"
}

run_database_tool single "$test_root/db-good" "$test_root/mock-good" > "$test_root/db-good.stdout"
[ "$(wc -l < "$test_root/mock-good/calls" | tr -d ' ')" -eq 8 ]
jq -e '.status == "inspection-captured-operator-classification-required" and .productionReady == false and (.artifacts | length) == 8' "$test_root/db-good/manifest.json" >/dev/null
! grep -Rqs 'postgresql://' "$test_root/db-good"
pass "mocked database capture uses two read-only inspections per explicit domain and stores no URL"

set +e
run_database_tool multiple "$test_root/db-multiple" "$test_root/mock-multiple" > "$test_root/db-multiple.stdout" 2> "$test_root/db-multiple.stderr"
multiple_status="$?"
set -e
[ "$multiple_status" -eq 2 ] || { echo "multiple-table status must be 2, observed $multiple_status" >&2; exit 1; }
[ "$(wc -l < "$test_root/mock-multiple/calls" | tr -d ' ')" -eq 4 ]
jq -e '.status == "stop" and (.stopReasons | length) == 4 and all(.stopReasons[]; .code == "ambiguous_migration_table")' "$test_root/db-multiple/manifest.json" >/dev/null
pass "multiple migration tables stop before any fingerprint inspection"

mkdir "$test_root/mock-unsafe-url"
set +e
A3_1_TEST_MODE=1 A3_1_PSQL_BIN="$test_root/mock-psql" A3_1_MOCK_MODE=single A3_1_MOCK_DIR="$test_root/mock-unsafe-url" \
A3_1_CORE_DATABASE_URL='https://fixture.invalid/core' \
A3_1_ANALYTICS_DATABASE_URL='postgresql://fixture.invalid/analytics' \
A3_1_NOTIFICATIONS_DATABASE_URL='postgresql://fixture.invalid/notifications' \
A3_1_PAYMENTS_DATABASE_URL='postgresql://fixture.invalid/payments' \
  "$database_tool" --output-dir "$test_root/db-unsafe-url" >/dev/null 2>&1
unsafe_url_status="$?"
set -e
[ "$unsafe_url_status" -eq 64 ]
[ ! -e "$test_root/mock-unsafe-url/calls" ]
[ ! -e "$test_root/db-unsafe-url" ]
pass "unsafe URL schemes fail before psql or artifact creation"

mkdir "$test_root/mock-control-url"
set +e
A3_1_TEST_MODE=1 A3_1_PSQL_BIN="$test_root/mock-psql" A3_1_MOCK_MODE=single A3_1_MOCK_DIR="$test_root/mock-control-url" \
A3_1_CORE_DATABASE_URL="$(printf 'postgresql://fixture.invalid/core\nextra')" \
A3_1_ANALYTICS_DATABASE_URL='postgresql://fixture.invalid/analytics' \
A3_1_NOTIFICATIONS_DATABASE_URL='postgresql://fixture.invalid/notifications' \
A3_1_PAYMENTS_DATABASE_URL='postgresql://fixture.invalid/payments' \
  "$database_tool" --output-dir "$test_root/db-control-url" >/dev/null 2>&1
control_url_status="$?"
set -e
[ "$control_url_status" -eq 64 ]
[ ! -e "$test_root/mock-control-url/calls" ]
pass "literal whitespace/control data in a URL fails before psql"

inside_db_output="$repo_root/docs/migration/a3-1-unsafe-db-output-$PPID"
[ ! -e "$inside_db_output" ] || { echo "unexpected pre-existing test output: $inside_db_output" >&2; exit 1; }
mkdir "$test_root/mock-inside-output"
set +e
A3_1_TEST_MODE=1 A3_1_PSQL_BIN="$test_root/mock-psql" A3_1_MOCK_MODE=single A3_1_MOCK_DIR="$test_root/mock-inside-output" \
A3_1_CORE_DATABASE_URL='postgresql://fixture.invalid/core' \
A3_1_ANALYTICS_DATABASE_URL='postgresql://fixture.invalid/analytics' \
A3_1_NOTIFICATIONS_DATABASE_URL='postgresql://fixture.invalid/notifications' \
A3_1_PAYMENTS_DATABASE_URL='postgresql://fixture.invalid/payments' \
  "$database_tool" --output-dir "$inside_db_output" >/dev/null 2>&1
inside_output_status="$?"
set -e
[ "$inside_output_status" -eq 64 ]
[ ! -e "$inside_db_output" ]
[ ! -e "$test_root/mock-inside-output/calls" ]
pass "database artifacts refuse every repository path before psql"

mkdir "$test_root/existing-output"
mkdir "$test_root/mock-existing-output"
set +e
A3_1_TEST_MODE=1 A3_1_PSQL_BIN="$test_root/mock-psql" A3_1_MOCK_MODE=single A3_1_MOCK_DIR="$test_root/mock-existing-output" \
A3_1_CORE_DATABASE_URL='postgresql://fixture.invalid/core' \
A3_1_ANALYTICS_DATABASE_URL='postgresql://fixture.invalid/analytics' \
A3_1_NOTIFICATIONS_DATABASE_URL='postgresql://fixture.invalid/notifications' \
A3_1_PAYMENTS_DATABASE_URL='postgresql://fixture.invalid/payments' \
  "$database_tool" --output-dir "$test_root/existing-output" >/dev/null 2>&1
existing_output_status="$?"
set -e
[ "$existing_output_status" -eq 64 ]
[ ! -e "$test_root/mock-existing-output/calls" ]
pass "database artifacts never overwrite an existing caller path"

mkdir "$test_root/mock-repo-tmpdir"
set +e
TMPDIR="$repo_root/docs/migration" \
A3_1_TEST_MODE=1 A3_1_PSQL_BIN="$test_root/mock-psql" A3_1_MOCK_MODE=single A3_1_MOCK_DIR="$test_root/mock-repo-tmpdir" \
A3_1_CORE_DATABASE_URL='postgresql://fixture.invalid/core' \
A3_1_ANALYTICS_DATABASE_URL='postgresql://fixture.invalid/analytics' \
A3_1_NOTIFICATIONS_DATABASE_URL='postgresql://fixture.invalid/notifications' \
A3_1_PAYMENTS_DATABASE_URL='postgresql://fixture.invalid/payments' \
  "$database_tool" >/dev/null 2>&1
repo_tmpdir_status="$?"
set -e
[ "$repo_tmpdir_status" -eq 64 ]
[ ! -e "$test_root/mock-repo-tmpdir/calls" ]
pass "default artifacts reject a TMPDIR inside the repository before psql"

printf '1..%d\n' "$pass_count"
