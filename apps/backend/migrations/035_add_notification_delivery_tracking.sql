-- Migration: Add notification delivery tracking for Redis pub/sub system
-- This adds columns to track Redis delivery attempts, offline queueing, and acknowledgments

ALTER TABLE wallet_notifications
ADD COLUMN IF NOT EXISTS queued_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
ADD COLUMN IF NOT EXISTS delivery_attempts INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS last_delivery_attempt_at TIMESTAMP WITH TIME ZONE,
ADD COLUMN IF NOT EXISTS delivery_error TEXT,
ADD COLUMN IF NOT EXISTS acknowledged_at TIMESTAMP WITH TIME ZONE;

-- Index for undelivered notifications (offline queue)
CREATE INDEX IF NOT EXISTS idx_wallet_notifications_undelivered
ON wallet_notifications(wallet_address, delivered_at)
WHERE delivered_at IS NULL;

-- Index for queued notifications that need delivery
CREATE INDEX IF NOT EXISTS idx_wallet_notifications_queued
ON wallet_notifications(wallet_address, queued_at, expires_at)
WHERE delivered_at IS NULL;

-- Index for cleanup of old notifications
CREATE INDEX IF NOT EXISTS idx_wallet_notifications_cleanup
ON wallet_notifications(created_at);

-- Index for acknowledgment tracking
CREATE INDEX IF NOT EXISTS idx_wallet_notifications_acknowledged
ON wallet_notifications(acknowledged_at)
WHERE acknowledged_at IS NOT NULL;

-- Add comments for documentation
COMMENT ON COLUMN wallet_notifications.queued_at IS 'When notification was queued for delivery via Redis';
COMMENT ON COLUMN wallet_notifications.delivery_attempts IS 'Number of delivery attempts via Redis pub/sub';
COMMENT ON COLUMN wallet_notifications.last_delivery_attempt_at IS 'Last attempt to deliver via Redis';
COMMENT ON COLUMN wallet_notifications.delivery_error IS 'Error message if Redis delivery failed';
COMMENT ON COLUMN wallet_notifications.acknowledged_at IS 'When client acknowledged receipt of notification';
