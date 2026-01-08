-- Add monitoring columns to payments table
-- These columns support the backend-driven transaction monitoring service

ALTER TABLE payments ADD COLUMN IF NOT EXISTS last_checked_at TIMESTAMPTZ;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS error_message TEXT;
ALTER TABLE payments ADD COLUMN IF NOT EXISTS network VARCHAR(50);

-- Add index for efficient pending transaction queries
CREATE INDEX IF NOT EXISTS idx_payments_status_pending ON payments(status) WHERE status = 'pending';
