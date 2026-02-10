-- Revert analytics database schema
DROP TABLE IF EXISTS assignment_audit_log;
DROP TABLE IF EXISTS payment_audit_log;
DROP TABLE IF EXISTS permission_audit_log;
DROP TABLE IF EXISTS aggregate_snapshots;
DROP TABLE IF EXISTS outbox_events;
DROP TABLE IF EXISTS event_store;
DROP TABLE IF EXISTS api_key_usage_logs;
