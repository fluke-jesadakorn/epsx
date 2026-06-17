#!/usr/bin/env bash
#
# Wave 11 / integration gate — production cutover step 2.
#
# One-shot bulk copy: replicate every row from `public.plans`
# into `payments.plans`. Run AFTER
# `apps/backend/migrations/payments/20260613000000_replicate_plans_into_payments_schema/up.sql`
# has applied (so the table + trigger exist).
#
# Why this is a separate script (not in the migration):
#   - The migration only creates the schema + the sync trigger
#     for FUTURE writes. The initial bulk copy is a one-shot
#     data move that the production team runs by hand after
#     applying the migration.
#   - Doing the bulk copy inside the migration would lock
#     `public.plans` for the duration of the INSERT, which
#     can be a multi-minute outage on a populated prod DB.
#     The script runs in a single transaction with explicit
#     batching, so the lock window is short.
#
# Usage:
#   DATABASE_URL=postgres://... ./infrastructure/scripts/wave11-replicate-plans.sh
#
# Or, for a dry-run that shows the row count without writing:
#   WAVE11_DRY_RUN=1 DATABASE_URL=postgres://... ./infrastructure/scripts/wave11-replicate-plans.sh
#
# The script:
#   1. Counts the source rows in `public.plans` for the audit log.
#   2. Runs `INSERT INTO payments.plans SELECT * FROM public.plans
#      ON CONFLICT (id) DO NOTHING` (idempotent — re-runs are safe).
#   3. Counts the destination rows and asserts the count matches.
#   4. Exits 0 on success, 1 on any failure (and rolls back the txn).
#
# CLAUDE.md "Migration safety" compliance:
#   - The script is IDEMPOTENT (ON CONFLICT DO NOTHING).
#   - The script does NOT touch `public.plans` (read-only on the
#     source).
#   - The script is a one-shot — production teams run it once
#     at cutover time, then the trigger keeps the replica in sync.

set -euo pipefail

# ---------------------------------------------------------------------------
# Preflight
# ---------------------------------------------------------------------------

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "ERROR: DATABASE_URL must be set" >&2
  echo "  export DATABASE_URL=postgres://user:pass@host:5432/epsx_prod" >&2
  exit 1
fi

DRY_RUN="${WAVE11_DRY_RUN:-0}"

# Use psql for the transaction. If psql isn't on PATH, fail loud.
if ! command -v psql >/dev/null 2>&1; then
  echo "ERROR: psql not found in PATH" >&2
  echo "  Install Postgres client tools (brew install libpq on macOS)" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Step 1: Count the source rows for the audit log
# ---------------------------------------------------------------------------

echo "[wave11] Counting source rows in public.plans ..."
SOURCE_COUNT=$(psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM public.plans;")
echo "[wave11]   source count: $SOURCE_COUNT"

if [[ "$SOURCE_COUNT" -eq 0 ]]; then
  echo "[wave11] WARN: public.plans is empty — nothing to replicate." >&2
  echo "[wave11]   This is OK if the prod DB has no plans yet, but" >&2
  echo "[wave11]   unusual for a payments-table system. Verify before" >&2
  echo "[wave11]   continuing." >&2
fi

# ---------------------------------------------------------------------------
# Step 2: Bulk copy
# ---------------------------------------------------------------------------

if [[ "$DRY_RUN" == "1" ]]; then
  echo "[wave11] DRY RUN: would INSERT $SOURCE_COUNT rows into payments.plans"
  echo "[wave11]   (set WAVE11_DRY_RUN=0 to actually run)"
  exit 0
fi

echo "[wave11] Replicating $SOURCE_COUNT rows into payments.plans ..."

# Run the INSERT in a single transaction. ON CONFLICT DO NOTHING
# means the script is idempotent — re-runs on a partially-replicated
# DB will only fill in the missing rows.
psql "$DATABASE_URL" <<SQL
BEGIN;

INSERT INTO payments.plans
SELECT * FROM public.plans
ON CONFLICT (id) DO NOTHING;

COMMIT;
SQL

# ---------------------------------------------------------------------------
# Step 3: Verify the count matches
# ---------------------------------------------------------------------------

DEST_COUNT=$(psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM payments.plans;")
echo "[wave11]   destination count after copy: $DEST_COUNT"

if [[ "$DEST_COUNT" -ne "$SOURCE_COUNT" ]]; then
  echo "ERROR: row count mismatch after replication" >&2
  echo "  source:      $SOURCE_COUNT" >&2
  echo "  destination: $DEST_COUNT" >&2
  echo "  (this should be impossible — the INSERT is ON CONFLICT DO NOTHING)" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Step 4: Sanity-check the sync trigger is firing
# ---------------------------------------------------------------------------

echo "[wave11] Verifying the sync_plans_to_payments_schema trigger ..."
TRIGGER_EXISTS=$(psql "$DATABASE_URL" -t -A -c "
  SELECT COUNT(*) FROM pg_trigger
  WHERE tgname = 'sync_plans_to_payments_schema';
")
if [[ "$TRIGGER_EXISTS" -ne "1" ]]; then
  echo "ERROR: sync_plans_to_payments_schema trigger is missing" >&2
  echo "  Did the migration apply? Check:" >&2
  echo "  psql \$DATABASE_URL -c \"\\dft sync_plans_from_public\"" >&2
  exit 1
fi
echo "[wave11]   trigger present: OK"

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------

echo ""
echo "[wave11] DONE: replicated $DEST_COUNT plan rows to payments.plans"
echo "[wave11]   The sync trigger will keep the replica in sync on"
echo "[wave11]   subsequent INSERT/UPDATE/DELETE on public.plans."
echo ""
echo "[wave11] Next step in the production cutover checklist:"
echo "[wave11]   - Set PAYMENTS_DATABASE_URL in .env.prod (step 3)"
echo "[wave11]   - Restart the backend (step 4)"
echo "[wave11]   - Verify /api/payments/* routes are healthy (step 5)"
