-- Persist the command and ledger result in the same transaction. Replays cannot
-- grant/revoke twice, including after an ambiguous network response.
CREATE TABLE IF NOT EXISTS public.admin_credit_commands (
    actor VARCHAR(42) NOT NULL,
    idempotency_key VARCHAR(128) NOT NULL,
    payload_hash CHAR(64) NOT NULL,
    transaction_id UUID REFERENCES public.credit_transactions(id),
    balance_after NUMERIC(10,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (actor, idempotency_key),
    CHECK ((transaction_id IS NULL) = (balance_after IS NULL))
);
