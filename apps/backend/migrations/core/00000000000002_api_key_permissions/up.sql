-- ================================================================================================
-- API KEY PERMISSIONS TABLE
-- ================================================================================================
-- Links API keys to permission groups for the new group-based permission system.
-- This allows API keys to inherit permissions from groups, similar to wallet_group_assignments.
-- ================================================================================================

CREATE TABLE api_key_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL,
    permission_group_id UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by VARCHAR(42),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    metadata JSONB DEFAULT '{}' NOT NULL,
    UNIQUE(api_key_id, permission_group_id)
);

-- Indexes for efficient lookups
CREATE INDEX idx_api_key_permissions_api_key ON api_key_permissions(api_key_id) WHERE is_active = TRUE;
CREATE INDEX idx_api_key_permissions_group ON api_key_permissions(permission_group_id) WHERE is_active = TRUE;
CREATE INDEX idx_api_key_permissions_expires ON api_key_permissions(expires_at) WHERE expires_at IS NOT NULL AND is_active = TRUE;
CREATE INDEX idx_api_key_permissions_combined ON api_key_permissions(api_key_id, permission_group_id, is_active) WHERE is_active = TRUE;

-- Foreign key constraints
ALTER TABLE api_key_permissions ADD CONSTRAINT api_key_permissions_api_key_fkey
    FOREIGN KEY (api_key_id) REFERENCES api_keys(id) ON DELETE CASCADE;

ALTER TABLE api_key_permissions ADD CONSTRAINT api_key_permissions_group_fkey
    FOREIGN KEY (permission_group_id) REFERENCES groups(id) ON DELETE CASCADE;

COMMENT ON TABLE api_key_permissions IS 'Links API keys to permission groups for group-based permissions';

SELECT 'API Key Permissions table created successfully! 🔑' AS success_message;
