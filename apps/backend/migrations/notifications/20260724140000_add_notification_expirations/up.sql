-- Additive expiry projection for notifications.
-- Expiry is kept outside the A3.11 base row so the reviewed 26-column
-- notifications contract remains immutable while owner reads and workers can
-- apply one durable expiry policy.

CREATE TABLE IF NOT EXISTS public.notification_expirations (
    notification_id VARCHAR(66) PRIMARY KEY,
    expires_at TIMESTAMPTZ(6) NOT NULL,
    created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),
    CONSTRAINT notification_expirations_id_shape
        CHECK (btrim(notification_id) <> '' AND length(notification_id) <= 66)
);

CREATE INDEX IF NOT EXISTS notification_expirations_due_idx
    ON public.notification_expirations (expires_at, notification_id);

SELECT 1;
