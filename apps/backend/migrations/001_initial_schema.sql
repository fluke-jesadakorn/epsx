-- ================================================================================================
-- EPSX INITIAL DATABASE SCHEMA
-- ================================================================================================
-- This is the consolidated initial schema for the EPSX platform.
-- It represents the final state after all migrations have been applied.
--
-- Core Components:
-- - Web3-first wallet authentication system
-- - Normalized permission management with groups
-- - OpenID Connect token management
-- - Route-based permission protection
-- - Materialized views for performance
--
-- Version: 1.0.0
-- Created: 2025-01-04
-- Based on: Migrations 002-027 consolidated
-- ================================================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- ================================================================================================
-- EXTENSIONS
-- ================================================================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;
COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;
COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';

CREATE EXTENSION IF NOT EXISTS btree_gist WITH SCHEMA public;
COMMENT ON EXTENSION btree_gist IS 'support for indexing common datatypes in GiST';

-- ================================================================================================
-- CORE TABLES
-- ================================================================================================

-- ------------------------------------------------------------------------------------------------
-- WALLET USERS TABLE - Primary user accounts
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS wallet_users (
    wallet_address VARCHAR(42) PRIMARY KEY,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    tier_level VARCHAR(20) DEFAULT 'Bronze' NOT NULL,
    wallet_metadata JSONB DEFAULT '{}' NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    last_auth_at TIMESTAMPTZ,
    permission_groups JSONB DEFAULT '[]',

    -- Constraints
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    ),
    CONSTRAINT valid_tier_level CHECK (
        tier_level IN ('Bronze', 'Silver', 'Gold', 'Platinum', 'Diamond')
    )
);

CREATE INDEX idx_wallet_users_active ON wallet_users(is_active, wallet_address);
CREATE INDEX idx_wallet_users_tier ON wallet_users(tier_level, is_active);
CREATE INDEX idx_wallet_users_created ON wallet_users(created_at DESC);
CREATE INDEX idx_wallet_users_updated ON wallet_users(updated_at DESC);
CREATE INDEX idx_wallet_users_last_auth ON wallet_users(last_auth_at DESC) WHERE last_auth_at IS NOT NULL;
CREATE INDEX idx_wallet_users_metadata_gin ON wallet_users USING gin(wallet_metadata);

COMMENT ON TABLE wallet_users IS 'Wallet user accounts - permissions granted via wallet_group_assignments and wallet_direct_permissions';

-- ------------------------------------------------------------------------------------------------
-- PERMISSIONS TABLE - Normalized permission definitions
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Permission identity
    permission_string VARCHAR(255) UNIQUE NOT NULL,
    platform VARCHAR(100) NOT NULL,
    resource VARCHAR(100) NOT NULL,
    action VARCHAR(100) NOT NULL,
    description TEXT,

    -- Web3 permission support
    permission_type VARCHAR(50) DEFAULT 'manual' NOT NULL,
    web3_contract_address VARCHAR(42),
    web3_chain_id BIGINT,
    web3_min_balance VARCHAR(78),
    web3_token_ids JSONB DEFAULT '[]',
    web3_metadata JSONB DEFAULT '{}',

    -- Status and audit
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_by VARCHAR(42),

    -- Constraints
    CONSTRAINT permissions_type_check CHECK (
        permission_type IN ('manual', 'nft_gated', 'token_gated', 'dao_governance')
    ),
    CONSTRAINT permissions_platform_not_empty CHECK (LENGTH(TRIM(platform)) > 0),
    CONSTRAINT permissions_resource_not_empty CHECK (LENGTH(TRIM(resource)) > 0),
    CONSTRAINT permissions_action_not_empty CHECK (LENGTH(TRIM(action)) > 0)
);

CREATE INDEX idx_permissions_lookup ON permissions(permission_string) WHERE is_active = TRUE;
CREATE INDEX idx_permissions_platform ON permissions(platform, resource, action);
CREATE INDEX idx_permissions_type ON permissions(permission_type, is_active);
CREATE INDEX idx_permissions_web3 ON permissions(permission_type, web3_chain_id, web3_contract_address)
    WHERE permission_type != 'manual';
CREATE INDEX idx_permissions_audit ON permissions(created_at, updated_at);

