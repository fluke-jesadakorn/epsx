CREATE TABLE IF NOT EXISTS public.pay_intents (
    id VARCHAR(66) PRIMARY KEY,
    chain_id VARCHAR(20) NOT NULL,
    payer VARCHAR(42) NOT NULL,
    payee VARCHAR(42) NOT NULL,
    amount VARCHAR(78) NOT NULL,
    token_address VARCHAR(42) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    escrow_id VARCHAR(66),
    tx_hash VARCHAR(66),
    description TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.escrows (
    id VARCHAR(66) PRIMARY KEY,
    chain_id VARCHAR(20) NOT NULL,
    payer VARCHAR(42) NOT NULL,
    payee VARCHAR(42) NOT NULL,
    amount VARCHAR(78) NOT NULL,
    token_address VARCHAR(42) NOT NULL,
    fee_amount VARCHAR(78) NOT NULL DEFAULT '0',
    status VARCHAR(30) NOT NULL DEFAULT 'active',
    on_chain_id VARCHAR(78),
    tx_hash VARCHAR(66),
    dispute_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.pay_links (
    id VARCHAR(66) PRIMARY KEY,
    slug VARCHAR(32) UNIQUE NOT NULL,
    intent_id VARCHAR(66) NOT NULL,
    max_uses INTEGER NOT NULL DEFAULT 1,
    current_uses INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public.pay_webhook_events (
    event_id VARCHAR(128) PRIMARY KEY,
    intent_id VARCHAR(66),
    escrow_id VARCHAR(66),
    event_type VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pay_intents_payer
    ON public.pay_intents (payer, status);
CREATE INDEX IF NOT EXISTS idx_pay_intents_payee
    ON public.pay_intents (payee, status);
CREATE INDEX IF NOT EXISTS idx_escrows_status
    ON public.escrows (status);
CREATE INDEX IF NOT EXISTS idx_pay_links_slug
    ON public.pay_links (slug);
CREATE INDEX IF NOT EXISTS idx_pay_links_intent
    ON public.pay_links (intent_id);
CREATE INDEX IF NOT EXISTS idx_pay_webhook_intent
    ON public.pay_webhook_events (intent_id);
