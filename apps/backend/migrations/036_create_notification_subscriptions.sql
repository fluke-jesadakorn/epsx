-- Migration: Create notification_subscriptions table for SSE connection tracking
-- This table tracks active SSE connections across multiple backend instances

CREATE TABLE IF NOT EXISTS notification_subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    instance_id VARCHAR(100) NOT NULL,
    connection_id VARCHAR(100) NOT NULL,
    connected_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_ping_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    disconnected_at TIMESTAMP WITH TIME ZONE,
    user_agent TEXT,
    ip_address INET,
    redis_channel VARCHAR(200),

    -- Constraints
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address = 'all' OR
        (wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND length(wallet_address) = 42)
    ),
    CONSTRAINT unique_connection UNIQUE (instance_id, connection_id)
);

-- Index for active subscriptions per wallet
CREATE INDEX idx_subscriptions_wallet_active
ON notification_subscriptions(wallet_address, connected_at)
WHERE disconnected_at IS NULL;

-- Index for active subscriptions per instance
CREATE INDEX idx_subscriptions_instance_active
ON notification_subscriptions(instance_id, connected_at)
WHERE disconnected_at IS NULL;

-- Index for cleanup of disconnected subscriptions
CREATE INDEX idx_subscriptions_disconnected
ON notification_subscriptions(disconnected_at)
WHERE disconnected_at IS NOT NULL;

-- Index for stale connection detection (use with runtime check for 1 hour)
CREATE INDEX idx_subscriptions_stale
ON notification_subscriptions(last_ping_at, disconnected_at)
WHERE disconnected_at IS NULL;

-- Comments for documentation
COMMENT ON TABLE notification_subscriptions IS 'Tracks active SSE connections across multiple backend instances for Redis pub/sub';
COMMENT ON COLUMN notification_subscriptions.wallet_address IS 'Wallet address subscribed to notifications (or ''all'' for broadcast)';
COMMENT ON COLUMN notification_subscriptions.instance_id IS 'Backend instance identifier (hostname or container ID)';
COMMENT ON COLUMN notification_subscriptions.connection_id IS 'Unique connection identifier (UUID per SSE connection)';
COMMENT ON COLUMN notification_subscriptions.connected_at IS 'When SSE connection was established';
COMMENT ON COLUMN notification_subscriptions.last_ping_at IS 'Last SSE keep-alive ping sent';
COMMENT ON COLUMN notification_subscriptions.disconnected_at IS 'When SSE connection was closed';
COMMENT ON COLUMN notification_subscriptions.redis_channel IS 'Redis channel name this connection is subscribed to';
