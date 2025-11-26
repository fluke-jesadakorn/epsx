-- ============================================================================
-- Notification Performance Indexes (DOWN)
-- ============================================================================
--
-- This migration removes all the notification indexes that were created
-- to improve query performance. Removing these indexes will make queries
-- slower but won't affect data integrity.
-- ============================================================================

-- ============================================================================
-- PRIMARY QUERY INDEXES
-- ============================================================================

DROP INDEX IF EXISTS idx_wallet_notifications_queue_fetch;
DROP INDEX IF EXISTS idx_wallet_notifications_user_query;
DROP INDEX IF EXISTS idx_wallet_notifications_admin_query;

-- ============================================================================
-- EXPIRY AND CLEANUP INDEXES
-- ============================================================================

DROP INDEX IF EXISTS idx_wallet_notifications_expiry;
DROP INDEX IF EXISTS idx_wallet_notifications_soft_deleted;
DROP INDEX IF EXISTS idx_wallet_notifications_read_cleanup;

-- ============================================================================
-- STATISTICS AND ANALYTICS INDEXES
-- ============================================================================

DROP INDEX IF EXISTS idx_wallet_notifications_timestamp_stats;
DROP INDEX IF EXISTS idx_wallet_notifications_type_stats;
DROP INDEX IF EXISTS idx_wallet_notifications_priority_stats;

-- ============================================================================
-- RATE CALCULATION INDEXES
-- ============================================================================

DROP INDEX IF EXISTS idx_wallet_notifications_read_rate;
DROP INDEX IF EXISTS idx_wallet_notifications_click_rate;
DROP INDEX IF EXISTS idx_wallet_notifications_delivery_rate;
DROP INDEX IF EXISTS idx_wallet_notifications_acknowledgement;

-- ============================================================================
-- UNREAD COUNT INDEX
-- ============================================================================

DROP INDEX IF EXISTS idx_wallet_notifications_unread_count;