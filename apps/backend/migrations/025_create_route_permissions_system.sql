-- Migration 025: Create Route Permissions System
-- Creates dynamic route-to-permission mapping system for centralized permission authority
-- Replaces hardcoded permission mapping with database-driven configuration

-- ============================================================================
-- CREATE ROUTE PERMISSIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS route_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Route configuration
    route_pattern VARCHAR(500) NOT NULL,
    http_method VARCHAR(10) NOT NULL DEFAULT '*',
    required_permission VARCHAR(255) NOT NULL,
    
    -- Priority and status
    priority INTEGER NOT NULL DEFAULT 100,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_public BOOLEAN NOT NULL DEFAULT false,
    
    -- Metadata
    description TEXT,
    route_category VARCHAR(100) DEFAULT 'api',
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),
    last_modified_by VARCHAR(255),
    
    -- Constraints
    CONSTRAINT route_permissions_method_check CHECK (
        http_method IN ('*', 'GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS', 'HEAD')
    ),
    CONSTRAINT route_permissions_priority_check CHECK (priority >= 0 AND priority <= 9999),
    CONSTRAINT route_permissions_pattern_not_empty CHECK (LENGTH(TRIM(route_pattern)) > 0),
    CONSTRAINT route_permissions_permission_not_empty CHECK (LENGTH(TRIM(required_permission)) > 0)
);

-- ============================================================================
-- CREATE INDEXES FOR PERFORMANCE
-- ============================================================================

-- Primary lookup index for route resolution
CREATE INDEX IF NOT EXISTS idx_route_permissions_lookup 
ON route_permissions (is_active, priority DESC, route_pattern, http_method)
WHERE is_active = true;

-- Pattern matching index for wildcard lookups
CREATE INDEX IF NOT EXISTS idx_route_permissions_patterns 
ON route_permissions USING gin (route_pattern gin_trgm_ops)
WHERE is_active = true;

-- Method-specific index
CREATE INDEX IF NOT EXISTS idx_route_permissions_method 
ON route_permissions (http_method, is_active)
WHERE is_active = true;

-- Permission lookup index
CREATE INDEX IF NOT EXISTS idx_route_permissions_permission 
ON route_permissions (required_permission, is_active)
WHERE is_active = true;

-- Category index for admin interfaces
CREATE INDEX IF NOT EXISTS idx_route_permissions_category 
ON route_permissions (route_category, is_active, priority DESC);

-- Audit index
CREATE INDEX IF NOT EXISTS idx_route_permissions_audit 
ON route_permissions (created_at, updated_at);

-- ============================================================================
-- CREATE UNIQUE CONSTRAINT
-- ============================================================================

-- Ensure unique route pattern and method combinations
CREATE UNIQUE INDEX IF NOT EXISTS idx_route_permissions_unique_route 
ON route_permissions (route_pattern, http_method)
WHERE is_active = true;

-- ============================================================================
-- CREATE TRIGGERS FOR UPDATED_AT
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_route_permissions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to automatically update updated_at
CREATE TRIGGER route_permissions_updated_at_trigger
    BEFORE UPDATE ON route_permissions
    FOR EACH ROW
    EXECUTE FUNCTION update_route_permissions_updated_at();

-- ============================================================================
-- INSERT DEFAULT ROUTE PERMISSIONS
-- ============================================================================

-- Public routes (highest priority)
INSERT INTO route_permissions 
    (route_pattern, http_method, required_permission, priority, is_public, description, route_category) 
VALUES
    ('/health', '*', 'public', 1000, true, 'Health check endpoint', 'health'),
    ('/readiness', '*', 'public', 1000, true, 'Readiness probe endpoint', 'health'),
    ('/liveness', '*', 'public', 1000, true, 'Liveness probe endpoint', 'health'),
    ('/api/v1/public/**', '*', 'public', 950, true, 'Public API endpoints', 'public'),
    ('/api/auth/web3/challenge', 'POST', 'public', 950, true, 'Web3 authentication challenge', 'auth'),
    ('/docs', 'GET', 'public', 900, true, 'API documentation', 'docs'),
    ('/api-docs/**', 'GET', 'public', 900, true, 'OpenAPI specification', 'docs')
ON CONFLICT (route_pattern, http_method) DO NOTHING;

-- Authentication routes
INSERT INTO route_permissions 
    (route_pattern, http_method, required_permission, priority, is_public, description, route_category) 
