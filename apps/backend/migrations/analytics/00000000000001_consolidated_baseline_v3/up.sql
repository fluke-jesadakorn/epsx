-- ============================================================================
-- EPSX Analytics Consolidated Schema v3 (renamed: schema infra_logs)
-- ============================================================================
-- Consolidates all analytics migrations into a single baseline.
--
-- Schema rename: the schema historically called `analytics` is actually
-- shared infrastructure (event store, outbox, audit logs, usage logs).
-- Per audit-analytics.md §5a and ROADMAP §4 wave-12 precondition #3, the
-- schema is renamed to `infra_logs` so future readers don't confuse it
-- with the analytics domain's own storage (analytics owns no PG tables).
--
-- Tables:
--   - api_key_usage_logs (partitioned by month)
--   - event_store
--   - outbox_events
--   - aggregate_snapshots
--   - permission_audit_log
--   - payment_audit_log
--   - assignment_audit_log
--   - wallet_activity_logs
--   - audit_logs
--   - analytics_events
--   - unified_audit_log
-- ============================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- Schema setup (idempotent: safe to re-run)
CREATE SCHEMA IF NOT EXISTS infra_logs;
SET search_path TO infra_logs;

-- Extensions (kept in public — uuid-ossp is a Postgres extension, not a table)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

-- ============================================================================
-- API KEY USAGE LOGS (Partitioned)
-- ============================================================================

CREATE TABLE infra_logs.api_key_usage_logs (
    id UUID DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL,
    module_id UUID,
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    response_status INTEGER,
    response_time_ms INTEGER,
    ip_address INET,
    user_agent TEXT,
    request_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, request_at)
) PARTITION BY RANGE (request_at);

-- Create monthly partitions for 2025-2026
CREATE TABLE infra_logs.api_key_usage_logs_2025_11 PARTITION OF infra_logs.api_key_usage_logs
    FOR VALUES FROM ('2025-11-01') TO ('2025-12-01');
CREATE TABLE infra_logs.api_key_usage_logs_2025_12 PARTITION OF infra_logs.api_key_usage_logs
    FOR VALUES FROM ('2025-12-01') TO ('2026-01-01');
CREATE TABLE infra_logs.api_key_usage_logs_2026_01 PARTITION OF infra_logs.api_key_usage_logs
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
CREATE TABLE infra_logs.api_key_usage_logs_2026_02 PARTITION OF infra_logs.api_key_usage_logs
    FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');
CREATE TABLE infra_logs.api_key_usage_logs_2026_03 PARTITION OF infra_logs.api_key_usage_logs
    FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

CREATE INDEX idx_usage_logs_api_key ON infra_logs.api_key_usage_logs(api_key_id);
CREATE INDEX idx_usage_logs_time ON infra_logs.api_key_usage_logs(request_at DESC);
CREATE INDEX idx_usage_logs_key_time ON infra_logs.api_key_usage_logs(api_key_id, request_at DESC);
CREATE INDEX idx_usage_logs_endpoint ON infra_logs.api_key_usage_logs(endpoint);

-- ============================================================================
-- EVENT STORE (Event Sourcing)
-- ============================================================================

CREATE TABLE infra_logs.event_store (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    causation_id UUID,
    correlation_id UUID,
    user_id VARCHAR(255),
    
    CONSTRAINT event_store_unique_version UNIQUE (aggregate_id, aggregate_version),
    CONSTRAINT event_store_version_positive CHECK (aggregate_version >= 0)
);

