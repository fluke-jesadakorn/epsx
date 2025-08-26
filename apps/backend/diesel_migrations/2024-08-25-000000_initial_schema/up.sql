-- ============================================================================
-- EPSX CONSOLIDATED DATABASE SCHEMA
-- ============================================================================
-- Complete production-ready database schema for the EPSX trading platform
-- This single file replaces all incremental migration files
-- Version: Consolidated 2024
-- Created: 2024-08-25

-- ============================================================================
-- EXTENSIONS
-- ============================================================================

-- UUID generation support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "btree_gist";

-- ============================================================================
-- ENUMS AND TYPES
-- ============================================================================

-- Permission system enums
CREATE TYPE permission_scope AS ENUM ('global', 'module', 'resource', 'user');
CREATE TYPE permission_level AS ENUM ('read', 'write', 'admin', 'owner');
CREATE TYPE admin_module AS ENUM (
    'user-management',
    'analytics-access', 
    'system-configuration',
    'audit-logs',
    'financial-oversight',
    'content-management',
    'support-access',
    'security-management'
);
CREATE TYPE admin_module_permission AS ENUM ('view', 'create', 'update', 'delete', 'admin', 'owner');

-- Package tier system
CREATE TYPE tier_feature AS ENUM (
    'basic-trading',
    'advanced-analytics',
    'api-access',
    'priority-support',
    'advanced-orders',
    'portfolio-tools',
    'research-reports',
    'institutional-features'
);
CREATE TYPE tier_limit_type AS ENUM (
    'requests-per-minute',
    'requests-per-hour', 
    'requests-per-day',
    'concurrent-connections',
    'data-export-mb',
    'storage-mb',
    'api-calls',
    'advanced-features',
    'support-tickets'
);
CREATE TYPE tier_reset_period AS ENUM ('minute', 'hour', 'day', 'week', 'month', 'year', 'never');
CREATE TYPE package_tier AS ENUM ('free', 'bronze', 'silver', 'gold', 'platinum', 'admin');
CREATE TYPE denial_reason AS ENUM (
    'insufficient-permissions',
    'expired-permission',
    'resource-not-found',
    'policy-violation',
    'rate-limited',
    'security-threat',
    'maintenance-mode',
    'insufficient-tier',
    'module-disabled',
    'temporary-block'
);

-- ============================================================================
-- CORE USER MANAGEMENT TABLES
-- ============================================================================

-- Users table (optimized for Firebase authentication)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) UNIQUE NOT NULL,
    email VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    name VARCHAR(255),
    avatar_url TEXT,
    package_tier VARCHAR(50) DEFAULT 'FREE',
    permissions TEXT[] DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sessions table (optimized for JWT-based auth)
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    provider VARCHAR(50) DEFAULT 'credentials',
    provider_account_id TEXT,
    session_token TEXT UNIQUE,
    jwt_token TEXT,
    user_agent TEXT,
    ip_address INET,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Firebase sessions table for authentication
CREATE TABLE firebase_sessions (
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

-- ============================================================================
-- ADMIN MODULE PERMISSION SYSTEM
-- ============================================================================

-- Admin modules definition - primary permission system
CREATE TABLE admin_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_code VARCHAR(50) UNIQUE NOT NULL,
    module_name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    category VARCHAR(50) NOT NULL,
    icon VARCHAR(50),
    color VARCHAR(20),
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    requires_modules TEXT[] DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- User to admin module assignments
CREATE TABLE user_admin_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL,
    module_code VARCHAR(50) NOT NULL REFERENCES admin_modules(module_code),
    granted_by VARCHAR(128),
    granted_reason TEXT,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    assignment_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_firebase_uid_module UNIQUE (firebase_uid, module_code)
);

-- Module permissions
CREATE TABLE admin_module_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_code VARCHAR(50) NOT NULL REFERENCES admin_modules(module_code),
    api_endpoints TEXT[] NOT NULL DEFAULT '{}',
    frontend_routes TEXT[] NOT NULL DEFAULT '{}',
    permissions TEXT[] NOT NULL DEFAULT '{}',
    resource_patterns TEXT[] NOT NULL DEFAULT '{}',
    access_level VARCHAR(20) DEFAULT 'read',
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Admin role assignment audit log
CREATE TABLE admin_role_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL,
    module_code VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    old_status JSONB,
    new_status JSONB,
    performed_by VARCHAR(128),
    reason TEXT,
    metadata JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- UNIFIED PERMISSION SYSTEM
