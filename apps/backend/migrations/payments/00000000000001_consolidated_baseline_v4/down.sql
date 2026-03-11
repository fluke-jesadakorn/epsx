-- ============================================================================
-- EPSX PAYMENTS CONSOLIDATED SCHEMA v4 - ROLLBACK
-- ============================================================================

DROP TRIGGER IF EXISTS payment_status_audit ON payments;
DROP FUNCTION IF EXISTS log_payment_status_changes();
DROP FUNCTION IF EXISTS add_credit_transaction(VARCHAR, NUMERIC, VARCHAR, UUID, VARCHAR, TEXT, VARCHAR, TIMESTAMPTZ, JSONB);
DROP TRIGGER IF EXISTS wallet_credits_updated_at ON wallet_credits;
DROP TRIGGER IF EXISTS payment_context_updated_at ON payment_contexts;
DROP TRIGGER IF EXISTS payments_updated_at ON payments;
DROP FUNCTION IF EXISTS update_timestamp_column();

DROP TABLE IF EXISTS credit_transactions CASCADE;
DROP TABLE IF EXISTS wallet_credits CASCADE;
DROP TABLE IF EXISTS payment_audit_log CASCADE;
DROP TABLE IF EXISTS payment_contexts CASCADE;
DROP TABLE IF EXISTS stock_ranking_assignments CASCADE;
DROP TABLE IF EXISTS subscriptions CASCADE;
DROP TABLE IF EXISTS payments CASCADE;

SELECT 'EPSX payments consolidated schema v4 rollback completed!' AS completion_message;
