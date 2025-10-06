-- ================================================================================================
-- EVENT STORE - Immutable Event Log for Event Sourcing
-- ================================================================================================
-- This migration creates the event sourcing infrastructure for CQRS pattern
--
-- Components:
-- 1. event_store: Immutable append-only event log
-- 2. outbox_events: Transactional outbox pattern for reliable event publishing
-- 3. aggregate_snapshots: Performance optimization for event sourcing
-- 4. Indexes for high-performance event queries
--
-- Version: 1.0.0
-- Created: 2025-10-06
-- Part of: Application-Level Event Publishing Architecture
-- ================================================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- ================================================================================================
-- 1. EVENT STORE TABLE
-- ================================================================================================
-- Immutable event log for event sourcing
-- All domain events are persisted here for audit trail and aggregate reconstruction

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

    -- Causation tracking (for debugging and tracing)
    causation_id UUID,           -- ID of command that caused this event
    correlation_id UUID,          -- Request trace ID for distributed tracing
    user_id VARCHAR(255),         -- Who triggered this event (wallet address)

    -- Constraints
    CONSTRAINT event_store_unique_version UNIQUE (aggregate_id, aggregate_version),
    CONSTRAINT event_store_version_positive CHECK (aggregate_version >= 0)
);

-- Indexes for event sourcing queries
CREATE INDEX idx_event_store_aggregate ON event_store(aggregate_id, aggregate_version);
CREATE INDEX idx_event_store_type_time ON event_store(event_type, occurred_at DESC);
CREATE INDEX idx_event_store_correlation ON event_store(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_event_store_aggregate_type ON event_store(aggregate_type, occurred_at DESC);
CREATE INDEX idx_event_store_occurred_at ON event_store(occurred_at DESC);

-- Comments
COMMENT ON TABLE event_store IS 'Immutable event log for event sourcing and audit trail';
COMMENT ON COLUMN event_store.event_id IS 'Unique identifier for this event instance';
COMMENT ON COLUMN event_store.aggregate_id IS 'ID of the aggregate that raised this event';
COMMENT ON COLUMN event_store.aggregate_type IS 'Type of aggregate (WalletUser, Subscription, etc.)';
COMMENT ON COLUMN event_store.aggregate_version IS 'Version of aggregate when event was raised (for optimistic concurrency)';
COMMENT ON COLUMN event_store.event_type IS 'Type of event (WalletUserCreated, PermissionGranted, etc.)';
COMMENT ON COLUMN event_store.event_data IS 'Full event payload as JSON';
COMMENT ON COLUMN event_store.causation_id IS 'ID of command that caused this event';
COMMENT ON COLUMN event_store.correlation_id IS 'Trace ID for request correlation across services';

-- ================================================================================================
-- 2. TRANSACTIONAL OUTBOX TABLE
-- ================================================================================================
-- Outbox pattern for reliable event publishing
-- Events are saved here atomically with aggregate, then published asynchronously

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

-- Indexes for outbox pattern
CREATE INDEX idx_outbox_unprocessed ON outbox_events(processed, sequence_number) WHERE processed = false;
CREATE INDEX idx_outbox_aggregate ON outbox_events(aggregate_id);
CREATE INDEX idx_outbox_retry ON outbox_events(next_retry_at) WHERE processed = false AND next_retry_at IS NOT NULL;
CREATE INDEX idx_outbox_created_at ON outbox_events(created_at DESC);

-- Comments
COMMENT ON TABLE outbox_events IS 'Transactional outbox for reliable event publishing to Redis Streams';
COMMENT ON COLUMN outbox_events.processed IS 'Whether event has been published to Redis Streams';
COMMENT ON COLUMN outbox_events.sequence_number IS 'Monotonically increasing sequence for event ordering';
COMMENT ON COLUMN outbox_events.retry_count IS 'Number of times we tried to publish this event';
COMMENT ON COLUMN outbox_events.next_retry_at IS 'When to retry publishing this event (exponential backoff)';

-- ================================================================================================
-- 3. AGGREGATE SNAPSHOTS TABLE
-- ================================================================================================
-- Snapshots for performance optimization
-- Instead of replaying 1000s of events, load latest snapshot + events after snapshot

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

-- Indexes for snapshot queries
CREATE INDEX idx_snapshots_type_version ON aggregate_snapshots(aggregate_type, aggregate_version DESC);
CREATE INDEX idx_snapshots_created_at ON aggregate_snapshots(created_at DESC);

-- Comments
COMMENT ON TABLE aggregate_snapshots IS 'Aggregate snapshots for performance optimization';
COMMENT ON COLUMN aggregate_snapshots.snapshot_data IS 'Full aggregate state at this version';
COMMENT ON COLUMN aggregate_snapshots.event_count_at_snapshot IS 'Number of events processed when snapshot was taken';

-- ================================================================================================
-- 4. EVENT STORE STATISTICS VIEW
-- ================================================================================================
-- Materialized view for event store metrics and monitoring

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

-- ================================================================================================
-- 5. FUNCTIONS AND TRIGGERS
-- ================================================================================================

-- Function to update event store stats
CREATE OR REPLACE FUNCTION refresh_event_store_stats()
RETURNS TRIGGER AS $$
BEGIN
    -- Refresh stats materialized view asynchronously
    -- In production, this would be done by a scheduled job
    REFRESH MATERIALIZED VIEW CONCURRENTLY event_store_stats;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger to refresh stats (disabled by default for performance)
-- Uncomment to enable automatic stats refresh
-- CREATE TRIGGER trigger_refresh_event_store_stats
--     AFTER INSERT ON event_store
--     FOR EACH STATEMENT
--     EXECUTE FUNCTION refresh_event_store_stats();

-- ================================================================================================
-- 6. GRANTS
-- ================================================================================================

-- Grant permissions to application user (adjust username as needed)
-- GRANT SELECT, INSERT ON event_store TO epsx_app;
-- GRANT SELECT, INSERT, UPDATE ON outbox_events TO epsx_app;
-- GRANT SELECT, INSERT, UPDATE ON aggregate_snapshots TO epsx_app;
-- GRANT SELECT ON event_store_stats TO epsx_app;

-- ================================================================================================
-- MIGRATION COMPLETE
-- ================================================================================================

-- Verify tables created
DO $$
DECLARE
    table_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO table_count
    FROM information_schema.tables
    WHERE table_name IN ('event_store', 'outbox_events', 'aggregate_snapshots');

    IF table_count = 3 THEN
        RAISE NOTICE 'Event sourcing infrastructure created successfully';
        RAISE NOTICE 'Tables created: event_store, outbox_events, aggregate_snapshots';
        RAISE NOTICE 'Ready for application-level event publishing';
    ELSE
        RAISE EXCEPTION 'Failed to create all required tables';
    END IF;
END $$;
