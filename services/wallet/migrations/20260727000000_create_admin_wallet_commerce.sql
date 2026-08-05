-- Forward-only commerce support for the admin wallet surfaces.
-- Existing wallet tables are intentionally left unchanged; these tables carry
-- operator state, immutable evidence, and the credit ledger separately.

CREATE TABLE IF NOT EXISTS public.wallet_admin_state (
    address VARCHAR(42) NOT NULL,
    chain_id VARCHAR(10) NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    version BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (address, chain_id),
    CONSTRAINT wallet_admin_state_status CHECK (status IN ('active', 'disabled')),
    CONSTRAINT wallet_admin_state_version_nonnegative CHECK (version >= 0),
    CONSTRAINT wallet_admin_state_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS public.wallet_admin_operations (
    operation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key VARCHAR(128) NOT NULL UNIQUE,
    address VARCHAR(42) NOT NULL,
    chain_id VARCHAR(10) NOT NULL,
    action VARCHAR(32) NOT NULL,
    actor VARCHAR(128) NOT NULL,
    version_before BIGINT NOT NULL,
    version_after BIGINT NOT NULL,
    result JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT wallet_admin_operations_result_object CHECK (jsonb_typeof(result) = 'object')
);

CREATE TABLE IF NOT EXISTS public.wallet_credit_accounts (
    address VARCHAR(42) PRIMARY KEY,
    balance_minor BIGINT NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT wallet_credit_accounts_balance_nonnegative CHECK (balance_minor >= 0),
    CONSTRAINT wallet_credit_accounts_version_nonnegative CHECK (version >= 0)
);

CREATE TABLE IF NOT EXISTS public.wallet_credit_ledger (
    entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key VARCHAR(128) NOT NULL UNIQUE,
    address VARCHAR(42) NOT NULL,
    operation VARCHAR(8) NOT NULL,
    delta_minor BIGINT NOT NULL,
    balance_after_minor BIGINT NOT NULL,
    reason VARCHAR(500) NOT NULL,
    actor VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT wallet_credit_ledger_operation CHECK (operation IN ('grant', 'revoke')),
    CONSTRAINT wallet_credit_ledger_balance_nonnegative CHECK (balance_after_minor >= 0)
);

CREATE INDEX IF NOT EXISTS wallet_admin_state_status_idx
    ON public.wallet_admin_state (status, updated_at DESC);
CREATE INDEX IF NOT EXISTS wallet_admin_operations_address_idx
    ON public.wallet_admin_operations (address, chain_id, created_at DESC);
CREATE INDEX IF NOT EXISTS wallet_credit_ledger_address_idx
    ON public.wallet_credit_ledger (address, created_at DESC);