COMMENT ON TABLE permissions IS 'Normalized permission definitions - single source of truth for all available permissions';
COMMENT ON COLUMN permissions.permission_string IS 'Full permission string in format platform:resource:action';
COMMENT ON COLUMN permissions.permission_type IS 'How permission is validated: manual, nft_gated, token_gated, or dao_governance';

-- ------------------------------------------------------------------------------------------------
-- PERMISSION GROUPS TABLE - Group definitions
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS permission_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    description TEXT DEFAULT '' NOT NULL,
    group_type VARCHAR(20) DEFAULT 'manual' NOT NULL,
    group_metadata JSONB DEFAULT '{}' NOT NULL,

    -- Subscription pricing
    price NUMERIC(10,2) DEFAULT 0.00,
    currency VARCHAR(3) DEFAULT 'USD',
    billing_cycle VARCHAR(20) DEFAULT 'monthly',

    -- Status and display
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    is_promoted BOOLEAN DEFAULT FALSE NOT NULL,
    display_order INTEGER DEFAULT 0,

    -- Auto-assignment
    max_members INTEGER,
    auto_assign_enabled BOOLEAN DEFAULT FALSE,
    assignment_rules JSONB DEFAULT '{}',

    -- Audit
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_by VARCHAR(42),
    last_modified_by VARCHAR(42),

    -- Constraints
    CONSTRAINT valid_group_type CHECK (
        group_type IN ('manual', 'subscription', 'web3_asset', 'dao_membership', 'admin')
    ),
    CONSTRAINT valid_currency CHECK (
        currency IN ('USD', 'EUR', 'BTC', 'ETH', 'BNB')
    ),
    CONSTRAINT valid_billing_cycle CHECK (
        billing_cycle IN ('monthly', 'yearly', 'one_time', 'lifetime')
    ),
    CONSTRAINT valid_wallet_addresses CHECK (
        (created_by IS NULL OR (created_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(created_by) = 42))
        AND (last_modified_by IS NULL OR (last_modified_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(last_modified_by) = 42))
    )
);

CREATE INDEX idx_permission_groups_active ON permission_groups(is_active, display_order);
CREATE INDEX idx_permission_groups_type ON permission_groups(group_type, is_active);
CREATE INDEX idx_permission_groups_slug ON permission_groups(slug);
CREATE INDEX idx_permission_groups_price ON permission_groups(price, currency) WHERE group_type = 'subscription';
CREATE INDEX idx_permission_groups_created ON permission_groups(created_at DESC);
CREATE INDEX idx_permission_groups_metadata_gin ON permission_groups USING gin(group_metadata);
CREATE INDEX idx_permission_groups_assignment_rules_gin ON permission_groups USING gin(assignment_rules);

COMMENT ON TABLE permission_groups IS 'Permission group definitions - groups are composed of permissions via permission_group_memberships';

-- ------------------------------------------------------------------------------------------------
-- PERMISSION GROUP MEMBERSHIPS TABLE - Group to Permission mapping
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS permission_group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Relationships
    group_id UUID NOT NULL,
    permission_id UUID NOT NULL,

    -- Metadata
    granted_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    granted_by VARCHAR(42),
    grant_reason TEXT,

    -- Constraints
    UNIQUE(group_id, permission_id)
);

CREATE INDEX idx_pg_memberships_group ON permission_group_memberships(group_id);
CREATE INDEX idx_pg_memberships_permission ON permission_group_memberships(permission_id);
CREATE INDEX idx_pg_memberships_audit ON permission_group_memberships(granted_at);

COMMENT ON TABLE permission_group_memberships IS 'Maps permission groups to individual permissions - enables permission group inheritance';

-- ------------------------------------------------------------------------------------------------
-- WALLET GROUP ASSIGNMENTS TABLE - Wallet to Group mapping
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS wallet_group_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Relationships
    wallet_address VARCHAR(42) NOT NULL,
    group_id UUID NOT NULL,

    -- Assignment metadata
    assigned_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ,
    assigned_by VARCHAR(42),
    assignment_reason TEXT,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,

    -- Constraints
    UNIQUE(wallet_address, group_id)
);

