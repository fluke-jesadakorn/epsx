-- ============================================================================
-- RBAC Permission System Migration
-- Creates normalized role-based + attribute-based access control system
-- Replaces string-based embedded timestamp permissions with proper RBAC
-- ============================================================================

-- Create enum types for better type safety
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
-- CORE PERMISSION DEFINITIONS (Master List)
-- ============================================================================
CREATE TABLE rbac_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL, -- "epsx:analytics:view"
    platform VARCHAR(50) NOT NULL,     -- "epsx", "admin", "epsx-pay" 
    resource VARCHAR(50) NOT NULL,      -- "analytics", "users", "rankings"
    action VARCHAR(50) NOT NULL,        -- "view", "manage", "export"
    description TEXT,
    metadata JSONB DEFAULT '{}',        -- Additional permission metadata
    is_system_permission BOOLEAN DEFAULT false, -- Cannot be deleted
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Ensure unique platform:resource:action combinations
    UNIQUE(platform, resource, action)
);

-- ============================================================================
-- ROLE DEFINITIONS (Permission Groups)
-- ============================================================================
CREATE TABLE rbac_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) UNIQUE NOT NULL,     -- "bronze_user", "admin", "premium"
    display_name VARCHAR(100) NOT NULL,   -- "Bronze Subscriber"
    description TEXT,
    platform_scope VARCHAR(50),           -- "epsx", "admin", "*" for cross-platform
    metadata JSONB DEFAULT '{}',          -- Additional role metadata
    is_system_role BOOLEAN DEFAULT false, -- Cannot be deleted (bronze, admin, etc.)
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- ROLE-PERMISSION MAPPING (M:N)
-- ============================================================================
CREATE TABLE rbac_role_permissions (
    role_id UUID NOT NULL REFERENCES rbac_roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES rbac_permissions(id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    granted_by UUID REFERENCES users(id), -- Who added this permission to the role
    metadata JSONB DEFAULT '{}',
    
    PRIMARY KEY (role_id, permission_id)
);

-- ============================================================================
-- USER-ROLE ASSIGNMENTS (M:N with Temporal Support)
-- ============================================================================
CREATE TABLE rbac_user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES rbac_roles(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id),     -- Who granted this role
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NULL,              -- NULL = permanent role
    is_active BOOLEAN DEFAULT true,
    reason TEXT,                              -- Why this role was granted
    metadata JSONB DEFAULT '{}',              -- Additional context
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Index for efficient queries - allows multiple entries for extending roles
    -- Business logic will handle preventing active overlaps
    CONSTRAINT rbac_user_roles_check CHECK (expires_at IS NULL OR expires_at > created_at)
);

-- ============================================================================
-- DIRECT USER PERMISSIONS (Overrides/Additions to Roles)
-- ============================================================================
CREATE TABLE rbac_user_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES rbac_permissions(id) ON DELETE CASCADE,
    permission_type permission_type_enum NOT NULL, -- 'grant' or 'revoke'
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NULL,               -- NULL = permanent
    is_active BOOLEAN DEFAULT true,
    reason TEXT,                               -- Why this permission was granted/revoked
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Business logic will handle preventing active conflicts
    CONSTRAINT rbac_user_permissions_check CHECK (expires_at IS NULL OR expires_at > created_at)
);

-- ============================================================================
-- COMPREHENSIVE AUDIT TRAIL
-- ============================================================================
CREATE TABLE rbac_permission_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation audit_operation_enum NOT NULL,
    actor_user_id UUID REFERENCES users(id), -- Who performed the action
    target_user_id UUID REFERENCES users(id) NULL, -- Who was affected (for user operations)
    role_id UUID REFERENCES rbac_roles(id) NULL,
    permission_id UUID REFERENCES rbac_permissions(id) NULL,
    old_values JSONB NULL,                    -- Previous state
    new_values JSONB NULL,                    -- New state
    expires_at TIMESTAMPTZ NULL,              -- If temporal change
    reason TEXT,                              -- Why the change was made
    ip_address INET,                          -- Source IP
    user_agent TEXT,                          -- User agent
    session_id UUID,                          -- Session identifier
    request_id VARCHAR(255),                  -- Request tracking
    metadata JSONB DEFAULT '{}',              -- Additional context
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- DYNAMIC USER LIMITS (Admin-Controlled Overrides)
-- ============================================================================
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
    
    -- Only one active limit per type per user (highest priority wins)
    UNIQUE(user_id, limit_type, priority)
);

