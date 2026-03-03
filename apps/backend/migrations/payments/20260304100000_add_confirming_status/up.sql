-- Add 'confirming' to payments status check constraint
ALTER TABLE payments DROP CONSTRAINT IF EXISTS payments_status_check;
ALTER TABLE payments ADD CONSTRAINT payments_status_check
    CHECK (status IN ('created', 'pending', 'confirming', 'confirmed', 'failed', 'refunded', 'expired'));
