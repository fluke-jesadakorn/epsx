-- Fresh notification system schema
-- Use DO block to create types only if they don't exist
DO $$ BEGIN
    -- Create notification_priority if not exists
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'notification_priority') THEN
        CREATE TYPE notification_priority AS ENUM ('low', 'normal', 'high', 'urgent');
    END IF;
    
    -- Create notification_type if not exists
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'notification_type') THEN
        CREATE TYPE notification_type AS ENUM ('system', 'admin', 'security', 'feature', 'marketing');
    END IF;
    
    -- delivery_channel and delivery_status already exist, skip them
END $$;

-- FCM Topics for broadcasting
CREATE TABLE fcm_topics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(200) NOT NULL,
    description TEXT,
    target_permissions JSONB, -- ["admin:*:*", "epsx:premium:*"]
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- User FCM tokens
CREATE TABLE fcm_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    token TEXT NOT NULL UNIQUE,
    platform VARCHAR(20) NOT NULL, -- 'web', 'android', 'ios'
    device_info JSONB,
    topics JSONB DEFAULT '[]'::jsonb, -- subscribed topics
    is_active BOOLEAN DEFAULT true,
    last_used_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Notifications
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_user_id UUID REFERENCES users(id), -- null for broadcast
    fcm_topic_id UUID REFERENCES fcm_topics(id), -- for broadcasts
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    notification_type notification_type NOT NULL,
    priority notification_priority NOT NULL,
    channels delivery_channel[] NOT NULL,
    data_payload JSONB,
    image_url TEXT,
    action_url TEXT,
    scheduled_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Delivery tracking
CREATE TABLE notification_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id),
    user_id UUID NOT NULL REFERENCES users(id),
    channel delivery_channel NOT NULL,
    status delivery_status NOT NULL DEFAULT 'pending',
    fcm_message_id TEXT, -- Firebase message ID
    error_message TEXT,
    delivered_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    clicked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- User notification preferences
CREATE TABLE user_notification_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    fcm_enabled BOOLEAN DEFAULT true,
    in_app_enabled BOOLEAN DEFAULT true,
    email_enabled BOOLEAN DEFAULT false,
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    timezone VARCHAR(50) DEFAULT 'UTC',
    blocked_topics JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Performance indexes
CREATE INDEX idx_notifications_recipient ON notifications(recipient_user_id, created_at DESC) WHERE recipient_user_id IS NOT NULL;
CREATE INDEX idx_notifications_topic ON notifications(fcm_topic_id, created_at DESC) WHERE fcm_topic_id IS NOT NULL;
CREATE INDEX idx_notification_deliveries_user ON notification_deliveries(user_id, created_at DESC);
CREATE INDEX idx_fcm_tokens_user_active ON fcm_tokens(user_id) WHERE is_active = true;
CREATE INDEX idx_notifications_scheduled ON notifications(scheduled_at) WHERE scheduled_at IS NOT NULL AND scheduled_at > NOW();

-- Create default topics
INSERT INTO fcm_topics (name, display_name, description, target_permissions) VALUES
('epsx_all_users', 'All Users', 'Notifications for all platform users', '["epsx:*:*"]'),
('epsx_admin_users', 'Admin Users', 'Notifications for administrators', '["admin:*:*"]'),
('epsx_premium_users', 'Premium Users', 'Notifications for premium subscribers', '["epsx:premium:*"]'),
('epsx_analytics_users', 'Analytics Users', 'Notifications for analytics users', '["epsx:analytics:*"]');