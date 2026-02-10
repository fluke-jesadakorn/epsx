DROP TABLE IF EXISTS wallet_notifications;

CREATE TABLE wallet_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_wallet_address VARCHAR(42),
    topic_name VARCHAR(255),
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    urgency VARCHAR(20) NOT NULL DEFAULT 'normal',
    notification_type VARCHAR(50) NOT NULL,
    priority VARCHAR(20) NOT NULL DEFAULT 'normal',
    channels JSONB NOT NULL DEFAULT '{}',
    schedule_type VARCHAR(20) NOT NULL DEFAULT 'immediate',
    scheduled_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) NOT NULL DEFAULT 'created',
    send_started_at TIMESTAMP WITH TIME ZONE,
    channel_status JSONB NOT NULL DEFAULT '{}',
    total_attempts INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(42),
    image_url TEXT,
    action_url TEXT,
    data_payload JSONB,
    tags TEXT[],
    notes TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_recipient ON wallet_notifications(recipient_wallet_address);
CREATE INDEX idx_notifications_status ON wallet_notifications(status);
CREATE INDEX idx_notifications_created_at ON wallet_notifications(created_at);
