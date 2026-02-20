-- Support Chat System
-- Tables: chat_topics, chat_conversations, chat_messages

CREATE TABLE IF NOT EXISTS chat_topics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    label VARCHAR(100) NOT NULL,
    description TEXT,
    icon VARCHAR(50),
    sort_order INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS chat_conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic_id UUID NOT NULL REFERENCES chat_topics(id),
    wallet_address VARCHAR(42) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    assigned_agent VARCHAR(42),
    last_message_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    unread_user INT NOT NULL DEFAULT 0,
    unread_agent INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES chat_conversations(id) ON DELETE CASCADE,
    sender_type VARCHAR(10) NOT NULL, -- user, agent, system, ai
    sender_address VARCHAR(42),
    content TEXT NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_chat_conversations_wallet ON chat_conversations(wallet_address);
CREATE INDEX idx_chat_conversations_status ON chat_conversations(status);
CREATE INDEX idx_chat_conversations_agent ON chat_conversations(assigned_agent);
CREATE INDEX idx_chat_conversations_last_msg ON chat_conversations(last_message_at DESC);
CREATE INDEX idx_chat_messages_conversation ON chat_messages(conversation_id, created_at);
CREATE INDEX idx_chat_messages_sender ON chat_messages(sender_address);

-- Seed default topics
INSERT INTO chat_topics (name, label, description, icon, sort_order) VALUES
    ('general', 'General', 'General questions and inquiries', 'message-circle', 0),
    ('billing', 'Billing', 'Payment and subscription issues', 'credit-card', 1),
    ('account', 'Account', 'Account and wallet management', 'user', 2),
    ('analytics', 'Analytics', 'Data and analytics questions', 'bar-chart', 3),
    ('bug', 'Bug Report', 'Report a bug or technical issue', 'bug', 4),
    ('feature', 'Feature Request', 'Suggest a new feature', 'lightbulb', 5);
