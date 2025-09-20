-- Stateless Notification System Migration
-- Replaces real-time FCM push notifications with database-stored notifications

-- 1. Create stateless notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Content
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    notification_type VARCHAR(50) NOT NULL, -- 'info', 'warning', 'error', 'success', 'security'
    category VARCHAR(50), -- 'account', 'trading', 'system', 'security', 'promotion'
    
    -- Priority and urgency
    priority VARCHAR(20) DEFAULT 'normal', -- 'low', 'normal', 'high', 'urgent'
    is_urgent BOOLEAN DEFAULT false,
    
    -- Status tracking
    is_read BOOLEAN DEFAULT false,
    read_at TIMESTAMPTZ,
    is_archived BOOLEAN DEFAULT false,
    archived_at TIMESTAMPTZ,
    
    -- Metadata
    data JSONB, -- Additional structured data
    action_url VARCHAR(500), -- Optional action URL
    action_label VARCHAR(100), -- Optional action button label
    icon VARCHAR(100), -- Optional icon identifier
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ -- Optional expiry for temporary notifications
);

-- Indexes for efficient querying
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, is_read) WHERE is_read = false;
CREATE INDEX idx_notifications_user_unarchived ON notifications(user_id, is_archived) WHERE is_archived = false;
CREATE INDEX idx_notifications_type ON notifications(notification_type);
CREATE INDEX idx_notifications_category ON notifications(category);
CREATE INDEX idx_notifications_priority ON notifications(priority);
CREATE INDEX idx_notifications_urgent ON notifications(is_urgent) WHERE is_urgent = true;
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_expires ON notifications(expires_at) WHERE expires_at IS NOT NULL;

-- Updated trigger for notification updates
CREATE TRIGGER update_notifications_updated_at BEFORE UPDATE ON notifications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 2. Create notification preferences table
CREATE TABLE notification_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Global preferences
    notifications_enabled BOOLEAN DEFAULT true,
    email_notifications BOOLEAN DEFAULT true,
    
    -- Category preferences
    account_notifications BOOLEAN DEFAULT true,
    trading_notifications BOOLEAN DEFAULT true,
    system_notifications BOOLEAN DEFAULT true,
    security_notifications BOOLEAN DEFAULT true,
    promotion_notifications BOOLEAN DEFAULT false,
    
    -- Priority filtering
    minimum_priority VARCHAR(20) DEFAULT 'normal', -- Only show notifications >= this priority
    urgent_only BOOLEAN DEFAULT false, -- Only show urgent notifications
    
    -- Cleanup preferences
    auto_archive_days INTEGER DEFAULT 30, -- Auto-archive read notifications after N days
    auto_delete_days INTEGER DEFAULT 90, -- Auto-delete archived notifications after N days
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(user_id)
);

CREATE INDEX idx_notification_preferences_user ON notification_preferences(user_id);

CREATE TRIGGER update_notification_preferences_updated_at BEFORE UPDATE ON notification_preferences
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 3. Create notification delivery log for audit
CREATE TABLE notification_delivery_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    delivery_method VARCHAR(20) NOT NULL, -- 'database', 'email', 'webhook'
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'delivered', 'failed', 'skipped'
    error_message TEXT,
    attempts INTEGER DEFAULT 0,
    delivered_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_notification_delivery_log_notification ON notification_delivery_log(notification_id);
CREATE INDEX idx_notification_delivery_log_status ON notification_delivery_log(status);
CREATE INDEX idx_notification_delivery_log_method ON notification_delivery_log(delivery_method);

CREATE TRIGGER update_notification_delivery_log_updated_at BEFORE UPDATE ON notification_delivery_log
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 4. Create cleanup function for expired and old notifications
CREATE OR REPLACE FUNCTION cleanup_notifications()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
    archived_count INTEGER := 0;
BEGIN
    -- Delete expired notifications
    DELETE FROM notifications 
    WHERE expires_at IS NOT NULL AND expires_at < CURRENT_TIMESTAMP;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Auto-archive old read notifications based on user preferences
    UPDATE notifications 
    SET is_archived = true, archived_at = CURRENT_TIMESTAMP
    FROM notification_preferences np
    WHERE notifications.user_id = np.user_id
      AND notifications.is_read = true
      AND notifications.is_archived = false
      AND notifications.read_at < (CURRENT_TIMESTAMP - INTERVAL '1 day' * np.auto_archive_days);
    
    GET DIAGNOSTICS archived_count = ROW_COUNT;
    
    -- Delete old archived notifications based on user preferences
    DELETE FROM notifications 
    USING notification_preferences np
    WHERE notifications.user_id = np.user_id
      AND notifications.is_archived = true
      AND notifications.archived_at < (CURRENT_TIMESTAMP - INTERVAL '1 day' * np.auto_delete_days);
    
    GET DIAGNOSTICS deleted_count = deleted_count + ROW_COUNT;
    
    RETURN deleted_count + archived_count;
