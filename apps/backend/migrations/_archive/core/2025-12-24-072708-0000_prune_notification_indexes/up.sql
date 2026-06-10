-- Drop redundant and excessive indexes
DROP INDEX IF EXISTS idx_wallet_notifications_user_query;
DROP INDEX IF EXISTS idx_wallet_notifications_soft_deleted;
DROP INDEX IF EXISTS idx_wallet_notifications_read_cleanup;
DROP INDEX IF EXISTS idx_wallet_notifications_timestamp_stats;
DROP INDEX IF EXISTS idx_wallet_notifications_type_stats;
DROP INDEX IF EXISTS idx_wallet_notifications_priority_stats;
DROP INDEX IF EXISTS idx_wallet_notifications_read_rate;
DROP INDEX IF EXISTS idx_wallet_notifications_click_rate;
DROP INDEX IF EXISTS idx_wallet_notifications_delivery_rate;
DROP INDEX IF EXISTS idx_wallet_notifications_acknowledgement;
DROP INDEX IF EXISTS idx_wallet_notifications_wallet;
DROP INDEX IF EXISTS idx_wallet_notifications_timestamp;
DROP INDEX IF EXISTS idx_wallet_notifications_read_at;
DROP INDEX IF EXISTS idx_wallet_notifications_type;
DROP INDEX IF EXISTS idx_wallet_notifications_priority;
DROP INDEX IF EXISTS idx_wallet_notifications_expires;
DROP INDEX IF EXISTS idx_wallet_notifications_wallet_unread;
DROP INDEX IF EXISTS idx_wallet_notifications_undelivered;
DROP INDEX IF EXISTS idx_wallet_notifications_queued;
DROP INDEX IF EXISTS idx_wallet_notifications_cleanup;
DROP INDEX IF EXISTS idx_wallet_notifications_acknowledged;
DROP INDEX IF EXISTS idx_wallet_notifications_active;
DROP INDEX IF EXISTS idx_wallet_notifications_deleted;
DROP INDEX IF EXISTS idx_wallet_notifications_offline_queue;
DROP INDEX IF EXISTS idx_wallet_notifications_unread_active;

-- We are keeping these 4 core indexes for functionality:
-- 1. idx_wallet_notifications_queue_fetch (Feed)
-- 2. idx_wallet_notifications_unread_count (Badges)
-- 3. idx_wallet_notifications_admin_query (Admin)
-- 4. idx_wallet_notifications_expiry (Cleanup)
