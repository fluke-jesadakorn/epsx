CREATE TABLE IF NOT EXISTS unified_audit_log (
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

CREATE INDEX idx_ual_created_at ON unified_audit_log (created_at DESC);
CREATE INDEX idx_ual_actor ON unified_audit_log (actor);
CREATE INDEX idx_ual_category ON unified_audit_log (category);
CREATE INDEX idx_ual_resource ON unified_audit_log (resource_type, resource_id);
