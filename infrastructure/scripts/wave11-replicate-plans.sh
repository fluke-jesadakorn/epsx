#!/usr/bin/env bash
# Reconcile the canonical core public.plans table into payments.plans.
#
# This tool supports both the legacy single-database topology and the current
# split core/payments databases. It never deletes destination-only rows: such
# drift stops the transaction for operator review. Dry-run is the default.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  CORE_DATABASE_URL=postgresql://... \
  PAYMENTS_DATABASE_URL=postgresql://... \
    ./infrastructure/scripts/wave11-replicate-plans.sh --dry-run

  CORE_DATABASE_URL=postgresql://... \
  PAYMENTS_DATABASE_URL=postgresql://... \
    ./infrastructure/scripts/wave11-replicate-plans.sh \
      --apply --environment development|staging

Production requires all of:
  --apply --environment production --confirm-production PLAN-PROJECTION

The source is read through a repeatable-read, read-only snapshot. Apply uses
one destination transaction, refuses destination-only rows, upserts every
source column, and commits only when the staged and destination rows match
exactly. At most 10,000 plan rows are accepted. URLs and row contents are
never printed.
EOF
}

fail() {
  echo "plan-projection: ERROR: $1" >&2
  exit "${2:-1}"
}

mode="dry-run"
environment=""
production_confirmation=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      mode="dry-run"
      shift
      ;;
    --apply)
      mode="apply"
      shift
      ;;
    --environment)
      [[ $# -ge 2 ]] || fail "--environment requires a value" 64
      environment="$2"
      shift 2
      ;;
    --confirm-production)
      [[ $# -ge 2 ]] || fail "--confirm-production requires a value" 64
      production_confirmation="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown option: $1" 64
      ;;
  esac
done

command -v psql >/dev/null 2>&1 || fail "psql is required" 64
if command -v shasum >/dev/null 2>&1; then
  hash_file() { shasum -a 256 "$1" | awk '{print $1}'; }
elif command -v sha256sum >/dev/null 2>&1; then
  hash_file() { sha256sum "$1" | awk '{print $1}'; }
else
  fail "shasum or sha256sum is required" 64
fi

source_url="${CORE_DATABASE_URL:-${DATABASE_URL:-}}"
target_url="${PAYMENTS_DATABASE_URL:-}"
[[ -n "$source_url" ]] || fail "CORE_DATABASE_URL is required" 64
[[ -n "$target_url" ]] || fail "PAYMENTS_DATABASE_URL is required" 64

validate_url() {
  local label="$1"
  local value="$2"
  case "$value" in
    postgres://*|postgresql://*) ;;
    *) fail "$label must use postgres:// or postgresql://" 64 ;;
  esac
  local sanitized
  sanitized="$(printf '%s' "$value" | LC_ALL=C tr -d '[:space:]#')"
  [[ "$sanitized" == "$value" ]] || fail "$label contains whitespace, control data, or a fragment" 64
}
validate_url CORE_DATABASE_URL "$source_url"
validate_url PAYMENTS_DATABASE_URL "$target_url"

if [[ "$mode" == "apply" ]]; then
  case "$environment" in
    development|staging) ;;
    production)
      [[ "$production_confirmation" == "PLAN-PROJECTION" ]] || \
        fail "production apply requires --confirm-production PLAN-PROJECTION" 64
      ;;
    *) fail "--apply requires --environment development, staging, or production" 64 ;;
  esac
elif [[ -n "$environment" || -n "$production_confirmation" ]]; then
  fail "environment/production confirmation are accepted only with --apply" 64
fi

work_root="$(mktemp -d "/tmp/epsx-plan-projection.XXXXXX")"
chmod 700 "$work_root"
snapshot_file="$work_root/plans.snapshot"
target_file="$work_root/plans.target"
trap 'rm -rf -- "$work_root"' EXIT

psql_read() {
  local url="$1"
  local sql="$2"
  PGOPTIONS='-c default_transaction_read_only=on -c statement_timeout=30000 -c lock_timeout=3000' \
    psql -X -v ON_ERROR_STOP=1 -qAt --dbname "$url" -c "$sql"
}

