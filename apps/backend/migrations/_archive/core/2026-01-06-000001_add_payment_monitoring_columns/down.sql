-- Remove monitoring columns
ALTER TABLE payments
DROP COLUMN last_checked_at,
DROP COLUMN error_message,
DROP COLUMN network;
