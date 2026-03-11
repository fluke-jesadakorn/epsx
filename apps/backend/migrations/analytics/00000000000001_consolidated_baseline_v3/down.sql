-- ============================================================================
-- EPSX Analytics Consolidated Schema v3 - ROLLBACK
-- ============================================================================

DROP TABLE IF EXISTS unified_audit_log CASCADE;
DROP TABLE IF EXISTS api_key_usage_logs CASCADE;
DROP TABLE IF EXISTS event_store CASCADE;
DROP TABLE IF EXISTS outbox_events CASCADE;
DROP TABLE IF EXISTS aggregate_snapshots CASCADE;
DROP TABLE IF EXISTS permission_audit_log CASCADE;
DROP TABLE IF EXISTS payment_audit_log CASCADE;
DROP TABLE IF EXISTS assignment_audit_log CASCADE;
DROP TABLE IF EXISTS wallet_activity_logs CASCADE;
DROP TABLE IF EXISTS audit_logs CASCADE;
DROP TABLE IF EXISTS analytics_events CASCADE;

SELECT 'EPSX analytics consolidated schema v3 rollback completed!' AS completion_message;