source_relation="$(psql_read "$source_url" "SELECT to_regclass('public.plans')::text")"
target_relation="$(psql_read "$target_url" "SELECT to_regclass('payments.plans')::text")"
[[ "$source_relation" == "plans" || "$source_relation" == "public.plans" ]] || \
  fail "source public.plans is missing"
[[ "$target_relation" == "payments.plans" ]] || fail "target payments.plans is missing"

schema_sql() {
  local schema="$1"
  cat <<SQL
SELECT string_agg(
  ordinal_position::text || ':' || column_name || ':' || data_type || ':' ||
  udt_name || ':' || is_nullable,
  ',' ORDER BY ordinal_position
)
FROM information_schema.columns
WHERE table_schema = '$schema' AND table_name = 'plans'
SQL
}

source_schema="$(psql_read "$source_url" "$(schema_sql public)")"
target_schema="$(psql_read "$target_url" "$(schema_sql payments)")"
[[ -n "$source_schema" && "$source_schema" == "$target_schema" ]] || \
  fail "source and target plan schemas differ; refusing projection"

plan_json_sql() {
  local relation="$1"
  cat <<SQL
SELECT jsonb_build_object(
  'id', id,
  'name', name,
  'slug', slug,
  'description', description,
  'plan_type', plan_type,
  'plan_category', plan_category,
  'plan_group', plan_group,
  'plan_metadata', plan_metadata,
  'price', price,
  'currency', currency,
  'billing_cycle', billing_cycle,
  'is_active', is_active,
  'is_promoted', is_promoted,
  'is_public', is_public,
  'is_system', is_system,
  'tier_level', tier_level,
  'max_members', max_members,
  'auto_assign_enabled', auto_assign_enabled,
  'assignment_rules', assignment_rules,
  'grace_period_hours', grace_period_hours,
  'rate_limit_per_minute', rate_limit_per_minute,
  'rate_limit_per_hour', rate_limit_per_hour,
  'rate_limit_per_day', rate_limit_per_day,
  'burst_capacity', burst_capacity,
  'created_at', created_at,
  'updated_at', updated_at,
  'created_by', created_by,
  'last_modified_by', last_modified_by
)::text
FROM $relation
ORDER BY id
SQL
}

source_export_sql="COPY ($(plan_json_sql public.plans)) TO STDOUT"
target_export_sql="COPY ($(plan_json_sql payments.plans)) TO STDOUT"

PGOPTIONS='-c default_transaction_read_only=on -c statement_timeout=30000 -c lock_timeout=3000' \
  psql -X -v ON_ERROR_STOP=1 -qAt --dbname "$source_url" <<SQL > "$snapshot_file"
BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY;
$source_export_sql;
COMMIT;
SQL
chmod 600 "$snapshot_file"

source_count="$(wc -l < "$snapshot_file" | tr -d ' ')"
[[ "$source_count" =~ ^[0-9]+$ ]] || fail "source row count is invalid"
(( source_count <= 10000 )) || fail "source exceeds the 10,000-row safety bound"
source_digest="$(hash_file "$snapshot_file")"

PGOPTIONS='-c default_transaction_read_only=on -c statement_timeout=30000 -c lock_timeout=3000' \
  psql -X -v ON_ERROR_STOP=1 -qAt --dbname "$target_url" -c "$target_export_sql" > "$target_file"
chmod 600 "$target_file"
target_count="$(wc -l < "$target_file" | tr -d ' ')"
target_digest="$(hash_file "$target_file")"

echo "plan-projection: mode=$mode source_rows=$source_count target_rows=$target_count"
echo "plan-projection: source_sha256=$source_digest target_sha256=$target_digest"

if [[ "$mode" == "dry-run" ]]; then
  if [[ "$source_count" == "$target_count" && "$source_digest" == "$target_digest" ]]; then
    echo "plan-projection: PASS — source and target are already exact; writes=0"
    exit 0
  fi
  echo "plan-projection: DRIFT — run an approved --apply for development/staging; writes=0" >&2
  exit 2
fi

# The temporary stage is populated from a bounded, read-only source snapshot.
# Destination-only IDs are never removed; they abort before the upsert. The
# final full-row comparison runs inside the same transaction as the write.
PGOPTIONS='-c statement_timeout=60000 -c lock_timeout=5000' \
  psql -X -v ON_ERROR_STOP=1 -q --dbname "$target_url" <<SQL
