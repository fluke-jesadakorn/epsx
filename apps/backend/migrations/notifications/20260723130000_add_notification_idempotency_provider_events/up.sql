-- Additive notification lifecycle extensions.
--
-- These tables make publisher admission, provider reconciliation, and user
-- engagement durable without changing the A3.11-compatible notification row.
-- They remain empty until N3/N4 cutover is explicitly approved.

CREATE TABLE IF NOT EXISTS public.notification_request_idempotency (
    principal_subject VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    idempotency_key VARCHAR(255) NOT NULL,
    request_hash CHAR(64) NOT NULL,
    response_status SMALLINT NOT NULL CHECK (response_status >= 100 AND response_status < 600),
    response_body JSONB NOT NULL,
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ(6),
    PRIMARY KEY (principal_subject, event_type, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_notification_request_idempotency_expiry
    ON public.notification_request_idempotency (expires_at)
    WHERE expires_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS public.notification_provider_events (
    provider VARCHAR(64) NOT NULL,
    provider_event_id VARCHAR(255) NOT NULL,
    job_id VARCHAR(128)
        REFERENCES public.notification_channel_jobs(id) ON DELETE RESTRICT,
    event_type VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    occurred_at TIMESTAMPTZ(6),
    received_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    PRIMARY KEY (provider, provider_event_id)
);

CREATE INDEX IF NOT EXISTS idx_notification_provider_events_job
    ON public.notification_provider_events (job_id, received_at);

CREATE TABLE IF NOT EXISTS public.notification_engagement (
    notification_id VARCHAR(66) NOT NULL
        REFERENCES public.notifications(id) ON DELETE RESTRICT,
    owner_id VARCHAR(66) NOT NULL,
    read_at TIMESTAMPTZ(6),
    clicked_at TIMESTAMPTZ(6),
    dismissed_at TIMESTAMPTZ(6),
    updated_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    PRIMARY KEY (notification_id, owner_id),
    CHECK (length(btrim(owner_id)) = 42 AND owner_id ~ '^0x[0-9A-Fa-f]{40}$')
);

CREATE INDEX IF NOT EXISTS idx_notification_engagement_owner
    ON public.notification_engagement (owner_id, updated_at DESC);
