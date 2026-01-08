-- Add monitoring columns to payments table
ALTER TABLE payments
ADD COLUMN IF NOT EXISTS last_checked_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS error_message TEXT,
ADD COLUMN IF NOT EXISTS network VARCHAR(50);
