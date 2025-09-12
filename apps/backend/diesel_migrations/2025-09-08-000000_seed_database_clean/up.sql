-- ============================================================================
-- EPSX Database Complete Seed Migration
-- Consolidates all migrations into a single clean seed
-- Creates complete schema with RBAC, OIDC, notifications, and analytics
-- ============================================================================

-- ============================================================================
-- CUSTOM TYPES AND ENUMS
-- ============================================================================

-- Notification system enums
CREATE TYPE notification_type AS ENUM ('system', 'admin', 'user', 'security', 'marketing', 'feature', 'data');
CREATE TYPE notification_priority AS ENUM ('low', 'normal', 'high', 'urgent');
CREATE TYPE delivery_channel AS ENUM ('fcm', 'email', 'in_app', 'sms', 'webhook');
CREATE TYPE delivery_status AS ENUM ('pending', 'sent', 'delivered', 'failed', 'bounced');

-- RBAC system enums
CREATE TYPE permission_type_enum AS ENUM ('grant', 'revoke');
CREATE TYPE audit_operation_enum AS ENUM (
    'grant_role', 'revoke_role', 'extend_role', 
    'grant_permission', 'revoke_permission', 'extend_permission',
    'create_role', 'update_role', 'delete_role',
    'create_permission', 'update_permission', 'delete_permission',
    'bulk_grant', 'bulk_revoke', 'migration', 'cleanup'
);
CREATE TYPE limit_type_enum AS ENUM (
    'ranking_view', 'api_requests_minute', 'api_requests_hour', 
    'api_requests_day', 'export_requests_day', 'advanced_features'
);

-- ============================================================================
-- CORE USER MANAGEMENT
-- ============================================================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255),
    name VARCHAR(255),
    avatar_url TEXT,
    package_tier VARCHAR(50) DEFAULT 'free',
    email_verified BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    primary_platform_id UUID
);

CREATE INDEX idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = true;
CREATE INDEX idx_users_package_tier ON users(package_tier);

-- ============================================================================
-- SESSION MANAGEMENT
-- ============================================================================

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    provider VARCHAR(50),
    session_token TEXT,
    user_agent TEXT,
    ip_address INET,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_active ON sessions(user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_sessions_access_token ON sessions(access_token);

-- ============================================================================
-- REFRESH TOKEN MANAGEMENT
-- ============================================================================

CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    family_id UUID NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    device_info JSONB,
    ip_address INET,
    user_agent TEXT,
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    revoked_at TIMESTAMPTZ,
    revoked_reason TEXT
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_family_id ON refresh_tokens(family_id);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
CREATE INDEX idx_refresh_tokens_active ON refresh_tokens(user_id, is_revoked) WHERE NOT is_revoked;

CREATE TABLE revoked_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jti TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    token_type TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_by TEXT,
    revoked_reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_revoked_tokens_jti ON revoked_tokens(jti);
CREATE INDEX idx_revoked_tokens_user_id ON revoked_tokens(user_id);
CREATE INDEX idx_revoked_tokens_expires_at ON revoked_tokens(expires_at);

-- ============================================================================
-- RBAC PERMISSION SYSTEM
-- ============================================================================

-- Core permission definitions (master list)
CREATE TABLE rbac_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL, -- "epsx:analytics:view"
    platform VARCHAR(50) NOT NULL,     -- "epsx", "admin", "epsx-pay" 
    resource VARCHAR(50) NOT NULL,      -- "analytics", "users", "rankings"
    action VARCHAR(50) NOT NULL,        -- "view", "manage", "export"
    description TEXT,
    metadata JSONB DEFAULT '{}',
    is_system_permission BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(platform, resource, action)
);

-- Role definitions (permission groups)
CREATE TABLE rbac_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) UNIQUE NOT NULL,     -- "bronze_user", "admin", "premium"
    display_name VARCHAR(100) NOT NULL,   -- "Bronze Subscriber"
    description TEXT,
    platform_scope VARCHAR(50),           -- "epsx", "admin", "*" for cross-platform
    metadata JSONB DEFAULT '{}',
    is_system_role BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Role-permission mapping (M:N)
