-- Keep the same-database plan projection byte-for-byte aligned on every
-- update. Split-database deployments use the separately reviewed bounded
-- reconciler because PostgreSQL triggers cannot cross database boundaries.
--
-- This is a forward-only replacement of the existing function. It changes no
-- table and deletes no row. Existing triggers keep their identity and invoke
-- the replaced function on the next source write.
CREATE OR REPLACE FUNCTION payments.sync_plans_from_public()
RETURNS TRIGGER AS $$
BEGIN
  IF TG_OP = 'DELETE' THEN
    -- Historical payments can still reference this projection. Preserve the
    -- row for audit/display while making it unavailable for new purchases.
    UPDATE payments.plans
    SET is_active = FALSE,
        is_public = FALSE,
        is_promoted = FALSE,
        updated_at = NOW()
    WHERE id = OLD.id;
    RETURN OLD;
  END IF;

  INSERT INTO payments.plans SELECT NEW.*
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
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
