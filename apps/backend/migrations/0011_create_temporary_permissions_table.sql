-- Temporary Permissions System
-- Allows granting time-bound permissions that automatically expire

CREATE TABLE IF NOT EXISTS temporary_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    action VARCHAR(255) NOT NULL,
    
    -- Time-bound attributes
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    auto_revoke BOOLEAN NOT NULL DEFAULT true,
    
    -- Assignment metadata
    granted_by UUID NOT NULL REFERENCES users(id),
    reason TEXT,
    conditions JSONB DEFAULT '{}',
    
    -- Status tracking
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    revoked_at TIMESTAMPTZ,
    revoked_by UUID REFERENCES users(id),
    revocation_reason TEXT,
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_temporary_permissions_user_id ON temporary_permissions(user_id);
CREATE INDEX IF NOT EXISTS idx_temporary_permissions_expires_at ON temporary_permissions(expires_at);
CREATE INDEX IF NOT EXISTS idx_temporary_permissions_status ON temporary_permissions(status);
CREATE INDEX IF NOT EXISTS idx_temporary_permissions_permission ON temporary_permissions(permission);
CREATE INDEX IF NOT EXISTS idx_temporary_permissions_resource_action ON temporary_permissions(resource, action);

-- Composite index for quick permission checks
CREATE INDEX IF NOT EXISTS idx_temporary_permissions_lookup 
ON temporary_permissions(user_id, permission, resource, action, status, expires_at);

-- Note: Automatic cleanup of expired permissions will be handled by a background job
-- rather than using database triggers for better performance and reliability