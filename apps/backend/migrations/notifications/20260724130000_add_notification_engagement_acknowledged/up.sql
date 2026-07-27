-- Additive engagement lifecycle extension.
-- Client receipt acknowledgement is independent from delivery and read state.
-- Existing rows remain unacknowledged until a verified owner acknowledges them.

ALTER TABLE public.notification_engagement
    ADD COLUMN IF NOT EXISTS acknowledged_at TIMESTAMPTZ(6);

