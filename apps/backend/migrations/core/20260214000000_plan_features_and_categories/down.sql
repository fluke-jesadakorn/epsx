-- Revert plan features & categories
DROP TABLE IF EXISTS plan_features;
DROP TABLE IF EXISTS features;
ALTER TABLE plans DROP CONSTRAINT IF EXISTS valid_plan_category;
ALTER TABLE plans DROP COLUMN IF EXISTS plan_category;
DROP INDEX IF EXISTS idx_plans_category;
