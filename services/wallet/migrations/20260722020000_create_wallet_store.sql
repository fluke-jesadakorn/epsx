CREATE TABLE IF NOT EXISTS public.accounts (
    address VARCHAR(42) NOT NULL,
    chain_id VARCHAR(10) NOT NULL,
    label TEXT,
    role VARCHAR(50) DEFAULT 'user',
    encrypted_pk TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (address, chain_id)
);

CREATE TABLE IF NOT EXISTS public.nonces (
    address VARCHAR(42) NOT NULL,
    chain_id VARCHAR(10) NOT NULL,
    nonce BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (address, chain_id)
);

CREATE TABLE IF NOT EXISTS public.signed_transactions (
    id SERIAL PRIMARY KEY,
    chain_id VARCHAR(10) NOT NULL,
    sender VARCHAR(42) NOT NULL,
    recipient VARCHAR(42),
    value VARCHAR(78),
    data_hash VARCHAR(66),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