END;
$$ LANGUAGE plpgsql;

-- 5. Helper functions for notification management
CREATE OR REPLACE FUNCTION get_user_notification_count(
    p_user_id UUID,
    p_unread_only BOOLEAN DEFAULT false
)
RETURNS INTEGER AS $$
DECLARE
    count_result INTEGER;
BEGIN
    IF p_unread_only THEN
        SELECT COUNT(*)
        INTO count_result
        FROM notifications n
        JOIN notification_preferences np ON n.user_id = np.user_id
        WHERE n.user_id = p_user_id
          AND n.is_read = false
          AND n.is_archived = false
          AND (n.expires_at IS NULL OR n.expires_at > CURRENT_TIMESTAMP)
          AND (
              np.notifications_enabled = true
              AND (
                  (n.category = 'account' AND np.account_notifications = true) OR
                  (n.category = 'trading' AND np.trading_notifications = true) OR
                  (n.category = 'system' AND np.system_notifications = true) OR
                  (n.category = 'security' AND np.security_notifications = true) OR
                  (n.category = 'promotion' AND np.promotion_notifications = true) OR
                  n.category IS NULL
              )
              AND (
                  (np.urgent_only = false) OR 
                  (np.urgent_only = true AND n.is_urgent = true)
              )
          );
    ELSE
        SELECT COUNT(*)
        INTO count_result
        FROM notifications n
        JOIN notification_preferences np ON n.user_id = np.user_id
        WHERE n.user_id = p_user_id
          AND n.is_archived = false
          AND (n.expires_at IS NULL OR n.expires_at > CURRENT_TIMESTAMP)
          AND (
              np.notifications_enabled = true
              AND (
                  (n.category = 'account' AND np.account_notifications = true) OR
                  (n.category = 'trading' AND np.trading_notifications = true) OR
                  (n.category = 'system' AND np.system_notifications = true) OR
                  (n.category = 'security' AND np.security_notifications = true) OR
                  (n.category = 'promotion' AND np.promotion_notifications = true) OR
                  n.category IS NULL
              )
              AND (
                  (np.urgent_only = false) OR 
                  (np.urgent_only = true AND n.is_urgent = true)
              )
          );
    END IF;
    
    RETURN COALESCE(count_result, 0);
END;
$$ LANGUAGE plpgsql;

-- 6. Function to mark notifications as read
CREATE OR REPLACE FUNCTION mark_notifications_read(
    p_user_id UUID,
    p_notification_ids UUID[] DEFAULT NULL
)
RETURNS INTEGER AS $$
DECLARE
    updated_count INTEGER;
BEGIN
    IF p_notification_ids IS NULL THEN
        -- Mark all unread notifications as read
        UPDATE notifications 
        SET is_read = true, read_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
        WHERE user_id = p_user_id 
          AND is_read = false;
    ELSE
        -- Mark specific notifications as read
        UPDATE notifications 
        SET is_read = true, read_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
        WHERE user_id = p_user_id 
          AND id = ANY(p_notification_ids) 
          AND is_read = false;
    END IF;
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RETURN updated_count;
END;
$$ LANGUAGE plpgsql;

-- 7. Create default notification preferences for existing users
INSERT INTO notification_preferences (user_id)
SELECT id FROM users 
WHERE id NOT IN (SELECT user_id FROM notification_preferences);

-- 8. Add comments for documentation
COMMENT ON TABLE notifications IS 'Stateless notification system - replaces real-time FCM with database storage';
COMMENT ON TABLE notification_preferences IS 'User preferences for notification filtering and cleanup';
COMMENT ON TABLE notification_delivery_log IS 'Audit log for notification delivery attempts';
COMMENT ON COLUMN notifications.data IS 'Additional structured data in JSON format';
COMMENT ON COLUMN notifications.expires_at IS 'Optional expiry timestamp for temporary notifications';
COMMENT ON COLUMN notification_preferences.minimum_priority IS 'Only show notifications with priority >= this level';
COMMENT ON COLUMN notification_preferences.urgent_only IS 'If true, only show urgent notifications regardless of other settings';