BEGIN;
LOCK TABLE payments.plans IN SHARE ROW EXCLUSIVE MODE;
CREATE TEMP TABLE plan_projection_stage (raw jsonb NOT NULL) ON COMMIT DROP;
\copy plan_projection_stage(raw) FROM '$snapshot_file' WITH (FORMAT text)

DO \$\$
DECLARE
  staged_count bigint;
  destination_only bigint;
BEGIN
  SELECT COUNT(*) INTO staged_count FROM plan_projection_stage;
  IF staged_count > 10000 THEN
    RAISE EXCEPTION 'plan projection exceeds bounded row count';
  END IF;
  IF EXISTS (
    SELECT 1
    FROM plan_projection_stage
    GROUP BY raw->>'id'
    HAVING COUNT(*) > 1 OR raw->>'id' IS NULL
  ) THEN
    RAISE EXCEPTION 'plan projection contains duplicate or missing IDs';
  END IF;
  SELECT COUNT(*) INTO destination_only
  FROM payments.plans target
  WHERE NOT EXISTS (
    SELECT 1 FROM plan_projection_stage source
    WHERE (source.raw->>'id')::uuid = target.id
  );
  IF destination_only <> 0 THEN
    RAISE EXCEPTION 'destination has % rows absent from source; refusing deletion', destination_only;
  END IF;
END
\$\$;

INSERT INTO payments.plans
SELECT (jsonb_populate_record(NULL::payments.plans, raw)).*
FROM plan_projection_stage
ON CONFLICT (id) DO UPDATE SET
  name = EXCLUDED.name,
  slug = EXCLUDED.slug,
  description = EXCLUDED.description,
  plan_type = EXCLUDED.plan_type,
  plan_category = EXCLUDED.plan_category,
  plan_group = EXCLUDED.plan_group,
  plan_metadata = EXCLUDED.plan_metadata,
  price = EXCLUDED.price,
  currency = EXCLUDED.currency,
  billing_cycle = EXCLUDED.billing_cycle,
  is_active = EXCLUDED.is_active,
  is_promoted = EXCLUDED.is_promoted,
  is_public = EXCLUDED.is_public,
  is_system = EXCLUDED.is_system,
  tier_level = EXCLUDED.tier_level,
  max_members = EXCLUDED.max_members,
  auto_assign_enabled = EXCLUDED.auto_assign_enabled,
  assignment_rules = EXCLUDED.assignment_rules,
  grace_period_hours = EXCLUDED.grace_period_hours,
  rate_limit_per_minute = EXCLUDED.rate_limit_per_minute,
  rate_limit_per_hour = EXCLUDED.rate_limit_per_hour,
  rate_limit_per_day = EXCLUDED.rate_limit_per_day,
  burst_capacity = EXCLUDED.burst_capacity,
  created_at = EXCLUDED.created_at,
  updated_at = EXCLUDED.updated_at,
  created_by = EXCLUDED.created_by,
  last_modified_by = EXCLUDED.last_modified_by;

DO \$\$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM payments.plans target
    FULL JOIN plan_projection_stage source
      ON (source.raw->>'id')::uuid = target.id
    WHERE target.id IS NULL
       OR source.raw IS NULL
       OR to_jsonb(target) IS DISTINCT FROM source.raw
  ) THEN
    RAISE EXCEPTION 'post-upsert plan projection reconciliation failed';
  END IF;
END
\$\$;
COMMIT;
SQL

PGOPTIONS='-c default_transaction_read_only=on -c statement_timeout=30000 -c lock_timeout=3000' \
  psql -X -v ON_ERROR_STOP=1 -qAt --dbname "$target_url" -c "$target_export_sql" > "$target_file"
target_count="$(wc -l < "$target_file" | tr -d ' ')"
target_digest="$(hash_file "$target_file")"
[[ "$source_count" == "$target_count" && "$source_digest" == "$target_digest" ]] || \
  fail "post-commit checksum reconciliation failed"

echo "plan-projection: PASS — rows=$target_count sha256=$target_digest"
echo "plan-projection: source snapshot was applied atomically; rerun before cutover if canonical plans changed"
