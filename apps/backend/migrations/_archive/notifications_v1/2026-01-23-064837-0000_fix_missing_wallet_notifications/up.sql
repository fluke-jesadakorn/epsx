
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

    -- Delivery tracking columns
    queued_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    delivery_attempts INTEGER DEFAULT 0,
    last_delivery_attempt_at TIMESTAMP WITH TIME ZONE,
    delivery_error TEXT,
    acknowledged_at TIMESTAMP WITH TIME ZONE,

    -- Soft delete column
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,

    -- Constraints
    CONSTRAINT valid_notification_type CHECK (notification_type IN ('system', 'security', 'permission', 'wallet_management', 'wallet', 'payment', 'general', 'admin', 'data', 'feature')),
    CONSTRAINT valid_priority CHECK (priority IN ('low', 'normal', 'high', 'critical', 'urgent')),
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address = 'all' OR
        (wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND length(wallet_address) = 42)
    )
);

-- Performance indexes for SSE streaming
CREATE INDEX IF NOT EXISTS idx_notif_queue_fetch ON wallet_notifications (wallet_address, deleted_at, created_at, timestamp DESC) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_notif_user_query ON wallet_notifications (deleted_at, wallet_address, read_at, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_notif_admin_query ON wallet_notifications (deleted_at, notification_type, priority, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_notif_expiry ON wallet_notifications (expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_notif_unread_count ON wallet_notifications (wallet_address, read_at, deleted_at) WHERE read_at IS NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_notif_wallet ON wallet_notifications(wallet_address);
CREATE INDEX IF NOT EXISTS idx_notif_timestamp ON wallet_notifications(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_notif_active ON wallet_notifications(wallet_address, deleted_at, created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_notif_offline_queue ON wallet_notifications(wallet_address, created_at DESC, deleted_at, expires_at) WHERE deleted_at IS NULL;

COMMENT ON TABLE wallet_notifications IS 'Notifications for wallet users with SSE delivery tracking';
