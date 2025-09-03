-- Rollback notification system tables

-- Drop indexes first
DROP INDEX IF EXISTS idx_fcm_tokens_platform;
DROP INDEX IF EXISTS idx_fcm_tokens_active;
DROP INDEX IF EXISTS idx_fcm_tokens_user_id;

DROP INDEX IF EXISTS idx_user_prefs_user_id;

DROP INDEX IF EXISTS idx_user_notifications_notification_id;
DROP INDEX IF EXISTS idx_user_notifications_delivered;
DROP INDEX IF EXISTS idx_user_notifications_read;
DROP INDEX IF EXISTS idx_user_notifications_user_id;

DROP INDEX IF EXISTS idx_notifications_fcm_topic;
DROP INDEX IF EXISTS idx_notifications_priority;
DROP INDEX IF EXISTS idx_notifications_type;
DROP INDEX IF EXISTS idx_notifications_created_at;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS fcm_tokens CASCADE;
DROP TABLE IF EXISTS user_notification_preferences CASCADE;
DROP TABLE IF EXISTS user_notifications CASCADE;
DROP TABLE IF EXISTS notifications CASCADE;