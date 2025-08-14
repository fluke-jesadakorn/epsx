-- EPSX Initial Database Schema
-- Single comprehensive migration file for clean EPSX database structure
-- This replaces 26+ incremental migrations with a clean, optimized schema

-- ============================================================================
-- ENABLE REQUIRED EXTENSIONS
-- ============================================================================

-- UUID generation support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

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

-- ============================================================================
-- MODERN ADMIN MODULE PERMISSION SYSTEM
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
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id UUID REFERENCES sessions(id)
);

-- ============================================================================
-- FEATURE TABLES
-- ============================================================================

-- Notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB
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

-- ============================================================================
-- PERFORMANCE INDEXES
-- ============================================================================

-- Users table indexes
CREATE INDEX idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_package_tier ON users(package_tier);

-- Sessions table indexes  
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_session_token ON sessions(session_token);

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

-- Audit logs indexes
CREATE INDEX idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);

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

-- ============================================================================
-- HELPER FUNCTIONS FOR JWT INTEGRATION
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
            WHEN 'system_admin' = ANY(upv.admin_modules) THEN 'super_admin'
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
            OR 'system_admin' = ANY(upv.admin_modules) -- System admin has all permissions
            OR '*' = ANY(upv.base_permissions) -- Wildcard permission
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

-- ============================================================================
-- SEED DATA: ADMIN MODULES
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
 'Database management, system configuration, and infrastructure monitoring')

ON CONFLICT DO NOTHING;

-- ============================================================================
-- TABLE COMMENTS AND DOCUMENTATION
-- ============================================================================

COMMENT ON TABLE users IS 'Core user table with Firebase authentication - cleaned and optimized';
COMMENT ON TABLE sessions IS 'JWT-based user sessions - optimized for modern auth';
COMMENT ON TABLE audit_logs IS 'Comprehensive audit logging - cleaned of legacy references';
COMMENT ON TABLE temporary_permissions IS 'Time-bound permissions system - optimized';
COMMENT ON TABLE admin_modules IS 'Defines granular admin functional modules';
COMMENT ON TABLE user_admin_roles IS 'Maps Firebase UIDs to specific admin modules';
COMMENT ON TABLE admin_module_permissions IS 'Defines resource access permissions for each admin module';
COMMENT ON TABLE admin_role_audit IS 'Audit trail for admin role assignments and changes';

COMMENT ON VIEW user_permissions_view IS 'Modern view for JWT token generation with user permissions';
COMMENT ON FUNCTION get_user_jwt_claims(VARCHAR) IS 'Returns JWT claims for a user based on their permissions and admin modules';
COMMENT ON FUNCTION user_has_permission(VARCHAR, TEXT) IS 'Checks if user has a specific permission for backend validation';

-- ============================================================================
-- MIGRATION COMPLETION NOTES
-- ============================================================================

-- This single migration replaces 26 incremental migrations:
-- - Consolidated schema creation from multiple iterations
-- - Removed unused complex module system (5 tables)
-- - Removed legacy permission profiles system (3 tables)  
-- - Optimized for modern JWT-based authentication
-- - Streamlined to 10 core tables with clear separation of concerns
-- - Includes comprehensive indexing and performance optimizations
-- - Modern admin modules system as primary permission mechanism
--
-- Result: Clean, performant database schema ready for production deployment