-- ============================================================================

-- Core permissions table
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    permission_id VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL DEFAULT '*',
    scope permission_scope NOT NULL DEFAULT 'global',
    level permission_level NOT NULL DEFAULT 'read',
    conditions JSONB,
    expires_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(permission_id, resource, scope)
);

-- User permission profiles (combining all permission sources)
CREATE TABLE user_permission_profiles (
    user_id VARCHAR(255) PRIMARY KEY,
    package_tier package_tier NOT NULL DEFAULT 'free',
    enabled_admin_modules admin_module[] DEFAULT '{}',
    enabled_tier_features tier_feature[] DEFAULT '{}',
    direct_permissions UUID[] DEFAULT '{}',
    inherited_permissions UUID[] DEFAULT '{}',
    temporary_permissions UUID[] DEFAULT '{}',
    denied_permissions UUID[] DEFAULT '{}',
    last_updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cache_version BIGINT NOT NULL DEFAULT 1,
    expires_at TIMESTAMPTZ,
    trial_mode BOOLEAN DEFAULT FALSE,
    auto_renew BOOLEAN DEFAULT FALSE
);

-- Admin module access configurations
CREATE TABLE admin_module_access (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    module admin_module NOT NULL,
    permissions admin_module_permission[] NOT NULL DEFAULT '{}',
    granted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    granted_by VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ,
    conditions JSONB,
    reason TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    UNIQUE(user_id, module)
);

-- Package tier access configurations
CREATE TABLE package_tier_access (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    package_tier package_tier NOT NULL,
    enabled_features tier_feature[] DEFAULT '{}',
    upgraded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ,
    auto_renew BOOLEAN DEFAULT FALSE,
    trial_mode BOOLEAN DEFAULT FALSE,
    billing_cycle_days INTEGER DEFAULT 30,
    last_billing_date TIMESTAMPTZ,
    next_billing_date TIMESTAMPTZ
);

-- Tier limits for rate limiting and quotas
CREATE TABLE tier_limits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    feature tier_feature NOT NULL,
    limit_type tier_limit_type NOT NULL,
    max_value BIGINT NOT NULL,
    current_value BIGINT NOT NULL DEFAULT 0,
    reset_period tier_reset_period,
    burst_allowance BIGINT,
    last_reset TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, feature, limit_type)
);

-- Permission inheritance mappings
CREATE TABLE permission_inheritance (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    parent_permission_id UUID NOT NULL,
    child_permission_id UUID NOT NULL,
    inheritance_type VARCHAR(50) NOT NULL DEFAULT 'explicit',
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_permission_id) REFERENCES permissions(id) ON DELETE CASCADE,
    FOREIGN KEY (child_permission_id) REFERENCES permissions(id) ON DELETE CASCADE,
    UNIQUE(parent_permission_id, child_permission_id)
);

-- Temporary permissions table (optimized)
CREATE TABLE temporary_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    action VARCHAR(255) NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    auto_revoke BOOLEAN NOT NULL DEFAULT true,
    granted_by UUID NOT NULL REFERENCES users(id),
    reason TEXT,
    conditions JSONB DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    revoked_at TIMESTAMPTZ,
    revoked_by UUID REFERENCES users(id),
    revocation_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Permission audit logs
CREATE TABLE permission_audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(50) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    permission VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    action VARCHAR(100) NOT NULL,
    result VARCHAR(20) NOT NULL,
    denial_reason denial_reason,
    request_id UUID,
    session_id VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    context_data JSONB DEFAULT '{}',
    security_score INTEGER DEFAULT 0,
    performance_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- SECURITY INFRASTRUCTURE
-- ============================================================================

-- Security events table for middleware security logging
CREATE TABLE security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    source VARCHAR(50) NOT NULL,
    user_id VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    request_path TEXT,
    request_method VARCHAR(10),
    request_headers JSONB,
    response_status INTEGER,
    event_data JSONB DEFAULT '{}',
    risk_score INTEGER DEFAULT 0,
    country_code VARCHAR(2),
    device_fingerprint TEXT,
    correlation_id UUID,
    alert_triggered BOOLEAN DEFAULT FALSE,
    blocked BOOLEAN DEFAULT FALSE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed BOOLEAN DEFAULT FALSE
);

