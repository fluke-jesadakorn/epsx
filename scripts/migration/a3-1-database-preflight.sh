#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: a3-1-database-preflight.sh [--output-dir ABSOLUTE_NEW_DIRECTORY]

Required environment variables:
  A3_1_CORE_DATABASE_URL
  A3_1_ANALYTICS_DATABASE_URL
  A3_1_NOTIFICATIONS_DATABASE_URL
  A3_1_PAYMENTS_DATABASE_URL

The script performs catalog SELECTs only. Every psql invocation uses -X,
ON_ERROR_STOP=1, default_transaction_read_only=on, BEGIN READ ONLY, and ROLLBACK.
The default artifact directory is a new mode-0700 directory below TMPDIR.

Exit status: 0 = inspection captured, 2 = ambiguous migration history, 64 = unsafe invocation.
No result from this tool is a production-readiness approval.
EOF
}

fail_usage() {
  echo "a3-1-database-preflight: ERROR: $1" >&2
  usage >&2
  exit 64
}

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_root="$(git -C "$script_dir" rev-parse --show-toplevel)"
repo_root="$(cd "$repo_root" && pwd -P)"
requested_output=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --help)
      usage
      exit 0
      ;;
    --output-dir)
      [ "$#" -ge 2 ] || fail_usage "--output-dir requires a value"
      requested_output="$2"
      shift 2
      ;;
    *)
      fail_usage "unknown option: $1"
      ;;
  esac
done

command -v jq >/dev/null 2>&1 || fail_usage "jq is required"
command -v shasum >/dev/null 2>&1 || fail_usage "shasum is required"

if [ -n "${A3_1_PSQL_BIN:-}" ]; then
  [ "${A3_1_TEST_MODE:-}" = "1" ] || fail_usage "A3_1_PSQL_BIN is permitted only with A3_1_TEST_MODE=1"
  [ -x "$A3_1_PSQL_BIN" ] || fail_usage "A3_1_PSQL_BIN must be an executable path"
  psql_bin="$A3_1_PSQL_BIN"
else
  psql_bin="$(command -v psql || true)"
  [ -n "$psql_bin" ] || fail_usage "psql is required"
fi

validate_url() {
  variable_name="$1"
  case "$variable_name" in
    A3_1_CORE_DATABASE_URL) database_url="${A3_1_CORE_DATABASE_URL:-}" ;;
    A3_1_ANALYTICS_DATABASE_URL) database_url="${A3_1_ANALYTICS_DATABASE_URL:-}" ;;
    A3_1_NOTIFICATIONS_DATABASE_URL) database_url="${A3_1_NOTIFICATIONS_DATABASE_URL:-}" ;;
    A3_1_PAYMENTS_DATABASE_URL) database_url="${A3_1_PAYMENTS_DATABASE_URL:-}" ;;
    *) fail_usage "unknown database URL variable" ;;
  esac
  [ -n "$database_url" ] || fail_usage "$variable_name is required"
  case "$database_url" in
    postgres://*|postgresql://*) ;;
    *) fail_usage "$variable_name must use postgres:// or postgresql://" ;;
  esac
  sanitized_url="$(printf '%s' "$database_url" | LC_ALL=C tr -d '[:space:]#')"
  if [ "$sanitized_url" != "$database_url" ]; then
    fail_usage "$variable_name contains whitespace, control data, or a fragment"
  fi
}

validate_url A3_1_CORE_DATABASE_URL
validate_url A3_1_ANALYTICS_DATABASE_URL
validate_url A3_1_NOTIFICATIONS_DATABASE_URL
validate_url A3_1_PAYMENTS_DATABASE_URL

umask 077
if [ -z "$requested_output" ]; then
  temp_parent="${TMPDIR:-/tmp}"
  [ -d "$temp_parent" ] || fail_usage "TMPDIR must name an existing directory"
  real_temp_parent="$(cd "$temp_parent" && pwd -P)"
  case "$real_temp_parent" in
    "$repo_root"|"$repo_root/"*) fail_usage "TMPDIR must be outside the repository" ;;
  esac
  artifact_root="$(mktemp -d "$real_temp_parent/epsx-a3-1-db-preflight.XXXXXX")"
