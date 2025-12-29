-- Revert split permissions schema

-- Note: This is an imperfect revert because we dropped columns and tables. We recreate them but data might be lost or hard to restore perfectly.

-- 1. Recreate 'permission_definitions' table
CREATE TABLE permission_definitions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    permission VARCHAR(255) NOT NULL,
    name VARCHAR(100),
    description TEXT,
    platform VARCHAR(50) NOT NULL,
    category VARCHAR(50),
    is_system BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by VARCHAR(42),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Restore columns to 'permissions'
ALTER TABLE permissions 
    ADD COLUMN wallet_address VARCHAR(42),
    ADD COLUMN source_type VARCHAR(20),
    ADD COLUMN source_id UUID,
    ADD COLUMN granted_at TIMESTAMPTZ,
    ADD COLUMN expires_at TIMESTAMPTZ,
    ADD COLUMN granted_by VARCHAR(255),
    ADD COLUMN grant_reason TEXT,
    ADD COLUMN web3_contract_address VARCHAR(42),
    ADD COLUMN web3_chain_id BIGINT,
    ADD COLUMN web3_min_balance VARCHAR(78),
    ADD COLUMN web3_token_ids JSONB,
    ADD COLUMN web3_metadata JSONB;

-- 3. Move grants back from 'wallet_direct_permissions' to 'permissions'
-- (This creates "grant rows" again)
INSERT INTO permissions (
    id, permission_string, platform, resource, action,
    wallet_address, granted_at, expires_at, granted_by, grant_reason, is_active,
    permission_type, source_type
)
SELECT 
    uuid_generate_v4(), 
    p.permission_string, p.platform, p.resource, p.action,
    wdp.wallet_address, wdp.granted_at, wdp.expires_at, wdp.granted_by, wdp.grant_reason, wdp.is_active,
    p.permission_type, 'direct'
FROM wallet_direct_permissions wdp
JOIN permissions p ON wdp.permission_id = p.id;

-- 4. Note: We do NOT delete from wallet_direct_permissions in the revert to avoid data loss if revert is accidental.
-- But strictly speaking, a full revert implies moving data back.
