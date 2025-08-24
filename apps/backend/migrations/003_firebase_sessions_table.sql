-- Add missing Firebase sessions table for authentication
-- Based on the usage patterns found in firebase_session_service.rs

CREATE TABLE IF NOT EXISTS firebase_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL,
    session_token TEXT UNIQUE NOT NULL,
    firebase_token_id TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    user_agent TEXT,
    ip_address INET,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ DEFAULT NOW()
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_firebase_uid ON firebase_sessions(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_session_token ON firebase_sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_expires_at ON firebase_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_is_active ON firebase_sessions(is_active) WHERE is_active = true;

-- Cleanup expired sessions function
CREATE OR REPLACE FUNCTION cleanup_expired_firebase_sessions()
RETURNS INTEGER
LANGUAGE plpgsql
AS $BODY$
DECLARE
    affected_rows INTEGER;
BEGIN
    UPDATE firebase_sessions SET is_active = false 
    WHERE expires_at <= NOW() AND is_active = true;
    
    GET DIAGNOSTICS affected_rows = ROW_COUNT;
    RETURN affected_rows;
END;
$BODY$;

COMMENT ON TABLE firebase_sessions IS 'Firebase authentication session management';