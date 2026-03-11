-- ============================================================================
-- EPSX Notifications Consolidated Schema v2
-- ============================================================================
-- Consolidates all notifications migrations into a single baseline.
--
-- Tables:
--   - wallet_notifications (v2 schema)
--   - notification_subscriptions
-- ============================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

-- ============================================================================
-- WALLET NOTIFICATIONS (v2 Schema)
-- ============================================================================

CREATE TABLE wallet_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_wallet_address VARCHAR(42),
    topic_name VARCHAR(255),
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    urgency VARCHAR(20) NOT NULL DEFAULT 'normal',
    notification_type VARCHAR(50) NOT NULL,
    priority VARCHAR(20) NOT NULL DEFAULT 'normal',
    channels JSONB NOT NULL DEFAULT '{}',
    schedule_type VARCHAR(20) NOT NULL DEFAULT 'immediate',
    scheduled_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) NOT NULL DEFAULT 'created',
    send_started_at TIMESTAMP WITH TIME ZONE,
    channel_status JSONB NOT NULL DEFAULT '{}',
    total_attempts INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(42),
    image_url TEXT,
    action_url TEXT,
    data_payload JSONB,
    tags TEXT[],
    notes TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_recipient ON wallet_notifications(recipient_wallet_address);
CREATE INDEX idx_notifications_status ON wallet_notifications(status);
CREATE INDEX idx_notifications_created_at ON wallet_notifications(created_at);

COMMENT ON TABLE wallet_notifications IS 'Notifications for wallet users with multi-channel delivery tracking';

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

    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address = 'all' OR
        (wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND length(wallet_address) = 42)
    ),
    CONSTRAINT unique_connection UNIQUE (instance_id, connection_id)
);

CREATE INDEX idx_subs_wallet_active ON notification_subscriptions(wallet_address, connected_at) WHERE disconnected_at IS NULL;
CREATE INDEX idx_subs_instance_active ON notification_subscriptions(instance_id, connected_at) WHERE disconnected_at IS NULL;
CREATE INDEX idx_subs_stale ON notification_subscriptions(last_ping_at, disconnected_at) WHERE disconnected_at IS NULL;

COMMENT ON TABLE notification_subscriptions IS 'Tracks active SSE connections for multi-instance Redis pub/sub';

SELECT 'EPSX NOTIFICATIONS CONSOLIDATED SCHEMA v2 CREATED SUCCESSFULLY! 🎉' AS success_message;
