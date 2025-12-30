-- Revert plan access fields from wallet_users table

-- Remove foreign key constraint first
ALTER TABLE wallet_users DROP CONSTRAINT IF EXISTS fk_wallet_users_current_plan;

-- Remove index
DROP INDEX IF EXISTS idx_wallet_users_plan_expires_at;

-- Remove columns
ALTER TABLE wallet_users 
DROP COLUMN IF EXISTS plan_expires_at,
DROP COLUMN IF EXISTS current_plan_id;
