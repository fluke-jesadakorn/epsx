-- Additive template lifecycle audit records.
--
-- Rollback and other template mutations must leave a durable, replayable
-- version trail without storing rendered bodies or recipient data in audit
-- metadata. The table is empty until the template service writes it.

CREATE TABLE IF NOT EXISTS public.notification_template_audit (
    id VARCHAR(128) PRIMARY KEY,
    template_id VARCHAR(66) NOT NULL
        REFERENCES public.templates(id) ON DELETE RESTRICT,
    action VARCHAR(32) NOT NULL
        CHECK (action IN ('created', 'updated', 'deleted', 'rollback')),
    from_version INTEGER,
    to_version INTEGER,
    actor_subject VARCHAR(255) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    CHECK (from_version IS NULL OR from_version > 0),
    CHECK (to_version IS NULL OR to_version > 0),
    CHECK (length(btrim(actor_subject)) > 0),
    CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS idx_notification_template_audit_template
    ON public.notification_template_audit (template_id, created_at DESC, id DESC);