VALUES
    ('/api/auth/web3/verify', 'POST', 'auth:web3:verify', 800, false, 'Web3 signature verification', 'auth'),
    ('/api/v1/auth/session/verify', 'POST', 'auth:session:verify', 800, false, 'Session verification', 'auth'),
    ('/api/v1/auth/web3/token', 'POST', 'auth:token:create', 800, false, 'Token creation', 'auth'),
    ('/api/v1/auth/userinfo', 'GET', 'auth:userinfo:read', 800, false, 'User information', 'auth'),
    ('/api/v1/auth/token/revoke', 'POST', 'auth:token:revoke', 800, false, 'Token revocation', 'auth')
ON CONFLICT (route_pattern, http_method) DO NOTHING;

-- Admin routes (high priority)
INSERT INTO route_permissions 
    (route_pattern, http_method, required_permission, priority, is_public, description, route_category) 
VALUES
    ('/admin/users/**', '*', 'admin:users:manage', 900, false, 'Admin user management', 'admin'),
    ('/admin/permission-groups/**', '*', 'admin:permission-groups:manage', 900, false, 'Admin permission group management', 'admin'),
    ('/admin/web3/**', '*', 'admin:web3:manage', 900, false, 'Admin Web3 management', 'admin'),
    ('/admin/analytics/**', 'GET', 'admin:analytics:read', 850, false, 'Admin analytics access', 'admin'),
    ('/admin/security/**', 'GET', 'admin:security:read', 850, false, 'Admin security monitoring', 'admin'),
    ('/admin/performance/**', '*', 'admin:performance:manage', 850, false, 'Admin performance management', 'admin')
ON CONFLICT (route_pattern, http_method) DO NOTHING;

-- API Admin routes (for frontend compatibility)
INSERT INTO route_permissions 
    (route_pattern, http_method, required_permission, priority, is_public, description, route_category) 
VALUES
    ('/api/admin/**', '*', 'admin:api:access', 800, false, 'Admin API access', 'admin-api'),
    ('/api/v1/admin/**', '*', 'admin:api:access', 800, false, 'Admin API v1 access', 'admin-api')
ON CONFLICT (route_pattern, http_method) DO NOTHING;

-- Analytics routes
INSERT INTO route_permissions 
    (route_pattern, http_method, required_permission, priority, is_public, description, route_category) 
VALUES
    ('/api/v1/analytics/**', 'GET', 'epsx:analytics:read', 700, false, 'Analytics data access', 'analytics'),
    ('/analytics/**', 'GET', 'epsx:analytics:read', 700, false, 'Analytics direct access', 'analytics')
ON CONFLICT (route_pattern, http_method) DO NOTHING;

-- User data routes
INSERT INTO route_permissions 
    (route_pattern, http_method, required_permission, priority, is_public, description, route_category) 
VALUES
    ('/api/v1/users/permissions', 'GET', 'epsx:data:read', 600, false, 'User permissions access', 'user-data'),
    ('/api/v1/users/holdings', 'GET', 'epsx:data:read', 600, false, 'User holdings access', 'user-data'),
    ('/api/v1/users/verify', 'POST', 'epsx:data:verify', 600, false, 'User verification', 'user-data'),
    ('/api/v1/users/**', '*', 'epsx:data:access', 550, false, 'General user data access', 'user-data')
ON CONFLICT (route_pattern, http_method) DO NOTHING;

-- Permission validation routes
INSERT INTO route_permissions 
    (route_pattern, http_method, required_permission, priority, is_public, description, route_category) 
VALUES
    ('/api/permissions/validate', 'POST', 'admin:permissions:validate', 750, false, 'Permission validation', 'permissions'),
    ('/api/permissions/validate-bulk', 'POST', 'admin:permissions:validate', 750, false, 'Bulk permission validation', 'permissions'),
    ('/api/permissions/user/*', 'GET', 'admin:permissions:read', 750, false, 'User permission read', 'permissions')
ON CONFLICT (route_pattern, http_method) DO NOTHING;

-- Default fallback (lowest priority)
INSERT INTO route_permissions 
    (route_pattern, http_method, required_permission, priority, is_public, description, route_category) 
VALUES
    ('/**', '*', 'epsx:basic:access', 1, false, 'Default authenticated access', 'default')
ON CONFLICT (route_pattern, http_method) DO NOTHING;

-- ============================================================================
-- CREATE HELPER FUNCTIONS
-- ============================================================================

