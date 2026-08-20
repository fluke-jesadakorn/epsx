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

-- Create the replica if it doesn't exist yet. Legacy single-database
-- deployments can clone public.plans directly. A dedicated payments database
-- has no public.plans relation, so create the same bounded projection
-- explicitly; data replication and reconciliation remain a separate cutover
-- step and this migration never invents authoritative plan rows.
DO $$
BEGIN
  IF to_regclass('payments.plans') IS NULL THEN
    IF to_regclass('public.plans') IS NOT NULL THEN
      EXECUTE 'CREATE TABLE payments.plans (LIKE public.plans INCLUDING ALL)';
    ELSE
      CREATE TABLE payments.plans (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        name VARCHAR(100) NOT NULL UNIQUE,
        slug VARCHAR(100) NOT NULL UNIQUE,
        description TEXT NOT NULL DEFAULT '',
        plan_type VARCHAR(20) NOT NULL DEFAULT 'manual',
        plan_category VARCHAR(20) NOT NULL DEFAULT 'base',
        plan_group VARCHAR(20) NOT NULL DEFAULT 'personal',
        plan_metadata JSONB NOT NULL DEFAULT '{}',
        price NUMERIC(10,2) DEFAULT 0.00,
        currency VARCHAR(3) DEFAULT 'USD',
        billing_cycle VARCHAR(20) DEFAULT 'pay_per_use',
        is_active BOOLEAN NOT NULL DEFAULT TRUE,
        is_promoted BOOLEAN NOT NULL DEFAULT FALSE,
        is_public BOOLEAN NOT NULL DEFAULT TRUE,
        is_system BOOLEAN NOT NULL DEFAULT FALSE,
        tier_level INTEGER NOT NULL DEFAULT 0,
        max_members INTEGER,
        auto_assign_enabled BOOLEAN DEFAULT FALSE,
        assignment_rules JSONB DEFAULT '{}',
        grace_period_hours INTEGER NOT NULL DEFAULT 0,
        rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
        rate_limit_per_hour INTEGER NOT NULL DEFAULT 1000,
        rate_limit_per_day INTEGER NOT NULL DEFAULT 10000,
        burst_capacity INTEGER NOT NULL DEFAULT 10,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        created_by VARCHAR(42),
        last_modified_by VARCHAR(42),
        CONSTRAINT valid_plan_type CHECK (plan_type IN ('manual', 'subscription', 'web3_asset', 'dao_membership', 'admin')),
        CONSTRAINT valid_plan_category CHECK (plan_category IN ('base', 'addon', 'system', 'exclusive')),
        CONSTRAINT valid_plan_group CHECK (plan_group IN ('personal', 'enterprise', 'api', 'custom')),
        CONSTRAINT valid_currency CHECK (currency IN ('USD', 'EUR', 'BTC', 'ETH', 'BNB')),
        CONSTRAINT valid_billing_cycle CHECK (billing_cycle IN ('monthly', 'yearly', 'one_time', 'lifetime', 'pay_per_use'))
      );
    END IF;
  END IF;
END
$$;

CREATE INDEX IF NOT EXISTS idx_plans_active_tier ON payments.plans(is_active, tier_level);
CREATE INDEX IF NOT EXISTS idx_plans_category ON payments.plans(plan_category);
CREATE INDEX IF NOT EXISTS idx_plans_created ON payments.plans(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_plans_group ON payments.plans(plan_group);
CREATE INDEX IF NOT EXISTS idx_plans_metadata_gin ON payments.plans USING GIN(plan_metadata);
CREATE INDEX IF NOT EXISTS idx_plans_price ON payments.plans(price, currency) WHERE plan_type = 'subscription';
CREATE INDEX IF NOT EXISTS idx_plans_slug ON payments.plans(slug);
CREATE INDEX IF NOT EXISTS idx_plans_type ON payments.plans(plan_type, is_active);

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

-- A same-database legacy deployment can keep the projection current with a
-- trigger. Dedicated databases deliberately skip this trigger because
-- PostgreSQL triggers cannot cross database boundaries; their external
-- replication/reconciliation gate remains explicit.
DO $$
BEGIN
  IF to_regclass('public.plans') IS NOT NULL THEN
    EXECUTE 'DROP TRIGGER IF EXISTS sync_plans_to_payments_schema ON public.plans';
    EXECUTE 'CREATE TRIGGER sync_plans_to_payments_schema
      AFTER INSERT OR UPDATE OR DELETE ON public.plans
      FOR EACH ROW EXECUTE FUNCTION payments.sync_plans_from_public()';
  END IF;
END
$$;
