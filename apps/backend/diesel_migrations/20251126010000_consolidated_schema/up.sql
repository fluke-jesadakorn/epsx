-- ================================================================================================
-- EPSX CONSOLIDATED DEVELOPMENT SCHEMA - DIESEL VERSION
-- ================================================================================================
-- Version: 3.0.0 (Consolidated for Development)
-- Created: 2025-11-26
-- Compatible with: Diesel ORM CLI
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

-- Set session configuration
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
-- SECTION 2: CORE TABLES
-- ================================================================================================

-- ------------------------------------------------------------------------------------------------
-- WALLET USERS TABLE - Primary user accounts
-- ------------------------------------------------------------------------------------------------

CREATE TABLE wallet_users (
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

CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

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

    -- UNIFIED PERMISSION FIELDS
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

-- Enhanced permission indexes
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

-- ------------------------------------------------------------------------------------------------
-- PERMISSION GROUPS TABLE - Enhanced with pay-per-use billing
-- ------------------------------------------------------------------------------------------------

CREATE TABLE permission_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
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

CREATE TABLE permission_group_memberships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

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

CREATE TABLE wallet_group_assignments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

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

CREATE TABLE wallet_direct_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

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

CREATE TABLE web3_auth_nonces (
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

CREATE TABLE openid_refresh_tokens (
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

CREATE TABLE route_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
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
CREATE INDEX idx_route_permissions_patterns ON route_permissions(route_pattern) WHERE is_active = TRUE;
CREATE INDEX idx_route_permissions_audit ON route_permissions(created_at, updated_at);
CREATE UNIQUE INDEX idx_route_permissions_unique_route ON route_permissions(route_pattern, http_method) WHERE is_active = TRUE;

COMMENT ON TABLE route_permissions IS 'Enhanced API route protection with permission requirements and category support';

-- ================================================================================================
-- SECTION 3: SESSIONS TABLE
-- ================================================================================================

CREATE TABLE sessions (
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
    WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX idx_sessions_last_accessed ON sessions(last_accessed_at DESC)
    WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_ip_address ON sessions(ip_address, wallet_address)
    WHERE ip_address IS NOT NULL AND is_revoked = FALSE;

COMMENT ON TABLE sessions IS 'Active user sessions for Web3-authenticated wallets with token management and security tracking';

-- ================================================================================================
-- SECTION 4: STOCK RANKINGS
-- ================================================================================================

CREATE TABLE stock_ranking_assignments (
    assignment_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
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

CREATE INDEX idx_stock_ranking_wallet ON stock_ranking_assignments(wallet_address);
CREATE INDEX idx_stock_ranking_package ON stock_ranking_assignments(package_id);
CREATE INDEX idx_stock_ranking_active ON stock_ranking_assignments(is_active, expires_at);

COMMENT ON TABLE stock_ranking_assignments IS 'Tracks stock ranking package assignments to wallet users with expiration and access levels';
COMMENT ON COLUMN stock_ranking_assignments.rank_access_level IS 'Maximum rank position user can access (e.g., 1000 = top 1000 stocks)';
COMMENT ON COLUMN stock_ranking_assignments.assignment_source IS 'Source of assignment: "purchase", "promotion", "manual", "trial"';

CREATE TABLE assignment_audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    assignment_id UUID NOT NULL REFERENCES stock_ranking_assignments(assignment_id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    reason TEXT,
    performed_by VARCHAR(42) NOT NULL,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_assignment ON assignment_audit_log(assignment_id);
CREATE INDEX idx_audit_performed_at ON assignment_audit_log(performed_at);

COMMENT ON TABLE assignment_audit_log IS 'Audit trail for all assignment modifications (extend, revoke, etc.)';

-- ================================================================================================
-- SECTION 5: NOTIFICATIONS SYSTEM WITH PERFORMANCE INDEXES
-- ================================================================================================

CREATE TABLE wallet_notifications (
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

-- Enhanced notification performance indexes
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

-- ------------------------------------------------------------------------------------------------
-- NOTIFICATION SUBSCRIPTIONS
-- ------------------------------------------------------------------------------------------------

CREATE TABLE notification_subscriptions (
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
-- SECTION 6: PERMISSION AUDIT SYSTEM
-- ================================================================================================

CREATE TABLE permission_audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

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
CREATE INDEX idx_audit_log_timestamp_month ON permission_audit_log(event_timestamp);

COMMENT ON TABLE permission_audit_log IS 'Complete audit trail of all permission-related events';

-- ================================================================================================
-- SECTION 7: FOREIGN KEY CONSTRAINTS
-- ================================================================================================

-- Sessions foreign key
ALTER TABLE sessions
    ADD CONSTRAINT sessions_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

-- Permission group memberships foreign keys
ALTER TABLE permission_group_memberships
    ADD CONSTRAINT permission_group_memberships_group_id_fkey
    FOREIGN KEY (group_id) REFERENCES permission_groups(id) ON DELETE CASCADE;

ALTER TABLE permission_group_memberships
    ADD CONSTRAINT permission_group_memberships_permission_id_fkey
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE;

-- Wallet group assignments foreign keys
ALTER TABLE wallet_group_assignments
    ADD CONSTRAINT wallet_group_assignments_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

ALTER TABLE wallet_group_assignments
    ADD CONSTRAINT wallet_group_assignments_group_id_fkey
    FOREIGN KEY (group_id) REFERENCES permission_groups(id) ON DELETE CASCADE;

-- Wallet direct permissions foreign keys
ALTER TABLE wallet_direct_permissions
    ADD CONSTRAINT wallet_direct_permissions_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

ALTER TABLE wallet_direct_permissions
    ADD CONSTRAINT wallet_direct_permissions_permission_id_fkey
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE;

-- OpenID refresh tokens foreign key
ALTER TABLE openid_refresh_tokens
    ADD CONSTRAINT fk_openid_refresh_tokens_wallet_address
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

-- Stock ranking foreign keys (inline in table definition)

-- Additional validation constraints
ALTER TABLE permission_group_memberships
    ADD CONSTRAINT valid_granted_by_format CHECK (
        granted_by IS NULL OR
        (granted_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(granted_by) = 42)
);

ALTER TABLE wallet_group_assignments
    ADD CONSTRAINT valid_assigned_by_format CHECK (
        assigned_by IS NULL OR
        (assigned_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(assigned_by) = 42)
);

ALTER TABLE wallet_direct_permissions
    ADD CONSTRAINT valid_granted_by_format_wdp CHECK (
        granted_by IS NULL OR
        (granted_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(granted_by) = 42)
);

-- ================================================================================================
-- SECTION 8: PERFORMANCE INDEXES
-- ================================================================================================

-- Performance indexes for foreign keys
CREATE INDEX idx_pgm_group_fk ON permission_group_memberships(group_id);
CREATE INDEX idx_pgm_permission_fk ON permission_group_memberships(permission_id);
CREATE INDEX idx_wga_wallet_fk ON wallet_group_assignments(wallet_address);
CREATE INDEX idx_wga_group_fk ON wallet_group_assignments(group_id);
CREATE INDEX idx_wdp_wallet_fk ON wallet_direct_permissions(wallet_address);
CREATE INDEX idx_wdp_permission_fk ON wallet_direct_permissions(permission_id);

-- Enhanced performance indexes
CREATE INDEX idx_wga_active_lookup
ON wallet_group_assignments(wallet_address, is_active)
WHERE is_active = TRUE;

CREATE INDEX idx_wga_expires_lookup
ON wallet_group_assignments(wallet_address, expires_at)
WHERE is_active = TRUE AND expires_at IS NOT NULL;

CREATE INDEX idx_wdp_active_lookup
ON wallet_direct_permissions(wallet_address, is_active)
WHERE is_active = TRUE;

CREATE INDEX idx_wdp_expires_lookup
ON wallet_direct_permissions(wallet_address, expires_at)
WHERE is_active = TRUE AND expires_at IS NOT NULL;

CREATE INDEX idx_pg_active ON permission_groups(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_permissions_active ON permissions(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_permissions_string_pattern ON permissions(permission_string varchar_pattern_ops);

-- ================================================================================================
-- SECTION 9: ESSENTIAL SEED DATA - SUBSCRIPTION PLANS
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
    uuid_generate_v4(),
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
);

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
    uuid_generate_v4(),
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
);

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
    uuid_generate_v4(),
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
);

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
    uuid_generate_v4(),
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
);

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
    uuid_generate_v4(),
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
);

-- Default route permissions
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

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

SELECT 'EPSX CONSOLIDATED DEVELOPMENT SCHEMA CREATED SUCCESSFULLY FOR DIESEL! 🎉' AS success_message;