CREATE TABLE rbac_role_permissions (
    role_id UUID NOT NULL REFERENCES rbac_roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES rbac_permissions(id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    granted_by UUID REFERENCES users(id),
    metadata JSONB DEFAULT '{}',
    
    PRIMARY KEY (role_id, permission_id)
);

-- User-role assignments (M:N with temporal support)
CREATE TABLE rbac_user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES rbac_roles(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NULL,              -- NULL = permanent role
    is_active BOOLEAN DEFAULT true,
    reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT rbac_user_roles_check CHECK (expires_at IS NULL OR expires_at > created_at)
);

-- Direct user permissions (overrides/additions to roles)
CREATE TABLE rbac_user_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES rbac_permissions(id) ON DELETE CASCADE,
    permission_type permission_type_enum NOT NULL, -- 'grant' or 'revoke'
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NULL,               -- NULL = permanent
    is_active BOOLEAN DEFAULT true,
    reason TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT rbac_user_permissions_check CHECK (expires_at IS NULL OR expires_at > created_at)
);

-- Comprehensive audit trail
CREATE TABLE rbac_permission_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation audit_operation_enum NOT NULL,
    actor_user_id UUID REFERENCES users(id),
    target_user_id UUID REFERENCES users(id) NULL,
    role_id UUID REFERENCES rbac_roles(id) NULL,
    permission_id UUID REFERENCES rbac_permissions(id) NULL,
    old_values JSONB NULL,
    new_values JSONB NULL,
    expires_at TIMESTAMPTZ NULL,
    reason TEXT,
    ip_address INET,
    user_agent TEXT,
    session_id UUID,
    request_id VARCHAR(255),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Dynamic user limits (admin-controlled overrides)
CREATE TABLE rbac_user_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    limit_type limit_type_enum NOT NULL,
    limit_value BIGINT NOT NULL,              -- -1 = unlimited
    granted_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    priority INTEGER DEFAULT 0,               -- Higher priority wins conflicts
    expires_at TIMESTAMPTZ NULL,              -- NULL = permanent
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(user_id, limit_type, priority)
);

-- ============================================================================
-- LEGACY USER PERMISSIONS (for backward compatibility)
-- ============================================================================

CREATE TABLE user_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(255) NOT NULL,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    granted_by UUID REFERENCES users(id),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE user_dynamic_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ranking_limit INTEGER,
    requests_per_minute INTEGER,
    requests_per_hour INTEGER,
    requests_per_day INTEGER,
    api_endpoints TEXT[],
    assigned_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    priority INTEGER NOT NULL,
    effective_from TIMESTAMP NOT NULL,
    expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    previous_limits JSONB,
    change_source VARCHAR(50) NOT NULL
);

-- ============================================================================
-- NOTIFICATION SYSTEM
-- ============================================================================

CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    fcm_topic_id UUID,
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    notification_type notification_type NOT NULL,
    priority notification_priority NOT NULL,
    channels delivery_channel[] NOT NULL DEFAULT '{fcm,in_app}',
    data_payload JSONB,
    image_url TEXT,
    action_url TEXT,
    scheduled_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE user_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR NOT NULL, -- Firebase UID
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    read_at TIMESTAMPTZ,
    clicked_at TIMESTAMPTZ,
    dismissed_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ DEFAULT NOW(),
    delivery_status VARCHAR DEFAULT 'delivered',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE notification_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel delivery_channel NOT NULL,
    status delivery_status NOT NULL,
    fcm_message_id TEXT,
    error_message TEXT,
    delivered_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    clicked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE user_notification_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    fcm_enabled BOOLEAN DEFAULT true,
    in_app_enabled BOOLEAN DEFAULT true,
    email_enabled BOOLEAN DEFAULT false,
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    timezone VARCHAR(50) DEFAULT 'UTC',
    blocked_topics JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- FCM TOKEN MANAGEMENT
-- ============================================================================