CREATE INDEX idx_wg_assignments_wallet ON wallet_group_assignments(wallet_address, is_active);
CREATE INDEX idx_wg_assignments_group ON wallet_group_assignments(group_id, is_active);
CREATE INDEX idx_wg_assignments_expires ON wallet_group_assignments(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_wg_assignments_audit ON wallet_group_assignments(assigned_at);

COMMENT ON TABLE wallet_group_assignments IS 'Assigns wallets to permission groups with optional expiration';

-- ------------------------------------------------------------------------------------------------
-- WALLET DIRECT PERMISSIONS TABLE - Direct permission grants
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS wallet_direct_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Relationships
    wallet_address VARCHAR(42) NOT NULL,
    permission_id UUID NOT NULL,

    -- Grant metadata
    granted_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ,
    granted_by VARCHAR(42),
    grant_reason TEXT,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,

    -- Constraints
    UNIQUE(wallet_address, permission_id)
);

CREATE INDEX idx_direct_perms_wallet ON wallet_direct_permissions(wallet_address, is_active);
CREATE INDEX idx_direct_perms_permission ON wallet_direct_permissions(permission_id);
CREATE INDEX idx_direct_perms_expires ON wallet_direct_permissions(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_direct_perms_audit ON wallet_direct_permissions(granted_at);

COMMENT ON TABLE wallet_direct_permissions IS 'Direct permission grants to wallets (exceptions, temporary access)';

-- ------------------------------------------------------------------------------------------------
-- WEB3 AUTH NONCES TABLE - SIWE authentication challenges
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS web3_auth_nonces (
    wallet_address VARCHAR(42) PRIMARY KEY,
    nonce VARCHAR(64) NOT NULL,
    message TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,

    -- Constraints
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    )
);

CREATE INDEX idx_web3_auth_nonces_expires_at ON web3_auth_nonces(expires_at);

COMMENT ON TABLE web3_auth_nonces IS 'Temporary nonces for Web3 SIWE authentication challenges';
COMMENT ON COLUMN web3_auth_nonces.wallet_address IS 'Wallet address (primary key)';
COMMENT ON COLUMN web3_auth_nonces.nonce IS 'Cryptographic nonce for challenge';
COMMENT ON COLUMN web3_auth_nonces.message IS 'SIWE message containing challenge details';
COMMENT ON COLUMN web3_auth_nonces.expires_at IS 'When this nonce expires';

-- ------------------------------------------------------------------------------------------------
-- OPENID REFRESH TOKENS TABLE - Token management
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS openid_refresh_tokens (
    token_id VARCHAR(36) PRIMARY KEY,
    wallet_address VARCHAR(42) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    is_revoked BOOLEAN DEFAULT FALSE NOT NULL,

    -- Constraints
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    )
);

CREATE INDEX idx_openid_refresh_tokens_wallet_address ON openid_refresh_tokens(wallet_address);
CREATE INDEX idx_openid_refresh_tokens_expires_at ON openid_refresh_tokens(expires_at);
CREATE INDEX idx_openid_refresh_tokens_active ON openid_refresh_tokens(wallet_address, is_revoked, expires_at)
    WHERE is_revoked = FALSE;

COMMENT ON TABLE openid_refresh_tokens IS 'OpenID Connect refresh tokens for Web3-authenticated users';
COMMENT ON COLUMN openid_refresh_tokens.token_id IS 'Unique refresh token identifier (UUID)';
COMMENT ON COLUMN openid_refresh_tokens.wallet_address IS 'Web3 wallet address of the token owner';
COMMENT ON COLUMN openid_refresh_tokens.expires_at IS 'When this refresh token expires';
COMMENT ON COLUMN openid_refresh_tokens.created_at IS 'When this refresh token was created';
COMMENT ON COLUMN openid_refresh_tokens.is_revoked IS 'Whether this refresh token has been revoked';