-- Security alert rules configuration
CREATE TABLE security_alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    event_pattern JSONB NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    threshold_count INTEGER DEFAULT 1,
    time_window_seconds INTEGER DEFAULT 300,
    enabled BOOLEAN DEFAULT TRUE,
    notification_channels TEXT[] DEFAULT '{}',
    auto_block BOOLEAN DEFAULT FALSE,
    block_duration_seconds INTEGER DEFAULT 3600,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_triggered TIMESTAMPTZ
);

-- Attack attempts tracking
CREATE TABLE attack_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL,
    attack_type VARCHAR(100) NOT NULL,
    target_user VARCHAR(255),
    request_path TEXT,
    user_agent TEXT,
    severity VARCHAR(20) DEFAULT 'medium',
    success BOOLEAN DEFAULT FALSE,
    blocked BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}',
    detection_method VARCHAR(100),
    risk_score INTEGER DEFAULT 0,
    geolocation JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- IP blacklist for security blocking
CREATE TABLE ip_blacklist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL UNIQUE,
    reason VARCHAR(255) NOT NULL,
    blocked_by VARCHAR(255),
    auto_generated BOOLEAN DEFAULT FALSE,
    expires_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_hit TIMESTAMPTZ
);

-- Alert notifications
CREATE TABLE alert_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_rule_id UUID NOT NULL REFERENCES security_alert_rules(id),
    event_id UUID REFERENCES security_events(id),
    notification_type VARCHAR(50) NOT NULL,
    recipient TEXT NOT NULL,
    subject TEXT,
    message TEXT NOT NULL,
    sent_at TIMESTAMPTZ,
    delivery_status VARCHAR(20) DEFAULT 'pending',
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- AUDIT AND LOGGING SYSTEM
-- ============================================================================

-- Primary audit logs table (cleaned)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES users(id),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    result VARCHAR(50) DEFAULT 'success',
    event_category VARCHAR(50) DEFAULT 'system_security',
    severity VARCHAR(20) DEFAULT 'medium',
    success BOOLEAN DEFAULT true,
    details JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    ip_address INET,
    client_ip INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id UUID REFERENCES sessions(id)
);

-- ============================================================================
-- ANALYTICS AND FEATURE TABLES
-- ============================================================================

-- EPS growth analytics table
CREATE TABLE eps_growth_analytics (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    name VARCHAR(100) NOT NULL,
    country VARCHAR(50) NOT NULL,
    sector VARCHAR(100),
    exchange VARCHAR(50),
    current_eps DECIMAL(10,4),
    qoq_growth_rate DECIMAL(8,4),
    price_current DECIMAL(10,2),
    market_cap BIGINT,
    volume BIGINT,
    ranking_score DECIMAL(10,4),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Enhanced notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    user_firebase_uid VARCHAR(255),
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    is_read BOOLEAN NOT NULL DEFAULT false,
    delivery_status VARCHAR(50) DEFAULT 'pending',
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    metadata JSONB
);

-- ============================================================================
-- PERFORMANCE INDEXES
-- ============================================================================

-- Users table indexes
CREATE INDEX idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_package_tier ON users(package_tier);
CREATE INDEX idx_users_email_search ON users USING gin(to_tsvector('english', email));
CREATE INDEX idx_users_email_lower ON users (LOWER(email));
CREATE INDEX idx_users_display_name_search ON users USING gin(to_tsvector('english', display_name));
CREATE INDEX idx_users_name_search ON users USING gin(to_tsvector('english', name));
CREATE INDEX idx_users_search_composite ON users (email, display_name, is_active, package_tier);

-- Sessions table indexes  
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_session_token ON sessions(session_token);

-- Firebase sessions indexes
CREATE INDEX idx_firebase_sessions_firebase_uid ON firebase_sessions(firebase_uid);
CREATE INDEX idx_firebase_sessions_session_token ON firebase_sessions(session_token);
CREATE INDEX idx_firebase_sessions_expires_at ON firebase_sessions(expires_at);
CREATE INDEX idx_firebase_sessions_is_active ON firebase_sessions(is_active) WHERE is_active = true;

