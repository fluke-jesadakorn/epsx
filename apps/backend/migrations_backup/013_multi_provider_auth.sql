-- Multi-Provider Authentication System Migration
-- This migration adds tables for supporting multiple authentication providers
-- while maintaining compatibility with existing Firebase-based authentication

-- Firebase UID to Backend user_id mapping table
-- This allows multiple Firebase UIDs to map to the same user (account linking)
CREATE TABLE IF NOT EXISTS firebase_user_mappings (
    firebase_uid VARCHAR(128) PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    provider_type VARCHAR(50) NOT NULL DEFAULT 'firebase', -- 'firebase', 'google', 'github', etc.
    is_primary BOOLEAN NOT NULL DEFAULT true, -- Primary authentication method for the user
    email_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    
    -- Ensure one primary provider per user
    CONSTRAINT unique_primary_per_user UNIQUE (user_id, is_primary) DEFERRABLE INITIALLY DEFERRED
);

-- Index for efficient user lookups
CREATE INDEX IF NOT EXISTS idx_firebase_user_mappings_user_id ON firebase_user_mappings(user_id);
CREATE INDEX IF NOT EXISTS idx_firebase_user_mappings_email ON firebase_user_mappings(email);
CREATE INDEX IF NOT EXISTS idx_firebase_user_mappings_provider_type ON firebase_user_mappings(provider_type);

-- Multi-provider unified sessions table
-- Tracks sessions across different authentication providers
CREATE TABLE IF NOT EXISTS unified_sessions (
    session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_type VARCHAR(50) NOT NULL, -- 'firebase', 'oidc', 'custom'
    provider_user_id VARCHAR(255) NOT NULL, -- Original provider user ID (Firebase UID, OIDC sub, etc.)
    unified_jwt_jti VARCHAR(255) UNIQUE, -- JWT ID for token revocation
    access_token_hash VARCHAR(255), -- Hash of the access token for revocation
    refresh_token_hash VARCHAR(255), -- Hash of refresh token if applicable
    session_metadata JSONB DEFAULT '{}', -- Additional session data (IP, user agent, etc.)
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,
    
    -- Composite index for efficient session validation
    CONSTRAINT unique_active_provider_session UNIQUE (user_id, provider_type, is_active) DEFERRABLE INITIALLY DEFERRED
);

