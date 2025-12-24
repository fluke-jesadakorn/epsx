-- Split permissions schema: definitions vs grants

-- 1. Ensure all existing grants in 'permissions' have a corresponding definition in 'permissions' (where wallet_address is NULL)
INSERT INTO permissions (
    id, permission_string, platform, resource, action, 
    permission_type, is_active, created_at, updated_at
)
SELECT 
    uuid_generate_v4(), permission_string, platform, resource, action,
    permission_type, true, NOW(), NOW()
FROM permissions 
WHERE wallet_address IS NOT NULL 
GROUP BY permission_string, platform, resource, action, permission_type
ON CONFLICT (permission_string) DO NOTHING;

-- 2. Migrate existing grants from 'permissions' to 'wallet_direct_permissions'
-- using the canonical definition ID
INSERT INTO wallet_direct_permissions (
    wallet_address, permission_id, granted_at, expires_at, 
    granted_by, grant_reason, is_active
)
SELECT 
    p_grant.wallet_address,
    p_def.id,
    COALESCE(p_grant.granted_at, NOW()),
    p_grant.expires_at,
    p_grant.granted_by,
    p_grant.grant_reason,
    p_grant.is_active
FROM permissions p_grant
JOIN permissions p_def ON p_grant.permission_string = p_def.permission_string
WHERE p_grant.wallet_address IS NOT NULL
  AND p_def.wallet_address IS NULL -- Ensure we join against the definition
ON CONFLICT (wallet_address, permission_id) DO NOTHING;

-- 3. Migrate data from 'permission_definitions' (if any) into 'permissions'
-- First add the missing columns to 'permissions' to support the consolidation
ALTER TABLE permissions
    ADD COLUMN name VARCHAR(100),
    ADD COLUMN category VARCHAR(50),
    ADD COLUMN is_system BOOLEAN DEFAULT FALSE NOT NULL;

-- This consolidates our catalogs.
INSERT INTO permissions (
    permission_string, platform, resource, action, 
    name, category, is_system,
    description, is_active, created_at, updated_at
)
SELECT 
    permission, platform, 
    SPLIT_PART(permission, ':', 2), -- resource guess from string
    SPLIT_PART(permission, ':', 3), -- action guess from string
    name, category, is_system,
    description, is_active, created_at, updated_at
FROM permission_definitions
ON CONFLICT (permission_string) 
DO UPDATE SET 
    name = EXCLUDED.name,
    category = EXCLUDED.category,
    is_system = EXCLUDED.is_system,
    description = EXCLUDED.description;

-- 4. Clean up 'permissions' table
-- Delete "grant rows" leaving only "definition rows"
DELETE FROM permissions WHERE wallet_address IS NOT NULL;

-- Drop assignment-related columns
ALTER TABLE permissions 
    DROP COLUMN wallet_address,
    DROP COLUMN source_type,
    DROP COLUMN source_id,
    DROP COLUMN granted_at,
    DROP COLUMN expires_at,
    DROP COLUMN granted_by,
    DROP COLUMN grant_reason,
    DROP COLUMN web3_contract_address, -- These seem better suited for 'permission_definitions' metadata or a separate web3_permissions table if needed, but for now strict cleanup
    DROP COLUMN web3_chain_id,
    DROP COLUMN web3_min_balance,
    DROP COLUMN web3_token_ids,
    DROP COLUMN web3_metadata;

-- 5. Drop the redundant 'permission_definitions' table
DROP TABLE permission_definitions;
