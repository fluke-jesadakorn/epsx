-- Granular Admin Module System Migration
-- Creates a modular admin role system with 10 distinct functional areas

-- Admin modules definition - each represents a specific functional area
CREATE TABLE IF NOT EXISTS admin_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_code VARCHAR(50) UNIQUE NOT NULL, -- e.g., 'user_operations'
    module_name VARCHAR(100) NOT NULL, -- e.g., 'User Operations Manager'
    description TEXT NOT NULL,
    category VARCHAR(50) NOT NULL, -- 'management', 'analytics', 'system', etc.
    icon VARCHAR(50), -- Frontend icon identifier
    color VARCHAR(20), -- UI color theme
    sort_order INTEGER DEFAULT 0, -- Display ordering
    is_active BOOLEAN DEFAULT true,
    requires_modules TEXT[] DEFAULT '{}', -- Array of required prerequisite modules
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- User to admin module assignments - maps Firebase UIDs to specific modules
CREATE TABLE IF NOT EXISTS user_admin_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL, -- Direct Firebase reference
    module_code VARCHAR(50) NOT NULL REFERENCES admin_modules(module_code),
    granted_by VARCHAR(128), -- Firebase UID of admin who granted role
    granted_reason TEXT, -- Reason for granting access
    expires_at TIMESTAMPTZ, -- Optional expiration date
    is_active BOOLEAN DEFAULT true,
    assignment_metadata JSONB DEFAULT '{}', -- Additional context data
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Ensure unique Firebase UID + module combinations
    CONSTRAINT unique_firebase_uid_module UNIQUE (firebase_uid, module_code)
);

-- Module permissions - defines what resources each module can access
CREATE TABLE IF NOT EXISTS admin_module_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_code VARCHAR(50) NOT NULL REFERENCES admin_modules(module_code),
    api_endpoints TEXT[] NOT NULL DEFAULT '{}', -- Array of allowed API patterns
    frontend_routes TEXT[] NOT NULL DEFAULT '{}', -- Array of allowed frontend routes
    permissions TEXT[] NOT NULL DEFAULT '{}', -- Array of specific permissions
    resource_patterns TEXT[] NOT NULL DEFAULT '{}', -- Array of resource access patterns
    access_level VARCHAR(20) DEFAULT 'read', -- 'read', 'write', 'admin'
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Role assignment audit log
CREATE TABLE IF NOT EXISTS admin_role_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL, -- User whose role was changed
    module_code VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL, -- 'granted', 'revoked', 'expired', 'modified'
    old_status JSONB, -- Previous state
    new_status JSONB, -- New state
    performed_by VARCHAR(128), -- Firebase UID of admin who made change
    reason TEXT, -- Reason for change
    metadata JSONB DEFAULT '{}', -- Additional context
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_admin_modules_code ON admin_modules(module_code);
CREATE INDEX IF NOT EXISTS idx_admin_modules_category ON admin_modules(category);
CREATE INDEX IF NOT EXISTS idx_admin_modules_active ON admin_modules(is_active) WHERE is_active = true;

CREATE INDEX IF NOT EXISTS idx_user_admin_roles_firebase_uid ON user_admin_roles(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_module ON user_admin_roles(module_code);
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_active ON user_admin_roles(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_expires ON user_admin_roles(expires_at) WHERE expires_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_module_permissions_module ON admin_module_permissions(module_code);
CREATE INDEX IF NOT EXISTS idx_module_permissions_access_level ON admin_module_permissions(access_level);

CREATE INDEX IF NOT EXISTS idx_admin_role_audit_firebase_uid ON admin_role_audit(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_admin_role_audit_module ON admin_role_audit(module_code);
CREATE INDEX IF NOT EXISTS idx_admin_role_audit_timestamp ON admin_role_audit(timestamp);

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

-- Add comments for documentation
COMMENT ON TABLE admin_modules IS 'Defines granular admin functional modules';
COMMENT ON TABLE user_admin_roles IS 'Maps Firebase UIDs to specific admin modules';
COMMENT ON TABLE admin_module_permissions IS 'Defines resource access permissions for each admin module';
COMMENT ON TABLE admin_role_audit IS 'Audit trail for admin role assignments and changes';

-- Utility functions for querying user modules
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