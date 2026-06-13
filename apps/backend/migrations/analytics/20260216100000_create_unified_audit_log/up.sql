-- 20260216100000_create_unified_audit_log
--
-- Originally created unified_audit_log at the search_path default (i.e. in
-- the `analytics` schema). With the v3 baseline now owning the
-- `infra_logs` schema (which already creates infra_logs.unified_audit_log
-- in the same shape), this migration is a no-op when search_path is
-- infra_logs. We re-assert the table and indexes idempotently so this
-- file can be replayed on a fresh DB that pre-dates the v3 baseline rename
-- (e.g. an environment that only has the v2 baseline + this migration).
--
-- All statements are guarded with IF NOT EXISTS — safe to re-run.

SET search_path TO infra_logs;

CREATE TABLE IF NOT EXISTS infra_logs.unified_audit_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor           VARCHAR(42),
    actor_type      VARCHAR(20) NOT NULL DEFAULT 'admin',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resource_type   VARCHAR(50) NOT NULL,
    resource_id     VARCHAR(255),
    action          VARCHAR(50) NOT NULL,
    effect          VARCHAR(20) NOT NULL DEFAULT 'success',
    before_state    JSONB,
    after_state     JSONB,
    ip_address      VARCHAR(45),
    user_agent      TEXT,
    metadata        JSONB,
    category        VARCHAR(30) NOT NULL DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_ual_created_at ON infra_logs.unified_audit_log (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ual_actor ON infra_logs.unified_audit_log (actor);
CREATE INDEX IF NOT EXISTS idx_ual_category ON infra_logs.unified_audit_log (category);
CREATE INDEX IF NOT EXISTS idx_ual_resource ON infra_logs.unified_audit_log (resource_type, resource_id);
