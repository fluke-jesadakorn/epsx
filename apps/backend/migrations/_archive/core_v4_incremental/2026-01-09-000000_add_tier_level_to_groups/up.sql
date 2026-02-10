-- Add tier_level column to groups table for plan upgrade/downgrade logic
-- Higher tier_level = better plan. 0 = free tier.

ALTER TABLE groups ADD COLUMN tier_level INTEGER NOT NULL DEFAULT 0;

COMMENT ON COLUMN groups.tier_level IS 'Plan tier level for upgrade/downgrade logic. Higher = better plan. 0 = free tier.';

-- Set initial tier levels based on existing plan names/prices
UPDATE groups SET tier_level = CASE
    WHEN price IS NULL OR price = 0 THEN 0
    WHEN price < 15 THEN 1
    WHEN price < 30 THEN 2
    WHEN price < 50 THEN 3
    ELSE 4
END WHERE group_type IN ('subscription', 'plan');
