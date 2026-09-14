-- Forward-only evidence for administrator payment mutations.

CREATE TABLE IF NOT EXISTS public.pay_admin_operations (
    operation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key VARCHAR(128) NOT NULL UNIQUE,
    action VARCHAR(32) NOT NULL,
    resource_id VARCHAR(66) NOT NULL,
    actor VARCHAR(128) NOT NULL,
    version_before TIMESTAMPTZ,
    version_after TIMESTAMPTZ,
    result JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT pay_admin_operations_result_object CHECK (jsonb_typeof(result) = 'object')
);

CREATE TABLE IF NOT EXISTS public.pay_ledger_entries (
    entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation_id UUID NOT NULL REFERENCES public.pay_admin_operations(operation_id) ON DELETE RESTRICT,
    resource_id VARCHAR(66) NOT NULL,
    entry_type VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    amount VARCHAR(78) NOT NULL,
    token_address VARCHAR(42) NOT NULL,
    chain_id VARCHAR(20) NOT NULL,
    tx_hash VARCHAR(66),
    finalized_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT pay_ledger_entries_finality CHECK (
        (status = 'pending' AND finalized_at IS NULL)
        OR (status = 'finalized' AND finalized_at IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS public.pay_link_admin_state (
    link_id VARCHAR(66) PRIMARY KEY REFERENCES public.pay_links(id) ON DELETE RESTRICT,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    version BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT pay_link_admin_state_status CHECK (status IN ('active', 'disabled')),
    CONSTRAINT pay_link_admin_state_version_nonnegative CHECK (version >= 0)
);

CREATE INDEX IF NOT EXISTS pay_admin_operations_resource_idx
    ON public.pay_admin_operations (resource_id, created_at DESC);
CREATE INDEX IF NOT EXISTS pay_ledger_entries_resource_idx
    ON public.pay_ledger_entries (resource_id, created_at DESC);
