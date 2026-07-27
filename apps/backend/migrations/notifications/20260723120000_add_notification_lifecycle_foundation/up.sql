-- Additive notification lifecycle foundation.
--
-- This migration deliberately does not alter or drop the A3.11-compatible
-- templates/notifications tables. The new tables are empty until an inbox/
-- outbox cutover is explicitly approved and reconciled.

CREATE TABLE IF NOT EXISTS public.notification_template_versions (
    id VARCHAR(128) PRIMARY KEY,
    template_id VARCHAR(66) NOT NULL REFERENCES public.templates(id) ON DELETE RESTRICT,
    version INTEGER NOT NULL CHECK (version > 0),
    subject TEXT,
    body TEXT NOT NULL,
    variables JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    UNIQUE (template_id, version)
);

CREATE INDEX IF NOT EXISTS idx_notification_template_versions_template
    ON public.notification_template_versions (template_id, version DESC);

CREATE TABLE IF NOT EXISTS public.notification_preferences (
    user_id VARCHAR(66) PRIMARY KEY,
    channels JSONB NOT NULL DEFAULT '{}'::jsonb,
    quiet_hours JSONB,
    timezone VARCHAR(64),
    updated_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    CHECK (length(btrim(user_id)) = 42 AND user_id ~ '^0x[0-9A-Fa-f]{40}$')
);

CREATE TABLE IF NOT EXISTS public.notification_inbox (
    principal_subject VARCHAR(255) NOT NULL,
    event_id VARCHAR(128) NOT NULL,
    request_hash CHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    state VARCHAR(16) NOT NULL DEFAULT 'received'
        CHECK (state IN ('received', 'processed', 'rejected')),
    received_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ(6),
    PRIMARY KEY (principal_subject, event_id)
);

CREATE INDEX IF NOT EXISTS idx_notification_inbox_state
    ON public.notification_inbox (state, received_at);

CREATE TABLE IF NOT EXISTS public.notification_outbox (
    event_id VARCHAR(128) PRIMARY KEY,
    event_type VARCHAR(100) NOT NULL,
    aggregate_id VARCHAR(128) NOT NULL,
    payload JSONB NOT NULL,
    state VARCHAR(16) NOT NULL DEFAULT 'pending'
        CHECK (state IN ('pending', 'published', 'failed')),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    available_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    claimed_until TIMESTAMPTZ(6),
    occurred_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ(6)
);

CREATE INDEX IF NOT EXISTS idx_notification_outbox_claimable
    ON public.notification_outbox (state, available_at, occurred_at);

CREATE TABLE IF NOT EXISTS public.notification_channel_jobs (
    id VARCHAR(128) PRIMARY KEY,
    source_event_id VARCHAR(128) NOT NULL
        REFERENCES public.notification_outbox(event_id) ON DELETE RESTRICT,
    notification_id VARCHAR(66) NOT NULL
        REFERENCES public.notifications(id) ON DELETE RESTRICT,
    channel VARCHAR(20) NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    state VARCHAR(20) NOT NULL DEFAULT 'queued'
        CHECK (state IN (
            'queued', 'leased', 'attempting', 'retry_wait',
            'provider_accepted', 'terminal_failed', 'dead_lettered'
        )),
    idempotency_key VARCHAR(255) NOT NULL,
    provider_message_id VARCHAR(255),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    available_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    lease_until TIMESTAMPTZ(6),
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    UNIQUE (source_event_id, notification_id, channel),
    UNIQUE (idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_notification_channel_jobs_claimable
    ON public.notification_channel_jobs (state, available_at, lease_until);

CREATE TABLE IF NOT EXISTS public.notification_delivery_attempts (
    job_id VARCHAR(128) NOT NULL
        REFERENCES public.notification_channel_jobs(id) ON DELETE RESTRICT,
    attempt_no INTEGER NOT NULL CHECK (attempt_no > 0),
    outcome VARCHAR(24) NOT NULL
        CHECK (outcome IN ('transient_failure', 'permanent_failure', 'accepted')),
    provider_message_id VARCHAR(255),
    error_code VARCHAR(100),
    attempted_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    PRIMARY KEY (job_id, attempt_no)
);

CREATE TABLE IF NOT EXISTS public.notification_dead_letters (
    job_id VARCHAR(128) PRIMARY KEY
        REFERENCES public.notification_channel_jobs(id) ON DELETE RESTRICT,
    reason VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    first_failed_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    redrive_count INTEGER NOT NULL DEFAULT 0 CHECK (redrive_count >= 0),
    last_redriven_at TIMESTAMPTZ(6),
    resolved_at TIMESTAMPTZ(6)
);

CREATE TABLE IF NOT EXISTS public.notification_replay_cursors (
    owner_id VARCHAR(66) NOT NULL,
    stream VARCHAR(32) NOT NULL,
    last_event_id VARCHAR(128),
    updated_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    PRIMARY KEY (owner_id, stream)
);

CREATE TABLE IF NOT EXISTS public.notification_push_subscriptions (
    endpoint TEXT PRIMARY KEY,
    user_id VARCHAR(66) NOT NULL,
    p256dh TEXT NOT NULL,
    auth TEXT NOT NULL,
    user_agent TEXT,
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ(6),
    CHECK (length(btrim(user_id)) = 42 AND user_id ~ '^0x[0-9A-Fa-f]{40}$')
);

CREATE INDEX IF NOT EXISTS idx_notification_push_subscriptions_user
    ON public.notification_push_subscriptions (user_id, revoked_at);
