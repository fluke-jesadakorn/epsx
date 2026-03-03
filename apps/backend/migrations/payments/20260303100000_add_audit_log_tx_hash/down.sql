DROP INDEX IF EXISTS idx_payment_audit_tx;
ALTER TABLE payment_audit_log DROP COLUMN IF EXISTS tx_hash;
