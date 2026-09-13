-- Consolidate display_order to tier_level
-- This migration removes the redundant display_order field and uses only tier_level

-- Step 1: Migrate legacy display_order values only when upgrading a schema that
-- predates the consolidated v6 baseline. Fresh databases already have only
-- tier_level, so an unconditional reference to display_order is invalid.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'plans'
          AND column_name = 'display_order'
    ) THEN
        EXECUTE 'UPDATE plans
                 SET tier_level = COALESCE(display_order, 0)
                 WHERE tier_level = 0 OR tier_level IS NULL';
    END IF;
END
$$;

-- Step 2: For safety, ensure no tier_level is NULL (set to 0 if needed)
UPDATE plans
SET tier_level = 0
WHERE tier_level IS NULL;

-- Step 3: Drop the old index that used display_order
DROP INDEX IF EXISTS idx_plans_active;

-- Step 4: Drop the display_order column
ALTER TABLE plans DROP COLUMN IF EXISTS display_order;

-- Step 5: Create new index using tier_level instead
CREATE INDEX IF NOT EXISTS idx_plans_active_tier ON plans(is_active, tier_level);
