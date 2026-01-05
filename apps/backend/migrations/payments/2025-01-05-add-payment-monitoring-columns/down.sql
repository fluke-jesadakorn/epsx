-- Revert monitoring columns
DROP INDEX IF EXISTS idx_payments_status_pending;
ALTER TABLE payments DROP COLUMN IF EXISTS network;
ALTER TABLE payments DROP COLUMN IF EXISTS error_message;
ALTER TABLE payments DROP COLUMN IF EXISTS last_checked_at;
