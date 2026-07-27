-- Additive VAPID key lineage for safe active/previous key rotation.
-- Existing subscriptions remain associated with the historical default key ID;
-- new subscriptions record the currently active deployment key.

ALTER TABLE public.notification_push_subscriptions
    ADD COLUMN IF NOT EXISTS vapid_key_id VARCHAR(128) NOT NULL DEFAULT 'active';

DO $constraint$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'notification_push_subscription_vapid_key_id_check'
    ) THEN
        ALTER TABLE public.notification_push_subscriptions
            ADD CONSTRAINT notification_push_subscription_vapid_key_id_check
            CHECK (
                length(btrim(vapid_key_id)) BETWEEN 1 AND 128
                AND vapid_key_id ~ '^[A-Za-z0-9_-]+$'
            );
    END IF;
END
$constraint$;

CREATE INDEX IF NOT EXISTS idx_notification_push_subscriptions_vapid_key
    ON public.notification_push_subscriptions (vapid_key_id, revoked_at);

SELECT 1;
