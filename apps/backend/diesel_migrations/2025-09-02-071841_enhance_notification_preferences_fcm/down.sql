-- Drop index first
DROP INDEX IF EXISTS idx_notification_preferences_timezone;

-- Remove FCM-related columns from notification_preferences table (keep push_enabled as it existed before)
ALTER TABLE notification_preferences 
DROP COLUMN IF EXISTS platform_preferences,
DROP COLUMN IF EXISTS fcm_topics,
DROP COLUMN IF EXISTS timezone,
DROP COLUMN IF EXISTS quiet_hours_end,
DROP COLUMN IF EXISTS quiet_hours_start;