-- Indexes for efficient session management
CREATE INDEX IF NOT EXISTS idx_unified_sessions_user_id ON unified_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_unified_sessions_jti ON unified_sessions(unified_jwt_jti);
CREATE INDEX IF NOT EXISTS idx_unified_sessions_expires_at ON unified_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_unified_sessions_active ON unified_sessions(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_unified_sessions_provider ON unified_sessions(provider_type, provider_user_id);

-- OAuth provider configurations table
-- Stores configuration for different OAuth providers
CREATE TABLE IF NOT EXISTS oauth_provider_configs (
    provider_id VARCHAR(50) PRIMARY KEY,
    provider_name VARCHAR(100) NOT NULL,
    provider_type VARCHAR(50) NOT NULL, -- 'oidc', 'oauth2', 'saml'
    client_id VARCHAR(255),
    client_secret_hash VARCHAR(255), -- Encrypted/hashed client secret
    authorization_endpoint TEXT,
    token_endpoint TEXT,
    userinfo_endpoint TEXT,
    jwks_uri TEXT,
    issuer VARCHAR(255),
    scopes JSONB DEFAULT '["openid", "profile", "email"]',
    additional_config JSONB DEFAULT '{}', -- Provider-specific configuration
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default configurations
INSERT INTO oauth_provider_configs (provider_id, provider_name, provider_type, is_active) VALUES
('firebase', 'Firebase Authentication', 'firebase', true),
('oidc', 'Internal OIDC Provider', 'oidc', true)
ON CONFLICT (provider_id) DO NOTHING;

-- Provider-specific user attributes table
-- Stores additional attributes from different providers
CREATE TABLE IF NOT EXISTS provider_user_attributes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_type VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    attribute_name VARCHAR(100) NOT NULL,
    attribute_value JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Unique constraint to prevent duplicate attributes per user per provider
    CONSTRAINT unique_user_provider_attribute UNIQUE (user_id, provider_type, attribute_name)
);

-- Index for efficient attribute lookups
CREATE INDEX IF NOT EXISTS idx_provider_user_attributes_user_id ON provider_user_attributes(user_id);
CREATE INDEX IF NOT EXISTS idx_provider_user_attributes_provider ON provider_user_attributes(provider_type, provider_user_id);

-- Authentication audit log table
-- Tracks authentication events for security monitoring
CREATE TABLE IF NOT EXISTS auth_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    provider_type VARCHAR(50) NOT NULL,
    event_type VARCHAR(50) NOT NULL, -- 'login', 'logout', 'token_refresh', 'token_revoked', 'failed_login'
    session_id UUID REFERENCES unified_sessions(session_id) ON DELETE SET NULL,
    ip_address INET,
    user_agent TEXT,
    location JSONB, -- Geolocation data if available
    success BOOLEAN NOT NULL,
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for audit log queries
CREATE INDEX IF NOT EXISTS idx_auth_audit_log_user_id ON auth_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_auth_audit_log_timestamp ON auth_audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_auth_audit_log_event_type ON auth_audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_auth_audit_log_provider ON auth_audit_log(provider_type);
CREATE INDEX IF NOT EXISTS idx_auth_audit_log_success ON auth_audit_log(success) WHERE success = false;

-- Token revocation list table
-- Tracks revoked JWT tokens to prevent replay attacks
CREATE TABLE IF NOT EXISTS revoked_tokens (
    jti VARCHAR(255) PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token_type VARCHAR(50) NOT NULL, -- 'access', 'refresh'
    issued_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ DEFAULT NOW(),
    revocation_reason VARCHAR(100), -- 'logout', 'security', 'expired'
    
    -- Automatically clean up expired tokens
    CONSTRAINT check_expires_future CHECK (expires_at > issued_at)
);

-- Index for efficient revocation checks
CREATE INDEX IF NOT EXISTS idx_revoked_tokens_jti ON revoked_tokens(jti);
CREATE INDEX IF NOT EXISTS idx_revoked_tokens_expires_at ON revoked_tokens(expires_at);

-- Note: PL/pgSQL functions and triggers can be added later via separate migrations or manual setup
-- For now, we'll skip the functions to avoid SQLx parsing issues

-- Add a note about manual cleanup that can be scheduled
-- Cleanup queries that should be run periodically:
-- DELETE FROM unified_sessions WHERE expires_at < NOW() - INTERVAL '1 day';
-- DELETE FROM revoked_tokens WHERE expires_at < NOW() - INTERVAL '7 days'; 
-- DELETE FROM auth_audit_log WHERE timestamp < NOW() - INTERVAL '90 days';

-- Add comments for documentation
COMMENT ON TABLE firebase_user_mappings IS 'Maps Firebase UIDs to backend user IDs, supports multiple providers per user';
COMMENT ON TABLE unified_sessions IS 'Tracks authentication sessions across multiple providers with unified JWT tokens';
COMMENT ON TABLE oauth_provider_configs IS 'Configuration for different OAuth/OIDC providers';
COMMENT ON TABLE provider_user_attributes IS 'Stores provider-specific user attributes and claims';
COMMENT ON TABLE auth_audit_log IS 'Comprehensive audit log for all authentication events';
COMMENT ON TABLE revoked_tokens IS 'Tracks revoked JWT tokens to prevent replay attacks';

-- Functions for periodic cleanup and session management can be added later

-- Grant appropriate permissions (adjust based on your application user)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO your_app_user;
-- GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO your_app_user;