-- Create comprehensive notification system tables

-- Main notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    notification_type VARCHAR NOT NULL, -- 'system', 'admin', 'data', 'feature', 'security'
    priority VARCHAR NOT NULL, -- 'low', 'normal', 'high', 'urgent'
    sender_type VARCHAR NOT NULL, -- 'system', 'admin', 'automated'
    image_url VARCHAR,
    action_url VARCHAR,
    custom_data JSONB,
    fcm_topic VARCHAR, -- 'epsx_all_users', 'epsx_admin_users', etc.
    target_user_id VARCHAR, -- for direct user notifications
    channels TEXT[] DEFAULT '{fcm,in_app}', -- notification channels
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    created_by_user_id VARCHAR -- admin who created it
);

-- User-specific notification tracking
CREATE TABLE user_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR NOT NULL, -- maps to firebase_uid from users table
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    read_at TIMESTAMP WITH TIME ZONE,
    clicked_at TIMESTAMP WITH TIME ZONE,
    dismissed_at TIMESTAMP WITH TIME ZONE,
    delivered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    delivery_status VARCHAR DEFAULT 'delivered', -- 'pending', 'delivered', 'failed'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User notification preferences
CREATE TABLE user_notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR NOT NULL UNIQUE, -- maps to firebase_uid
    fcm_enabled BOOLEAN DEFAULT true,
    in_app_enabled BOOLEAN DEFAULT true,
    email_enabled BOOLEAN DEFAULT false,
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    timezone VARCHAR DEFAULT 'UTC',
    blocked_topics TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- FCM tokens storage
CREATE TABLE fcm_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR NOT NULL, -- maps to firebase_uid
    token VARCHAR NOT NULL UNIQUE,
    platform VARCHAR NOT NULL, -- 'ios', 'android', 'web'
    device_info JSONB,
    subscribed_topics TEXT[] DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance optimization
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_type ON notifications(notification_type);
CREATE INDEX idx_notifications_priority ON notifications(priority);
CREATE INDEX idx_notifications_fcm_topic ON notifications(fcm_topic);

CREATE INDEX idx_user_notifications_user_id ON user_notifications(user_id);
CREATE INDEX idx_user_notifications_read ON user_notifications(user_id, read_at);
CREATE INDEX idx_user_notifications_delivered ON user_notifications(delivery_status, delivered_at);
CREATE INDEX idx_user_notifications_notification_id ON user_notifications(notification_id);

CREATE INDEX idx_user_prefs_user_id ON user_notification_preferences(user_id);

CREATE INDEX idx_fcm_tokens_user_id ON fcm_tokens(user_id);
CREATE INDEX idx_fcm_tokens_active ON fcm_tokens(user_id, is_active);
CREATE INDEX idx_fcm_tokens_platform ON fcm_tokens(platform);

-- Insert initial test data for info@epsx.io
INSERT INTO notifications (title, body, notification_type, priority, sender_type, fcm_topic, created_at) VALUES
('🎉 Welcome to EPSX Analytics', 'Your account is now active with full access to market data insights and real-time analytics.', 'system', 'normal', 'system', 'epsx_all_users', NOW() - INTERVAL '2 days'),
('📊 Weekly Data Report Ready', 'Your personalized analytics report for this week is now available in the dashboard.', 'data', 'normal', 'automated', 'epsx_all_users', NOW() - INTERVAL '1 day'),
('🔒 Security Alert: New Login', 'New login detected from Chrome on macOS in San Francisco, CA. If this wasn''t you, secure your account immediately.', 'security', 'high', 'system', NULL, NOW() - INTERVAL '3 hours'),
('✨ New Feature: Advanced Charts', 'Enhanced charting tools with technical indicators are now available in your analytics dashboard.', 'feature', 'normal', 'admin', 'epsx_all_users', NOW() - INTERVAL '6 hours'),
('⚠️ System Maintenance Notice', 'Scheduled maintenance tonight 2-4 AM UTC. Services may be briefly unavailable during this window.', 'system', 'high', 'admin', 'epsx_all_users', NOW() - INTERVAL '12 hours'),
('📈 Market Alert: High Volatility', 'Significant market movement detected in your watchlist. Review your positions and risk management.', 'data', 'urgent', 'automated', 'epsx_all_users', NOW() - INTERVAL '30 minutes'),
('🔐 Two-Factor Authentication Enabled', 'Your account security has been enhanced with two-factor authentication. Keep your backup codes safe.', 'security', 'normal', 'system', NULL, NOW() - INTERVAL '4 hours'),
('💰 Premium Features Unlocked', 'Congratulations! You now have access to advanced analytics, real-time data, and priority support.', 'admin', 'normal', 'admin', NULL, NOW() - INTERVAL '8 hours');

-- Link notifications to info@epsx.io user with realistic read states
INSERT INTO user_notifications (user_id, notification_id, read_at, clicked_at, delivery_status) 
SELECT 
    'info@epsx.io', 
    id, 
    CASE 
        WHEN notification_type = 'system' AND priority = 'high' THEN created_at + INTERVAL '5 minutes'
        WHEN notification_type = 'feature' THEN created_at + INTERVAL '15 minutes'  
        WHEN notification_type = 'admin' THEN created_at + INTERVAL '1 hour'
        ELSE NULL -- Keep security and recent notifications unread
    END,
    CASE 
        WHEN notification_type = 'feature' THEN created_at + INTERVAL '20 minutes'
        WHEN notification_type = 'admin' THEN created_at + INTERVAL '1 hour 5 minutes'
        ELSE NULL
    END,
    'delivered'
FROM notifications;

-- Set up notification preferences for test user
INSERT INTO user_notification_preferences (user_id, fcm_enabled, in_app_enabled, email_enabled, blocked_topics, timezone)
VALUES ('info@epsx.io', true, true, false, '{marketing}', 'America/New_York');

-- Add FCM token for test user  
INSERT INTO fcm_tokens (user_id, token, platform, subscribed_topics, device_info)
VALUES ('info@epsx.io', 'test-fcm-token-info-epsx-web-' || extract(epoch from now())::text, 'web', '{epsx_all_users,epsx_admin_users}', '{"browser": "Chrome", "os": "macOS", "version": "119.0.0.0"}');