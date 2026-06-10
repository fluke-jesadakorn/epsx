-- Revert: remove 'confirming' from payments status check constraint
-- First update any confirming rows to pending
UPDATE payments SET status = 'pending' WHERE status = 'confirming';
ALTER TABLE payments DROP CONSTRAINT IF EXISTS payments_status_check;
ALTER TABLE payments ADD CONSTRAINT payments_status_check
    CHECK (status IN ('created', 'pending', 'confirmed', 'failed', 'refunded', 'expired'));
