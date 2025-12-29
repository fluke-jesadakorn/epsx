-- Rollback payment_contexts table
DROP TRIGGER IF EXISTS payment_context_updated_at ON payment_contexts;
DROP FUNCTION IF EXISTS update_payment_context_updated_at();
DROP TABLE IF EXISTS payment_contexts;
