-- Rollback: Dynamic Plan System with Resource Tracking
-- Removes all tables and structures created for flexible plan management

-- Drop views first (depends on tables)
DROP VIEW IF EXISTS subscription_details;

-- Drop triggers
DROP TRIGGER IF EXISTS update_usage_analytics_summary_updated_at ON usage_analytics_summary;
DROP TRIGGER IF EXISTS update_user_plan_subscriptions_updated_at ON user_plan_subscriptions;
DROP TRIGGER IF EXISTS update_plan_features_updated_at ON plan_features;
DROP TRIGGER IF EXISTS update_access_contexts_updated_at ON access_contexts;

-- Drop the trigger function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop main tables (in reverse dependency order)
DROP TABLE IF EXISTS subscription_changes;
DROP TABLE IF EXISTS real_time_usage_cache;
DROP TABLE IF EXISTS usage_analytics_summary;
DROP TABLE IF EXISTS resource_usage_events;
DROP TABLE IF EXISTS user_plan_subscriptions;
DROP TABLE IF EXISTS plan_features;
DROP TABLE IF EXISTS access_contexts;

-- Revert pricing_plans table changes
ALTER TABLE pricing_plans 
DROP COLUMN IF EXISTS plan_metadata,
DROP COLUMN IF EXISTS billing_model,
DROP COLUMN IF EXISTS target_audience,
DROP COLUMN IF EXISTS plan_category;

-- Note: We preserve existing pricing_plans data and structure
-- as this migration only adds new columns without removing existing ones