-- Add plan access fields to wallet_users table for Direct Payment model
-- These columns replace the active_subscriptions table

ALTER TABLE wallet_users 
ADD COLUMN IF NOT EXISTS plan_expires_at TIMESTAMPTZ DEFAULT NULL,
ADD COLUMN IF NOT EXISTS current_plan_id INTEGER DEFAULT NULL;

-- Add index for efficient expiry queries
CREATE INDEX IF NOT EXISTS idx_wallet_users_plan_expires_at 
ON wallet_users (plan_expires_at) 
WHERE plan_expires_at IS NOT NULL;

-- Add foreign key reference to pricing_plans
-- Note: Using constraint check only if pricing_plans exists
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'pricing_plans') THEN
        ALTER TABLE wallet_users 
        ADD CONSTRAINT fk_wallet_users_current_plan 
        FOREIGN KEY (current_plan_id) REFERENCES pricing_plans(id) ON DELETE SET NULL;
    END IF;
EXCEPTION
    WHEN duplicate_object THEN
        -- Constraint already exists, ignore
        NULL;
END $$;

COMMENT ON COLUMN wallet_users.plan_expires_at IS 'When the user current plan access expires (Direct Payment model)';
COMMENT ON COLUMN wallet_users.current_plan_id IS 'Reference to the user current pricing plan';
