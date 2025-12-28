-- ============================================================================
-- EPSX Notifications Database Schema
-- ============================================================================
-- Separate database for real-time notifications (SSE streaming)
-- Isolates notification queries from core transaction paths
-- 
-- Tables:
--   - wallet_notifications
--   - notification_subscriptions
-- ============================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

-- ============================================================================
-- WALLET NOTIFICATIONS
-- ============================================================================

CREATE TABLE wallet_notifications (
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
CREATE INDEX idx_notif_queue_fetch ON wallet_notifications (wallet_address, deleted_at, created_at, timestamp DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_notif_user_query ON wallet_notifications (deleted_at, wallet_address, read_at, timestamp DESC);
CREATE INDEX idx_notif_admin_query ON wallet_notifications (deleted_at, notification_type, priority, timestamp DESC);
CREATE INDEX idx_notif_expiry ON wallet_notifications (expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_notif_unread_count ON wallet_notifications (wallet_address, read_at, deleted_at) WHERE read_at IS NULL AND deleted_at IS NULL;
CREATE INDEX idx_notif_wallet ON wallet_notifications(wallet_address);
CREATE INDEX idx_notif_timestamp ON wallet_notifications(timestamp DESC);
CREATE INDEX idx_notif_active ON wallet_notifications(wallet_address, deleted_at, created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_notif_offline_queue ON wallet_notifications(wallet_address, created_at DESC, deleted_at, expires_at) WHERE deleted_at IS NULL;

COMMENT ON TABLE wallet_notifications IS 'Notifications for wallet users with SSE delivery tracking';

-- ============================================================================
-- NOTIFICATION SUBSCRIPTIONS (SSE Connections)
-- ============================================================================

CREATE TABLE notification_subscriptions (
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

CREATE INDEX idx_subs_wallet_active ON notification_subscriptions(wallet_address, connected_at) WHERE disconnected_at IS NULL;
CREATE INDEX idx_subs_instance_active ON notification_subscriptions(instance_id, connected_at) WHERE disconnected_at IS NULL;
CREATE INDEX idx_subs_stale ON notification_subscriptions(last_ping_at, disconnected_at) WHERE disconnected_at IS NULL;

COMMENT ON TABLE notification_subscriptions IS 'Active SSE connections for Redis pub/sub routing';

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE wallet_notifications IS 'Real-time notifications with SSE delivery tracking';
COMMENT ON TABLE notification_subscriptions IS 'Tracks active SSE connections for multi-instance Redis pub/sub';
