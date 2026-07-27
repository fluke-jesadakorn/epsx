-- Additive optimistic-concurrency state for admin plan operations.
-- This is intentionally separate from the legacy plan baseline so an
-- already-applied baseline is never rewritten.

CREATE TABLE IF NOT EXISTS public.subscription_plan_state (
    plan_id UUID PRIMARY KEY REFERENCES public.subscription_plans(id) ON DELETE RESTRICT,
    version BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT subscription_plan_state_version_nonnegative CHECK (version >= 0)
);

