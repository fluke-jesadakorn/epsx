-- Migration: Rename permission_groups to permission_plans in wallet_users table
ALTER TABLE wallet_users RENAME COLUMN permission_groups TO permission_plans;
COMMENT ON COLUMN wallet_users.permission_plans IS 'JSONB array of permission plan IDs (legacy, formerly permission_groups)';
