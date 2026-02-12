-- Consolidate display_order to tier_level
-- This migration removes the redundant display_order field and uses only tier_level

-- Step 1: Migrate display_order values to tier_level for plans where tier_level is 0 or NULL
UPDATE plans
SET tier_level = COALESCE(display_order, 0)
WHERE tier_level = 0 OR tier_level IS NULL;

-- Step 2: For safety, ensure no tier_level is NULL (set to 0 if needed)
UPDATE plans
SET tier_level = 0
WHERE tier_level IS NULL;

-- Step 3: Drop the old index that used display_order
DROP INDEX IF EXISTS idx_plans_active;

-- Step 4: Drop the display_order column
ALTER TABLE plans DROP COLUMN IF EXISTS display_order;

-- Step 5: Create new index using tier_level instead
CREATE INDEX idx_plans_active_tier ON plans(is_active, tier_level);
