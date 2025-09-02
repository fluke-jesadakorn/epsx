-- Drop indexes first
DROP INDEX IF EXISTS idx_notifications_fcm_delivered_at;
DROP INDEX IF EXISTS idx_notifications_fcm_sent;

-- Remove FCM-related columns from notifications table
ALTER TABLE notifications 
DROP COLUMN IF EXISTS fcm_data,
DROP COLUMN IF EXISTS delivery_attempts,
DROP COLUMN IF EXISTS fcm_failed_reason,
DROP COLUMN IF EXISTS fcm_delivered_at,
DROP COLUMN IF EXISTS fcm_message_id,
DROP COLUMN IF EXISTS fcm_sent;
