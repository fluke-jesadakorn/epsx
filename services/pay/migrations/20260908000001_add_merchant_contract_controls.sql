CREATE TABLE IF NOT EXISTS pay_merchant_contract_controls (
 id TEXT PRIMARY KEY, environment TEXT NOT NULL, mode TEXT NOT NULL,
 chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL,
 actor TEXT NOT NULL, paused BOOLEAN NOT NULL, transaction_parameters JSONB NOT NULL,
 tx_hash TEXT, status TEXT NOT NULL DEFAULT 'awaiting_signature',
 verified_block BIGINT, verified_block_hash TEXT,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 CHECK (environment IN ('test','live')), CHECK (mode IN ('direct','escrow')),
 CHECK (status IN ('awaiting_signature','pending','confirmed','failed','verification_required'))
);
