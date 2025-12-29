-- ================================================================================================
-- ADD EVENT SOURCING TABLES MIGRATION
-- ================================================================================================
-- Version: 2025-11-27-045240-0000
-- Created: 2025-11-27
-- Description: Add event sourcing infrastructure tables for CQRS pattern
--
-- Tables Added:
-- - event_store: Immutable event log for event sourcing and audit trail
-- - outbox_events: Transactional outbox for reliable event publishing to Redis Streams
-- - aggregate_snapshots: Aggregate snapshots for performance optimization
--
-- Features:
-- - CQRS event sourcing with proper indexing
-- - Outbox pattern for reliable async event publishing
-- - Snapshot pattern for performance optimization
-- - Foreign key constraints for data integrity
-- ================================================================================================

-- ================================================================================================
-- SECTION 1: EVENT STORE TABLE
-- ================================================================================================

-- Event store for immutable event log
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

-- Indexes for event_store (idempotent)
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate ON event_store(aggregate_id, aggregate_version);
CREATE INDEX IF NOT EXISTS idx_event_store_type_time ON event_store(event_type, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_event_store_correlation ON event_store(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate_type ON event_store(aggregate_type, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_event_store_occurred_at ON event_store(occurred_at DESC);

COMMENT ON TABLE event_store IS 'Immutable event log for event sourcing and audit trail';

-- ================================================================================================
-- SECTION 2: OUTBOX EVENTS TABLE
-- ================================================================================================

-- Transactional outbox for reliable event publishing
CREATE TABLE IF NOT EXISTS outbox_events (
    -- Outbox identification
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL,

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

-- Indexes for outbox_events (idempotent)
CREATE INDEX IF NOT EXISTS idx_outbox_unprocessed ON outbox_events(processed, sequence_number) WHERE processed = false;
CREATE INDEX IF NOT EXISTS idx_outbox_aggregate ON outbox_events(aggregate_id);
CREATE INDEX IF NOT EXISTS idx_outbox_retry ON outbox_events(next_retry_at) WHERE processed = false AND next_retry_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_outbox_created_at ON outbox_events(created_at DESC);

-- Foreign key to event_store (idempotent)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'outbox_events_event_id_fkey'
        AND table_name = 'outbox_events'
    ) THEN
        ALTER TABLE outbox_events
        ADD CONSTRAINT outbox_events_event_id_fkey
        FOREIGN KEY (event_id) REFERENCES event_store(event_id) ON DELETE CASCADE;
    END IF;
END $$;

COMMENT ON TABLE outbox_events IS 'Transactional outbox for reliable event publishing to Redis Streams';

-- ================================================================================================
-- SECTION 3: AGGREGATE SNAPSHOTS TABLE
-- ================================================================================================

-- Aggregate snapshots for performance optimization
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

-- Indexes for aggregate_snapshots (idempotent)
CREATE INDEX IF NOT EXISTS idx_snapshots_type_version ON aggregate_snapshots(aggregate_type, aggregate_version DESC);
CREATE INDEX IF NOT EXISTS idx_snapshots_created_at ON aggregate_snapshots(created_at DESC);

COMMENT ON TABLE aggregate_snapshots IS 'Aggregate snapshots for performance optimization';

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'EVENT SOURCING TABLES CREATED SUCCESSFULLY! 🎉';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Event Sourcing Tables Added:';
    RAISE NOTICE '  ✅ event_store (immutable event log)';
    RAISE NOTICE '  ✅ outbox_events (reliable event publishing)';
    RAISE NOTICE '  ✅ aggregate_snapshots (performance optimization)';
    RAISE NOTICE '';
    RAISE NOTICE 'Features:';
    RAISE NOTICE '  ✅ CQRS event sourcing with proper indexing';
    RAISE NOTICE '  ✅ Outbox pattern for reliable async event publishing';
    RAISE NOTICE '  ✅ Snapshot pattern for performance optimization';
    RAISE NOTICE '  ✅ Foreign key constraints for data integrity';
    RAISE NOTICE '';
    RAISE NOTICE '🚀 Event sourcing infrastructure ready for EPSX platform!';
    RAISE NOTICE '=================================================================================';
END $$;
