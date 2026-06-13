-- 20260216100000_create_unified_audit_log rollback
--
-- Idempotent — DROP TABLE IF EXISTS guards the operation. Note that v3
-- baseline also creates infra_logs.unified_audit_log, so the rollback
-- here drops the table even though the v3 baseline would recreate it.
-- That is the desired behavior: this migration is a no-op in the modern
-- stack (v3 baseline owns the table), and the rollback cleanly removes
-- the index/table if the migration was applied to a v2-baseline DB.

DROP TABLE IF EXISTS infra_logs.unified_audit_log CASCADE;
