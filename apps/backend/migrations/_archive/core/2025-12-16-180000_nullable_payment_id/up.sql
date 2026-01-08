-- Make payment_id nullable for admin-assigned subscriptions
ALTER TABLE subscriptions ALTER COLUMN payment_id DROP NOT NULL;

-- Add comment explaining the change
COMMENT ON COLUMN subscriptions.payment_id IS 'Payment record ID - nullable for admin-assigned free subscriptions';