CREATE TABLE fcm_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    platform VARCHAR(20) NOT NULL,
    device_info JSONB,
    topics JSONB,
    is_active BOOLEAN DEFAULT true,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE fcm_topics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(200) NOT NULL,
    description TEXT,
    target_permissions JSONB,
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- SECURITY AND AUDIT LOGGING
-- ============================================================================

CREATE TABLE security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    user_id VARCHAR(255),
    ip_address INET,
    request_path TEXT,
    event_data JSONB,
    risk_score INTEGER,
    blocked BOOLEAN,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES users(id),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    result VARCHAR(50),
    severity VARCHAR(20),
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id UUID REFERENCES sessions(id),
    platform_id UUID
);

-- ============================================================================
-- EPS ANALYTICS
-- ============================================================================

CREATE TABLE eps_growth_analytics (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    name VARCHAR(100) NOT NULL,
    country VARCHAR(50) NOT NULL,
    sector VARCHAR(100),
    exchange VARCHAR(50),
    current_eps DECIMAL,
    qoq_growth_rate DECIMAL,
    price_current DECIMAL,
    market_cap BIGINT,
    volume BIGINT,
    ranking_score DECIMAL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- INDEXES FOR PERFORMANCE
-- ============================================================================

-- RBAC Indexes
CREATE INDEX idx_rbac_permissions_platform_resource_action ON rbac_permissions(platform, resource, action);
CREATE INDEX idx_rbac_permissions_active ON rbac_permissions(is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_permissions_platform ON rbac_permissions(platform);

CREATE INDEX idx_rbac_roles_active ON rbac_roles(is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_roles_platform_scope ON rbac_roles(platform_scope);
CREATE INDEX idx_rbac_roles_system ON rbac_roles(is_system_role);

CREATE INDEX idx_rbac_role_permissions_role ON rbac_role_permissions(role_id);
CREATE INDEX idx_rbac_role_permissions_permission ON rbac_role_permissions(permission_id);

CREATE INDEX idx_rbac_user_roles_user_active ON rbac_user_roles(user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_user_roles_expires_at ON rbac_user_roles(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_rbac_user_roles_active_valid ON rbac_user_roles(user_id, expires_at, is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_user_roles_expiring_soon ON rbac_user_roles(expires_at, user_id) WHERE expires_at IS NOT NULL;

CREATE INDEX idx_rbac_user_permissions_user_active ON rbac_user_permissions(user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_user_permissions_expires_at ON rbac_user_permissions(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_rbac_user_permissions_type ON rbac_user_permissions(permission_type);
CREATE INDEX idx_rbac_user_permissions_validation ON rbac_user_permissions(user_id, permission_id, permission_type, is_active, expires_at);

CREATE INDEX idx_rbac_audit_log_actor ON rbac_permission_audit_log(actor_user_id, created_at DESC);
CREATE INDEX idx_rbac_audit_log_target ON rbac_permission_audit_log(target_user_id, created_at DESC);
CREATE INDEX idx_rbac_audit_log_operation ON rbac_permission_audit_log(operation, created_at DESC);
CREATE INDEX idx_rbac_audit_log_created_at ON rbac_permission_audit_log(created_at DESC);

CREATE INDEX idx_rbac_user_limits_user_active ON rbac_user_limits(user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_user_limits_type ON rbac_user_limits(limit_type);
CREATE INDEX idx_rbac_user_limits_expires_at ON rbac_user_limits(expires_at) WHERE expires_at IS NOT NULL;

-- Legacy permission indexes
CREATE INDEX idx_user_permissions_user_id ON user_permissions(user_id);
CREATE INDEX idx_user_permissions_active ON user_permissions(user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_user_permissions_expires_at ON user_permissions(expires_at) WHERE expires_at IS NOT NULL;

-- Notification indexes
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_type ON notifications(notification_type);
CREATE INDEX idx_notifications_priority ON notifications(priority);
CREATE INDEX idx_notifications_recipient ON notifications(recipient_user_id);

CREATE INDEX idx_user_notifications_user_id ON user_notifications(user_id);
CREATE INDEX idx_user_notifications_read ON user_notifications(user_id, read_at);
CREATE INDEX idx_user_notifications_delivered ON user_notifications(delivery_status, delivered_at);
CREATE INDEX idx_user_notifications_notification_id ON user_notifications(notification_id);

CREATE INDEX idx_notification_deliveries_user_id ON notification_deliveries(user_id);
CREATE INDEX idx_notification_deliveries_status ON notification_deliveries(status);

-- FCM indexes
CREATE INDEX idx_fcm_tokens_user_id ON fcm_tokens(user_id);
CREATE INDEX idx_fcm_tokens_active ON fcm_tokens(user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_fcm_tokens_platform ON fcm_tokens(platform);

CREATE INDEX idx_fcm_topics_active ON fcm_topics(is_active) WHERE is_active = true;

-- Security indexes
CREATE INDEX idx_security_events_timestamp ON security_events(timestamp DESC);
CREATE INDEX idx_security_events_user_id ON security_events(user_id);
CREATE INDEX idx_security_events_severity ON security_events(severity);
CREATE INDEX idx_security_events_event_type ON security_events(event_type);

CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);

-- Analytics indexes
CREATE INDEX idx_eps_analytics_symbol ON eps_growth_analytics(symbol);
CREATE INDEX idx_eps_analytics_country ON eps_growth_analytics(country);
CREATE INDEX idx_eps_analytics_ranking_score ON eps_growth_analytics(ranking_score DESC);
CREATE INDEX idx_eps_analytics_created_at ON eps_growth_analytics(created_at DESC);

-- ============================================================================
-- FUNCTIONS AND PROCEDURES
-- ============================================================================

-- RBAC permission checking function
CREATE OR REPLACE FUNCTION rbac_user_has_permission(
    target_user_id UUID,
    permission_platform TEXT,
    permission_resource TEXT, 
    permission_action TEXT
) RETURNS BOOLEAN AS $$
DECLARE
    has_perm BOOLEAN := false;
    perm_id UUID;
BEGIN
    -- Get permission ID
    SELECT id INTO perm_id FROM rbac_permissions p
    WHERE p.platform = permission_platform 
      AND p.resource = permission_resource 
      AND p.action = permission_action
      AND p.is_active = true;
      
    IF perm_id IS NULL THEN
        RETURN false;
    END IF;
    
    -- Check if user has permission through roles (and not revoked)
    SELECT EXISTS(
        SELECT 1 
        FROM rbac_user_roles ur
        JOIN rbac_role_permissions rp ON ur.role_id = rp.role_id
        JOIN rbac_roles r ON ur.role_id = r.id
        WHERE ur.user_id = target_user_id
          AND rp.permission_id = perm_id
          AND ur.is_active = true
          AND r.is_active = true
          AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
          -- Check if not revoked by direct permission
          AND NOT EXISTS (
              SELECT 1 FROM rbac_user_permissions up
              WHERE up.user_id = target_user_id
                AND up.permission_id = perm_id
                AND up.permission_type = 'revoke'
                AND up.is_active = true
                AND (up.expires_at IS NULL OR up.expires_at > NOW())
          )
    ) INTO has_perm;
    
    -- If not granted through roles, check direct grant
    IF NOT has_perm THEN
        SELECT EXISTS(
            SELECT 1 FROM rbac_user_permissions up
            WHERE up.user_id = target_user_id
              AND up.permission_id = perm_id
              AND up.permission_type = 'grant'
              AND up.is_active = true
              AND (up.expires_at IS NULL OR up.expires_at > NOW())
        ) INTO has_perm;
    END IF;
    
    -- Check wildcard permissions
    IF NOT has_perm THEN
        SELECT EXISTS(
            SELECT 1 
            FROM rbac_user_roles ur
            JOIN rbac_role_permissions rp ON ur.role_id = rp.role_id
            JOIN rbac_roles r ON ur.role_id = r.id
            JOIN rbac_permissions p ON rp.permission_id = p.id
            WHERE ur.user_id = target_user_id
              AND ur.is_active = true
              AND r.is_active = true
              AND p.is_active = true
              AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
              AND (
                  -- Wildcard matching
                  (p.platform = permission_platform AND p.resource = permission_resource AND p.action = '*') OR
                  (p.platform = permission_platform AND p.resource = '*' AND p.action = '*') OR
                  (p.platform = '*' AND p.resource = '*' AND p.action = '*')
              )
              -- Check if not revoked
              AND NOT EXISTS (
                  SELECT 1 FROM rbac_user_permissions up
                  WHERE up.user_id = target_user_id
                    AND up.permission_id = p.id
                    AND up.permission_type = 'revoke'
                    AND up.is_active = true
                    AND (up.expires_at IS NULL OR up.expires_at > NOW())
              )
        ) INTO has_perm;
    END IF;
    
    RETURN has_perm;
END;
$$ LANGUAGE plpgsql;

-- RBAC cleanup function
CREATE OR REPLACE FUNCTION rbac_cleanup_expired_permissions() RETURNS TABLE(
    expired_roles BIGINT,
    expired_permissions BIGINT,
    expired_limits BIGINT
) AS $$
DECLARE
    expired_role_count BIGINT;
    expired_perm_count BIGINT;
    expired_limit_count BIGINT;
BEGIN
    -- Deactivate expired user roles
    WITH expired_roles AS (
        UPDATE rbac_user_roles 
        SET is_active = false, updated_at = NOW()
        WHERE is_active = true 
          AND expires_at IS NOT NULL 
          AND expires_at <= NOW()
        RETURNING id
    )
    SELECT COUNT(*) INTO expired_role_count FROM expired_roles;
    
    -- Deactivate expired user permissions  
    WITH expired_perms AS (
        UPDATE rbac_user_permissions
        SET is_active = false, updated_at = NOW()
        WHERE is_active = true
          AND expires_at IS NOT NULL
          AND expires_at <= NOW()
        RETURNING id
    )
    SELECT COUNT(*) INTO expired_perm_count FROM expired_perms;
    
    -- Deactivate expired user limits
    WITH expired_limits AS (
        UPDATE rbac_user_limits
        SET is_active = false, updated_at = NOW()
        WHERE is_active = true
          AND expires_at IS NOT NULL
          AND expires_at <= NOW()
        RETURNING id
    )
    SELECT COUNT(*) INTO expired_limit_count FROM expired_limits;
    
    -- Log cleanup operation
    INSERT INTO rbac_permission_audit_log (
        operation, actor_user_id, reason, metadata
    ) VALUES (
        'cleanup', 
        NULL,
        'Automatic cleanup of expired permissions',
        jsonb_build_object(
            'expired_roles', expired_role_count,
            'expired_permissions', expired_perm_count, 
            'expired_limits', expired_limit_count,
            'cleanup_time', NOW()
        )
    );
    
    RETURN QUERY SELECT expired_role_count, expired_perm_count, expired_limit_count;
END;
$$ LANGUAGE plpgsql;

-- Token cleanup function
CREATE OR REPLACE FUNCTION cleanup_expired_tokens(older_than_hours INTEGER DEFAULT 24)
RETURNS TABLE(deleted_count BIGINT) AS $$
DECLARE
    cutoff_time TIMESTAMPTZ;
    result_count BIGINT;
BEGIN
    cutoff_time := NOW() - (older_than_hours || ' hours')::INTERVAL;
    
    -- Delete expired refresh tokens
    WITH deleted AS (
        DELETE FROM refresh_tokens 
        WHERE expires_at < cutoff_time
        RETURNING id
    )
    SELECT COUNT(*) INTO result_count FROM deleted;
    
    -- Delete old revoked tokens
    DELETE FROM revoked_tokens 
    WHERE expires_at < cutoff_time;
    
    RETURN QUERY SELECT result_count;
END;
$$ LANGUAGE plpgsql;

-- Enable updated_at trigger management
SELECT diesel_manage_updated_at('users');
SELECT diesel_manage_updated_at('sessions');
SELECT diesel_manage_updated_at('refresh_tokens');
SELECT diesel_manage_updated_at('rbac_permissions');
SELECT diesel_manage_updated_at('rbac_roles');
SELECT diesel_manage_updated_at('rbac_user_roles');
SELECT diesel_manage_updated_at('rbac_user_permissions');
SELECT diesel_manage_updated_at('rbac_user_limits');
SELECT diesel_manage_updated_at('user_permissions');
SELECT diesel_manage_updated_at('user_dynamic_limits');
SELECT diesel_manage_updated_at('user_notification_preferences');
SELECT diesel_manage_updated_at('fcm_tokens');
SELECT diesel_manage_updated_at('fcm_topics');
SELECT diesel_manage_updated_at('eps_growth_analytics');

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- System permissions
INSERT INTO rbac_permissions (name, platform, resource, action, description, is_system_permission) VALUES
-- Admin permissions
('admin:*:*', 'admin', '*', '*', 'Full administrative access', true),
('admin:users:view', 'admin', 'users', 'view', 'View user information', true),
('admin:users:manage', 'admin', 'users', 'manage', 'Manage users (create, update, delete)', true),
('admin:permissions:view', 'admin', 'permissions', 'view', 'View permission information', true),
('admin:permissions:manage', 'admin', 'permissions', 'manage', 'Manage permissions and roles', true),
('admin:analytics:view', 'admin', 'analytics', 'view', 'View admin analytics', true),
('admin:system:manage', 'admin', 'system', 'manage', 'System administration', true),

-- EPSX platform permissions
('epsx:rankings:view:5', 'epsx', 'rankings', 'view:5', 'View top 5 rankings (Bronze)', true),
('epsx:rankings:view:25', 'epsx', 'rankings', 'view:25', 'View top 25 rankings (Silver)', true), 
('epsx:rankings:view:50', 'epsx', 'rankings', 'view:50', 'View top 50 rankings (Gold)', true),
('epsx:rankings:view:100', 'epsx', 'rankings', 'view:100', 'View top 100 rankings (Platinum)', true),
('epsx:rankings:view:unlimited', 'epsx', 'rankings', 'view:unlimited', 'View unlimited rankings (Enterprise)', true),
('epsx:analytics:view', 'epsx', 'analytics', 'view', 'View analytics dashboard', true),
('epsx:analytics:export', 'epsx', 'analytics', 'export', 'Export analytics data', true),
('epsx:analytics:premium', 'epsx', 'analytics', 'premium', 'Access premium analytics features', true),
('epsx:trading:basic', 'epsx', 'trading', 'basic', 'Basic trading features', true),
('epsx:trading:advanced', 'epsx', 'trading', 'advanced', 'Advanced trading features', true),
('epsx:trading:premium', 'epsx', 'trading', 'premium', 'Premium trading features', true),
('epsx:portfolio:view', 'epsx', 'portfolio', 'view', 'View portfolio data', true),
('epsx:portfolio:manage', 'epsx', 'portfolio', 'manage', 'Manage portfolio settings', true),
('epsx:notifications:receive', 'epsx', 'notifications', 'receive', 'Receive notifications', true),
('epsx:notifications:manage', 'epsx', 'notifications', 'manage', 'Manage notification preferences', true),

-- Payment platform permissions  
('epsx-pay:*:*', 'epsx-pay', '*', '*', 'Full payment platform access', true),
('epsx-pay:payments:view', 'epsx-pay', 'payments', 'view', 'View payment information', true),
('epsx-pay:payments:process', 'epsx-pay', 'payments', 'process', 'Process payments', true),

-- Token platform permissions
('epsx-token:*:*', 'epsx-token', '*', '*', 'Full token platform access', true),
('epsx-token:tokens:view', 'epsx-token', 'tokens', 'view', 'View token information', true),
('epsx-token:tokens:trade', 'epsx-token', 'tokens', 'trade', 'Trade tokens', true);

-- System roles
INSERT INTO rbac_roles (name, display_name, description, platform_scope, is_system_role) VALUES
('super_admin', 'Super Administrator', 'Full system access across all platforms', '*', true),
('admin', 'Administrator', 'Administrative access to admin platform', 'admin', true),
('bronze_user', 'Bronze Subscriber', 'Basic EPSX platform access', 'epsx', true),
('silver_user', 'Silver Subscriber', 'Enhanced EPSX platform access', 'epsx', true),
('gold_user', 'Gold Subscriber', 'Premium EPSX platform access', 'epsx', true),
('platinum_user', 'Platinum Subscriber', 'Advanced EPSX platform access', 'epsx', true),
('enterprise_user', 'Enterprise User', 'Enterprise EPSX platform access', 'epsx', true),
('free_user', 'Free User', 'Limited free access', 'epsx', true);

-- Role-permission mappings
INSERT INTO rbac_role_permissions (role_id, permission_id) 
SELECT r.id, p.id FROM rbac_roles r, rbac_permissions p WHERE
-- Super admin gets everything
(r.name = 'super_admin' AND p.name = 'admin:*:*')
OR
-- Admin role gets admin permissions
(r.name = 'admin' AND p.platform = 'admin')
OR
-- Bronze user permissions
(r.name = 'bronze_user' AND p.name IN (
    'epsx:rankings:view:5', 'epsx:trading:basic', 'epsx:portfolio:view', 'epsx:notifications:receive'
))
OR
-- Silver user permissions (bronze + more)
(r.name = 'silver_user' AND p.name IN (
    'epsx:rankings:view:25', 'epsx:trading:basic', 'epsx:trading:advanced', 
    'epsx:portfolio:view', 'epsx:analytics:view', 'epsx:notifications:receive'
))
OR
-- Gold user permissions (silver + more)
(r.name = 'gold_user' AND p.name IN (
    'epsx:rankings:view:50', 'epsx:trading:basic', 'epsx:trading:advanced', 'epsx:trading:premium',
    'epsx:portfolio:view', 'epsx:portfolio:manage', 'epsx:analytics:view', 'epsx:analytics:premium',
    'epsx:notifications:receive', 'epsx:notifications:manage'
))
OR
-- Platinum user permissions (gold + more)  
(r.name = 'platinum_user' AND p.name IN (
    'epsx:rankings:view:100', 'epsx:trading:basic', 'epsx:trading:advanced', 'epsx:trading:premium',
    'epsx:portfolio:view', 'epsx:portfolio:manage', 'epsx:analytics:view', 'epsx:analytics:export', 
    'epsx:analytics:premium', 'epsx:notifications:receive', 'epsx:notifications:manage'
))
OR
-- Enterprise user permissions (everything)
(r.name = 'enterprise_user' AND (p.platform = 'epsx' OR p.name = 'epsx:rankings:view:unlimited'))
OR
-- Free user minimal permissions
(r.name = 'free_user' AND p.name IN ('epsx:rankings:view:5', 'epsx:portfolio:view'));

-- Test user
INSERT INTO users (firebase_uid, email, display_name, name, package_tier, email_verified, is_active, created_at) VALUES
('info@epsx.io', 'info@epsx.io', 'EPSX Test User', 'Test User', 'admin', true, true, NOW());

-- Assign admin role to test user
INSERT INTO rbac_user_roles (user_id, role_id, granted_by, reason)
SELECT u.id, r.id, u.id, 'Initial admin setup'
FROM users u, rbac_roles r
WHERE u.email = 'info@epsx.io' AND r.name = 'admin';

-- FCM topics
INSERT INTO fcm_topics (name, display_name, description, is_active) VALUES
('epsx_all_users', 'All EPSX Users', 'General notifications for all platform users', true),
('epsx_admin_users', 'Admin Users', 'Administrative notifications', true),
('epsx_premium_users', 'Premium Users', 'Premium feature notifications', true);

-- Test notifications
INSERT INTO notifications (title, body, notification_type, priority, channels, created_at) VALUES
('🎉 Welcome to EPSX Analytics', 'Your account is now active with full access to market data insights and real-time analytics.', 'system', 'normal', '{fcm,in_app}', NOW() - INTERVAL '2 days'),
('📊 Weekly Data Report Ready', 'Your personalized analytics report for this week is now available in the dashboard.', 'data', 'normal', '{fcm,in_app}', NOW() - INTERVAL '1 day'),
('🔒 Security Alert: New Login', 'New login detected from Chrome on macOS in San Francisco, CA. If this wasn''t you, secure your account immediately.', 'security', 'high', '{fcm,in_app}', NOW() - INTERVAL '3 hours'),
('✨ New Feature: Advanced Charts', 'Enhanced charting tools with technical indicators are now available in your analytics dashboard.', 'feature', 'normal', '{fcm,in_app}', NOW() - INTERVAL '6 hours'),
('⚠️ System Maintenance Notice', 'Scheduled maintenance tonight 2-4 AM UTC. Services may be briefly unavailable during this window.', 'system', 'high', '{fcm,in_app}', NOW() - INTERVAL '12 hours'),
('📈 Market Alert: High Volatility', 'Significant market movement detected in your watchlist. Review your positions and risk management.', 'data', 'urgent', '{fcm,in_app}', NOW() - INTERVAL '30 minutes');

-- Link notifications to test user
INSERT INTO user_notifications (user_id, notification_id, read_at, clicked_at, delivery_status) 
SELECT 
    'info@epsx.io', 
    id, 
    CASE 
        WHEN notification_type = 'system' AND priority = 'high' THEN created_at + INTERVAL '5 minutes'
        WHEN notification_type = 'feature' THEN created_at + INTERVAL '15 minutes'  
        ELSE NULL
    END,
    CASE 
        WHEN notification_type = 'feature' THEN created_at + INTERVAL '20 minutes'
        ELSE NULL
    END,
    'delivered'
FROM notifications;

-- Set up notification preferences for test user
INSERT INTO user_notification_preferences (user_id, fcm_enabled, in_app_enabled, email_enabled, timezone)
SELECT id, true, true, false, 'America/New_York'
FROM users WHERE email = 'info@epsx.io';

-- Add FCM token for test user  
INSERT INTO fcm_tokens (user_id, token, platform, topics, device_info)
SELECT id, 'test-fcm-token-info-epsx-web-' || extract(epoch from now())::text, 'web', '["epsx_all_users","epsx_admin_users"]', '{"browser": "Chrome", "os": "macOS", "version": "119.0.0.0"}'
FROM users WHERE email = 'info@epsx.io';

-- Sample EPS analytics data
INSERT INTO eps_growth_analytics (symbol, name, country, sector, exchange, current_eps, qoq_growth_rate, price_current, market_cap, volume, ranking_score) VALUES
('AAPL', 'Apple Inc.', 'US', 'Technology', 'NASDAQ', 6.05, 15.2, 175.50, 2800000000000, 45000000, 95.8),
('MSFT', 'Microsoft Corporation', 'US', 'Technology', 'NASDAQ', 11.05, 12.8, 415.25, 3100000000000, 25000000, 94.2),
('GOOGL', 'Alphabet Inc.', 'US', 'Technology', 'NASDAQ', 4.56, 18.7, 2840.75, 1900000000000, 15000000, 92.5),
('AMZN', 'Amazon.com Inc.', 'US', 'Consumer Discretionary', 'NASDAQ', 0.65, 85.4, 3320.50, 1700000000000, 20000000, 89.3),
('TSLA', 'Tesla Inc.', 'US', 'Consumer Discretionary', 'NASDAQ', 4.90, 25.6, 1025.75, 1050000000000, 35000000, 87.1);

-- Log initial setup
INSERT INTO rbac_permission_audit_log (operation, actor_user_id, reason, metadata) VALUES (
    'migration',
    NULL,
    'Complete EPSX database seed setup',
    jsonb_build_object(
        'permissions_created', (SELECT COUNT(*) FROM rbac_permissions),
        'roles_created', (SELECT COUNT(*) FROM rbac_roles), 
        'mappings_created', (SELECT COUNT(*) FROM rbac_role_permissions),
        'users_created', (SELECT COUNT(*) FROM users),
        'notifications_created', (SELECT COUNT(*) FROM notifications),
        'analytics_records', (SELECT COUNT(*) FROM eps_growth_analytics),
        'setup_time', NOW()
    )
);

-- Add comment to schema
COMMENT ON SCHEMA public IS 'EPSX complete database seed - consolidated from all migrations - 2025-09-08';