-- Admin modules indexes
CREATE INDEX idx_admin_modules_code ON admin_modules(module_code);
CREATE INDEX idx_admin_modules_category ON admin_modules(category);
CREATE INDEX idx_admin_modules_active ON admin_modules(is_active) WHERE is_active = true;

-- Admin roles indexes
CREATE INDEX idx_user_admin_roles_firebase_uid ON user_admin_roles(firebase_uid);
CREATE INDEX idx_user_admin_roles_module ON user_admin_roles(module_code);
CREATE INDEX idx_user_admin_roles_active ON user_admin_roles(is_active) WHERE is_active = true;
CREATE INDEX idx_user_admin_roles_expires ON user_admin_roles(expires_at) WHERE expires_at IS NOT NULL;

-- Module permissions indexes
CREATE INDEX idx_module_permissions_module ON admin_module_permissions(module_code);
CREATE INDEX idx_module_permissions_access_level ON admin_module_permissions(access_level);

-- Security events indexes
CREATE INDEX idx_security_events_event_type ON security_events(event_type);
CREATE INDEX idx_security_events_timestamp ON security_events(timestamp);
CREATE INDEX idx_security_events_ip_address ON security_events(ip_address);
CREATE INDEX idx_security_events_user_id ON security_events(user_id);
CREATE INDEX idx_security_events_risk_score ON security_events(risk_score DESC) WHERE risk_score IS NOT NULL;
CREATE INDEX idx_security_events_country_code ON security_events(country_code) WHERE country_code IS NOT NULL;
CREATE INDEX idx_security_events_device_fingerprint ON security_events(device_fingerprint) WHERE device_fingerprint IS NOT NULL;
CREATE INDEX idx_security_events_correlation_id ON security_events(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_security_events_alert_triggered ON security_events(alert_triggered, timestamp DESC) WHERE alert_triggered = TRUE;

-- Attack attempts indexes
CREATE INDEX idx_attack_attempts_ip_address ON attack_attempts(ip_address);
CREATE INDEX idx_attack_attempts_timestamp ON attack_attempts(timestamp);
CREATE INDEX idx_attack_attempts_attack_type ON attack_attempts(attack_type);
CREATE INDEX idx_attack_attempts_target_user ON attack_attempts(target_user);

-- IP blacklist indexes
CREATE INDEX idx_ip_blacklist_ip_address ON ip_blacklist(ip_address);
CREATE INDEX idx_ip_blacklist_expires_at ON ip_blacklist(expires_at) WHERE expires_at IS NOT NULL;

-- Audit logs indexes
CREATE INDEX idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_client_ip ON audit_logs(client_ip);

-- Notifications indexes
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);
CREATE INDEX idx_notifications_is_read ON notifications(is_read);
CREATE INDEX idx_notifications_expires_at ON notifications(expires_at) WHERE expires_at IS NOT NULL;

-- Temporary permissions indexes
CREATE INDEX idx_temporary_permissions_user_id ON temporary_permissions(user_id);
CREATE INDEX idx_temporary_permissions_expires_at ON temporary_permissions(expires_at);
CREATE INDEX idx_temporary_permissions_status ON temporary_permissions(status);
CREATE INDEX idx_temporary_permissions_permission ON temporary_permissions(permission);

-- EPS analytics indexes
CREATE INDEX idx_eps_country ON eps_growth_analytics (country);
CREATE INDEX idx_eps_qoq_growth ON eps_growth_analytics (qoq_growth_rate DESC);
CREATE INDEX idx_eps_ranking_score ON eps_growth_analytics (ranking_score DESC);
CREATE INDEX idx_eps_symbol ON eps_growth_analytics (symbol);
CREATE INDEX idx_eps_sector ON eps_growth_analytics (sector);
CREATE INDEX idx_eps_updated_at ON eps_growth_analytics (updated_at DESC);

