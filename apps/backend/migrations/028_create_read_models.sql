-- ================================================================================================
-- READ MODEL SCHEMA - Denormalized Query-Optimized Views
-- ================================================================================================
-- This migration creates the read model schema for CQRS read side
--
-- Components:
-- 1. read_model schema: Separate namespace for read models
-- 2. wallet_details: Denormalized wallet view (no joins!)
-- 3. permission_summary: Flattened permission view
-- 4. analytics_rankings: Pre-computed analytics cache
-- 5. projection_checkpoints: Track projection progress
--
-- Version: 1.0.0
-- Created: 2025-10-06
-- Part of: CQRS Dual Database Architecture
-- ================================================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- ================================================================================================
-- 1. CREATE READ MODEL SCHEMA
-- ================================================================================================

CREATE SCHEMA IF NOT EXISTS read_model;

COMMENT ON SCHEMA read_model IS 'Read-optimized denormalized views for CQRS query side';

-- ================================================================================================
-- 2. WALLET READ MODEL - Denormalized Wallet Details
-- ================================================================================================
-- Single table with all wallet information (no joins required)
-- Updated by WalletReadModelProjection from domain events

CREATE TABLE IF NOT EXISTS read_model.wallet_details (
    -- Primary key
    wallet_address VARCHAR(42) PRIMARY KEY,

    -- Core wallet info
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL,
    last_auth_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Embedded permissions (denormalized - no joins!)
    active_permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
    permission_groups JSONB NOT NULL DEFAULT '[]'::jsonb,

    -- Pre-computed statistics (no aggregations needed!)
    total_permissions INT NOT NULL DEFAULT 0,
    active_permission_count INT NOT NULL DEFAULT 0,
    expired_permission_count INT NOT NULL DEFAULT 0,

    -- Subscription data (embedded)
    subscription_tier VARCHAR(50),
    subscription_status VARCHAR(20),
    subscription_expires_at TIMESTAMPTZ,
    subscription_plan_id VARCHAR(255),

    -- Session data (pre-counted)
    total_sessions INT NOT NULL DEFAULT 0,
    active_session_count INT NOT NULL DEFAULT 0,

    -- Analytics (materialized)
    last_activity_at TIMESTAMPTZ,
    total_logins INT NOT NULL DEFAULT 0,
    account_age_days INT, -- Computed by projection instead of generated column
    engagement_score DECIMAL(5,2) DEFAULT 0.0,

    -- Metadata
    wallet_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- Projection tracking
    projection_version BIGINT NOT NULL DEFAULT 0,
    last_event_id UUID,
    last_projected_at TIMESTAMPTZ DEFAULT NOW()
);

-- Query-optimized indexes
CREATE INDEX idx_wallet_details_active ON read_model.wallet_details(is_active, last_auth_at DESC);
CREATE INDEX idx_wallet_details_tier ON read_model.wallet_details(subscription_tier) WHERE subscription_tier IS NOT NULL;
CREATE INDEX idx_wallet_details_permissions_gin ON read_model.wallet_details USING GIN(active_permissions);
CREATE INDEX idx_wallet_details_groups_gin ON read_model.wallet_details USING GIN(permission_groups);
CREATE INDEX idx_wallet_details_activity ON read_model.wallet_details(last_activity_at DESC) WHERE is_active = true;
CREATE INDEX idx_wallet_details_engagement ON read_model.wallet_details(engagement_score DESC);
CREATE INDEX idx_wallet_details_created_at ON read_model.wallet_details(created_at DESC);

-- Comments
COMMENT ON TABLE read_model.wallet_details IS 'Denormalized wallet view for fast queries (updated by projections)';
COMMENT ON COLUMN read_model.wallet_details.active_permissions IS 'Array of active permissions (denormalized for speed)';
COMMENT ON COLUMN read_model.wallet_details.permission_groups IS 'Array of permission group memberships';
COMMENT ON COLUMN read_model.wallet_details.engagement_score IS 'User engagement score (0-100)';
COMMENT ON COLUMN read_model.wallet_details.projection_version IS 'Version of projection (for idempotency)';
COMMENT ON COLUMN read_model.wallet_details.last_event_id IS 'Last event ID processed by projection';

-- ================================================================================================
-- 3. PERMISSION SUMMARY - Flattened Permission View
-- ================================================================================================
-- Flattened view of all permissions with source tracking

