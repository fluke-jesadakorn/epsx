CREATE TABLE IF NOT EXISTS public.blocks (
    chain_id VARCHAR(10) NOT NULL,
    number BIGINT NOT NULL,
    hash VARCHAR(66) NOT NULL,
    parent_hash VARCHAR(66) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    miner VARCHAR(42),
    gas_used BIGINT NOT NULL,
    gas_limit BIGINT NOT NULL,
    tx_count INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT blocks_pkey PRIMARY KEY (chain_id, number),
    CONSTRAINT blocks_chain_hash_key UNIQUE (chain_id, hash),
    CONSTRAINT blocks_chain_id_check CHECK (chain_id ~ '^[1-9][0-9]{0,9}$'),
    CONSTRAINT blocks_number_check CHECK (number >= 0),
    CONSTRAINT blocks_hash_check CHECK (hash ~ '^0x[0-9a-f]{64}$'),
    CONSTRAINT blocks_parent_hash_check CHECK (parent_hash ~ '^0x[0-9a-f]{64}$'),
    CONSTRAINT blocks_miner_check CHECK (miner IS NULL OR miner ~ '^0x[0-9a-f]{40}$'),
    CONSTRAINT blocks_gas_used_check CHECK (gas_used >= 0),
    CONSTRAINT blocks_gas_limit_check CHECK (gas_limit >= 0),
    CONSTRAINT blocks_gas_bounds_check CHECK (gas_used <= gas_limit),
    CONSTRAINT blocks_tx_count_check CHECK (tx_count >= 0)
);

CREATE TABLE IF NOT EXISTS public.transactions (
    chain_id VARCHAR(10) NOT NULL,
    hash VARCHAR(66) NOT NULL,
    from_address VARCHAR(42) NOT NULL,
    to_address VARCHAR(42),
    value VARCHAR(78) NOT NULL,
    block_number BIGINT NOT NULL,
    status INTEGER,
    timestamp TIMESTAMPTZ NOT NULL,
    input_data BYTEA NOT NULL,
    CONSTRAINT transactions_pkey PRIMARY KEY (chain_id, hash),
    CONSTRAINT transactions_chain_hash_block_key UNIQUE (chain_id, hash, block_number),
    CONSTRAINT transactions_block_fkey FOREIGN KEY (chain_id, block_number)
        REFERENCES public.blocks (chain_id, number) ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT transactions_chain_id_check CHECK (chain_id ~ '^[1-9][0-9]{0,9}$'),
    CONSTRAINT transactions_hash_check CHECK (hash ~ '^0x[0-9a-f]{64}$'),
    CONSTRAINT transactions_from_address_check CHECK (from_address ~ '^0x[0-9a-f]{40}$'),
    CONSTRAINT transactions_to_address_check CHECK (to_address IS NULL OR to_address ~ '^0x[0-9a-f]{40}$'),
    CONSTRAINT transactions_value_check CHECK (
        CASE
            WHEN value ~ '^(0|[1-9][0-9]{0,77})$'
                THEN value::NUMERIC <= NUMERIC '115792089237316195423570985008687907853269984665640564039457584007913129639935'
            ELSE FALSE
        END
    ),
    CONSTRAINT transactions_block_number_check CHECK (block_number >= 0),
    CONSTRAINT transactions_status_check CHECK (status IS NULL OR status = ANY (ARRAY[0, 1]))
);

CREATE TABLE IF NOT EXISTS public.token_transfers (
    chain_id VARCHAR(10) NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    log_index INTEGER NOT NULL,
    token_address VARCHAR(42) NOT NULL,
    from_address VARCHAR(42) NOT NULL,
    to_address VARCHAR(42) NOT NULL,
    value VARCHAR(78) NOT NULL,
    block_number BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    CONSTRAINT token_transfers_pkey PRIMARY KEY (chain_id, tx_hash, log_index),
    CONSTRAINT token_transfers_transaction_fkey FOREIGN KEY (chain_id, tx_hash, block_number)
        REFERENCES public.transactions (chain_id, hash, block_number) ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT token_transfers_chain_id_check CHECK (chain_id ~ '^[1-9][0-9]{0,9}$'),
    CONSTRAINT token_transfers_tx_hash_check CHECK (tx_hash ~ '^0x[0-9a-f]{64}$'),
    CONSTRAINT token_transfers_log_index_check CHECK (log_index >= 0),
    CONSTRAINT token_transfers_token_address_check CHECK (token_address ~ '^0x[0-9a-f]{40}$'),
    CONSTRAINT token_transfers_from_address_check CHECK (from_address ~ '^0x[0-9a-f]{40}$'),
    CONSTRAINT token_transfers_to_address_check CHECK (to_address ~ '^0x[0-9a-f]{40}$'),
    CONSTRAINT token_transfers_value_check CHECK (
        CASE
            WHEN value ~ '^(0|[1-9][0-9]{0,77})$'
                THEN value::NUMERIC <= NUMERIC '115792089237316195423570985008687907853269984665640564039457584007913129639935'
            ELSE FALSE
        END
    ),
    CONSTRAINT token_transfers_block_number_check CHECK (block_number >= 0)
);

CREATE INDEX IF NOT EXISTS idx_blocks_timestamp
    ON public.blocks USING btree (chain_id, timestamp DESC, number DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_block
    ON public.transactions USING btree (chain_id, block_number DESC, hash DESC);
CREATE INDEX IF NOT EXISTS idx_transfers_token
    ON public.token_transfers USING btree (chain_id, token_address, block_number DESC, tx_hash DESC, log_index DESC);
CREATE INDEX IF NOT EXISTS idx_transfers_from
    ON public.token_transfers USING btree (chain_id, from_address, block_number DESC, tx_hash DESC, log_index DESC);
CREATE INDEX IF NOT EXISTS idx_transfers_to
    ON public.token_transfers USING btree (chain_id, to_address, block_number DESC, tx_hash DESC, log_index DESC);