-- Permission system indexes
CREATE INDEX idx_permissions_permission_id ON permissions(permission_id);
CREATE INDEX idx_permissions_resource ON permissions(resource);
CREATE INDEX idx_permissions_scope ON permissions(scope);
CREATE INDEX idx_user_permission_profiles_user_id ON user_permission_profiles(user_id);
CREATE INDEX idx_admin_module_access_user_id ON admin_module_access(user_id);
CREATE INDEX idx_admin_module_access_module ON admin_module_access(module);
CREATE INDEX idx_package_tier_access_user_id ON package_tier_access(user_id);
CREATE INDEX idx_tier_limits_user_id ON tier_limits(user_id);
CREATE INDEX idx_permission_audit_logs_user_id ON permission_audit_logs(user_id);
CREATE INDEX idx_permission_audit_logs_timestamp ON permission_audit_logs(created_at);

-- ============================================================================
-- OPTIMIZED VIEWS
-- ============================================================================

-- Admin system summary view
CREATE VIEW admin_system_summary AS
SELECT 
    'admin_modules' as component,
    COUNT(*) as count,
    'Core admin functional modules' as description
FROM admin_modules WHERE is_active = true
UNION ALL
SELECT 
    'user_admin_roles' as component,
    COUNT(*) as count,
    'Admin role assignments' as description
FROM user_admin_roles WHERE is_active = true
UNION ALL
SELECT 
    'admin_module_permissions' as component,
    COUNT(*) as count,
    'Module permission definitions' as description
FROM admin_module_permissions
UNION ALL
SELECT 
    'admin_role_audit' as component,
    COUNT(*) as count,
    'Admin role audit entries' as description
FROM admin_role_audit;

-- User permissions view for JWT generation
CREATE VIEW user_permissions_view AS
SELECT 
    u.id,
    u.firebase_uid,
    u.email,
    u.package_tier,
    u.permissions as base_permissions,
    COALESCE(
        ARRAY_AGG(DISTINCT uar.module_code) FILTER (WHERE uar.module_code IS NOT NULL),
        '{}'
    ) as admin_modules,
    COALESCE(
        ARRAY_AGG(DISTINCT amp.permissions) FILTER (WHERE amp.permissions IS NOT NULL),
        '{}'
    ) as module_permissions,
    u.is_active,
    u.created_at,
    u.last_login_at
FROM users u
LEFT JOIN user_admin_roles uar ON u.firebase_uid = uar.firebase_uid 
    AND uar.is_active = true 
    AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
LEFT JOIN admin_module_permissions amp ON uar.module_code = amp.module_code
WHERE u.is_active = true
GROUP BY u.id, u.firebase_uid, u.email, u.package_tier, u.permissions, u.is_active, u.created_at, u.last_login_at;

-- Role history view for compatibility
CREATE VIEW role_history AS
SELECT 
    ara.id,
    ara.firebase_uid as user_id,
    ara.module_code as role_name,
    ara.action as change_type,
    ara.performed_by as changed_by,
    ara.reason as change_reason,
    ara.timestamp as changed_at,
    ara.old_status->>'is_active' as old_role,
    ara.new_status->>'is_active' as new_role
FROM admin_role_audit ara
ORDER BY ara.timestamp DESC;

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function to get user JWT claims
CREATE OR REPLACE FUNCTION get_user_jwt_claims(user_firebase_uid VARCHAR(128))
RETURNS JSONB
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT jsonb_build_object(
        'firebase_uid', upv.firebase_uid,
        'email', upv.email,
        'package_tier', upv.package_tier,
        'permissions', upv.base_permissions,
        'admin_modules', upv.admin_modules,
        'role', CASE 
            WHEN array_length(upv.admin_modules, 1) > 0 THEN 'admin'
            ELSE 'user'
        END,
        'is_active', upv.is_active,
        'last_login_at', upv.last_login_at
    )
    FROM user_permissions_view upv
    WHERE upv.firebase_uid = user_firebase_uid;
$BODY$;

-- Function to check if user has specific permission  
CREATE OR REPLACE FUNCTION user_has_permission(
    user_firebase_uid VARCHAR(128), 
    required_permission TEXT
)
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT EXISTS(
        SELECT 1 FROM user_permissions_view upv
        WHERE upv.firebase_uid = user_firebase_uid
        AND (
            required_permission = ANY(upv.base_permissions)
            OR required_permission = ANY(upv.module_permissions)
            OR 'system_admin' = ANY(upv.admin_modules)
            OR '*' = ANY(upv.base_permissions)
        )
    );
