-- ================================================================================================
-- EPSX CONSOLIDATED DEVELOPMENT SCHEMA
-- ================================================================================================
-- Version: 3.0.0 (Consolidated for Development)
-- Created: 2025-11-26
-- Consolidates: All individual migrations into single comprehensive schema
--
-- Description: Complete production-ready schema for EPSX Web3-first platform with all
-- November 2025 enhancements for simplified development setup.
--
-- Core Components:
-- - Web3-first wallet authentication system (SIWE compliant)
-- - Unified permission management with wallet-specific fields
-- - OpenID Connect token management
-- - Enhanced route-based permission protection
-- - Event sourcing infrastructure (CQRS)
-- - Read model schema for optimized queries
-- - Notification system with SSE support and performance indexes
-- - Stock ranking package assignments
-- - Permission audit logging
-- - Pay-per-use billing model
-- - Performance optimization functions
--
-- Database Requirements:
-- - PostgreSQL 14+
-- - Extensions: uuid-ossp, pg_trgm, btree_gist
-- ================================================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- ================================================================================================
-- SECTION 1: EXTENSIONS
-- ================================================================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;
COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;
COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';

CREATE EXTENSION IF NOT EXISTS btree_gist WITH SCHEMA public;
COMMENT ON EXTENSION btree_gist IS 'support for indexing common datatypes in GiST';

-- ================================================================================================
-- SECTION 2: CORE TABLES (from migration 001 + enhancements)
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
-- PERMISSIONS TABLE - Enhanced unified permission definitions
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

    -- UNIFIED PERMISSION FIELDS (from 20251118_005 and 20251126)
    wallet_address VARCHAR(42),
    source_type VARCHAR(20) CHECK (source_type IN ('direct', 'group', 'route')),
    source_id UUID,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    granted_by VARCHAR(255),
    grant_reason TEXT,

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
    CONSTRAINT permissions_action_not_empty CHECK (LENGTH(TRIM(action)) > 0),
    CONSTRAINT permissions_valid_wallet CHECK (wallet_address IS NULL OR wallet_address ~ '^0x[a-fA-F0-9]{40}$'),
    CONSTRAINT permissions_valid_dates CHECK (expires_at IS NULL OR expires_at > granted_at),
    CONSTRAINT permissions_active_expires CHECK (is_active = false OR expires_at IS NULL OR expires_at > NOW())
);

-- Enhanced permission indexes (from 20251118_005)
CREATE INDEX idx_permissions_lookup ON permissions(permission_string) WHERE is_active = TRUE;
CREATE INDEX idx_permissions_platform ON permissions(platform, resource, action);
CREATE INDEX idx_permissions_type ON permissions(permission_type, is_active);
CREATE INDEX idx_permissions_web3 ON permissions(permission_type, web3_chain_id, web3_contract_address)
    WHERE permission_type != 'manual';
CREATE INDEX idx_permissions_audit ON permissions(created_at, updated_at);

-- Strategic indexes for unified permissions
CREATE INDEX idx_permissions_wallet_lookup ON permissions (wallet_address, is_active, expires_at) WHERE wallet_address IS NOT NULL;
CREATE INDEX idx_permissions_platform_lookup ON permissions (platform, resource, action, is_active) WHERE is_active = true;
CREATE INDEX idx_permissions_source_lookup ON permissions (source_type, source_id, is_active) WHERE is_active = true AND source_type IS NOT NULL;
CREATE INDEX idx_permissions_expiry ON permissions (expires_at, is_active) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_permissions_active_time ON permissions (is_active, granted_at, expires_at) WHERE is_active = true;
CREATE INDEX idx_permissions_full_search ON permissions (permission_string, wallet_address, is_active) WHERE is_active = true;

COMMENT ON TABLE permissions IS 'Enhanced unified permissions table with Web3-first support and direct wallet assignments';
COMMENT ON COLUMN permissions.permission_string IS 'Full permission string in format platform:resource:action';
COMMENT ON COLUMN permissions.permission_type IS 'How permission is validated: manual, nft_gated, token_gated, or dao_governance';
COMMENT ON COLUMN permissions.wallet_address IS 'Wallet address that has this permission (for unified permissions table)';
COMMENT ON COLUMN permissions.source_type IS 'Source type: direct, group, or route (for unified permissions table)';
COMMENT ON COLUMN permissions.source_id IS 'Source ID for group-based permissions (for unified permissions table)';
COMMENT ON COLUMN permissions.granted_at IS 'When this permission was granted (for unified permissions table)';
COMMENT ON COLUMN permissions.expires_at IS 'When this permission expires (NULL for permanent)';
COMMENT ON COLUMN permissions.granted_by IS 'Who granted this permission (admin wallet address)';
COMMENT ON COLUMN permissions.grant_reason IS 'Reason for granting this permission';

-- ------------------------------------------------------------------------------------------------
-- PERMISSION GROUPS TABLE - Enhanced with pay-per-use billing
-- ------------------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS permission_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    description TEXT DEFAULT '' NOT NULL,
    group_type VARCHAR(20) DEFAULT 'manual' NOT NULL,
    group_metadata JSONB DEFAULT '{}' NOT NULL,

    -- Subscription pricing (Enhanced with pay-per-use)
    price NUMERIC(10,2) DEFAULT 0.00,
    currency VARCHAR(3) DEFAULT 'USD',
    billing_cycle VARCHAR(20) DEFAULT 'pay_per_use',

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

    -- Constraints (Updated for pay-per-use)
    CONSTRAINT valid_group_type CHECK (
        group_type IN ('manual', 'subscription', 'web3_asset', 'dao_membership', 'admin')
    ),
    CONSTRAINT valid_currency CHECK (
        currency IN ('USD', 'EUR', 'BTC', 'ETH', 'BNB')
    ),
    CONSTRAINT valid_billing_cycle CHECK (
        billing_cycle IN ('monthly', 'yearly', 'one_time', 'lifetime', 'pay_per_use')
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

COMMENT ON TABLE permission_groups IS 'Enhanced permission group definitions with pay-per-use billing support';

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

-- ------------------------------------------------------------------------------------------------
-- ENHANCED ROUTE PERMISSIONS TABLE - API route protection
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

COMMENT ON TABLE route_permissions IS 'Enhanced API route protection with permission requirements and category support';

-- ================================================================================================
-- SECTION 3: SESSIONS TABLE
-- ================================================================================================

CREATE TABLE IF NOT EXISTS sessions (
    -- Identity
    id UUID PRIMARY KEY,
    wallet_address VARCHAR(42) NOT NULL,

    -- Session tokens
    access_token TEXT NOT NULL,
    refresh_token TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    last_accessed_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,

    -- Security metadata
    ip_address VARCHAR(45),  -- IPv4 (15 chars) or IPv6 (45 chars)
    user_agent TEXT,
    is_revoked BOOLEAN DEFAULT FALSE NOT NULL,

    -- Aggregate versioning for optimistic concurrency
    version BIGINT DEFAULT 1 NOT NULL,

    -- Constraints
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    ),
    CONSTRAINT access_token_not_empty CHECK (LENGTH(TRIM(access_token)) > 0),
    CONSTRAINT expires_at_future CHECK (expires_at > created_at),
    CONSTRAINT version_positive CHECK (version > 0)
);