-- Function to resolve permission for a route
CREATE OR REPLACE FUNCTION resolve_route_permission(
    p_method VARCHAR(10),
    p_path VARCHAR(500)
)
RETURNS TABLE (
    permission VARCHAR(255),
    is_public BOOLEAN,
    matched_pattern VARCHAR(500),
    priority INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        rp.required_permission,
        rp.is_public,
        rp.route_pattern,
        rp.priority
    FROM route_permissions rp
    WHERE rp.is_active = true
      AND (rp.http_method = '*' OR rp.http_method = UPPER(p_method))
      AND (
          rp.route_pattern = p_path OR
          (rp.route_pattern LIKE '%*%' AND p_path ~ replace(replace(rp.route_pattern, '*', '.*'), '/', '\/')) OR
          (rp.route_pattern LIKE '%:%' AND p_path ~ replace(replace(rp.route_pattern, ':', '[^/]+'), '/', '\/'))
      )
    ORDER BY rp.priority DESC, LENGTH(rp.route_pattern) DESC
    LIMIT 1;
END;
$$ LANGUAGE plpgsql;

-- Function to register new route permission
CREATE OR REPLACE FUNCTION register_route_permission(
    p_route_pattern VARCHAR(500),
    p_method VARCHAR(10),
    p_permission VARCHAR(255),
    p_priority INTEGER DEFAULT 100,
    p_is_public BOOLEAN DEFAULT false,
    p_description TEXT DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_id UUID;
BEGIN
    INSERT INTO route_permissions 
        (route_pattern, http_method, required_permission, priority, is_public, description)
    VALUES 
        (p_route_pattern, UPPER(p_method), p_permission, p_priority, p_is_public, p_description)
    ON CONFLICT (route_pattern, http_method) 
    DO UPDATE SET 
        required_permission = EXCLUDED.required_permission,
        priority = EXCLUDED.priority,
        is_public = EXCLUDED.is_public,
        description = EXCLUDED.description,
        updated_at = NOW()
    RETURNING id INTO v_id;
    
    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- CREATE ADMIN VIEWS FOR MONITORING
-- ============================================================================

-- View for route permission summary
CREATE OR REPLACE VIEW route_permissions_summary AS
SELECT 
    route_category,
    COUNT(*) as total_routes,
    COUNT(*) FILTER (WHERE is_public = true) as public_routes,
    COUNT(*) FILTER (WHERE is_public = false) as protected_routes,
    COUNT(*) FILTER (WHERE is_active = true) as active_routes,
    COUNT(*) FILTER (WHERE is_active = false) as inactive_routes,
    AVG(priority) as avg_priority
FROM route_permissions
GROUP BY route_category
ORDER BY total_routes DESC;

-- View for permission usage analysis
CREATE OR REPLACE VIEW permission_usage_analysis AS
SELECT 
    required_permission,
    COUNT(*) as route_count,
    ARRAY_AGG(DISTINCT route_category) as categories,
    ARRAY_AGG(route_pattern ORDER BY priority DESC) as patterns,
    AVG(priority) as avg_priority,
    COUNT(*) FILTER (WHERE is_public = true) as public_count,
    COUNT(*) FILTER (WHERE is_public = false) as protected_count
FROM route_permissions
WHERE is_active = true
GROUP BY required_permission
ORDER BY route_count DESC;

-- ============================================================================
-- GRANT PERMISSIONS
-- ============================================================================

-- Grant permissions to application user (adjust as needed)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON route_permissions TO your_app_user;
-- GRANT EXECUTE ON FUNCTION resolve_route_permission TO your_app_user;
-- GRANT EXECUTE ON FUNCTION register_route_permission TO your_app_user;
-- GRANT SELECT ON route_permissions_summary TO your_app_user;
-- GRANT SELECT ON permission_usage_analysis TO your_app_user;

-- ============================================================================
-- MIGRATION COMPLETION
-- ============================================================================

-- Log migration completion
DO $$
BEGIN
    RAISE NOTICE 'Migration 025: Route Permissions System created successfully';
    RAISE NOTICE '- Created route_permissions table with % initial routes', 
        (SELECT COUNT(*) FROM route_permissions);
    RAISE NOTICE '- Created indexes for performance optimization';
    RAISE NOTICE '- Created helper functions for route resolution';
    RAISE NOTICE '- Created monitoring views for admin interfaces';
END $$;