$BODY$;

-- Function to get user admin modules
CREATE OR REPLACE FUNCTION get_user_admin_modules(user_firebase_uid VARCHAR(128))
RETURNS TABLE(module_code VARCHAR(50), module_name VARCHAR(100), permissions TEXT[]) 
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT 
        am.module_code,
        am.module_name,
        amp.permissions
    FROM user_admin_roles uar
    JOIN admin_modules am ON uar.module_code = am.module_code
    JOIN admin_module_permissions amp ON am.module_code = amp.module_code
    WHERE uar.firebase_uid = user_firebase_uid
      AND uar.is_active = true
      AND am.is_active = true
      AND (uar.expires_at IS NULL OR uar.expires_at > NOW());
$BODY$;

-- Function to check if user has specific module access
CREATE OR REPLACE FUNCTION user_has_admin_module(user_firebase_uid VARCHAR(128), check_module_code VARCHAR(50))
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT EXISTS(
        SELECT 1 FROM user_admin_roles
        WHERE firebase_uid = user_firebase_uid
          AND module_code = check_module_code
          AND is_active = true
          AND (expires_at IS NULL OR expires_at > NOW())
    );
$BODY$;

-- Cleanup expired firebase sessions function
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

-- ============================================================================
-- INITIAL SEED DATA
-- ============================================================================

-- Insert the 10 granular admin modules
INSERT INTO admin_modules (module_code, module_name, description, category, icon, color, sort_order, requires_modules) VALUES
('user_operations', 'User Operations Manager', 'User CRUD operations, status management, and basic profile editing', 'management', 'users', 'blue', 1, '{}'),
('permission_admin', 'Permission Administrator', 'Permission profiles, assignments, and temporary permission management', 'management', 'shield-check', 'green', 2, '{}'),
('role_policy_manager', 'Role & Policy Manager', 'Casbin roles, policies, and access control management', 'management', 'key', 'purple', 3, '{}'),
('analytics_specialist', 'Analytics Specialist', 'Reporting, dashboards, and data analysis (read-only access)', 'analytics', 'chart-bar', 'yellow', 4, '{}'),
('billing_admin', 'Billing Administrator', 'Payment management, subscriptions, and package assignments', 'commerce', 'credit-card', 'emerald', 5, '{}'),
('system_admin', 'System Administrator', 'Database management, health monitoring, and system settings', 'system', 'server', 'red', 6, '{}'),
('developer_relations', 'Developer Relations', 'API keys, developer portal, and integration management', 'technical', 'code', 'indigo', 7, '{}'),
('module_coordinator', 'Module Coordinator', 'Feature module assignments and access control', 'management', 'puzzle', 'pink', 8, '{}'),
('compliance_audit', 'Compliance & Audit Officer', 'Security, audit reports, backups, and compliance management', 'security', 'clipboard-check', 'orange', 9, '{}'),
('support_specialist', 'Support Specialist', 'User support and troubleshooting (read-only access)', 'support', 'support', 'cyan', 10, '{}')
ON CONFLICT (module_code) DO NOTHING;

-- Insert module permissions for each admin module
INSERT INTO admin_module_permissions (module_code, api_endpoints, frontend_routes, permissions, resource_patterns, access_level, description) VALUES
-- User Operations Manager
('user_operations', 
 ARRAY['/api/v1/admin/users/*', '/api/v1/admin/firebase/users/*', '/api/v1/admin/users/*/unified'],
 ARRAY['/users', '/users/*'],
 ARRAY['user:read', 'user:write', 'user:status', 'profile:edit'],
 ARRAY['users/*', 'firebase_users/*'],
 'write',
 'Full user management except deletion and role assignment'),

-- Permission Administrator  
('permission_admin',
 ARRAY['/api/v1/admin/permission-profiles/*', '/api/v1/admin/temporary-permissions/*', '/api/v1/admin/permissions/*'],
 ARRAY['/permission-profiles', '/permission-profiles/*'],
 ARRAY['permission:read', 'permission:write', 'profile:assign', 'temp_permission:manage'],
 ARRAY['permission_profiles/*', 'temporary_permissions/*'],
 'write',
 'Permission profile creation, assignment, and temporary permission management'),

