-- Revert: Make payment_id NOT NULL again
-- Note: This may fail if there are subscriptions with NULL payment_id
ALTER TABLE subscriptions ALTER COLUMN payment_id SET NOT NULL;
