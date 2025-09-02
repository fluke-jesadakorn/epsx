-- Create separate session tables for admin and user contexts
-- This migration supports the separated JWT architecture with optimized storage per context

-- Admin sessions table with enhanced security tracking
CREATE TABLE admin_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(255) NOT NULL UNIQUE,
    user_id UUID NOT NULL,
    email VARCHAR(255) NOT NULL,
    
    -- JWT token information
    access_token_hash VARCHAR(255) NOT NULL, -- SHA-256 hash for security
    id_token_hash VARCHAR(255),
    refresh_token_hash VARCHAR(255),
    
    -- Session metadata
    issued_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_activity TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Security context
    ip_address INET,
    user_agent TEXT,
    device_fingerprint VARCHAR(255),
    mfa_verified BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Admin-specific context
    admin_permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
    security_level VARCHAR(50) NOT NULL DEFAULT 'standard',
    audit_context JSONB,
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMP WITH TIME ZONE,
    revoke_reason VARCHAR(255),
    
    -- Foreign key constraints
    CONSTRAINT fk_admin_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- User sessions table with performance optimization
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(255) NOT NULL UNIQUE,
    user_id UUID NOT NULL,
    email VARCHAR(255) NOT NULL,
    
    -- JWT token information (lighter storage)
    access_token_hash VARCHAR(255) NOT NULL,
    refresh_token_hash VARCHAR(255),
    
    -- Session metadata
    issued_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_activity TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Lightweight security context
    ip_address INET,
    user_agent TEXT,
    
    -- User-specific context (optimized)
    user_permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
    package_tier VARCHAR(50) NOT NULL DEFAULT 'FREE',
    platform_context VARCHAR(50) NOT NULL DEFAULT 'epsx',
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMP WITH TIME ZONE,
    
    -- Foreign key constraints
    CONSTRAINT fk_user_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for optimal performance

-- Admin sessions indexes (security-focused)
CREATE INDEX idx_admin_sessions_user_id ON admin_sessions(user_id);
CREATE INDEX idx_admin_sessions_session_id ON admin_sessions(session_id);
CREATE INDEX idx_admin_sessions_email ON admin_sessions(email);
CREATE INDEX idx_admin_sessions_expires_at ON admin_sessions(expires_at);
CREATE INDEX idx_admin_sessions_last_activity ON admin_sessions(last_activity);
CREATE INDEX idx_admin_sessions_ip_address ON admin_sessions(ip_address);
CREATE INDEX idx_admin_sessions_revoked_at ON admin_sessions(revoked_at) WHERE revoked_at IS NULL;

-- GIN index for admin permissions (supports efficient permission queries)
CREATE INDEX idx_admin_sessions_permissions_gin ON admin_sessions USING GIN(admin_permissions);

-- User sessions indexes (performance-focused)
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_session_id ON user_sessions(session_id);
CREATE INDEX idx_user_sessions_email ON user_sessions(email);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX idx_user_sessions_last_activity ON user_sessions(last_activity);
CREATE INDEX idx_user_sessions_package_tier ON user_sessions(package_tier);
CREATE INDEX idx_user_sessions_platform_context ON user_sessions(platform_context);
CREATE INDEX idx_user_sessions_revoked_at ON user_sessions(revoked_at) WHERE revoked_at IS NULL;

-- Composite index for efficient session cleanup
CREATE INDEX idx_user_sessions_cleanup ON user_sessions(expires_at, revoked_at, last_activity);

-- GIN index for user permissions (lighter than admin)
CREATE INDEX idx_user_sessions_permissions_gin ON user_sessions USING GIN(user_permissions);

-- Add triggers for automatic updated_at maintenance
CREATE OR REPLACE FUNCTION update_session_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER admin_sessions_updated_at
    BEFORE UPDATE ON admin_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_session_updated_at();

CREATE TRIGGER user_sessions_updated_at
    BEFORE UPDATE ON user_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_session_updated_at();

-- Add session cleanup function for expired sessions
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
BEGIN
    -- Clean up expired admin sessions
    DELETE FROM admin_sessions 
    WHERE expires_at < NOW() - INTERVAL '1 hour'
       OR (revoked_at IS NOT NULL AND revoked_at < NOW() - INTERVAL '24 hours')
       OR last_activity < NOW() - INTERVAL '7 days';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Clean up expired user sessions
    DELETE FROM user_sessions 
    WHERE expires_at < NOW() - INTERVAL '1 hour'
       OR (revoked_at IS NOT NULL AND revoked_at < NOW() - INTERVAL '24 hours')
       OR last_activity < NOW() - INTERVAL '30 days';
    
    GET DIAGNOSTICS deleted_count = deleted_count + ROW_COUNT;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE admin_sessions IS 'Admin sessions with enhanced security tracking and audit trails';
COMMENT ON TABLE user_sessions IS 'User sessions optimized for performance and lightweight storage';
COMMENT ON COLUMN admin_sessions.security_level IS 'Security level: standard, high, critical';
COMMENT ON COLUMN admin_sessions.mfa_verified IS 'Whether MFA was verified for this session';
COMMENT ON COLUMN admin_sessions.device_fingerprint IS 'Unique device identification for security';
COMMENT ON COLUMN user_sessions.platform_context IS 'Platform context: epsx, epsx-pay, epsx-token';
COMMENT ON FUNCTION cleanup_expired_sessions() IS 'Cleanup function for expired sessions - run periodically';
