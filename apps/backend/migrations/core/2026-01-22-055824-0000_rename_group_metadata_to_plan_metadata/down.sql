-- Rollback: Rename plan_metadata back to group_metadata
ALTER TABLE plans RENAME COLUMN plan_metadata TO group_metadata;
