-- Additive lifecycle constraints. Existing rows are intentionally checked when
-- this migration is executed; no data is rewritten or deleted here.

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'templates_channel_check') THEN
        ALTER TABLE public.templates
            ADD CONSTRAINT templates_channel_check
            CHECK (channel IN ('email', 'in_app', 'push'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'templates_variables_object_check') THEN
        ALTER TABLE public.templates
            ADD CONSTRAINT templates_variables_object_check
            CHECK (jsonb_typeof(variables) = 'object');
    END IF;
END
$constraint$;

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notifications_channel_check') THEN
        ALTER TABLE public.notifications
            ADD CONSTRAINT notifications_channel_check
            CHECK (channel IN ('email', 'in_app', 'push'));
    END IF;
END
$constraint$;

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_preferences_channels_object_check') THEN
        ALTER TABLE public.notification_preferences
            ADD CONSTRAINT notification_preferences_channels_object_check
            CHECK (jsonb_typeof(channels) = 'object');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_preferences_quiet_hours_object_check') THEN
        ALTER TABLE public.notification_preferences
            ADD CONSTRAINT notification_preferences_quiet_hours_object_check
            CHECK (quiet_hours IS NULL OR jsonb_typeof(quiet_hours) = 'object');
    END IF;
END
$constraint$;

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_inbox_identity_check') THEN
        ALTER TABLE public.notification_inbox
            ADD CONSTRAINT notification_inbox_identity_check
            CHECK (length(btrim(principal_subject)) > 0 AND length(btrim(event_id)) > 0);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_inbox_hash_check') THEN
        ALTER TABLE public.notification_inbox
            ADD CONSTRAINT notification_inbox_hash_check
            CHECK (request_hash ~ '^[0-9a-fA-F]{64}$');
    END IF;
END
$constraint$;

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_outbox_identity_check') THEN
        ALTER TABLE public.notification_outbox
            ADD CONSTRAINT notification_outbox_identity_check
            CHECK (length(btrim(event_id)) > 0 AND length(btrim(event_type)) > 0 AND length(btrim(aggregate_id)) > 0);
    END IF;
END
$constraint$;

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_channel_jobs_recipient_check') THEN
        ALTER TABLE public.notification_channel_jobs
            ADD CONSTRAINT notification_channel_jobs_recipient_check
            CHECK (channel IN ('email', 'in_app', 'push') AND length(btrim(recipient)) > 0);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_channel_jobs_identity_check') THEN
        ALTER TABLE public.notification_channel_jobs
            ADD CONSTRAINT notification_channel_jobs_identity_check
            CHECK (length(btrim(id)) > 0 AND length(btrim(source_event_id)) > 0 AND length(btrim(idempotency_key)) > 0);
    END IF;
END
$constraint$;

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_replay_cursors_owner_check') THEN
        ALTER TABLE public.notification_replay_cursors
            ADD CONSTRAINT notification_replay_cursors_owner_check
            CHECK (length(btrim(owner_id)) = 42 AND owner_id ~ '^0x[0-9A-Fa-f]{40}$');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_replay_cursors_stream_check') THEN
        ALTER TABLE public.notification_replay_cursors
            ADD CONSTRAINT notification_replay_cursors_stream_check
            CHECK (stream IN ('owner'));
    END IF;
END
$constraint$;

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_push_subscription_payload_check') THEN
        ALTER TABLE public.notification_push_subscriptions
            ADD CONSTRAINT notification_push_subscription_payload_check
            CHECK (length(btrim(endpoint)) BETWEEN 1 AND 4096 AND length(btrim(p256dh)) BETWEEN 1 AND 512 AND length(btrim(auth)) BETWEEN 1 AND 512);
    END IF;
END
$constraint$;

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_request_idempotency_identity_check') THEN
        ALTER TABLE public.notification_request_idempotency
            ADD CONSTRAINT notification_request_idempotency_identity_check
            CHECK (length(btrim(principal_subject)) > 0 AND length(btrim(event_type)) > 0 AND length(btrim(idempotency_key)) > 0);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_request_idempotency_hash_check') THEN
        ALTER TABLE public.notification_request_idempotency
            ADD CONSTRAINT notification_request_idempotency_hash_check
            CHECK (request_hash ~ '^[0-9a-fA-F]{64}$');
    END IF;
END
$constraint$;

DO $constraint$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'notification_provider_events_identity_check') THEN
        ALTER TABLE public.notification_provider_events
            ADD CONSTRAINT notification_provider_events_identity_check
            CHECK (length(btrim(provider)) > 0 AND length(btrim(provider_event_id)) > 0 AND length(btrim(event_type)) > 0);
    END IF;
END
$constraint$;
