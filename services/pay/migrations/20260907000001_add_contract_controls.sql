ALTER TABLE pay_v1_deals ADD COLUMN IF NOT EXISTS verification_error TEXT;
CREATE TABLE IF NOT EXISTS pay_v1_contract_operations (
 id TEXT PRIMARY KEY, chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL,
 actor TEXT NOT NULL, paused BOOLEAN NOT NULL, transaction_parameters JSONB NOT NULL,
 tx_hash TEXT, status TEXT NOT NULL DEFAULT 'awaiting_signature',
 confirmed_block BIGINT, confirmed_block_hash TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 CHECK(status IN ('awaiting_signature','pending','confirmed','failed','verification_required'))
);
