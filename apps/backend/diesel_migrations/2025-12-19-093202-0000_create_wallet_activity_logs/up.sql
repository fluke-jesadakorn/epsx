-- Create wallet_activity_logs table for tracking wallet events
CREATE TABLE wallet_activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    performed_by VARCHAR(42),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient queries by wallet address
CREATE INDEX idx_wallet_activity_logs_wallet ON wallet_activity_logs(wallet_address);

-- Index for efficient queries by event type
CREATE INDEX idx_wallet_activity_logs_type ON wallet_activity_logs(event_type);

-- Index for efficient queries by timestamp
CREATE INDEX idx_wallet_activity_logs_created ON wallet_activity_logs(created_at DESC);

-- Add disable_info field to wallet_users table for tracking wallet disable state
ALTER TABLE wallet_users 
ADD COLUMN IF NOT EXISTS disable_info JSONB DEFAULT NULL;

-- Comment on table
COMMENT ON TABLE wallet_activity_logs IS 'Tracks wallet activity events like permission changes, logins, disables';
COMMENT ON COLUMN wallet_activity_logs.event_type IS 'Event types: permission_granted, permission_revoked, subscription_started, subscription_cancelled, wallet_disabled, wallet_enabled, wallet_created, login';
COMMENT ON COLUMN wallet_activity_logs.performed_by IS 'Wallet address of the admin who performed the action (null for system events)';