-- Indexes
CREATE INDEX idx_sessions_wallet_address ON sessions(wallet_address, is_revoked, expires_at)
    WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_access_token ON sessions(access_token)
    WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_refresh_token ON sessions(refresh_token)
    WHERE refresh_token IS NOT NULL AND is_revoked = FALSE;
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at)
    WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_active ON sessions(wallet_address, is_revoked, expires_at, last_accessed_at)
    WHERE is_revoked = FALSE AND expires_at > NOW();
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX idx_sessions_last_accessed ON sessions(last_accessed_at DESC)
    WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_ip_address ON sessions(ip_address, wallet_address)
    WHERE ip_address IS NOT NULL AND is_revoked = FALSE;

COMMENT ON TABLE sessions IS 'Active user sessions for Web3-authenticated wallets with token management and security tracking';

-- ================================================================================================
-- SECTION 4: STOCK RANKINGS
-- ================================================================================================

CREATE TABLE IF NOT EXISTS stock_ranking_assignments (
    assignment_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) NOT NULL,
    package_id VARCHAR(255) NOT NULL,
    package_name VARCHAR(255) NOT NULL,
    rank_access_level INTEGER NOT NULL DEFAULT 1000,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    assignment_source VARCHAR(50) NOT NULL,
    auto_renew BOOLEAN NOT NULL DEFAULT false,
    payment_reference VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_stock_ranking_wallet ON stock_ranking_assignments(wallet_address);
CREATE INDEX IF NOT EXISTS idx_stock_ranking_package ON stock_ranking_assignments(package_id);
CREATE INDEX IF NOT EXISTS idx_stock_ranking_active ON stock_ranking_assignments(is_active, expires_at);

COMMENT ON TABLE stock_ranking_assignments IS 'Tracks stock ranking package assignments to wallet users with expiration and access levels';
COMMENT ON COLUMN stock_ranking_assignments.rank_access_level IS 'Maximum rank position user can access (e.g., 1000 = top 1000 stocks)';
COMMENT ON COLUMN stock_ranking_assignments.assignment_source IS 'Source of assignment: "purchase", "promotion", "manual", "trial"';

