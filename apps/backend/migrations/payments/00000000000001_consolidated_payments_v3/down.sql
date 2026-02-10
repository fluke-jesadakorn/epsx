-- ============================================================================
-- EPSX PAYMENTS CONSOLIDATED SCHEMA v3 - ROLLBACK
-- ============================================================================

SELECT 'Starting EPSX payments schema v3 rollback...' AS status_message;

DROP TRIGGER IF EXISTS payment_status_audit ON payments;
DROP TRIGGER IF EXISTS payment_context_updated_at ON payment_contexts;
DROP TRIGGER IF EXISTS payment_updated_at ON payments;

DROP FUNCTION IF EXISTS log_payment_status_changes();
DROP FUNCTION IF EXISTS update_payment_context_updated_at();
DROP FUNCTION IF EXISTS update_payment_updated_at();

DROP TABLE IF EXISTS payment_audit_log CASCADE;
DROP TABLE IF EXISTS payment_contexts CASCADE;
DROP TABLE IF EXISTS stock_ranking_assignments CASCADE;
DROP TABLE IF EXISTS subscriptions CASCADE;
DROP TABLE IF EXISTS payments CASCADE;

SELECT 'EPSX payments schema v3 rollback completed!' AS completion_message;