else
  case "$requested_output" in
    /*) ;;
    *) fail_usage "--output-dir must be an absolute path" ;;
  esac
  [ "$requested_output" != "/" ] || fail_usage "--output-dir may not be filesystem root"
  [ ! -e "$requested_output" ] && [ ! -L "$requested_output" ] || fail_usage "--output-dir refuses an existing path"
  requested_parent="$(dirname -- "$requested_output")"
  [ -d "$requested_parent" ] || fail_usage "--output-dir parent must exist"
  real_parent="$(cd "$requested_parent" && pwd -P)"
  artifact_root="$real_parent/$(basename -- "$requested_output")"
  case "$artifact_root" in
    "$repo_root"|"$repo_root/"*)
      fail_usage "--output-dir must be outside the repository"
      ;;
  esac
  [ "$artifact_root" != "${HOME:-}" ] || fail_usage "--output-dir may not be HOME"
  mkdir -m 700 "$artifact_root"
fi

stop_lines="$artifact_root/.stop-reasons.jsonl"
: > "$stop_lines"

url_for_domain() {
  case "$1" in
    core) printf '%s' "$A3_1_CORE_DATABASE_URL" ;;
    analytics) printf '%s' "$A3_1_ANALYTICS_DATABASE_URL" ;;
    notifications) printf '%s' "$A3_1_NOTIFICATIONS_DATABASE_URL" ;;
    payments) printf '%s' "$A3_1_PAYMENTS_DATABASE_URL" ;;
    *) return 64 ;;
  esac
}

run_psql() {
  database_url="$1"
  sql="$2"
  PGOPTIONS="-c default_transaction_read_only=on -c statement_timeout=30000 -c lock_timeout=3000" \
    "$psql_bin" -X -v ON_ERROR_STOP=1 -qAt --dbname "$database_url" <<<"$sql"
}

discovery_sql=$(cat <<'SQL'
BEGIN READ ONLY;
SELECT jsonb_build_object(
  'identity', jsonb_build_object(
    'database', current_database(),
    'user', current_user,
    'serverAddress', inet_server_addr(),
    'serverPort', inet_server_port(),
    'serverVersion', current_setting('server_version'),
    'searchPath', current_setting('search_path'),
    'transactionReadOnly', current_setting('transaction_read_only')
  ),
  'migrationTables', COALESCE((
    SELECT jsonb_agg(
      jsonb_build_object('schema', n.nspname, 'name', c.relname)
      ORDER BY n.nspname, c.relname
    )
    FROM pg_catalog.pg_class c
    JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
    WHERE c.relname = '__diesel_schema_migrations'
      AND c.relkind = 'r'
  ), '[]'::jsonb)
)::text;
ROLLBACK;
SQL
)

for domain in core analytics notifications payments; do
  database_url="$(url_for_domain "$domain")"
  discovery_raw="$(run_psql "$database_url" "$discovery_sql")" || {
    jq -cn --arg domain "$domain" --arg code "connection_or_discovery_failed" \
      '{domain:$domain,code:$code,detail:"psql discovery failed; no inspection was attempted"}' >> "$stop_lines"
    continue
  }
  if ! printf '%s\n' "$discovery_raw" | jq -e 'type == "object" and (.identity | type == "object") and (.migrationTables | type == "array")' >/dev/null; then
    jq -cn --arg domain "$domain" --arg code "invalid_discovery_output" \
      '{domain:$domain,code:$code,detail:"discovery did not return the expected single JSON object"}' >> "$stop_lines"
    continue
  fi
  printf '%s\n' "$discovery_raw" | jq -S . > "$artifact_root/$domain.discovery.json"
  table_count="$(jq '.migrationTables | length' "$artifact_root/$domain.discovery.json")"
  if [ "$table_count" -ne 1 ]; then
    jq -cn --arg domain "$domain" --arg code "ambiguous_migration_table" --argjson count "$table_count" \
      '{domain:$domain,code:$code,detail:("expected exactly one __diesel_schema_migrations table; observed " + ($count|tostring))}' >> "$stop_lines"
    continue
  fi
  migration_schema="$(jq -r '.migrationTables[0].schema' "$artifact_root/$domain.discovery.json")"
  case "$migration_schema" in
    ''|*[!A-Za-z0-9_'$']*|[0-9]*)
      jq -cn --arg domain "$domain" --arg code "unsafe_migration_schema_identifier" \
        '{domain:$domain,code:$code,detail:"migration table schema is not a safely quotable PostgreSQL identifier"}' >> "$stop_lines"
      ;;
  esac
done

if [ -s "$stop_lines" ]; then
  jq -s . "$stop_lines" > "$artifact_root/.stops.json"
  discovery_files=()
  for discovery_file in "$artifact_root"/*.discovery.json; do
    [ -f "$discovery_file" ] && discovery_files+=("$discovery_file")
  done
  if [ "${#discovery_files[@]}" -gt 0 ]; then
    jq -s '.' "${discovery_files[@]}" > "$artifact_root/.discoveries.json"
  else
    printf '[]\n' > "$artifact_root/.discoveries.json"
  fi
  jq -n --slurpfile stops "$artifact_root/.stops.json" --slurpfile discoveries "$artifact_root/.discoveries.json" \
    '{schemaVersion:1,package:"A3.1a",purpose:"read-only-database-preflight",productionReady:false,status:"stop",stopReasons:$stops[0],discoveries:$discoveries[0]}' \
    > "$artifact_root/manifest.json"
  rm -f "$stop_lines" "$artifact_root/.stops.json" "$artifact_root/.discoveries.json"
  echo "a3-1-database-preflight: STOP — inspect $artifact_root/manifest.json" >&2
  exit 2
fi

for domain in core analytics notifications payments; do
  database_url="$(url_for_domain "$domain")"
  migration_schema="$(jq -r '.migrationTables[0].schema' "$artifact_root/$domain.discovery.json")"
  inspection_sql=$(cat <<SQL
BEGIN READ ONLY;
WITH
migration_history AS (
  SELECT version::text, run_on::text
  FROM "$migration_schema".__diesel_schema_migrations
  ORDER BY version
),
relations AS (
  SELECT n.nspname AS schema_name, c.relname AS relation_name, c.relkind::text AS relation_kind,
         COALESCE(s.n_live_tup, 0)::bigint AS estimated_rows
  FROM pg_catalog.pg_class c
  JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
  LEFT JOIN pg_catalog.pg_stat_user_tables s ON s.relid = c.oid
  WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
    AND c.relkind IN ('r', 'p', 'v', 'm', 'S')
  ORDER BY n.nspname, c.relname, c.relkind
),
columns AS (
  SELECT n.nspname AS schema_name, c.relname AS relation_name, a.attnum AS ordinal,
         a.attname AS column_name, pg_catalog.format_type(a.atttypid, a.atttypmod) AS data_type,
         a.attnotnull AS not_null, pg_catalog.pg_get_expr(d.adbin, d.adrelid) AS default_expression
  FROM pg_catalog.pg_attribute a
  JOIN pg_catalog.pg_class c ON c.oid = a.attrelid
  JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
  LEFT JOIN pg_catalog.pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum
  WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
    AND c.relkind IN ('r', 'p', 'v', 'm') AND a.attnum > 0 AND NOT a.attisdropped
  ORDER BY n.nspname, c.relname, a.attnum
),
constraints AS (
  SELECT n.nspname AS schema_name, c.relname AS relation_name, con.conname AS constraint_name,
         con.contype::text AS constraint_type, con.convalidated AS validated,
         pg_catalog.pg_get_constraintdef(con.oid, true) AS definition
  FROM pg_catalog.pg_constraint con
  JOIN pg_catalog.pg_class c ON c.oid = con.conrelid
  JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
  WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
  ORDER BY n.nspname, c.relname, con.conname
),
indexes AS (
  SELECT ns.nspname AS schema_name, tbl.relname AS relation_name, idx.relname AS index_name,
         pg_catalog.pg_get_indexdef(i.indexrelid) AS definition
  FROM pg_catalog.pg_index i
  JOIN pg_catalog.pg_class idx ON idx.oid = i.indexrelid
  JOIN pg_catalog.pg_class tbl ON tbl.oid = i.indrelid
  JOIN pg_catalog.pg_namespace ns ON ns.oid = tbl.relnamespace
  WHERE ns.nspname !~ '^pg_' AND ns.nspname <> 'information_schema'
  ORDER BY ns.nspname, tbl.relname, idx.relname
),
triggers AS (
  SELECT n.nspname AS schema_name, c.relname AS relation_name, t.tgname AS trigger_name,
         pg_catalog.pg_get_triggerdef(t.oid, true) AS definition
  FROM pg_catalog.pg_trigger t
  JOIN pg_catalog.pg_class c ON c.oid = t.tgrelid
  JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
  WHERE NOT t.tgisinternal AND n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
  ORDER BY n.nspname, c.relname, t.tgname
),
functions AS (
  SELECT n.nspname AS schema_name, p.proname AS function_name,
         pg_catalog.pg_get_function_identity_arguments(p.oid) AS identity_arguments,
         pg_catalog.pg_get_functiondef(p.oid) AS definition
  FROM pg_catalog.pg_proc p
  JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
  WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema' AND p.prokind = 'f'
  ORDER BY n.nspname, p.proname, pg_catalog.pg_get_function_identity_arguments(p.oid)
),
partitions AS (
  SELECT pn.nspname AS parent_schema, p.relname AS parent_name,
         cn.nspname AS child_schema, c.relname AS child_name,
         pg_catalog.pg_get_expr(c.relpartbound, c.oid) AS partition_bound
  FROM pg_catalog.pg_inherits i
  JOIN pg_catalog.pg_class p ON p.oid = i.inhparent
  JOIN pg_catalog.pg_namespace pn ON pn.oid = p.relnamespace
  JOIN pg_catalog.pg_class c ON c.oid = i.inhrelid
  JOIN pg_catalog.pg_namespace cn ON cn.oid = c.relnamespace
  WHERE pn.nspname !~ '^pg_' AND pn.nspname <> 'information_schema'
  ORDER BY pn.nspname, p.relname, cn.nspname, c.relname
),
views AS (
  SELECT n.nspname AS schema_name, c.relname AS view_name, c.relkind::text AS view_kind,
         pg_catalog.pg_get_viewdef(c.oid, true) AS definition
  FROM pg_catalog.pg_class c
  JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
  WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema' AND c.relkind IN ('v', 'm')
  ORDER BY n.nspname, c.relname
)
SELECT jsonb_build_object(
  'migrationTable', jsonb_build_object('schema', '$migration_schema', 'name', '__diesel_schema_migrations'),
  'migrationHistory', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM migration_history x), '[]'::jsonb),
  'relations', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM relations x), '[]'::jsonb),
  'columns', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM columns x), '[]'::jsonb),
  'constraints', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM constraints x), '[]'::jsonb),
  'indexes', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM indexes x), '[]'::jsonb),
  'triggers', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM triggers x), '[]'::jsonb),
  'functions', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM functions x), '[]'::jsonb),
  'partitions', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM partitions x), '[]'::jsonb),
  'views', COALESCE((SELECT jsonb_agg(to_jsonb(x)) FROM views x), '[]'::jsonb)
)::text;
ROLLBACK;
SQL
)
  inspection_raw="$(run_psql "$database_url" "$inspection_sql")" || {
    jq -cn --arg domain "$domain" --arg code "inspection_failed" \
      '{domain:$domain,code:$code,detail:"read-only catalog inspection failed"}' >> "$stop_lines"
    continue
  }
  if ! printf '%s\n' "$inspection_raw" | jq -e 'type == "object" and (.migrationHistory | type == "array") and (.relations | type == "array")' >/dev/null; then
    jq -cn --arg domain "$domain" --arg code "invalid_inspection_output" \
      '{domain:$domain,code:$code,detail:"inspection did not return the expected single JSON object"}' >> "$stop_lines"
    continue
  fi
  printf '%s\n' "$inspection_raw" | jq -S . > "$artifact_root/$domain.inspection.json"
done

if [ -s "$stop_lines" ]; then
  jq -s . "$stop_lines" > "$artifact_root/.stops.json"
  jq -n --slurpfile stops "$artifact_root/.stops.json" \
    '{schemaVersion:1,package:"A3.1a",purpose:"read-only-database-preflight",productionReady:false,status:"stop",stopReasons:$stops[0]}' \
    > "$artifact_root/manifest.json"
  rm -f "$stop_lines" "$artifact_root/.stops.json"
  echo "a3-1-database-preflight: STOP — inspect $artifact_root/manifest.json" >&2
  exit 2
fi

digest_lines="$artifact_root/.digests.jsonl"
: > "$digest_lines"
for artifact in "$artifact_root"/*.discovery.json "$artifact_root"/*.inspection.json; do
  digest="$(shasum -a 256 "$artifact" | awk '{print $1}')"
  jq -cn --arg file "$(basename -- "$artifact")" --arg sha256 "$digest" '{file:$file,sha256:$sha256}' >> "$digest_lines"
done
jq -s . "$digest_lines" > "$artifact_root/.digests.json"
jq -n --slurpfile artifacts "$artifact_root/.digests.json" \
  '{schemaVersion:1,package:"A3.1a",purpose:"read-only-database-preflight",productionReady:false,status:"inspection-captured-operator-classification-required",artifacts:$artifacts[0],stopReasons:[]}' \
  > "$artifact_root/manifest.json"
rm -f "$stop_lines" "$digest_lines" "$artifact_root/.digests.json"

echo "a3-1-database-preflight: read-only artifacts written to $artifact_root"
echo "a3-1-database-preflight: NOT production-ready; operator classification is still required"