-- ------------------------------------------------------------------------------------------------
-- ROUTE PERMISSIONS TABLE - API route protection
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS route_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_pattern VARCHAR(500) NOT NULL,
    http_method VARCHAR(10) DEFAULT '*' NOT NULL,
    required_permission VARCHAR(255) NOT NULL,
    priority INTEGER DEFAULT 100 NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    is_public BOOLEAN DEFAULT FALSE NOT NULL,
    description TEXT,
    route_category VARCHAR(100) DEFAULT 'api',
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_by VARCHAR(255),
    last_modified_by VARCHAR(255),

    -- Constraints
    CONSTRAINT route_permissions_method_check CHECK (
        http_method IN ('*', 'GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS', 'HEAD')
    ),
    CONSTRAINT route_permissions_pattern_not_empty CHECK (LENGTH(TRIM(route_pattern)) > 0),
    CONSTRAINT route_permissions_permission_not_empty CHECK (LENGTH(TRIM(required_permission)) > 0),
    CONSTRAINT route_permissions_priority_check CHECK (priority >= 0 AND priority <= 9999)
);

CREATE INDEX idx_route_permissions_lookup ON route_permissions(is_active, priority DESC, route_pattern, http_method)
    WHERE is_active = TRUE;
CREATE INDEX idx_route_permissions_method ON route_permissions(http_method, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_route_permissions_permission ON route_permissions(required_permission, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_route_permissions_category ON route_permissions(route_category, is_active, priority DESC);
CREATE INDEX idx_route_permissions_patterns ON route_permissions USING gin(route_pattern gin_trgm_ops) WHERE is_active = TRUE;
CREATE INDEX idx_route_permissions_audit ON route_permissions(created_at, updated_at);
CREATE UNIQUE INDEX idx_route_permissions_unique_route ON route_permissions(route_pattern, http_method) WHERE is_active = TRUE;

-- ================================================================================================
-- MATERIALIZED VIEWS
-- ================================================================================================

-- ------------------------------------------------------------------------------------------------
-- USER EFFECTIVE PERMISSIONS VIEW - Pre-computed permission lookups
-- ------------------------------------------------------------------------------------------------

CREATE MATERIALIZED VIEW IF NOT EXISTS user_effective_permissions AS
-- Permissions from groups
SELECT DISTINCT
    wga.wallet_address,
    p.id AS permission_id,
    p.permission_string,
    p.platform,
    p.resource,
    p.action,
    'group'::text AS source,
    pg.name AS source_name,
    wga.expires_at
FROM wallet_group_assignments wga
JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
JOIN permissions p ON pgm.permission_id = p.id
JOIN permission_groups pg ON wga.group_id = pg.id
WHERE wga.is_active = TRUE
  AND p.is_active = TRUE
  AND pg.is_active = TRUE
  AND (wga.expires_at IS NULL OR wga.expires_at > NOW())

UNION

-- Direct permissions
SELECT DISTINCT
    wdp.wallet_address,
    p.id AS permission_id,
    p.permission_string,
    p.platform,
    p.resource,
    p.action,
    'direct'::text AS source,
    'Direct Grant'::text AS source_name,
    wdp.expires_at
FROM wallet_direct_permissions wdp
JOIN permissions p ON wdp.permission_id = p.id
WHERE wdp.is_active = TRUE
  AND p.is_active = TRUE
  AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW());

-- Indexes on materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_eff_perms_unique
ON user_effective_permissions(wallet_address, permission_id);

CREATE INDEX IF NOT EXISTS idx_user_eff_perms_wallet
ON user_effective_permissions(wallet_address);

CREATE INDEX IF NOT EXISTS idx_user_eff_perms_permission
ON user_effective_permissions(permission_string);

CREATE INDEX IF NOT EXISTS idx_user_eff_perms_platform
ON user_effective_permissions(platform, resource, action);

COMMENT ON MATERIALIZED VIEW user_effective_permissions IS 'Pre-computed effective permissions for each wallet - refresh periodically';

-- ================================================================================================
-- FUNCTIONS
-- ================================================================================================

-- ------------------------------------------------------------------------------------------------
-- UPDATE TIMESTAMP TRIGGER FUNCTIONS
-- ------------------------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION trigger_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION trigger_update_wallet_users_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_route_permissions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ------------------------------------------------------------------------------------------------
-- REFRESH MATERIALIZED VIEW FUNCTION
-- ------------------------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION refresh_user_effective_permissions()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_effective_permissions;
    RAISE NOTICE 'Refreshed user_effective_permissions materialized view';
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION refresh_user_effective_permissions() IS 'Refresh the user_effective_permissions materialized view - call periodically or after permission changes';

-- ================================================================================================
-- TRIGGERS
-- ================================================================================================

-- Update timestamp triggers
DROP TRIGGER IF EXISTS trigger_wallet_users_updated_at ON wallet_users;
CREATE TRIGGER trigger_wallet_users_updated_at
    BEFORE UPDATE ON wallet_users
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_wallet_users_timestamp();

DROP TRIGGER IF EXISTS trigger_permission_groups_updated_at ON permission_groups;
CREATE TRIGGER trigger_permission_groups_updated_at
    BEFORE UPDATE ON permission_groups
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_timestamp();

DROP TRIGGER IF EXISTS route_permissions_updated_at_trigger ON route_permissions;
CREATE TRIGGER route_permissions_updated_at_trigger
    BEFORE UPDATE ON route_permissions
    FOR EACH ROW
    EXECUTE FUNCTION update_route_permissions_updated_at();

-- ================================================================================================
-- FOREIGN KEYS
-- ================================================================================================

-- Permission group memberships foreign keys
ALTER TABLE permission_group_memberships
    DROP CONSTRAINT IF EXISTS permission_group_memberships_group_id_fkey,
    ADD CONSTRAINT permission_group_memberships_group_id_fkey
    FOREIGN KEY (group_id) REFERENCES permission_groups(id) ON DELETE CASCADE;

ALTER TABLE permission_group_memberships
    DROP CONSTRAINT IF EXISTS permission_group_memberships_permission_id_fkey,
    ADD CONSTRAINT permission_group_memberships_permission_id_fkey
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE;

-- Wallet group assignments foreign keys
ALTER TABLE wallet_group_assignments
    DROP CONSTRAINT IF EXISTS wallet_group_assignments_wallet_address_fkey,
    ADD CONSTRAINT wallet_group_assignments_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

ALTER TABLE wallet_group_assignments
    DROP CONSTRAINT IF EXISTS wallet_group_assignments_group_id_fkey,
    ADD CONSTRAINT wallet_group_assignments_group_id_fkey
    FOREIGN KEY (group_id) REFERENCES permission_groups(id) ON DELETE CASCADE;

-- Wallet direct permissions foreign keys
ALTER TABLE wallet_direct_permissions
    DROP CONSTRAINT IF EXISTS wallet_direct_permissions_wallet_address_fkey,
    ADD CONSTRAINT wallet_direct_permissions_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

ALTER TABLE wallet_direct_permissions
    DROP CONSTRAINT IF EXISTS wallet_direct_permissions_permission_id_fkey,
    ADD CONSTRAINT wallet_direct_permissions_permission_id_fkey
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE;

-- OpenID refresh tokens foreign key
ALTER TABLE openid_refresh_tokens
    DROP CONSTRAINT IF EXISTS fk_openid_refresh_tokens_wallet_address,
    ADD CONSTRAINT fk_openid_refresh_tokens_wallet_address
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'EPSX INITIAL SCHEMA CREATED SUCCESSFULLY! 🎉';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Core Tables Created:';
    RAISE NOTICE '  ✅ wallet_users (Web3 user accounts)';
    RAISE NOTICE '  ✅ permissions (normalized permission definitions)';
    RAISE NOTICE '  ✅ permission_groups (permission group definitions)';
    RAISE NOTICE '  ✅ permission_group_memberships (group → permission mapping)';
    RAISE NOTICE '  ✅ wallet_group_assignments (wallet → group mapping)';
    RAISE NOTICE '  ✅ wallet_direct_permissions (direct permission grants)';
    RAISE NOTICE '  ✅ web3_auth_nonces (SIWE authentication)';
    RAISE NOTICE '  ✅ openid_refresh_tokens (token management)';
    RAISE NOTICE '  ✅ route_permissions (API route protection)';
    RAISE NOTICE '';
    RAISE NOTICE 'Additional Objects:';
    RAISE NOTICE '  ✅ user_effective_permissions (materialized view)';
    RAISE NOTICE '  ✅ Trigger functions (timestamp updates)';
    RAISE NOTICE '  ✅ Foreign key constraints';
    RAISE NOTICE '  ✅ Indexes for performance';
    RAISE NOTICE '';
    RAISE NOTICE '🎯 Database ready for EPSX Web3-First Platform!';
    RAISE NOTICE '=================================================================================';
END $$;
