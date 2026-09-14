CREATE TABLE IF NOT EXISTS admin_chat_operations (
    operation_id UUID PRIMARY KEY,
    idempotency_key VARCHAR(128) NOT NULL UNIQUE,
    conversation_id UUID NOT NULL,
    action VARCHAR(32) NOT NULL,
    actor VARCHAR(128) NOT NULL,
    result JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_admin_chat_operations_conversation
    ON admin_chat_operations (conversation_id, created_at DESC);
