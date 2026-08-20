-- Restore the core CQRS tables that were present in the pre-consolidation
-- migration history but were omitted from the active v6 baseline.
--
-- Every object is additive so databases that still have the legacy tables
-- retain their event history when this migration is recorded.

CREATE TABLE IF NOT EXISTS event_store (
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

CREATE INDEX IF NOT EXISTS idx_event_store_aggregate
    ON event_store(aggregate_id, aggregate_version);
CREATE INDEX IF NOT EXISTS idx_event_store_type_time
    ON event_store(event_type, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_event_store_correlation
    ON event_store(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate_type
    ON event_store(aggregate_type, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_event_store_occurred_at
    ON event_store(occurred_at DESC);

CREATE TABLE IF NOT EXISTS outbox_events (
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_payload JSONB NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at TIMESTAMPTZ,
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    next_retry_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sequence_number BIGSERIAL NOT NULL,
    CONSTRAINT outbox_retry_count_positive CHECK (retry_count >= 0),
    CONSTRAINT outbox_retry_count_limit CHECK (retry_count <= 10)
);

CREATE INDEX IF NOT EXISTS idx_outbox_unprocessed
    ON outbox_events(processed, sequence_number) WHERE processed = FALSE;
CREATE INDEX IF NOT EXISTS idx_outbox_aggregate
    ON outbox_events(aggregate_id);
CREATE INDEX IF NOT EXISTS idx_outbox_retry
    ON outbox_events(next_retry_at)
    WHERE processed = FALSE AND next_retry_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_outbox_created_at
    ON outbox_events(created_at DESC);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'outbox_events'::regclass
          AND conname = 'outbox_events_event_id_fkey'
    ) THEN
        ALTER TABLE outbox_events
            ADD CONSTRAINT outbox_events_event_id_fkey
            FOREIGN KEY (event_id)
            REFERENCES event_store(event_id)
            ON DELETE CASCADE;
    END IF;
END
$$;

CREATE TABLE IF NOT EXISTS aggregate_snapshots (
    aggregate_id VARCHAR(255) PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_count_at_snapshot INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT snapshot_version_positive CHECK (aggregate_version >= 0)
);

CREATE INDEX IF NOT EXISTS idx_snapshots_type_version
    ON aggregate_snapshots(aggregate_type, aggregate_version DESC);
CREATE INDEX IF NOT EXISTS idx_snapshots_created_at
    ON aggregate_snapshots(created_at DESC);

COMMENT ON TABLE event_store IS
    'Immutable event log for event sourcing and audit trail';
COMMENT ON TABLE outbox_events IS
    'Transactional outbox for reliable event publishing to Redis Streams';
COMMENT ON TABLE aggregate_snapshots IS
    'Aggregate snapshots for event replay performance';
