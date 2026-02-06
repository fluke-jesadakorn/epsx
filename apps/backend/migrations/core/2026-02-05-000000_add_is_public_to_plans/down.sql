-- Rollback: Remove is_public column from plans table
ALTER TABLE plans DROP COLUMN is_public;
