-- ============================================================================
-- EPSX Infra Logs Consolidated Schema v3 - ROLLBACK
-- ============================================================================
-- Down migration drops all tables created in up.sql, then drops the
-- infra_logs schema itself for symmetric rollback. Idempotent: every
-- statement uses IF EXISTS so re-running is safe.
--
-- NOTE: this is the inverse of up.sql. To re-apply the schema, run up.sql
-- again. The audit_log_repository and other Rust code imports from
-- `crate::schemas::analytics::*` (the Rust module path, not the DB schema
-- name) — the underlying table objects are dropped here but the Rust
-- compile-time surface is unaffected.
-- ============================================================================

SET search_path TO infra_logs;

DROP TABLE IF EXISTS infra_logs.unified_audit_log CASCADE;
DROP TABLE IF EXISTS infra_logs.api_key_usage_logs CASCADE;
DROP TABLE IF EXISTS infra_logs.event_store CASCADE;
DROP TABLE IF EXISTS infra_logs.outbox_events CASCADE;
DROP TABLE IF EXISTS infra_logs.aggregate_snapshots CASCADE;
DROP TABLE IF EXISTS infra_logs.permission_audit_log CASCADE;
DROP TABLE IF EXISTS infra_logs.payment_audit_log CASCADE;
DROP TABLE IF EXISTS infra_logs.assignment_audit_log CASCADE;
DROP TABLE IF EXISTS infra_logs.wallet_activity_logs CASCADE;
DROP TABLE IF EXISTS infra_logs.audit_logs CASCADE;
DROP TABLE IF EXISTS infra_logs.analytics_events CASCADE;

-- Drop the schema itself. Per CLAUDE.md "Migration safety", DO NOT touch
-- the `public.*` tables — only drop the schema that this migration owns.
DROP SCHEMA IF EXISTS infra_logs CASCADE;

SELECT 'EPSX infra_logs consolidated schema v3 rollback completed!' AS completion_message;
