-- Add UNIQUE constraint on transaction_hash to prevent replay attacks
-- NULL values are excluded (credit-only payments have no tx hash)
CREATE UNIQUE INDEX IF NOT EXISTS idx_payments_unique_tx_hash
    ON payments (transaction_hash)
    WHERE transaction_hash IS NOT NULL;

-- Add index for pending payment expiry queries
CREATE INDEX IF NOT EXISTS idx_payments_pending_created
    ON payments (created_at)
    WHERE status = 'pending';

-- Add index for confirming status queries
CREATE INDEX IF NOT EXISTS idx_payments_confirming
    ON payments (status)
    WHERE status = 'confirming';
