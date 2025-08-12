-- Firebase-Native Minimal Schema Migration
-- This migration simplifies the authentication schema to be Firebase-first
-- with minimal database storage as per the approved architecture plan

-- Drop complex multi-provider tables that duplicate Firebase data
DROP TABLE IF EXISTS firebase_user_mappings CASCADE;
DROP TABLE IF EXISTS provider_user_attributes CASCADE;
DROP TABLE IF EXISTS oauth_provider_configs CASCADE;

-- The existing users table already has the minimal structure we need:
-- users(id, firebase_uid, email, created_at, updated_at)
-- No changes needed to users table - it's already Firebase-native

-- Simplify sessions to minimal Firebase token references only
-- Replace complex unified_sessions with simple Firebase-validated sessions
DROP TABLE IF EXISTS unified_sessions CASCADE;

CREATE TABLE IF NOT EXISTS firebase_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL, -- Direct Firebase reference (no local user FK)
    session_token VARCHAR(255) UNIQUE NOT NULL, -- Our internal session token
    firebase_token_id VARCHAR(255) NOT NULL, -- Firebase ID token JTI for validation
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ DEFAULT NOW(),
    user_agent TEXT,
    ip_address INET,
    is_active BOOLEAN DEFAULT true
);

-- Indexes for efficient session management
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_firebase_uid ON firebase_sessions(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_token ON firebase_sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_firebase_token_id ON firebase_sessions(firebase_token_id);
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_expires_at ON firebase_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_active ON firebase_sessions(is_active) WHERE is_active = true;

-- Firebase token validation cache (for performance)
CREATE TABLE IF NOT EXISTS firebase_token_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id VARCHAR(255) UNIQUE NOT NULL, -- Firebase token JTI
    firebase_uid VARCHAR(128) NOT NULL,
    token_data JSONB NOT NULL, -- Cached Firebase user data and claims
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for token cache
CREATE INDEX IF NOT EXISTS idx_firebase_token_cache_token_id ON firebase_token_cache(token_id);
CREATE INDEX IF NOT EXISTS idx_firebase_token_cache_firebase_uid ON firebase_token_cache(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_firebase_token_cache_expires_at ON firebase_token_cache(expires_at);

-- User roles and permissions (stored in database, not Firebase custom claims)
CREATE TABLE IF NOT EXISTS user_roles_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL, -- Direct Firebase reference
    role VARCHAR(100) NOT NULL DEFAULT 'user-basic-001', -- IAM role (user-basic-001, admin-full-004, etc.)
    permissions TEXT[] DEFAULT '{}', -- Array of permission strings
    access_level VARCHAR(50) DEFAULT 'none', -- none, standard, full, super
    is_admin BOOLEAN DEFAULT FALSE,
    is_premium BOOLEAN DEFAULT FALSE,
    role_assigned_by VARCHAR(128), -- Firebase UID of admin who assigned role
    role_assigned_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- Optional role expiration
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Unique constraint to prevent duplicate roles per Firebase user
    CONSTRAINT unique_firebase_uid_roles UNIQUE (firebase_uid)
);

-- Indexes for role and permission lookups
CREATE INDEX IF NOT EXISTS idx_user_roles_firebase_uid ON user_roles_permissions(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_user_roles_role ON user_roles_permissions(role);
CREATE INDEX IF NOT EXISTS idx_user_roles_is_admin ON user_roles_permissions(is_admin) WHERE is_admin = true;
CREATE INDEX IF NOT EXISTS idx_user_roles_expires_at ON user_roles_permissions(expires_at) WHERE expires_at IS NOT NULL;

-- Role assignment audit log
CREATE TABLE IF NOT EXISTS role_assignment_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL, -- User whose role was changed
    old_role VARCHAR(100), -- Previous role
    new_role VARCHAR(100) NOT NULL, -- New role assigned
    assigned_by VARCHAR(128), -- Firebase UID of admin who made change
    reason TEXT, -- Optional reason for role change
    metadata JSONB DEFAULT '{}', -- Additional context
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Index for audit queries
CREATE INDEX IF NOT EXISTS idx_role_audit_firebase_uid ON role_assignment_audit(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_role_audit_assigned_by ON role_assignment_audit(assigned_by);
CREATE INDEX IF NOT EXISTS idx_role_audit_timestamp ON role_assignment_audit(timestamp);

-- Application-specific data only (not user profiles - those stay in Firebase)
CREATE TABLE IF NOT EXISTS user_app_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL, -- Direct Firebase reference
    subscription_tier VARCHAR(50),
    trading_preferences JSONB DEFAULT '{}',
    notification_settings JSONB DEFAULT '{}',
    app_metadata JSONB DEFAULT '{}', -- Application-specific metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Unique constraint to prevent duplicate app data per Firebase user
    CONSTRAINT unique_firebase_uid_app_data UNIQUE (firebase_uid)
);

-- Index for app data lookups
CREATE INDEX IF NOT EXISTS idx_user_app_data_firebase_uid ON user_app_data(firebase_uid);

-- Simplify auth audit log to Firebase-native approach
-- Replace complex auth_audit_log with Firebase-focused audit
DROP TABLE IF EXISTS auth_audit_log CASCADE;

CREATE TABLE IF NOT EXISTS firebase_auth_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128), -- Can be null for failed authentication attempts
    event_type VARCHAR(50) NOT NULL, -- 'firebase_login', 'firebase_logout', 'session_created', 'session_expired'
    session_id UUID REFERENCES firebase_sessions(id) ON DELETE SET NULL,
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    firebase_metadata JSONB DEFAULT '{}', -- Firebase-specific event data
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for audit queries
CREATE INDEX IF NOT EXISTS idx_firebase_auth_audit_firebase_uid ON firebase_auth_audit(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_firebase_auth_audit_timestamp ON firebase_auth_audit(timestamp);
CREATE INDEX IF NOT EXISTS idx_firebase_auth_audit_event_type ON firebase_auth_audit(event_type);
CREATE INDEX IF NOT EXISTS idx_firebase_auth_audit_success ON firebase_auth_audit(success) WHERE success = false;

-- Keep revoked_tokens table but simplify for Firebase tokens only
-- Update existing revoked_tokens table to be Firebase-specific
DROP TABLE IF EXISTS revoked_tokens CASCADE;

CREATE TABLE IF NOT EXISTS firebase_revoked_tokens (
    firebase_token_id VARCHAR(255) PRIMARY KEY, -- Firebase token JTI
    firebase_uid VARCHAR(128) NOT NULL,
    session_id UUID, -- Reference to firebase_sessions if applicable
    token_type VARCHAR(50) NOT NULL DEFAULT 'firebase_id_token',
    issued_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ DEFAULT NOW(),
    revocation_reason VARCHAR(100) -- 'logout', 'security', 'expired', 'firebase_revoked'
);

-- Index for efficient revocation checks
CREATE INDEX IF NOT EXISTS idx_firebase_revoked_tokens_token_id ON firebase_revoked_tokens(firebase_token_id);
CREATE INDEX IF NOT EXISTS idx_firebase_revoked_tokens_firebase_uid ON firebase_revoked_tokens(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_firebase_revoked_tokens_expires_at ON firebase_revoked_tokens(expires_at);

-- Add comments for documentation
COMMENT ON TABLE firebase_sessions IS 'Minimal session management with Firebase token validation';
COMMENT ON TABLE firebase_token_cache IS 'Performance cache for Firebase token validation results';
COMMENT ON TABLE user_app_data IS 'Application-specific data only - user profiles stay in Firebase';
COMMENT ON TABLE firebase_auth_audit IS 'Firebase-native authentication event audit log';
COMMENT ON TABLE firebase_revoked_tokens IS 'Tracks revoked Firebase tokens for security';

-- Cleanup queries that should be run periodically:
-- DELETE FROM firebase_sessions WHERE expires_at < NOW() - INTERVAL '1 day';
-- DELETE FROM firebase_token_cache WHERE expires_at < NOW();
-- DELETE FROM firebase_revoked_tokens WHERE expires_at < NOW() - INTERVAL '7 days';
-- DELETE FROM firebase_auth_audit WHERE timestamp < NOW() - INTERVAL '90 days';

-- Migration notes:
-- 1. All user profile data (name, email, roles, permissions) will be queried from Firebase directly
-- 2. Database only stores session tokens and application-specific preferences
-- 3. No password storage - all authentication handled by Firebase
-- 4. Firebase UIDs are used as primary user identifiers throughout the system