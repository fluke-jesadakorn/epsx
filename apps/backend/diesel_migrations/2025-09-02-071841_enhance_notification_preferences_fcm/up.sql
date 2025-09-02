-- Add FCM-related preferences to notification_preferences table (skip push_enabled as it exists)
ALTER TABLE notification_preferences 
ADD COLUMN IF NOT EXISTS quiet_hours_start TIME,
ADD COLUMN IF NOT EXISTS quiet_hours_end TIME,
ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) DEFAULT 'UTC',
ADD COLUMN IF NOT EXISTS fcm_topics JSONB DEFAULT '[]',
ADD COLUMN IF NOT EXISTS platform_preferences JSONB DEFAULT '{"web": true, "android": true, "ios": true}';

-- Add index for timezone queries (for scheduling)
CREATE INDEX IF NOT EXISTS idx_notification_preferences_timezone ON notification_preferences(timezone);

-- Add comments for documentation
COMMENT ON COLUMN notification_preferences.push_enabled IS 'Whether FCM push notifications are enabled for user';
COMMENT ON COLUMN notification_preferences.quiet_hours_start IS 'Start time for quiet hours (no push notifications)';
COMMENT ON COLUMN notification_preferences.quiet_hours_end IS 'End time for quiet hours (no push notifications)';
COMMENT ON COLUMN notification_preferences.timezone IS 'User timezone for scheduling notifications';
COMMENT ON COLUMN notification_preferences.fcm_topics IS 'Array of FCM topics user is subscribed to';
COMMENT ON COLUMN notification_preferences.platform_preferences IS 'Per-platform notification preferences';