CREATE TABLE IF NOT EXISTS assignment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES stock_ranking_assignments(assignment_id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    reason TEXT,
    performed_by VARCHAR(42) NOT NULL,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_assignment ON assignment_audit_log(assignment_id);
CREATE INDEX IF NOT EXISTS idx_audit_performed_at ON assignment_audit_log(performed_at);

COMMENT ON TABLE assignment_audit_log IS 'Audit trail for all assignment modifications (extend, revoke, etc.)';

-- ================================================================================================
-- SECTION 5: EVENT SOURCING INFRASTRUCTURE
-- ================================================================================================

CREATE TABLE IF NOT EXISTS event_store (
    -- Event identification
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,

    -- Event data
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- Timing
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Causation tracking
    causation_id UUID,
    correlation_id UUID,
    user_id VARCHAR(255),

    -- Constraints
    CONSTRAINT event_store_unique_version UNIQUE (aggregate_id, aggregate_version),
    CONSTRAINT event_store_version_positive CHECK (aggregate_version >= 0)
);

CREATE INDEX idx_event_store_aggregate ON event_store(aggregate_id, aggregate_version);
CREATE INDEX idx_event_store_type_time ON event_store(event_type, occurred_at DESC);
CREATE INDEX idx_event_store_correlation ON event_store(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_event_store_aggregate_type ON event_store(aggregate_type, occurred_at DESC);
CREATE INDEX idx_event_store_occurred_at ON event_store(occurred_at DESC);

COMMENT ON TABLE event_store IS 'Immutable event log for event sourcing and audit trail';

CREATE TABLE IF NOT EXISTS outbox_events (
    -- Outbox identification
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES event_store(event_id) ON DELETE CASCADE,

    -- Event routing data
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_payload JSONB NOT NULL,

    -- Publishing status
    processed BOOLEAN NOT NULL DEFAULT false,
    processed_at TIMESTAMPTZ,
    retry_count INT NOT NULL DEFAULT 0,
    last_error TEXT,
    next_retry_at TIMESTAMPTZ,

    -- Ordering and timing
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sequence_number BIGSERIAL NOT NULL,

    -- Constraints
    CONSTRAINT outbox_retry_count_positive CHECK (retry_count >= 0),
    CONSTRAINT outbox_retry_count_limit CHECK (retry_count <= 10)
);

CREATE INDEX idx_outbox_unprocessed ON outbox_events(processed, sequence_number) WHERE processed = false;
CREATE INDEX idx_outbox_aggregate ON outbox_events(aggregate_id);
CREATE INDEX idx_outbox_retry ON outbox_events(next_retry_at) WHERE processed = false AND next_retry_at IS NOT NULL;
CREATE INDEX idx_outbox_created_at ON outbox_events(created_at DESC);

COMMENT ON TABLE outbox_events IS 'Transactional outbox for reliable event publishing to Redis Streams';

CREATE TABLE IF NOT EXISTS aggregate_snapshots (
    -- Snapshot identification
    aggregate_id VARCHAR(255) PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,

    -- Snapshot data
    snapshot_data JSONB NOT NULL,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_count_at_snapshot INT NOT NULL DEFAULT 0,

    -- Constraints
    CONSTRAINT snapshot_version_positive CHECK (aggregate_version >= 0)
);

CREATE INDEX idx_snapshots_type_version ON aggregate_snapshots(aggregate_type, aggregate_version DESC);
CREATE INDEX idx_snapshots_created_at ON aggregate_snapshots(created_at DESC);

COMMENT ON TABLE aggregate_snapshots IS 'Aggregate snapshots for performance optimization';

-- ================================================================================================
-- SECTION 6: READ MODEL SCHEMA
-- ================================================================================================

CREATE SCHEMA IF NOT EXISTS read_model;
COMMENT ON SCHEMA read_model IS 'Read-optimized denormalized views for CQRS query side';

CREATE TABLE IF NOT EXISTS read_model.wallet_details (
    -- Primary key
    wallet_address VARCHAR(42) PRIMARY KEY,

    -- Core wallet info
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL,
    last_auth_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Embedded permissions (denormalized)
    active_permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
    permission_groups JSONB NOT NULL DEFAULT '[]'::jsonb,

    -- Pre-computed statistics
    total_permissions INT NOT NULL DEFAULT 0,
    active_permission_count INT NOT NULL DEFAULT 0,
    expired_permission_count INT NOT NULL DEFAULT 0,

    -- Subscription data
    subscription_tier VARCHAR(50),
    subscription_status VARCHAR(20),
    subscription_expires_at TIMESTAMPTZ,
    subscription_plan_id VARCHAR(255),

    -- Session data
    total_sessions INT NOT NULL DEFAULT 0,
    active_session_count INT NOT NULL DEFAULT 0,

    -- Analytics
    last_activity_at TIMESTAMPTZ,
    total_logins INT NOT NULL DEFAULT 0,
    account_age_days INT,
    engagement_score DECIMAL(5,2) DEFAULT 0.0,

    -- Metadata
    wallet_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- Projection tracking
    projection_version BIGINT NOT NULL DEFAULT 0,
    last_event_id UUID,
    last_projected_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_wallet_details_active ON read_model.wallet_details(is_active, last_auth_at DESC);
CREATE INDEX idx_wallet_details_tier ON read_model.wallet_details(subscription_tier) WHERE subscription_tier IS NOT NULL;
CREATE INDEX idx_wallet_details_permissions_gin ON read_model.wallet_details USING GIN(active_permissions);
CREATE INDEX idx_wallet_details_groups_gin ON read_model.wallet_details USING GIN(permission_groups);
CREATE INDEX idx_wallet_details_activity ON read_model.wallet_details(last_activity_at DESC) WHERE is_active = true;
CREATE INDEX idx_wallet_details_engagement ON read_model.wallet_details(engagement_score DESC);
CREATE INDEX idx_wallet_details_created_at ON read_model.wallet_details(created_at DESC);

COMMENT ON TABLE read_model.wallet_details IS 'Denormalized wallet view for fast queries (updated by projections)';

CREATE TABLE IF NOT EXISTS read_model.permission_summary (
    id BIGSERIAL PRIMARY KEY,
    wallet_address VARCHAR(42) NOT NULL,
    permission_string VARCHAR(255) NOT NULL,

    -- Permission source
    source VARCHAR(50) NOT NULL,
    source_id VARCHAR(255),
    source_name VARCHAR(255),

    -- Time tracking
    granted_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN,

    -- Audit
    granted_by VARCHAR(255),
    revoked_at TIMESTAMPTZ,
    revoked_by VARCHAR(255),

    -- Metadata
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Constraints
    CONSTRAINT permission_summary_source_check CHECK (
        source IN ('direct', 'group', 'nft', 'token', 'dao')
    ),
    CONSTRAINT permission_summary_unique UNIQUE(wallet_address, permission_string, source, COALESCE(source_id, 'null'))
);

CREATE INDEX idx_permission_wallet ON read_model.permission_summary(wallet_address, is_active);
CREATE INDEX idx_permission_active ON read_model.permission_summary(is_active, expires_at);
CREATE INDEX idx_permission_source ON read_model.permission_summary(source, source_id);
CREATE INDEX idx_permission_string ON read_model.permission_summary(permission_string);
CREATE INDEX idx_permission_expires_at ON read_model.permission_summary(expires_at) WHERE expires_at IS NOT NULL;

COMMENT ON TABLE read_model.permission_summary IS 'Flattened permission view with source tracking';

CREATE TABLE IF NOT EXISTS read_model.analytics_rankings (
    symbol VARCHAR(20) PRIMARY KEY,
    company_name VARCHAR(255) NOT NULL,

    -- Pre-computed rankings
    eps_growth_rank INT,
    revenue_rank INT,
    overall_score DECIMAL(5,2),
    grade VARCHAR(2),

    -- Cached metrics
    current_eps DECIMAL(18,4),
    eps_growth_1y DECIMAL(8,2),
    eps_growth_3y DECIMAL(8,2),
    revenue_growth DECIMAL(8,2),
    profit_margin DECIMAL(8,2),

    -- Classification
    sector VARCHAR(100),
    industry VARCHAR(100),
    market_cap BIGINT,

    -- Freshness tracking
    calculated_at TIMESTAMPTZ NOT NULL,
    data_as_of DATE NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT analytics_rankings_score_range CHECK (overall_score BETWEEN 0 AND 100)
);

CREATE INDEX idx_analytics_rank ON read_model.analytics_rankings(eps_growth_rank) WHERE eps_growth_rank IS NOT NULL;
CREATE INDEX idx_analytics_sector ON read_model.analytics_rankings(sector, overall_score DESC);
CREATE INDEX idx_analytics_score ON read_model.analytics_rankings(overall_score DESC);
CREATE INDEX idx_analytics_updated_at ON read_model.analytics_rankings(updated_at DESC);

COMMENT ON TABLE read_model.analytics_rankings IS 'Pre-computed stock rankings for fast queries';

CREATE TABLE IF NOT EXISTS read_model.projection_checkpoints (
    projection_name VARCHAR(100) PRIMARY KEY,

    -- Checkpoint data
    last_processed_event_id UUID,
    last_processed_sequence BIGINT NOT NULL DEFAULT 0,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Statistics
    events_processed_count BIGINT NOT NULL DEFAULT 0,
    events_failed_count BIGINT NOT NULL DEFAULT 0,
    avg_processing_time_ms DECIMAL(10,2),

    -- Health
    last_error TEXT,
    last_error_at TIMESTAMPTZ,
    is_healthy BOOLEAN NOT NULL DEFAULT true,

    -- Metadata
    projection_version VARCHAR(50),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_checkpoint_health ON read_model.projection_checkpoints(is_healthy, processed_at);
CREATE INDEX idx_checkpoint_processed_at ON read_model.projection_checkpoints(processed_at DESC);

COMMENT ON TABLE read_model.projection_checkpoints IS 'Tracks projection progress for resumability';

-- ================================================================================================
-- SECTION 7: NOTIFICATIONS SYSTEM WITH PERFORMANCE INDEXES
-- ================================================================================================

CREATE TABLE IF NOT EXISTS wallet_notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    data JSONB DEFAULT NULL,
    priority VARCHAR(20) NOT NULL DEFAULT 'normal',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    read_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    clicked_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    delivered_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    action_url VARCHAR(500) DEFAULT NULL,
    image_url VARCHAR(500) DEFAULT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Delivery tracking columns
    queued_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    delivery_attempts INTEGER DEFAULT 0,
    last_delivery_attempt_at TIMESTAMP WITH TIME ZONE,
    delivery_error TEXT,
    acknowledged_at TIMESTAMP WITH TIME ZONE,

    -- Soft delete column
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,

    -- Constraints
    CONSTRAINT valid_notification_type CHECK (notification_type IN ('system', 'security', 'permission', 'wallet_management', 'wallet', 'payment', 'general', 'admin', 'data', 'feature')),
    CONSTRAINT valid_priority CHECK (priority IN ('low', 'normal', 'high', 'critical', 'urgent')),
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address = 'all' OR
        (wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND length(wallet_address) = 42)
    )
);

-- Enhanced notification performance indexes (from 001_notification_indexes.sql)
CREATE INDEX idx_wallet_notifications_queue_fetch ON wallet_notifications (wallet_address, deleted_at, created_at, timestamp DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_user_query ON wallet_notifications (deleted_at, wallet_address, read_at, timestamp DESC);
CREATE INDEX idx_wallet_notifications_admin_query ON wallet_notifications (deleted_at, notification_type, priority, timestamp DESC);
CREATE INDEX idx_wallet_notifications_expiry ON wallet_notifications (expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_wallet_notifications_soft_deleted ON wallet_notifications (deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX idx_wallet_notifications_read_cleanup ON wallet_notifications (read_at, deleted_at, created_at) WHERE read_at IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_timestamp_stats ON wallet_notifications (timestamp, deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_type_stats ON wallet_notifications (notification_type, deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_priority_stats ON wallet_notifications (priority, deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_read_rate ON wallet_notifications (read_at, deleted_at) WHERE read_at IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_click_rate ON wallet_notifications (clicked_at, deleted_at) WHERE clicked_at IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_delivery_rate ON wallet_notifications (delivered_at, deleted_at) WHERE delivered_at IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_acknowledgement ON wallet_notifications (acknowledged_at, deleted_at) WHERE acknowledged_at IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_unread_count ON wallet_notifications (wallet_address, read_at, deleted_at) WHERE read_at IS NULL AND deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_wallet ON wallet_notifications(wallet_address);
CREATE INDEX idx_wallet_notifications_timestamp ON wallet_notifications(timestamp DESC);
CREATE INDEX idx_wallet_notifications_read_at ON wallet_notifications(read_at) WHERE read_at IS NULL;
CREATE INDEX idx_wallet_notifications_type ON wallet_notifications(notification_type);
CREATE INDEX idx_wallet_notifications_priority ON wallet_notifications(priority);
CREATE INDEX idx_wallet_notifications_expires ON wallet_notifications(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_wallet_notifications_wallet_unread ON wallet_notifications(wallet_address, read_at) WHERE read_at IS NULL;
CREATE INDEX idx_wallet_notifications_undelivered ON wallet_notifications(wallet_address, delivered_at) WHERE delivered_at IS NULL;
CREATE INDEX idx_wallet_notifications_queued ON wallet_notifications(wallet_address, queued_at, expires_at) WHERE delivered_at IS NULL;
CREATE INDEX idx_wallet_notifications_cleanup ON wallet_notifications(created_at);
CREATE INDEX idx_wallet_notifications_acknowledged ON wallet_notifications(acknowledged_at) WHERE acknowledged_at IS NOT NULL;
CREATE INDEX idx_wallet_notifications_active ON wallet_notifications(wallet_address, deleted_at, created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_deleted ON wallet_notifications(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX idx_wallet_notifications_offline_queue ON wallet_notifications(wallet_address, created_at DESC, deleted_at, expires_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_wallet_notifications_unread_active ON wallet_notifications(wallet_address, read_at, deleted_at) WHERE read_at IS NULL AND deleted_at IS NULL;

COMMENT ON TABLE wallet_notifications IS 'Enhanced notifications system with performance indexes and delivery tracking';

-- ================================================================================================
-- SECTION 8: NOTIFICATION SUBSCRIPTIONS
-- ================================================================================================

CREATE TABLE IF NOT EXISTS notification_subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    instance_id VARCHAR(100) NOT NULL,
    connection_id VARCHAR(100) NOT NULL,
    connected_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_ping_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    disconnected_at TIMESTAMP WITH TIME ZONE,
    user_agent TEXT,
    ip_address INET,
    redis_channel VARCHAR(200),

    -- Constraints
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address = 'all' OR
        (wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND length(wallet_address) = 42)
    ),
    CONSTRAINT unique_connection UNIQUE (instance_id, connection_id)
);

CREATE INDEX idx_subscriptions_wallet_active ON notification_subscriptions(wallet_address, connected_at) WHERE disconnected_at IS NULL;
CREATE INDEX idx_subscriptions_instance_active ON notification_subscriptions(instance_id, connected_at) WHERE disconnected_at IS NULL;
CREATE INDEX idx_subscriptions_disconnected ON notification_subscriptions(disconnected_at) WHERE disconnected_at IS NOT NULL;
CREATE INDEX idx_subscriptions_stale ON notification_subscriptions(last_ping_at, disconnected_at) WHERE disconnected_at IS NULL;

COMMENT ON TABLE notification_subscriptions IS 'Tracks active SSE connections across multiple backend instances for Redis pub/sub';

-- ================================================================================================
-- SECTION 9: PERMISSION AUDIT SYSTEM
-- ================================================================================================

CREATE TABLE IF NOT EXISTS permission_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Event identification
    event_type VARCHAR(50) NOT NULL,
    event_timestamp TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    event_source VARCHAR(100) DEFAULT 'system' NOT NULL,

    -- Subject (who was affected)
    wallet_address VARCHAR(42) NOT NULL,

    -- Permission details
    permission_string VARCHAR(255),
    permission_id UUID,
    group_id UUID,
    group_name VARCHAR(100),

    -- Actor (who performed the action)
    performed_by VARCHAR(42),
    performed_by_name VARCHAR(255),

    -- Context
    reason TEXT,
    request_id VARCHAR(36),
    ip_address INET,
    user_agent TEXT,

    -- Before/After state
    previous_state JSONB,
    new_state JSONB,

    -- Temporal information
    expires_at TIMESTAMPTZ,
    valid_from TIMESTAMPTZ,
    valid_until TIMESTAMPTZ,

    -- Additional metadata
    metadata JSONB DEFAULT '{}'::JSONB,

    -- Constraints
    CONSTRAINT valid_event_type CHECK (
        event_type IN (
            'granted', 'revoked', 'modified', 'expired',
            'group_assigned', 'group_removed', 'group_updated',
            'direct_permission_granted', 'direct_permission_revoked'
        )
    ),
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    ),
    CONSTRAINT valid_performed_by_format CHECK (
        performed_by IS NULL OR
        (performed_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(performed_by) = 42)
    )
);

CREATE INDEX idx_audit_log_wallet ON permission_audit_log(wallet_address, event_timestamp DESC);
CREATE INDEX idx_audit_log_timestamp ON permission_audit_log(event_timestamp DESC);
CREATE INDEX idx_audit_log_event_type ON permission_audit_log(event_type, event_timestamp DESC);
CREATE INDEX idx_audit_log_permission ON permission_audit_log(permission_string, event_timestamp DESC) WHERE permission_string IS NOT NULL;
CREATE INDEX idx_audit_log_group ON permission_audit_log(group_id, event_timestamp DESC) WHERE group_id IS NOT NULL;
CREATE INDEX idx_audit_log_performed_by ON permission_audit_log(performed_by, event_timestamp DESC) WHERE performed_by IS NOT NULL;
CREATE INDEX idx_audit_log_request_id ON permission_audit_log(request_id) WHERE request_id IS NOT NULL;
CREATE INDEX idx_audit_log_previous_state_gin ON permission_audit_log USING gin(previous_state);
CREATE INDEX idx_audit_log_new_state_gin ON permission_audit_log USING gin(new_state);
CREATE INDEX idx_audit_log_metadata_gin ON permission_audit_log USING gin(metadata);
CREATE INDEX idx_audit_log_timestamp_month ON permission_audit_log((date_trunc('month', event_timestamp)));

COMMENT ON TABLE permission_audit_log IS 'Complete audit trail of all permission-related events';

-- ================================================================================================
-- SECTION 10: ENHANCED MATERIALIZED VIEWS
-- ================================================================================================

-- User effective permissions view
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

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_eff_perms_unique ON user_effective_permissions(wallet_address, permission_id);
CREATE INDEX IF NOT EXISTS idx_user_eff_perms_wallet ON user_effective_permissions(wallet_address);
CREATE INDEX IF NOT EXISTS idx_user_eff_perms_permission ON user_effective_permissions(permission_string);
CREATE INDEX IF NOT EXISTS idx_user_eff_perms_platform ON user_effective_permissions(platform, resource, action);

COMMENT ON MATERIALIZED VIEW user_effective_permissions IS 'Pre-computed effective permissions for each wallet';

-- ENHANCED: Wallet permissions view for unified permissions (from 20251118_005)
DROP MATERIALIZED VIEW IF EXISTS wallet_permissions_view;
CREATE MATERIALIZED VIEW wallet_permissions_view AS
SELECT
    -- Wallet information
    wu.wallet_address,
    wu.is_active as wallet_active,
    wu.wallet_metadata,
    wu.created_at as wallet_created,
    wu.updated_at as wallet_updated,
    wu.last_auth_at,

    -- Permission aggregation (JSON)
    COALESCE(
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'id', p.id,
                'permission_string', p.permission_string,
                'platform', p.platform,
                'resource', p.resource,
                'action', p.action,
                'source_type', p.source_type,
                'source_id', p.source_id,
                'granted_at', COALESCE(p.granted_at, p.created_at),
                'expires_at', p.expires_at,
                'granted_by', p.granted_by,
                'grant_reason', p.grant_reason
            )
        ) FILTER (WHERE p.id IS NOT NULL AND p.wallet_address IS NOT NULL),
        '[]'::json
    ) as permissions,

    -- Permission counts for quick stats
    COUNT(p.id) FILTER (WHERE p.id IS NOT NULL AND p.wallet_address IS NOT NULL) as total_permissions,
    COUNT(p.id) FILTER (WHERE p.source_type = 'direct' AND p.wallet_address IS NOT NULL) as direct_permissions,
    COUNT(p.id) FILTER (WHERE p.source_type = 'group' AND p.wallet_address IS NOT NULL) as group_permissions,
    COUNT(p.id) FILTER (WHERE p.expires_at IS NOT NULL AND p.wallet_address IS NOT NULL) as temporary_permissions,

    -- Wallet activity metrics
    CASE
        WHEN wu.last_auth_at IS NOT NULL THEN 'active'
        WHEN wu.created_at > NOW() - INTERVAL '30 days' THEN 'new'
        ELSE 'inactive'
    END as activity_status,

    -- Creation timestamp (for view refresh)
    NOW() as view_refreshed_at

FROM wallet_users wu
LEFT JOIN permissions p ON wu.wallet_address = p.wallet_address
    AND p.is_active = true
    AND (p.expires_at IS NULL OR p.expires_at > NOW())
GROUP BY
    wu.wallet_address, wu.is_active, wu.wallet_metadata,
    wu.created_at, wu.updated_at, wu.last_auth_at;

-- Create indexes for the materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_wallet_permissions_view_address
ON wallet_permissions_view (wallet_address);

CREATE INDEX IF NOT EXISTS idx_wallet_permissions_view_activity
ON wallet_permissions_view (activity_status, wallet_created DESC);

CREATE INDEX IF NOT EXISTS idx_wallet_permissions_view_stats
ON wallet_permissions_view (total_permissions DESC, direct_permissions DESC);

-- Event store statistics
CREATE MATERIALIZED VIEW IF NOT EXISTS event_store_stats AS
SELECT
    aggregate_type,
    event_type,
    COUNT(*) as event_count,
    MIN(occurred_at) as first_event_at,
    MAX(occurred_at) as last_event_at,
    COUNT(DISTINCT aggregate_id) as unique_aggregates
FROM event_store
GROUP BY aggregate_type, event_type;

CREATE UNIQUE INDEX idx_event_store_stats_pk ON event_store_stats(aggregate_type, event_type);
COMMENT ON MATERIALIZED VIEW event_store_stats IS 'Statistics for monitoring event store health';

-- Active wallets summary
CREATE MATERIALIZED VIEW IF NOT EXISTS read_model.mv_active_wallets_summary AS
SELECT
    COUNT(*) as total_wallets,
    COUNT(*) FILTER (WHERE is_active) as active_wallets,
    COUNT(*) FILTER (WHERE last_auth_at > NOW() - INTERVAL '7 days') as active_last_7_days,
    COUNT(*) FILTER (WHERE last_auth_at > NOW() - INTERVAL '30 days') as active_last_30_days,
    COUNT(DISTINCT subscription_tier) as unique_tiers,
    AVG(engagement_score) as avg_engagement_score,
    AVG(total_permissions) as avg_permissions_per_wallet
FROM read_model.wallet_details;

CREATE UNIQUE INDEX idx_mv_active_wallets_summary ON read_model.mv_active_wallets_summary((1));
COMMENT ON MATERIALIZED VIEW read_model.mv_active_wallets_summary IS 'Summary statistics for active wallets dashboard';

-- Wallet permission counts
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_wallet_permission_counts AS
SELECT
    wu.wallet_address,
    wu.tier_level,
    0 AS direct_permission_count,
    0 AS group_count,
    0 AS total_permission_count,
    NOW() AS last_permission_change
FROM wallet_users wu
WHERE wu.is_active = TRUE;

CREATE UNIQUE INDEX idx_mv_wallet_perm_counts_wallet ON mv_wallet_permission_counts(wallet_address);
CREATE INDEX idx_mv_wallet_perm_counts_tier ON mv_wallet_permission_counts(tier_level);
COMMENT ON MATERIALIZED VIEW mv_wallet_permission_counts IS 'Materialized view for fast permission count queries';

-- ================================================================================================
-- SECTION 11: ENHANCED FUNCTIONS
-- ================================================================================================

-- Update timestamp trigger functions
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

CREATE OR REPLACE FUNCTION trigger_update_sessions_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Refresh materialized view functions
CREATE OR REPLACE FUNCTION refresh_user_effective_permissions()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_effective_permissions;
    RAISE NOTICE 'Refreshed user_effective_permissions materialized view';
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION refresh_event_store_stats()
RETURNS TRIGGER AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY event_store_stats;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION refresh_wallet_permission_counts()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_wallet_permission_counts;
END;
$$ LANGUAGE plpgsql;

-- ENHANCED: Wallet permissions view functions (from 20251118_005)
CREATE OR REPLACE FUNCTION refresh_wallet_permissions_view()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY wallet_permissions_view;
END;
$$ LANGUAGE plpgsql;

-- Function: Check if wallet has specific permission (unified)
CREATE OR REPLACE FUNCTION wallet_has_permission(
    p_wallet_address VARCHAR(42),
    p_permission_string VARCHAR(255)
) RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM permissions
        WHERE wallet_address = p_wallet_address
          AND permission_string = p_permission_string
          AND is_active = true
          AND (expires_at IS NULL OR expires_at > NOW())
    );
END;
$$ LANGUAGE plpgsql;

-- Function: Get all permissions for wallet (unified)
CREATE OR REPLACE FUNCTION get_wallet_permissions(
    p_wallet_address VARCHAR(42)
) RETURNS JSON AS $$
BEGIN
    RETURN COALESCE(
        (
            SELECT JSON_AGG(
                JSON_BUILD_OBJECT(
                    'permission_string', permission_string,
                    'platform', platform,
                    'resource', resource,
                    'action', action,
                    'source_type', source_type,
                    'granted_at', COALESCE(granted_at, created_at),
                    'expires_at', expires_at
                )
            )
            FROM permissions
            WHERE wallet_address = p_wallet_address
              AND is_active = true
              AND (expires_at IS NULL OR expires_at > NOW())
        ),
        '[]'::json
    );
END;
$$ LANGUAGE plpgsql;

-- Function: Count active permissions by platform (unified)
CREATE OR REPLACE FUNCTION get_permission_stats_by_platform()
RETURNS TABLE(platform VARCHAR(50), permission_count BIGINT, wallet_count BIGINT) AS $$
BEGIN
    RETURN QUERY
    SELECT
        p.platform,
        COUNT(*)::BIGINT as permission_count,
        COUNT(DISTINCT p.wallet_address)::BIGINT as wallet_count
    FROM permissions p
    WHERE p.is_active = true
      AND p.wallet_address IS NOT NULL
      AND (p.expires_at IS NULL OR p.expires_at > NOW())
    GROUP BY p.platform
    ORDER BY permission_count DESC;
END;
$$ LANGUAGE plpgsql;

-- Read model helper function
CREATE OR REPLACE FUNCTION read_model.get_wallet_full_details(p_wallet_address VARCHAR(42))
RETURNS JSONB AS $$
DECLARE
    result JSONB;
BEGIN
    SELECT jsonb_build_object(
        'wallet_address', wallet_address,
        'is_active', is_active,
        'created_at', created_at,
        'last_auth_at', last_auth_at,
        'permissions', active_permissions,
        'groups', permission_groups,
        'stats', jsonb_build_object(
            'total_permissions', total_permissions,
            'active_permissions', active_permission_count,
            'total_sessions', total_sessions,
            'active_sessions', active_session_count,
            'total_logins', total_logins,
            'account_age_days', account_age_days,
            'engagement_score', engagement_score
        ),
        'subscription', jsonb_build_object(
            'tier', subscription_tier,
            'status', subscription_status,
            'expires_at', subscription_expires_at
        ),
        'metadata', wallet_metadata
    ) INTO result
    FROM read_model.wallet_details
    WHERE wallet_details.wallet_address = p_wallet_address;

    RETURN result;
END;
$$ LANGUAGE plpgsql STABLE;

-- Permission optimization functions
CREATE OR REPLACE FUNCTION get_wallet_permissions_detailed(p_wallet_address VARCHAR)
RETURNS TABLE (
    permission_string VARCHAR,
    permission_id UUID,
    source_type VARCHAR,
    source_id UUID,
    source_name VARCHAR,
    expires_at TIMESTAMPTZ,
    granted_at TIMESTAMPTZ,
    is_permanent BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    -- Group-based permissions
    SELECT
        p.permission_string,
        p.id AS permission_id,
        'group'::VARCHAR AS source_type,
        pg.id AS source_id,
        pg.name AS source_name,
        wga.expires_at,
        wga.assigned_at AS granted_at,
        (wga.expires_at IS NULL) AS is_permanent
    FROM wallet_group_assignments wga
    JOIN permission_groups pg ON wga.group_id = pg.id
    JOIN permission_group_memberships pgm ON pg.id = pgm.group_id
    JOIN permissions p ON pgm.permission_id = p.id
    WHERE wga.wallet_address = LOWER(p_wallet_address)
      AND wga.is_active = TRUE
      AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
      AND pg.is_active = TRUE
      AND p.is_active = TRUE

    UNION ALL

    -- Direct permissions
    SELECT
        p.permission_string,
        p.id AS permission_id,
        'direct'::VARCHAR AS source_type,
        wdp.id AS source_id,
        'Direct Grant'::VARCHAR AS source_name,
        wdp.expires_at,
        wdp.granted_at,
        (wdp.expires_at IS NULL) AS is_permanent
    FROM wallet_direct_permissions wdp
    JOIN permissions p ON wdp.permission_id = p.id
    WHERE wdp.wallet_address = LOWER(p_wallet_address)
      AND wdp.is_active = TRUE
      AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
      AND p.is_active = TRUE

    ORDER BY permission_string;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE OR REPLACE FUNCTION get_wallet_effective_permissions(p_wallet_address VARCHAR)
RETURNS JSONB AS $$
DECLARE
    v_permissions JSONB;
BEGIN
    SELECT COALESCE(jsonb_agg(DISTINCT permission_string ORDER BY permission_string), '[]'::JSONB)
    INTO v_permissions
    FROM (
        -- Group permissions
        SELECT p.permission_string
        FROM wallet_group_assignments wga
        JOIN permission_groups pg ON wga.group_id = pg.id
        JOIN permission_group_memberships pgm ON pg.id = pgm.group_id
        JOIN permissions p ON pgm.permission_id = p.id
        WHERE wga.wallet_address = LOWER(p_wallet_address)
          AND wga.is_active = TRUE
          AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
          AND pg.is_active = TRUE
          AND p.is_active = TRUE

        UNION

        -- Direct permissions
        SELECT p.permission_string
        FROM wallet_direct_permissions wdp
        JOIN permissions p ON wdp.permission_id = p.id
        WHERE wdp.wallet_address = LOWER(p_wallet_address)
          AND wdp.is_active = TRUE
          AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
          AND p.is_active = TRUE
    ) AS all_permissions;

    RETURN v_permissions;
END;
$$ LANGUAGE plpgsql STABLE;

-- Audit logging functions
CREATE OR REPLACE FUNCTION log_permission_granted(
    p_wallet_address VARCHAR,
    p_permission_string VARCHAR,
    p_permission_id UUID,
    p_granted_by VARCHAR,
    p_reason TEXT DEFAULT NULL,
    p_expires_at TIMESTAMPTZ DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
    v_audit_id UUID;
BEGIN
    INSERT INTO permission_audit_log (
        event_type,
        wallet_address,
        permission_string,
        permission_id,
        performed_by,
        reason,
        expires_at,
        valid_from,
        new_state,
        metadata
    ) VALUES (
        'direct_permission_granted',
        p_wallet_address,
        p_permission_string,
        p_permission_id,
        p_granted_by,
        p_reason,
        p_expires_at,
        NOW(),
        jsonb_build_object(
            'permission', p_permission_string,
            'expires_at', p_expires_at
        ),
        p_metadata
    )
    RETURNING id INTO v_audit_id;

    RETURN v_audit_id;
END;
$$ LANGUAGE plpgsql;

-- ================================================================================================
-- SECTION 12: TRIGGERS
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

DROP TRIGGER IF EXISTS trigger_sessions_updated_at ON sessions;
CREATE TRIGGER trigger_sessions_updated_at
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_sessions_timestamp();

DROP TRIGGER IF EXISTS update_wallet_notifications_updated_at ON wallet_notifications;
CREATE TRIGGER update_wallet_notifications_updated_at
    BEFORE UPDATE ON wallet_notifications
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ENHANCED: Permissions table update trigger (from 20251118_005)
DROP TRIGGER IF EXISTS update_permissions_updated_at ON permissions;
CREATE TRIGGER update_permissions_updated_at
    BEFORE UPDATE ON permissions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ================================================================================================
-- SECTION 13: FOREIGN KEY CONSTRAINTS
-- ================================================================================================

-- Sessions foreign key
ALTER TABLE sessions
    DROP CONSTRAINT IF EXISTS sessions_wallet_address_fkey,
    ADD CONSTRAINT sessions_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

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

-- Additional validation constraints
ALTER TABLE permission_group_memberships
DROP CONSTRAINT IF EXISTS valid_granted_by_format,
ADD CONSTRAINT valid_granted_by_format CHECK (
    granted_by IS NULL OR
    (granted_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(granted_by) = 42)
);

ALTER TABLE wallet_group_assignments
DROP CONSTRAINT IF EXISTS valid_assigned_by_format,
ADD CONSTRAINT valid_assigned_by_format CHECK (
    assigned_by IS NULL OR
    (assigned_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(assigned_by) = 42)
);

ALTER TABLE wallet_direct_permissions
DROP CONSTRAINT IF EXISTS valid_granted_by_format_wdp,
ADD CONSTRAINT valid_granted_by_format_wdp CHECK (
    granted_by IS NULL OR
    (granted_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(granted_by) = 42)
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_pgm_group_fk ON permission_group_memberships(group_id);
CREATE INDEX IF NOT EXISTS idx_pgm_permission_fk ON permission_group_memberships(permission_id);
CREATE INDEX IF NOT EXISTS idx_wga_wallet_fk ON wallet_group_assignments(wallet_address);
CREATE INDEX IF NOT EXISTS idx_wga_group_fk ON wallet_group_assignments(group_id);
CREATE INDEX IF NOT EXISTS idx_wdp_wallet_fk ON wallet_direct_permissions(wallet_address);
CREATE INDEX IF NOT EXISTS idx_wdp_permission_fk ON wallet_direct_permissions(permission_id);

-- Enhanced performance indexes
CREATE INDEX IF NOT EXISTS idx_wga_active_lookup
ON wallet_group_assignments(wallet_address, is_active)
WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_wga_expires_lookup
ON wallet_group_assignments(wallet_address, expires_at)
WHERE is_active = TRUE AND expires_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_wdp_active_lookup
ON wallet_direct_permissions(wallet_address, is_active)
WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_wdp_expires_lookup
ON wallet_direct_permissions(wallet_address, expires_at)
WHERE is_active = TRUE AND expires_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_pg_active ON permission_groups(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_permissions_active ON permissions(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_permissions_string_pattern ON permissions(permission_string varchar_pattern_ops);

-- ================================================================================================
-- SECTION 14: VIEWS
-- ================================================================================================

CREATE OR REPLACE VIEW v_recent_permission_changes AS
SELECT
    pal.id,
    pal.event_type,
    pal.event_timestamp,
    pal.wallet_address,
    wu.tier_level,
    pal.permission_string,
    pal.group_name,
    pal.performed_by,
    pal.reason,
    pal.expires_at
FROM permission_audit_log pal
LEFT JOIN wallet_users wu ON pal.wallet_address = wu.wallet_address
WHERE pal.event_timestamp > NOW() - INTERVAL '7 days'
ORDER BY pal.event_timestamp DESC;

CREATE OR REPLACE VIEW v_wallet_permission_history AS
SELECT
    pal.wallet_address,
    COUNT(*) FILTER (WHERE event_type IN ('granted', 'direct_permission_granted', 'group_assigned')) as total_grants,
    COUNT(*) FILTER (WHERE event_type IN ('revoked', 'direct_permission_revoked', 'group_removed')) as total_revocations,
    MAX(pal.event_timestamp) FILTER (WHERE event_type IN ('granted', 'direct_permission_granted', 'group_assigned')) as last_grant_at,
    MAX(pal.event_timestamp) FILTER (WHERE event_type IN ('revoked', 'direct_permission_revoked', 'group_removed')) as last_revoke_at
FROM permission_audit_log pal
GROUP BY pal.wallet_address;

COMMENT ON VIEW v_recent_permission_changes IS 'Recent permission changes (last 7 days) for monitoring';
COMMENT ON VIEW v_wallet_permission_history IS 'Permission change statistics per wallet';

-- ================================================================================================
-- SECTION 15: ESSENTIAL SEED DATA - SUBSCRIPTION PLANS
-- ================================================================================================

-- Free Plan
INSERT INTO permission_groups (
    id,
    name,
    slug,
    description,
    group_type,
    group_metadata,
    price,
    currency,
    billing_cycle,
    is_active,
    is_promoted,
    display_order,
    created_by
) VALUES (
    gen_random_uuid(),
    'Free Plan',
    'free',
    'Perfect for getting started with basic analytics',
    'subscription',
    '{
        "permissions": ["epsx:analytics:view:5", "epsx:rankings:view:5"],
        "features": [
            "Basic analytics view",
            "5 stock rankings limit",
            "Community support",
            "Daily market updates"
        ],
        "limits": {
            "analytics_queries_per_day": 10,
            "stocks_tracked": 5,
            "historical_data_months": 1
        }
    }'::jsonb,
    0.00,
    'USD',
    'monthly',
    true,
    false,
    1,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;

-- Starter Plan
INSERT INTO permission_groups (
    id,
    name,
    slug,
    description,
    group_type,
    group_metadata,
    price,
    currency,
    billing_cycle,
    is_active,
    is_promoted,
    display_order,
    created_by
) VALUES (
    gen_random_uuid(),
    'Starter Plan',
    'starter',
    'Ideal for individual investors and traders',
    'subscription',
    '{
        "permissions": ["epsx:analytics:view:25", "epsx:rankings:view:25", "epsx:analytics:export", "epsx:alerts:create"],
        "features": [
            "Advanced analytics",
            "25 stock rankings",
            "Export functionality",
            "Price alerts",
            "Email support"
        ],
        "limits": {
            "analytics_queries_per_day": 50,
            "stocks_tracked": 25,
            "historical_data_months": 6,
            "alerts": 10
        }
    }'::jsonb,
    14.99,
    'USD',
    'monthly',
    true,
    false,
    2,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;

-- Pro Plan
INSERT INTO permission_groups (
    id,
    name,
    slug,
    description,
    group_type,
    group_metadata,
    price,
    currency,
    billing_cycle,
    is_active,
    is_promoted,
    display_order,
    created_by
) VALUES (
    gen_random_uuid(),
    'Pro Plan',
    'pro',
    'For serious traders who need advanced tools',
    'subscription',
    '{
        "permissions": ["epsx:analytics:view:100", "epsx:rankings:view:100", "epsx:analytics:export", "epsx:analytics:advanced", "epsx:alerts:create", "epsx:alerts:manage", "epsx:portfolio:view", "epsx:portfolio:manage"],
        "features": [
            "Advanced analytics",
            "100 stock rankings",
            "Export functionality",
            "Advanced charting tools",
            "Portfolio management",
            "Unlimited price alerts",
            "Priority support",
            "Real-time data"
        ],
        "limits": {
            "analytics_queries_per_day": 200,
            "stocks_tracked": 100,
            "historical_data_months": 24,
            "alerts": -1,
            "portfolios": 5
        },
        "highlighted": true
    }'::jsonb,
    29.99,
    'USD',
    'monthly',
    true,
    true,
    3,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;

-- Enterprise Plan
INSERT INTO permission_groups (
    id,
    name,
    slug,
    description,
    group_type,
    group_metadata,
    price,
    currency,
    billing_cycle,
    is_active,
    is_promoted,
    display_order,
    created_by
) VALUES (
    gen_random_uuid(),
    'Enterprise Plan',
    'enterprise',
    'Complete solution for professional teams and institutions',
    'subscription',
    '{
        "permissions": ["epsx:*:*", "epsx:api:access", "epsx:enterprise:*"],
        "features": [
            "Unlimited stock analysis",
            "Unlimited rankings access",
            "Full API access",
            "Premium analytics suite",
            "Advanced portfolio tools",
            "Custom integrations",
            "Dedicated account manager",
            "24/7 priority support",
            "White-label options"
        ],
        "limits": {
            "analytics_queries_per_day": -1,
            "stocks_tracked": -1,
            "historical_data_months": -1,
            "alerts": -1,
            "portfolios": -1,
            "api_calls_per_month": 1000000
        },
        "highlighted": false,
        "contact_sales": true
    }'::jsonb,
    99.99,
    'USD',
    'monthly',
    true,
    false,
    4,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;

-- API Developer Plan
INSERT INTO permission_groups (
    id,
    name,
    slug,
    description,
    group_type,
    group_metadata,
    price,
    currency,
    billing_cycle,
    is_active,
    is_promoted,
    display_order,
    created_by
) VALUES (
    gen_random_uuid(),
    'API Developer',
    'api-developer',
    'For developers building on EPSX platform',
    'subscription',
    '{
        "permissions": ["epsx:api:access", "epsx:analytics:view:unlimited", "epsx:rankings:view:unlimited"],
        "features": [
            "Full REST API access",
            "Unlimited analytics queries",
            "Unlimited rankings access",
            "WebSocket support",
            "API documentation",
            "Developer support",
            "100k API calls/month"
        ],
        "limits": {
            "api_calls_per_month": 100000,
            "websocket_connections": 5,
            "rate_limit_per_second": 10
        },
        "plan_type": "api"
    }'::jsonb,
    49.99,
    'USD',
    'monthly',
    true,
    false,
    5,
    '0x0000000000000000000000000000000000000000'
) ON CONFLICT (slug) DO NOTHING;

-- ENHANCED: Default route permissions from 20251118_003
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
    ('/api/admin/permissions/routes', 'GET', 'admin:permissions:routes', 30, false)
ON CONFLICT (route_pattern, http_method) DO NOTHING;

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'EPSX CONSOLIDATED DEVELOPMENT SCHEMA CREATED SUCCESSFULLY! 🎉';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Core Tables Created:';
    RAISE NOTICE '  ✅ wallet_users (Web3 user accounts)';
    RAISE NOTICE '  ✅ permissions (enhanced unified permissions)';
    RAISE NOTICE '  ✅ permission_groups (with pay-per-use billing)';
    RAISE NOTICE '  ✅ permission_group_memberships (group → permission mapping)';
    RAISE NOTICE '  ✅ wallet_group_assignments (wallet → group mapping)';
    RAISE NOTICE '  ✅ wallet_direct_permissions (direct permission grants)';
    RAISE NOTICE '  ✅ web3_auth_nonces (SIWE authentication)';
    RAISE NOTICE '  ✅ openid_refresh_tokens (token management)';
    RAISE NOTICE '  ✅ route_permissions (enhanced API route protection)';
    RAISE NOTICE '  ✅ sessions (Web3 session management)';
    RAISE NOTICE '';
    RAISE NOTICE 'Enhanced Features:';
    RAISE NOTICE '  ✅ Unified permission fields (wallet_address, source_type, etc.)';
    RAISE NOTICE '  ✅ Pay-per-use billing support';
    RAISE NOTICE '  ✅ Enhanced route permissions with categories';
    RAISE NOTICE '  ✅ Optimized notification performance indexes';
    RAISE NOTICE '  ✅ Materialized view for wallet permissions';
    RAISE NOTICE '';
    RAISE NOTICE 'Additional Tables:';
    RAISE NOTICE '  ✅ stock_ranking_assignments, assignment_audit_log (stock rankings)';
    RAISE NOTICE '  ✅ event_store, outbox_events, aggregate_snapshots (event sourcing)';
    RAISE NOTICE '  ✅ read_model.* tables (CQRS read models)';
    RAISE NOTICE '  ✅ wallet_notifications (enhanced notification system)';
    RAISE NOTICE '  ✅ notification_subscriptions (SSE tracking)';
    RAISE NOTICE '  ✅ permission_audit_log (audit trail)';
    RAISE NOTICE '';
    RAISE NOTICE 'Essential Seed Data:';
    RAISE NOTICE '  ✅ 5 subscription plans (Free, Starter, Pro, Enterprise, API Developer)';
    RAISE NOTICE '  ✅ Default API route permissions';
    RAISE NOTICE '';
    RAISE NOTICE '🚀 Development database ready for EPSX Web3-First Platform!';
    RAISE NOTICE 'Consolidated from 15+ individual migrations';
    RAISE NOTICE 'Single migration for ultra-fast development setup';
    RAISE NOTICE '=================================================================================';
END $$;

-- Final refresh of materialized views
SELECT refresh_wallet_permissions_view();
SELECT refresh_user_effective_permissions();