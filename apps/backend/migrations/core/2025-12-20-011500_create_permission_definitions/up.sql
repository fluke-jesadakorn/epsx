-- Create permission_definitions table for storing available/custom permissions
CREATE TABLE IF NOT EXISTS permission_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- The permission string (e.g., "epsx:analytics:view")
    permission VARCHAR(255) NOT NULL UNIQUE,
    
    -- Human-readable name
    name VARCHAR(100),
    
    -- Description of what this permission grants
    description TEXT,
    
    -- Platform this permission belongs to (e.g., "epsx", "admin", "epsx-pay")
    platform VARCHAR(50) NOT NULL DEFAULT 'epsx',
    
    -- Category for grouping (e.g., "analytics", "trading", "admin")
    category VARCHAR(50),
    
    -- Whether this is a system-defined permission (cannot be deleted)
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Whether this permission is currently active/usable
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Audit fields
    created_by VARCHAR(42),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX idx_permission_definitions_platform ON permission_definitions(platform);
CREATE INDEX idx_permission_definitions_category ON permission_definitions(category);
CREATE INDEX idx_permission_definitions_is_active ON permission_definitions(is_active);
CREATE INDEX idx_permission_definitions_permission ON permission_definitions(permission);

-- Add updated_at trigger
CREATE OR REPLACE FUNCTION update_permission_definitions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_permission_definitions_updated_at
    BEFORE UPDATE ON permission_definitions
    FOR EACH ROW
    EXECUTE FUNCTION update_permission_definitions_updated_at();

-- Seed with default system permissions
INSERT INTO permission_definitions (permission, name, description, platform, category, is_system) VALUES
    ('epsx:analytics:view', 'View Analytics', 'View basic analytics and market data', 'epsx', 'analytics', TRUE),
    ('epsx:analytics:advanced', 'Advanced Analytics', 'Access advanced analytics features', 'epsx', 'analytics', TRUE),
    ('epsx:trading:basic', 'Basic Trading', 'Execute basic trading operations', 'epsx', 'trading', TRUE),
    ('epsx:trading:advanced', 'Advanced Trading', 'Access advanced trading features', 'epsx', 'trading', TRUE),
    ('epsx:trading:pro', 'Pro Trading', 'Access professional trading tools', 'epsx', 'trading', TRUE),
    ('epsx:data:export', 'Data Export', 'Export data to various formats', 'epsx', 'data', TRUE),
    ('epsx:api:read', 'API Read Access', 'Read-only API access', 'epsx', 'api', TRUE),
    ('epsx:api:write', 'API Write Access', 'Read and write API access', 'epsx', 'api', TRUE),
    ('epsx:notifications:manage', 'Manage Notifications', 'Manage notification settings', 'epsx', 'notifications', TRUE),
    ('admin:users:view', 'View Users', 'View user information', 'admin', 'users', TRUE),
    ('admin:users:manage', 'Manage Users', 'Create, update, and disable users', 'admin', 'users', TRUE),
    ('admin:permissions:view', 'View Permissions', 'View permission assignments', 'admin', 'permissions', TRUE),
    ('admin:permissions:manage', 'Manage Permissions', 'Assign and revoke permissions', 'admin', 'permissions', TRUE),
    ('admin:*:*', 'Full Admin Access', 'Full administrative access to all features', 'admin', 'admin', TRUE)
ON CONFLICT (permission) DO NOTHING;