CREATE INDEX idx_event_store_aggregate ON infra_logs.event_store(aggregate_id, aggregate_version);
CREATE INDEX idx_event_store_type_time ON infra_logs.event_store(event_type, occurred_at DESC);
CREATE INDEX idx_event_store_correlation ON infra_logs.event_store(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_event_store_occurred_at ON infra_logs.event_store(occurred_at DESC);

-- ============================================================================
-- OUTBOX EVENTS (Transactional Outbox Pattern)
-- ============================================================================

CREATE TABLE infra_logs.outbox_events (
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES infra_logs.event_store(event_id) ON DELETE CASCADE,
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_payload JSONB NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT false,
    processed_at TIMESTAMPTZ,
    retry_count INT NOT NULL DEFAULT 0,
    last_error TEXT,
    next_retry_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sequence_number BIGSERIAL NOT NULL,
    
    CONSTRAINT outbox_retry_count_positive CHECK (retry_count >= 0),
    CONSTRAINT outbox_retry_count_limit CHECK (retry_count <= 10)
);

CREATE INDEX idx_outbox_unprocessed ON infra_logs.outbox_events(processed, sequence_number) WHERE processed = false;
CREATE INDEX idx_outbox_aggregate ON infra_logs.outbox_events(aggregate_id);
CREATE INDEX idx_outbox_retry ON infra_logs.outbox_events(next_retry_at) WHERE processed = false AND next_retry_at IS NOT NULL;

-- ============================================================================
-- AGGREGATE SNAPSHOTS
-- ============================================================================

CREATE TABLE infra_logs.aggregate_snapshots (
    aggregate_id VARCHAR(255) PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_count_at_snapshot INT NOT NULL DEFAULT 0,
    
    CONSTRAINT snapshot_version_positive CHECK (aggregate_version >= 0)
);

CREATE INDEX idx_snapshots_type_version ON infra_logs.aggregate_snapshots(aggregate_type, aggregate_version DESC);

-- ============================================================================
-- UNIFIED AUDIT LOG (Consolidated v3)
-- ============================================================================

CREATE TABLE infra_logs.unified_audit_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor           VARCHAR(42),
    actor_type      VARCHAR(20) NOT NULL DEFAULT 'admin',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resource_type   VARCHAR(50) NOT NULL,
    resource_id     VARCHAR(255),
    action          VARCHAR(50) NOT NULL,
    effect          VARCHAR(20) NOT NULL DEFAULT 'success',
    before_state    JSONB,
    after_state     JSONB,
    ip_address      VARCHAR(45),
    user_agent      TEXT,
    metadata        JSONB,
    category        VARCHAR(30) NOT NULL DEFAULT 'system'
);

CREATE INDEX idx_ual_created_at ON infra_logs.unified_audit_log (created_at DESC);
CREATE INDEX idx_ual_actor ON infra_logs.unified_audit_log (actor);
CREATE INDEX idx_ual_category ON infra_logs.unified_audit_log (category);
CREATE INDEX idx_ual_resource ON infra_logs.unified_audit_log (resource_type, resource_id);

-- ============================================================================
-- LEGACY AUDIT TABLES (Kept for compatibility)
-- ============================================================================

CREATE TABLE infra_logs.permission_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,
    event_timestamp TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    event_source VARCHAR(100) DEFAULT 'system' NOT NULL,
    wallet_address VARCHAR(42) NOT NULL,
    permission_string VARCHAR(255),
    permission_id UUID,
    group_id UUID,
    group_name VARCHAR(100),
    performed_by VARCHAR(42),
    performed_by_name VARCHAR(255),
    reason TEXT,
    request_id VARCHAR(36),
    ip_address INET,
    user_agent TEXT,
    previous_state JSONB,
    new_state JSONB,
    expires_at TIMESTAMPTZ,
    valid_from TIMESTAMPTZ,
    valid_until TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::JSONB
);

CREATE INDEX idx_perm_audit_wallet ON infra_logs.permission_audit_log(wallet_address, event_timestamp DESC);
CREATE INDEX idx_perm_audit_timestamp ON infra_logs.permission_audit_log(event_timestamp DESC);

CREATE TABLE infra_logs.payment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    old_status VARCHAR(20),
    new_status VARCHAR(20) NOT NULL,
    reason TEXT,
    performed_by VARCHAR(42),
    performed_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE TABLE infra_logs.assignment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    reason TEXT,
    performed_by VARCHAR(42) NOT NULL,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE infra_logs.wallet_activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    performed_by VARCHAR(42),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE infra_logs.audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42),
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    result VARCHAR(20) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- ANALYTICS EVENTS
-- ============================================================================

CREATE TABLE infra_logs.analytics_events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    wallet_address VARCHAR(42),
    resource_path VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    status_code INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_analytics_events_wallet ON infra_logs.analytics_events(wallet_address);
CREATE INDEX idx_analytics_events_created_at ON infra_logs.analytics_events(created_at);

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON SCHEMA infra_logs IS 'Shared infrastructure logs (event store, outbox, audit, usage). Not analytics-domain storage.';
COMMENT ON TABLE infra_logs.api_key_usage_logs IS 'API usage tracking (partitioned by month) - high write volume';
COMMENT ON TABLE infra_logs.event_store IS 'Immutable event log for event sourcing';
COMMENT ON TABLE infra_logs.outbox_events IS 'Transactional outbox for reliable event publishing';
COMMENT ON TABLE infra_logs.unified_audit_log IS 'Consolidated audit trail for all system actions';

SELECT 'EPSX INFRA_LOGS CONSOLIDATED SCHEMA v3 CREATED SUCCESSFULLY! 🎉' AS success_message;
