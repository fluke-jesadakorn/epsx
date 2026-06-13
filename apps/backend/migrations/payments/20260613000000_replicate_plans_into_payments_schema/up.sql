-- Wave 11 / integration gate — schema cutover: replicate `plans`
-- into the `payments` schema so the in-process PaymentRepositoryPort
-- impl can JOIN `payments ⋈ payments.plans` single-pool instead of
-- the pre-cutover `payments_pool` + `get_diesel_pool()` cross-pool
-- pattern. Non-destructive: `CREATE TABLE IF NOT EXISTS` + a sync
-- trigger. NO `DROP` of the source `public.plans` table.
--
-- CLAUDE.md "Migration safety" compliance:
--   - `IF NOT EXISTS` guards on every CREATE
--   - `CREATE OR REPLACE FUNCTION` for the sync function
--   - `DROP TRIGGER IF EXISTS` (idempotent) on the down migration
--   - NO `DROP TABLE public.plans` anywhere in either direction
--   - The one-shot data sync is a separate script
--     (`infrastructure/scripts/wave11-replicate-plans.sh`) the
--     production team runs by hand AFTER this migration applies.

CREATE SCHEMA IF NOT EXISTS payments;

-- Create the replica if it doesn't exist yet. `LIKE ... INCLUDING
-- ALL` copies the schema (columns + indexes + constraints) but
-- NOT the data — the one-shot script handles the initial bulk
-- copy. The trigger (below) keeps the replica in sync on
-- subsequent INSERT/UPDATE/DELETE.
CREATE TABLE IF NOT EXISTS payments.plans (LIKE public.plans INCLUDING ALL);

-- A short comment so future readers understand the relationship
-- to the canonical `public.plans` table.
COMMENT ON TABLE payments.plans IS
  'Replica of public.plans for single-pool joins from the payments handlers. '
  'Source of truth is public.plans; this replica is read-only from the payments schema. '
  'Kept in sync by the sync_plans_from_public() trigger (see below).';

-- Sync function. Idempotent (CREATE OR REPLACE). Handles the 3
-- trigger ops:
--   INSERT: insert NEW into payments.plans (no conflict — the
--           one-shot script ensures uniqueness, and the trigger
--           will only fire ONCE for a given plan since after that
--           the same row exists in both schemas).
--   UPDATE: UPSERT NEW into payments.plans using EXCLUDED.*.
--   DELETE: delete the row from payments.plans by id.
CREATE OR REPLACE FUNCTION payments.sync_plans_from_public()
RETURNS TRIGGER AS $$
BEGIN
  IF TG_OP = 'DELETE' THEN
    DELETE FROM payments.plans WHERE id = OLD.id;
    RETURN OLD;
  ELSE
    INSERT INTO payments.plans SELECT NEW.*
      ON CONFLICT (id) DO UPDATE SET
        name = EXCLUDED.name,
        price = EXCLUDED.price,
        is_active = EXCLUDED.is_active,
        plan_type = EXCLUDED.plan_type,
        plan_metadata = EXCLUDED.plan_metadata,
        updated_at = EXCLUDED.updated_at;
    RETURN NEW;
  END IF;
END;
$$ LANGUAGE plpgsql;

-- The trigger itself. `AFTER` so the source row is committed
-- before we replicate. `FOR EACH ROW` — we need the NEW/OLD
-- values, not per-statement. Idempotent on the up migration via
-- the `DROP TRIGGER IF EXISTS` in the down file.
DROP TRIGGER IF EXISTS sync_plans_to_payments_schema ON public.plans;
CREATE TRIGGER sync_plans_to_payments_schema
  AFTER INSERT OR UPDATE OR DELETE ON public.plans
  FOR EACH ROW EXECUTE FUNCTION payments.sync_plans_from_public();
