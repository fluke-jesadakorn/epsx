-- Add tx_hash column to payment_audit_log for direct lookups
ALTER TABLE payment_audit_log ADD COLUMN IF NOT EXISTS tx_hash TEXT;
CREATE INDEX IF NOT EXISTS idx_payment_audit_tx ON payment_audit_log(tx_hash);
