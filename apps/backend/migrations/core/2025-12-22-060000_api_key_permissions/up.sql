-- Create junction table linking API keys to permission groups
CREATE TABLE api_key_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    permission_group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by VARCHAR(42),
    UNIQUE(api_key_id, permission_group_id)
);

-- Indexes for performance
CREATE INDEX idx_api_key_permissions_key ON api_key_permissions(api_key_id);
CREATE INDEX idx_api_key_permissions_group ON api_key_permissions(permission_group_id);
