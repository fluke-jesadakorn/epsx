-- Create read_model schema if it doesn't exist
CREATE SCHEMA IF NOT EXISTS read_model;

-- Create projection_checkpoints table
CREATE TABLE IF NOT EXISTS read_model.projection_checkpoints (
    projection_name VARCHAR(255) PRIMARY KEY,
    last_processed_event_id UUID,
    last_processed_sequence BIGINT NOT NULL DEFAULT 0,
    events_processed_count BIGINT NOT NULL DEFAULT 0,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_healthy BOOLEAN NOT NULL DEFAULT true,
    
    -- Constraints
    CONSTRAINT projection_checkpoints_sequence_positive CHECK (last_processed_sequence >= 0),
    CONSTRAINT projection_checkpoints_count_positive CHECK (events_processed_count >= 0)
);

-- Comments
COMMENT ON TABLE read_model.projection_checkpoints IS 'Tracks progress of CQRS read model projections';
COMMENT ON COLUMN read_model.projection_checkpoints.projection_name IS 'Unique identifier for the projection';
COMMENT ON COLUMN read_model.projection_checkpoints.last_processed_event_id IS 'ID of the last successfully processed event';
COMMENT ON COLUMN read_model.projection_checkpoints.last_processed_sequence IS 'Sequence number of the last successfully processed event';