CREATE TABLE IF NOT EXISTS read_model.permission_summary (
    id BIGSERIAL PRIMARY KEY,
    wallet_address VARCHAR(42) NOT NULL,
    permission_string VARCHAR(255) NOT NULL,

    -- Permission source
    source VARCHAR(50) NOT NULL, -- 'direct', 'group', 'nft', 'token', 'dao'
    source_id VARCHAR(255),
    source_name VARCHAR(255),

    -- Time tracking
    granted_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN, -- Computed by projection instead of generated column

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

-- Indexes for permission queries
CREATE INDEX idx_permission_wallet ON read_model.permission_summary(wallet_address, is_active);
CREATE INDEX idx_permission_active ON read_model.permission_summary(is_active, expires_at);
CREATE INDEX idx_permission_source ON read_model.permission_summary(source, source_id);
CREATE INDEX idx_permission_string ON read_model.permission_summary(permission_string);
CREATE INDEX idx_permission_expires_at ON read_model.permission_summary(expires_at) WHERE expires_at IS NOT NULL;

-- Comments
COMMENT ON TABLE read_model.permission_summary IS 'Flattened permission view with source tracking';
COMMENT ON COLUMN read_model.permission_summary.source IS 'Where permission came from (direct grant, group, NFT, token, DAO)';
COMMENT ON COLUMN read_model.permission_summary.is_active IS 'Computed: true if not expired';

-- ================================================================================================
-- 4. ANALYTICS READ MODEL - Pre-Aggregated Rankings
-- ================================================================================================
-- Pre-computed analytics for fast dashboard queries

CREATE TABLE IF NOT EXISTS read_model.analytics_rankings (
    symbol VARCHAR(20) PRIMARY KEY,
    company_name VARCHAR(255) NOT NULL,

    -- Pre-computed rankings (no calculation needed!)
    eps_growth_rank INT,
    revenue_rank INT,
    overall_score DECIMAL(5,2),
    grade VARCHAR(2), -- 'A+', 'A', 'B+', etc.

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

-- Indexes for analytics queries
CREATE INDEX idx_analytics_rank ON read_model.analytics_rankings(eps_growth_rank) WHERE eps_growth_rank IS NOT NULL;
CREATE INDEX idx_analytics_sector ON read_model.analytics_rankings(sector, overall_score DESC);
CREATE INDEX idx_analytics_score ON read_model.analytics_rankings(overall_score DESC);
CREATE INDEX idx_analytics_updated_at ON read_model.analytics_rankings(updated_at DESC);

-- Comments
COMMENT ON TABLE read_model.analytics_rankings IS 'Pre-computed stock rankings for fast queries';
COMMENT ON COLUMN read_model.analytics_rankings.overall_score IS 'Composite score 0-100';
COMMENT ON COLUMN read_model.analytics_rankings.calculated_at IS 'When these metrics were calculated';

-- ================================================================================================
-- 5. PROJECTION CHECKPOINTS - Track Projection Progress
-- ================================================================================================
-- Tracks which events have been processed by each projection

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

-- Indexes for checkpoint queries
CREATE INDEX idx_checkpoint_health ON read_model.projection_checkpoints(is_healthy, processed_at);
CREATE INDEX idx_checkpoint_processed_at ON read_model.projection_checkpoints(processed_at DESC);

-- Comments
COMMENT ON TABLE read_model.projection_checkpoints IS 'Tracks projection progress for resumability';
COMMENT ON COLUMN read_model.projection_checkpoints.last_processed_sequence IS 'Last outbox sequence number processed';
COMMENT ON COLUMN read_model.projection_checkpoints.is_healthy IS 'Whether projection is running without errors';

-- ================================================================================================
-- 6. MATERIALIZED VIEWS
-- ================================================================================================

-- Active wallets summary (refreshed periodically)
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

-- ================================================================================================
-- 7. FUNCTIONS FOR READ MODELS
-- ================================================================================================

-- Function to get wallet with all details (single query)
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

COMMENT ON FUNCTION read_model.get_wallet_full_details IS 'Get complete wallet details in single query';

-- ================================================================================================
-- 8. GRANTS
-- ================================================================================================

-- Grant read-only access to read model schema
-- GRANT USAGE ON SCHEMA read_model TO epsx_app_readonly;
-- GRANT SELECT ON ALL TABLES IN SCHEMA read_model TO epsx_app_readonly;

-- Grant read/write access to projection workers
-- GRANT USAGE ON SCHEMA read_model TO epsx_app;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA read_model TO epsx_app;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA read_model TO epsx_app;

-- ================================================================================================
-- MIGRATION COMPLETE
-- ================================================================================================

DO $$
DECLARE
    table_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO table_count
    FROM information_schema.tables
    WHERE table_schema = 'read_model'
    AND table_name IN ('wallet_details', 'permission_summary', 'analytics_rankings', 'projection_checkpoints');

    IF table_count = 4 THEN
        RAISE NOTICE 'Read model schema created successfully';
        RAISE NOTICE 'Tables created: wallet_details, permission_summary, analytics_rankings, projection_checkpoints';
        RAISE NOTICE 'Ready for projection workers to populate read models';
    ELSE
        RAISE EXCEPTION 'Failed to create all required read model tables';
    END IF;
END $$;
