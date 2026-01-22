-- Rollback: Rename plan_type back to group_type
ALTER TABLE plans RENAME COLUMN plan_type TO group_type;
