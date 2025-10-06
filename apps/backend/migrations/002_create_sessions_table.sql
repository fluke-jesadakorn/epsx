-- ================================================================================================
-- SESSIONS TABLE MIGRATION
-- ================================================================================================
-- Creates the sessions table for Web3 wallet session management
-- Tracks active user sessions with access tokens, refresh tokens, and security metadata
--
-- Version: 1.0.0
-- Created: 2025-01-05
-- ================================================================================================

-- ------------------------------------------------------------------------------------------------
-- SESSIONS TABLE - Session management for authenticated wallets
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS sessions (
    -- Identity
    id UUID PRIMARY KEY,
    wallet_address VARCHAR(42) NOT NULL,

    -- Session tokens
    access_token TEXT NOT NULL,
    refresh_token TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    last_accessed_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,

    -- Security metadata
    ip_address VARCHAR(45),  -- IPv4 (15 chars) or IPv6 (45 chars)
    user_agent TEXT,
    is_revoked BOOLEAN DEFAULT FALSE NOT NULL,

    -- Aggregate versioning for optimistic concurrency
    version BIGINT DEFAULT 1 NOT NULL,

    -- Constraints
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    ),
    CONSTRAINT access_token_not_empty CHECK (LENGTH(TRIM(access_token)) > 0),
    CONSTRAINT expires_at_future CHECK (expires_at > created_at),
    CONSTRAINT version_positive CHECK (version > 0)
);

-- Foreign key to wallet_users
ALTER TABLE sessions
    ADD CONSTRAINT sessions_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

-- ------------------------------------------------------------------------------------------------
-- INDEXES
-- ------------------------------------------------------------------------------------------------

-- Primary lookup by session ID (already indexed via PRIMARY KEY)

-- Lookup by wallet address (most common query pattern)
CREATE INDEX idx_sessions_wallet_address ON sessions(wallet_address, is_revoked, expires_at)
    WHERE is_revoked = FALSE;

-- Lookup by access token (for token validation)
CREATE INDEX idx_sessions_access_token ON sessions(access_token)
    WHERE is_revoked = FALSE;

-- Lookup by refresh token (for token refresh)
CREATE INDEX idx_sessions_refresh_token ON sessions(refresh_token)
    WHERE refresh_token IS NOT NULL AND is_revoked = FALSE;

-- Cleanup expired sessions
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at)
    WHERE is_revoked = FALSE;

-- Active sessions per wallet
CREATE INDEX idx_sessions_active ON sessions(wallet_address, is_revoked, expires_at, last_accessed_at)
    WHERE is_revoked = FALSE AND expires_at > NOW();

-- Audit and analytics
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX idx_sessions_last_accessed ON sessions(last_accessed_at DESC)
    WHERE is_revoked = FALSE;

-- IP address tracking (for security monitoring)
CREATE INDEX idx_sessions_ip_address ON sessions(ip_address, wallet_address)
    WHERE ip_address IS NOT NULL AND is_revoked = FALSE;

-- ------------------------------------------------------------------------------------------------
-- UPDATE TIMESTAMP TRIGGER
-- ------------------------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION trigger_update_sessions_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_sessions_updated_at ON sessions;
CREATE TRIGGER trigger_sessions_updated_at
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_sessions_timestamp();

-- ------------------------------------------------------------------------------------------------
-- COMMENTS
-- ------------------------------------------------------------------------------------------------

COMMENT ON TABLE sessions IS 'Active user sessions for Web3-authenticated wallets with token management and security tracking';
COMMENT ON COLUMN sessions.id IS 'Unique session identifier (UUID)';
COMMENT ON COLUMN sessions.wallet_address IS 'Web3 wallet address of the session owner';
COMMENT ON COLUMN sessions.access_token IS 'JWT access token for API authentication';
COMMENT ON COLUMN sessions.refresh_token IS 'Optional refresh token for session renewal';
COMMENT ON COLUMN sessions.created_at IS 'When this session was created';
COMMENT ON COLUMN sessions.updated_at IS 'When this session was last updated';
COMMENT ON COLUMN sessions.expires_at IS 'When this session expires';
COMMENT ON COLUMN sessions.last_accessed_at IS 'When this session was last accessed';
COMMENT ON COLUMN sessions.ip_address IS 'IP address of the client (IPv4 or IPv6)';
COMMENT ON COLUMN sessions.user_agent IS 'Browser/client user agent string';
COMMENT ON COLUMN sessions.is_revoked IS 'Whether this session has been revoked/invalidated';
COMMENT ON COLUMN sessions.version IS 'Version number for optimistic concurrency control';

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'SESSIONS TABLE CREATED SUCCESSFULLY! 🎉';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Created:';
    RAISE NOTICE '  ✅ sessions table with full security metadata';
    RAISE NOTICE '  ✅ Foreign key constraint to wallet_users';
    RAISE NOTICE '  ✅ 9 performance indexes for common query patterns';
    RAISE NOTICE '  ✅ Update timestamp trigger';
    RAISE NOTICE '  ✅ Optimistic concurrency support via version column';
    RAISE NOTICE '';
    RAISE NOTICE '🎯 Session management ready for Web3-First Platform!';
    RAISE NOTICE '=================================================================================';
END $$;
