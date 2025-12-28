-- ============================================================================
-- EPSX Analytics Database Schema
-- ============================================================================
-- Separate database for high-volume analytics and logging tables
-- This reduces write contention on the main database
-- 
-- Tables:
--   - api_key_usage_logs (partitioned by month)
--   - event_store
--   - outbox_events
--   - permission_audit_log
--   - payment_audit_log
--   - assignment_audit_log
-- ============================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

-- ============================================================================
-- API KEY USAGE LOGS (Partitioned)
-- ============================================================================
-- High-volume table partitioned by month for efficient data lifecycle management

CREATE TABLE api_key_usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL,  -- Reference only, no FK (cross-database)
    module_id UUID,
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    response_status INTEGER,
    response_time_ms INTEGER,
    ip_address INET,
    user_agent TEXT,
    request_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
) PARTITION BY RANGE (request_at);

-- Create monthly partitions for 2025
CREATE TABLE api_key_usage_logs_2025_01 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
CREATE TABLE api_key_usage_logs_2025_02 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-02-01') TO ('2025-03-01');
CREATE TABLE api_key_usage_logs_2025_03 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-03-01') TO ('2025-04-01');
CREATE TABLE api_key_usage_logs_2025_04 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-04-01') TO ('2025-05-01');
CREATE TABLE api_key_usage_logs_2025_05 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-05-01') TO ('2025-06-01');
CREATE TABLE api_key_usage_logs_2025_06 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-06-01') TO ('2025-07-01');
CREATE TABLE api_key_usage_logs_2025_07 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-07-01') TO ('2025-08-01');
CREATE TABLE api_key_usage_logs_2025_08 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-08-01') TO ('2025-09-01');
CREATE TABLE api_key_usage_logs_2025_09 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-09-01') TO ('2025-10-01');
CREATE TABLE api_key_usage_logs_2025_10 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-10-01') TO ('2025-11-01');
CREATE TABLE api_key_usage_logs_2025_11 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-11-01') TO ('2025-12-01');
CREATE TABLE api_key_usage_logs_2025_12 PARTITION OF api_key_usage_logs
    FOR VALUES FROM ('2025-12-01') TO ('2026-01-01');

-- Indexes for usage logs
CREATE INDEX idx_usage_logs_api_key ON api_key_usage_logs(api_key_id);
CREATE INDEX idx_usage_logs_time ON api_key_usage_logs(request_at DESC);
CREATE INDEX idx_usage_logs_key_time ON api_key_usage_logs(api_key_id, request_at DESC);
CREATE INDEX idx_usage_logs_endpoint ON api_key_usage_logs(endpoint);

-- ============================================================================
-- EVENT STORE (Event Sourcing)
-- ============================================================================

CREATE TABLE event_store (
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

CREATE INDEX idx_event_store_aggregate ON event_store(aggregate_id, aggregate_version);
CREATE INDEX idx_event_store_type_time ON event_store(event_type, occurred_at DESC);
CREATE INDEX idx_event_store_correlation ON event_store(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_event_store_occurred_at ON event_store(occurred_at DESC);

-- ============================================================================
-- OUTBOX EVENTS (Transactional Outbox Pattern)
-- ============================================================================

CREATE TABLE outbox_events (
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES event_store(event_id) ON DELETE CASCADE,
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

CREATE INDEX idx_outbox_unprocessed ON outbox_events(processed, sequence_number) WHERE processed = false;
CREATE INDEX idx_outbox_aggregate ON outbox_events(aggregate_id);
CREATE INDEX idx_outbox_retry ON outbox_events(next_retry_at) WHERE processed = false AND next_retry_at IS NOT NULL;

-- ============================================================================
-- AGGREGATE SNAPSHOTS
-- ============================================================================

CREATE TABLE aggregate_snapshots (
    aggregate_id VARCHAR(255) PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_count_at_snapshot INT NOT NULL DEFAULT 0,
    
    CONSTRAINT snapshot_version_positive CHECK (aggregate_version >= 0)
);

CREATE INDEX idx_snapshots_type_version ON aggregate_snapshots(aggregate_type, aggregate_version DESC);

-- ============================================================================
-- PERMISSION AUDIT LOG
-- ============================================================================

CREATE TABLE permission_audit_log (
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

CREATE INDEX idx_perm_audit_wallet ON permission_audit_log(wallet_address, event_timestamp DESC);
CREATE INDEX idx_perm_audit_timestamp ON permission_audit_log(event_timestamp DESC);
CREATE INDEX idx_perm_audit_event_type ON permission_audit_log(event_type, event_timestamp DESC);

-- ============================================================================
-- PAYMENT AUDIT LOG
-- ============================================================================

CREATE TABLE payment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL,  -- Reference only, no FK (cross-database)
    action VARCHAR(50) NOT NULL,
    old_status VARCHAR(20),
    new_status VARCHAR(20) NOT NULL,
    reason TEXT,
    performed_by VARCHAR(42),
    performed_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_pay_audit_payment ON payment_audit_log(payment_id);
CREATE INDEX idx_pay_audit_action ON payment_audit_log(action);
CREATE INDEX idx_pay_audit_performed_at ON payment_audit_log(performed_at DESC);

-- ============================================================================
-- ASSIGNMENT AUDIT LOG
-- ============================================================================

CREATE TABLE assignment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL,  -- Reference only, no FK (cross-database)
    action VARCHAR(50) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    reason TEXT,
    performed_by VARCHAR(42) NOT NULL,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_assign_audit_assignment ON assignment_audit_log(assignment_id);
CREATE INDEX idx_assign_audit_performed_at ON assignment_audit_log(performed_at DESC);

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE api_key_usage_logs IS 'API usage tracking (partitioned by month) - high write volume';
COMMENT ON TABLE event_store IS 'Immutable event log for event sourcing';
COMMENT ON TABLE outbox_events IS 'Transactional outbox for reliable event publishing';
COMMENT ON TABLE permission_audit_log IS 'Audit trail for permission changes';
COMMENT ON TABLE payment_audit_log IS 'Audit trail for payment status changes';
COMMENT ON TABLE assignment_audit_log IS 'Audit trail for subscription/assignment modifications';
