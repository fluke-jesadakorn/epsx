-- Create revoked_tokens table for JTI blacklist system
CREATE TABLE revoked_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jti TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    token_type TEXT NOT NULL, -- 'access_token', 'refresh_token', 'id_token'
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_by TEXT,
    revoked_reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for fast JTI lookup
CREATE INDEX idx_revoked_tokens_jti ON revoked_tokens(jti);
CREATE INDEX idx_revoked_tokens_user_id ON revoked_tokens(user_id);
CREATE INDEX idx_revoked_tokens_expires_at ON revoked_tokens(expires_at);
CREATE INDEX idx_revoked_tokens_token_type ON revoked_tokens(token_type);
CREATE INDEX idx_revoked_tokens_revoked_at ON revoked_tokens(revoked_at);

-- Composite index for cleanup queries
CREATE INDEX idx_revoked_tokens_cleanup ON revoked_tokens(expires_at, revoked_at);

-- Add foreign key constraint to users table
ALTER TABLE revoked_tokens ADD CONSTRAINT fk_revoked_tokens_user 
    FOREIGN KEY (user_id) REFERENCES users(firebase_uid) ON DELETE CASCADE;
