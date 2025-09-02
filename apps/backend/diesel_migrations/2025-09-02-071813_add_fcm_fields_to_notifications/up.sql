-- Add FCM-related fields to notifications table
ALTER TABLE notifications 
ADD COLUMN fcm_sent BOOLEAN DEFAULT false,
ADD COLUMN fcm_message_id TEXT,
ADD COLUMN fcm_delivered_at TIMESTAMPTZ,
ADD COLUMN fcm_failed_reason TEXT,
ADD COLUMN delivery_attempts INTEGER DEFAULT 0,
ADD COLUMN fcm_data JSONB DEFAULT '{}';

-- Add index for FCM delivery status queries
CREATE INDEX idx_notifications_fcm_sent ON notifications(fcm_sent);
CREATE INDEX idx_notifications_fcm_delivered_at ON notifications(fcm_delivered_at);

-- Add comment for documentation
COMMENT ON COLUMN notifications.fcm_sent IS 'Whether FCM push notification was sent';
COMMENT ON COLUMN notifications.fcm_message_id IS 'Firebase FCM message ID for tracking';
COMMENT ON COLUMN notifications.fcm_delivered_at IS 'Timestamp when FCM push was delivered';
COMMENT ON COLUMN notifications.fcm_failed_reason IS 'Reason if FCM push failed';
COMMENT ON COLUMN notifications.delivery_attempts IS 'Number of FCM delivery attempts';
COMMENT ON COLUMN notifications.fcm_data IS 'Additional FCM-specific data and metadata';
