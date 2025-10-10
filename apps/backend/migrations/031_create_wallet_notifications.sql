-- Migration: Create wallet_notifications table for Web3-based notifications
-- This table stores notifications for wallet users in the Web3-first system

CREATE TABLE IF NOT EXISTS wallet_notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    data JSONB DEFAULT NULL,
    priority VARCHAR(20) NOT NULL DEFAULT 'normal',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    read_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    clicked_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    delivered_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    action_url VARCHAR(500) DEFAULT NULL,
    image_url VARCHAR(500) DEFAULT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT valid_notification_type CHECK (notification_type IN ('system', 'security', 'permission', 'wallet_management', 'wallet', 'payment', 'general', 'admin', 'data', 'feature')),
    CONSTRAINT valid_priority CHECK (priority IN ('low', 'normal', 'high', 'critical', 'urgent')),
    -- Allow 'all' for broadcast or valid wallet address format
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address = 'all' OR
        (wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND length(wallet_address) = 42)
    )
);

-- Indexes for performance
CREATE INDEX idx_wallet_notifications_wallet ON wallet_notifications(wallet_address);
CREATE INDEX idx_wallet_notifications_timestamp ON wallet_notifications(timestamp DESC);
CREATE INDEX idx_wallet_notifications_read_at ON wallet_notifications(read_at) WHERE read_at IS NULL;
CREATE INDEX idx_wallet_notifications_type ON wallet_notifications(notification_type);
CREATE INDEX idx_wallet_notifications_priority ON wallet_notifications(priority);
CREATE INDEX idx_wallet_notifications_expires ON wallet_notifications(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_wallet_notifications_wallet_unread ON wallet_notifications(wallet_address, read_at) WHERE read_at IS NULL;

-- Trigger to update updated_at timestamp
CREATE TRIGGER update_wallet_notifications_updated_at
    BEFORE UPDATE ON wallet_notifications
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE wallet_notifications IS 'Stores notifications for wallet users in Web3-first system';
COMMENT ON COLUMN wallet_notifications.wallet_address IS 'Recipient wallet address (use ''all'' for broadcast notifications)';
COMMENT ON COLUMN wallet_notifications.notification_type IS 'Type: system, security, permission, wallet_management, wallet, payment, general, admin, data, feature';
COMMENT ON COLUMN wallet_notifications.priority IS 'Priority level: low, normal, high, critical, urgent';
COMMENT ON COLUMN wallet_notifications.data IS 'Additional structured data in JSON format';
COMMENT ON COLUMN wallet_notifications.timestamp IS 'When the notification was created/sent';
COMMENT ON COLUMN wallet_notifications.expires_at IS 'Optional expiration timestamp for time-sensitive notifications';
COMMENT ON COLUMN wallet_notifications.read_at IS 'When the user marked the notification as read';
COMMENT ON COLUMN wallet_notifications.clicked_at IS 'When the user clicked on the notification action';
COMMENT ON COLUMN wallet_notifications.delivered_at IS 'When the notification was successfully delivered';
