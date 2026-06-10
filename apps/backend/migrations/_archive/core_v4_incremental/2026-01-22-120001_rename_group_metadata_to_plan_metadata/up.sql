-- Migration: Rename group_metadata to plan_metadata
ALTER TABLE plans RENAME COLUMN group_metadata TO plan_metadata;
COMMENT ON COLUMN plans.plan_metadata IS 'JSONB metadata for the plan (formerly group_metadata)';
