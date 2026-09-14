-- The consolidated v6 baseline owns tier_level and idx_plans_active_tier and
-- intentionally omits display_order. A rollback must preserve that baseline
-- schema and existing plan data, so this compatibility migration is a no-op on
-- the way down.
SELECT 1 AS compatibility_rollback_preserved;
