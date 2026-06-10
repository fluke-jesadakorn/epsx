-- Rollback: Restore display_order column from tier_level

-- Step 1: Add display_order column back
ALTER TABLE plans ADD COLUMN display_order INTEGER DEFAULT 0;

-- Step 2: Copy tier_level values back to display_order
UPDATE plans SET display_order = tier_level;

-- Step 3: Drop the new index
DROP INDEX IF EXISTS idx_plans_active_tier;

-- Step 4: Restore original index
CREATE INDEX idx_plans_active ON plans(is_active, display_order);