-- ============================================================================
-- PERFORMANCE INDEXES
-- ============================================================================

-- Permissions table indexes
CREATE INDEX idx_rbac_permissions_platform_resource_action ON rbac_permissions(platform, resource, action);
CREATE INDEX idx_rbac_permissions_active ON rbac_permissions(is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_permissions_platform ON rbac_permissions(platform);

-- Roles table indexes  
CREATE INDEX idx_rbac_roles_active ON rbac_roles(is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_roles_platform_scope ON rbac_roles(platform_scope);
CREATE INDEX idx_rbac_roles_system ON rbac_roles(is_system_role);

-- Role permissions indexes
CREATE INDEX idx_rbac_role_permissions_role ON rbac_role_permissions(role_id);
CREATE INDEX idx_rbac_role_permissions_permission ON rbac_role_permissions(permission_id);

-- User roles indexes (most critical for performance)
CREATE INDEX idx_rbac_user_roles_user_active ON rbac_user_roles(user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_user_roles_expires_at ON rbac_user_roles(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_rbac_user_roles_active_valid ON rbac_user_roles(user_id, expires_at, is_active) 
    WHERE is_active = true;
CREATE INDEX idx_rbac_user_roles_expiring_soon ON rbac_user_roles(expires_at, user_id) 
    WHERE expires_at IS NOT NULL;

-- User permissions indexes
CREATE INDEX idx_rbac_user_permissions_user_active ON rbac_user_permissions(user_id, is_active) 
    WHERE is_active = true;
CREATE INDEX idx_rbac_user_permissions_expires_at ON rbac_user_permissions(expires_at) 
    WHERE expires_at IS NOT NULL;
CREATE INDEX idx_rbac_user_permissions_type ON rbac_user_permissions(permission_type);
CREATE INDEX idx_rbac_user_permissions_validation ON rbac_user_permissions(user_id, permission_id, permission_type, is_active, expires_at);

-- Audit log indexes
CREATE INDEX idx_rbac_audit_log_actor ON rbac_permission_audit_log(actor_user_id, created_at DESC);
CREATE INDEX idx_rbac_audit_log_target ON rbac_permission_audit_log(target_user_id, created_at DESC);
CREATE INDEX idx_rbac_audit_log_operation ON rbac_permission_audit_log(operation, created_at DESC);
CREATE INDEX idx_rbac_audit_log_created_at ON rbac_permission_audit_log(created_at DESC);

-- User limits indexes
CREATE INDEX idx_rbac_user_limits_user_active ON rbac_user_limits(user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_rbac_user_limits_type ON rbac_user_limits(limit_type);
CREATE INDEX idx_rbac_user_limits_expires_at ON rbac_user_limits(expires_at) WHERE expires_at IS NOT NULL;

-- ============================================================================
-- FUNCTIONS FOR PERMISSION MANAGEMENT
-- ============================================================================

-- Function to check if user has permission
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

-- Function to cleanup expired permissions
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

-- ============================================================================
-- SEED DATA - SYSTEM PERMISSIONS AND ROLES
-- ============================================================================

-- Core system permissions
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

-- Log initial setup
INSERT INTO rbac_permission_audit_log (operation, actor_user_id, reason, metadata) VALUES (
    'migration',
    NULL,
    'Initial RBAC system setup',
    jsonb_build_object(
        'permissions_created', (SELECT COUNT(*) FROM rbac_permissions),
        'roles_created', (SELECT COUNT(*) FROM rbac_roles), 
        'mappings_created', (SELECT COUNT(*) FROM rbac_role_permissions),
        'setup_time', NOW()
    )
);
