-- Create OpenID Connect refresh tokens table
-- Stores refresh tokens for OpenID Connect token renewal after Web3 authentication

CREATE TABLE IF NOT EXISTS openid_refresh_tokens (
    token_id VARCHAR(36) PRIMARY KEY,  -- UUID v4 string
    wallet_address VARCHAR(42) NOT NULL, -- Associated wallet address
    expires_at TIMESTAMPTZ NOT NULL,     -- Token expiration time
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL, -- Token creation time
    is_revoked BOOLEAN DEFAULT FALSE NOT NULL,      -- Revocation status
    
    -- Foreign key constraint to wallet_users
    CONSTRAINT fk_openid_refresh_tokens_wallet_address 
        FOREIGN KEY (wallet_address) 
        REFERENCES wallet_users(wallet_address) 
        ON DELETE CASCADE,
    
    -- Ensure wallet address format is valid
    CONSTRAINT valid_wallet_address_format 
        CHECK (wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42)
);

-- Index for efficient cleanup of expired tokens
CREATE INDEX IF NOT EXISTS idx_openid_refresh_tokens_expires_at 
    ON openid_refresh_tokens(expires_at);

-- Index for quick lookup by wallet address
CREATE INDEX IF NOT EXISTS idx_openid_refresh_tokens_wallet_address 
    ON openid_refresh_tokens(wallet_address);

-- Index for active tokens (not revoked and not expired)
CREATE INDEX IF NOT EXISTS idx_openid_refresh_tokens_active 
    ON openid_refresh_tokens(wallet_address, is_revoked, expires_at)
    WHERE is_revoked = FALSE;

-- Comments for documentation
COMMENT ON TABLE openid_refresh_tokens IS 'OpenID Connect refresh tokens for Web3-authenticated users';
COMMENT ON COLUMN openid_refresh_tokens.token_id IS 'Unique refresh token identifier (UUID)';
COMMENT ON COLUMN openid_refresh_tokens.wallet_address IS 'Web3 wallet address of the token owner';
COMMENT ON COLUMN openid_refresh_tokens.expires_at IS 'When this refresh token expires';
COMMENT ON COLUMN openid_refresh_tokens.created_at IS 'When this refresh token was created';
COMMENT ON COLUMN openid_refresh_tokens.is_revoked IS 'Whether this refresh token has been revoked';

-- Function to automatically clean up expired refresh tokens
CREATE OR REPLACE FUNCTION cleanup_expired_openid_refresh_tokens()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM openid_refresh_tokens 
    WHERE expires_at < NOW() OR is_revoked = TRUE;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Create a scheduled cleanup job (can be called by cron or app scheduler)
COMMENT ON FUNCTION cleanup_expired_openid_refresh_tokens() IS 'Cleans up expired and revoked OpenID refresh tokens';