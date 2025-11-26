-- Migration: Add Route Permissions (UP)
-- Description: Creates route_permissions table and inserts default API route mappings
-- Created: 2025-11-18

-- Create route_permissions table
CREATE TABLE IF NOT EXISTS route_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_pattern VARCHAR(255) NOT NULL,
    http_method VARCHAR(10) NOT NULL,
    required_permission VARCHAR(255) NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    is_public BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default route permissions for common API routes
INSERT INTO route_permissions (route_pattern, http_method, required_permission, priority, is_public) VALUES
    ('/api/auth/login', 'POST', 'auth:login', 10, false),
    ('/api/auth/logout', 'POST', 'auth:logout', 10, false),
    ('/api/auth/refresh', 'POST', 'auth:refresh', 10, false),
    ('/api/auth/register', 'POST', 'auth:register', 10, false),
    ('/api/auth/profile', 'GET', 'auth:profile', 10, false),
    ('/api/users/profile', 'GET', 'users:profile', 10, false),
    ('/api/users/profile', 'PUT', 'users:profile', 10, false),
    ('/api/users/profile', 'PATCH', 'users:profile', 10, false),
    ('/api/admin/auth/login', 'POST', 'admin:auth:login', 20, false),
    ('/api/admin/auth/logout', 'POST', 'admin:auth:logout', 20, false),
    ('/api/admin/users/list', 'GET', 'admin:users:list', 30, false),
    ('/api/admin/users/{wallet_address}', 'GET', 'admin:users:get', 30, false),
    ('/api/admin/users/{wallet_address}', 'PUT', 'admin:users:update', 30, false),
    ('/api/admin/permissions/validate', 'POST', 'admin:permissions:validate', 30, false),
    ('/api/admin/permissions/validate-bulk', 'POST', 'admin:permissions:validate_bulk', 30, false),
    ('/api/admin/permissions/wallet/{wallet_address}', 'GET', 'admin:permissions:wallet', 30, false),
    ('/api/admin/permissions/groups/list', 'GET', 'admin:permissions:groups', 30, false),
    ('/api/admin/permissions/groups/{group_id}', 'GET', 'admin:permissions:group', 30, false),
    ('/api/admin/permissions/grant', 'POST', 'admin:permissions:grant', 30, false),
    ('/api/admin/permissions/revoke', 'DELETE', 'admin:permissions:revoke', 30, false),
    ('/api/admin/permissions/bulk-grant', 'POST', 'admin:permissions:bulk_grant', 30, false),
    ('/api/admin/permissions/bulk-revoke', 'POST', 'admin:permissions:bulk_revoke', 30, false),
    ('/api/admin/permissions/register-route', 'POST', 'admin:permissions:register', 30, false),
    ('/api/admin/permissions/routes', 'GET', 'admin:permissions:routes', 30, false);

-- Add table comment
COMMENT ON TABLE route_permissions IS 'Table for storing route permission mappings for API authorization';