-- Role & Policy Manager
('role_policy_manager',
 ARRAY['/api/v1/admin/casbin/*', '/api/v1/admin/roles/*'],
 ARRAY['/iam', '/iam/*'],
 ARRAY['role:read', 'role:write', 'policy:manage', 'casbin:admin'],
 ARRAY['casbin_rules/*', 'roles/*'],
 'write',
 'Casbin policy and role management with testing capabilities'),

-- Analytics Specialist
('analytics_specialist',
 ARRAY['/api/v1/admin/analytics/*'],
 ARRAY['/analytics', '/analytics/*'],
 ARRAY['analytics:read', 'reports:generate', 'metrics:view'],
 ARRAY['analytics/*', 'reports/*'],
 'read',
 'Read-only access to analytics dashboards and report generation'),

-- Billing Administrator
('billing_admin',
 ARRAY['/api/v1/admin/users/*/billing', '/api/v1/admin/stock-ranking-packages/*'],
 ARRAY['/billing', '/billing/*'],
 ARRAY['billing:read', 'billing:write', 'subscription:manage', 'package:assign'],
 ARRAY['billing/*', 'subscriptions/*', 'packages/*'],
 'write',
 'Payment processing, subscription management, and package assignments'),

-- System Administrator
('system_admin',
 ARRAY['/api/v1/admin/database/*', '/api/v1/admin/settings/*', '/api/v1/admin/casbin/cache/*'],
 ARRAY['/database', '/settings', '/settings/*'],
 ARRAY['database:admin', 'system:settings', 'cache:manage', 'health:monitor'],
 ARRAY['database/*', 'system/*', 'cache/*'],
 'admin',
 'Database management, system configuration, and infrastructure monitoring'),

-- Developer Relations
('developer_relations',
 ARRAY['/api/v1/admin/api-keys/*', '/api/v1/admin/developer-portal/*'],
 ARRAY['/developer-portal', '/developer-portal/*'],
 ARRAY['api_key:manage', 'developer:tools', 'documentation:manage'],
 ARRAY['api_keys/*', 'developer_tools/*'],
 'write',
 'API key management and developer resource administration'),

-- Module Coordinator
('module_coordinator',
 ARRAY['/api/v1/admin/modules/*', '/api/v1/admin/users/*/modules'],
 ARRAY['/modules', '/modules/*'],
 ARRAY['module:read', 'module:write', 'module:assign', 'feature:manage'],
 ARRAY['modules/*', 'user_modules/*'],
 'write',
 'Feature module assignments and access control management'),

-- Compliance & Audit Officer  
('compliance_audit',
 ARRAY['/api/v1/admin/permissions/audit-report', '/api/v1/admin/permissions/system-backup/*', '/api/v1/admin/analytics/security-risks'],
 ARRAY['/compliance', '/audit/*'],
 ARRAY['audit:read', 'compliance:manage', 'backup:create', 'security:analyze'],
 ARRAY['audit/*', 'compliance/*', 'backups/*'],
 'read',
 'Security auditing, compliance reporting, and system backup management'),

-- Support Specialist
('support_specialist',
 ARRAY['/api/v1/admin/users/*/activity', '/api/v1/admin/support/*'],
 ARRAY['/support', '/users/*/support'],
 ARRAY['user:read', 'support:tickets', 'activity:view'],
 ARRAY['users/*/read', 'support/*'],
 'read',
 'Read-only user access for support and troubleshooting purposes')
ON CONFLICT DO NOTHING;

-- Insert sample EPS analytics data for testing
INSERT INTO eps_growth_analytics (symbol, name, country, sector, exchange, current_eps, qoq_growth_rate, price_current, market_cap, volume, ranking_score) VALUES
('AAPL', 'Apple Inc.', 'US', 'Technology', 'NASDAQ', 6.15, 8.50, 175.25, 2800000000000, 45000000, 95.8),
('MSFT', 'Microsoft Corporation', 'US', 'Technology', 'NASDAQ', 9.65, 12.30, 335.50, 2500000000000, 28000000, 94.2),
('GOOGL', 'Alphabet Inc.', 'US', 'Technology', 'NASDAQ', 5.80, 15.75, 138.75, 1750000000000, 22000000, 92.5)
ON CONFLICT DO NOTHING;