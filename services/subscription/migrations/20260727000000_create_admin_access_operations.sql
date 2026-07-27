-- Forward-only plan/access support for the admin commerce surfaces.

CREATE TABLE IF NOT EXISTS public.subscription_access_assignments (
    wallet_address VARCHAR(42) NOT NULL,
    plan_id UUID NOT NULL REFERENCES public.subscription_plans(id) ON DELETE RESTRICT,
    permission VARCHAR(128) NOT NULL,
    expires_at TIMESTAMPTZ,
    version BIGINT NOT NULL DEFAULT 0,
    assigned_by VARCHAR(128) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (wallet_address, plan_id, permission),
    CONSTRAINT subscription_access_assignments_permission CHECK (permission ~ '^[A-Za-z0-9:_-]{1,128}$'),
    CONSTRAINT subscription_access_assignments_version_nonnegative CHECK (version >= 0)
);

CREATE TABLE IF NOT EXISTS public.subscription_admin_operations (
    operation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key VARCHAR(128) NOT NULL UNIQUE,
    action VARCHAR(32) NOT NULL,
    resource_key VARCHAR(256) NOT NULL,
    actor VARCHAR(128) NOT NULL,
    version_before BIGINT,
    version_after BIGINT,
    result JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT subscription_admin_operations_result_object CHECK (jsonb_typeof(result) = 'object')
);

CREATE INDEX IF NOT EXISTS subscription_access_assignments_wallet_idx
    ON public.subscription_access_assignments (wallet_address, updated_at DESC);
CREATE INDEX IF NOT EXISTS subscription_access_assignments_plan_idx
    ON public.subscription_access_assignments (plan_id, updated_at DESC);
