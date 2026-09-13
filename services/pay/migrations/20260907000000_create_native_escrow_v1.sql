-- New contract authority: deliberately separate from legacy package and Pay rows.
CREATE TABLE IF NOT EXISTS pay_v1_deals (
 id TEXT PRIMARY KEY, chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL,
 contract_version INTEGER NOT NULL CHECK(contract_version=1), salt TEXT NOT NULL,
 on_chain_id TEXT NOT NULL, payer TEXT NOT NULL, payee TEXT NOT NULL, token_address TEXT NOT NULL,
 amount TEXT NOT NULL CHECK(amount ~ '^[1-9][0-9]{0,77}$'), description TEXT,
 status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','active','disputed','released','refunded','cancelled','verification_required')),
 fee_amount TEXT NOT NULL DEFAULT '0', tx_hash TEXT, verified_block BIGINT, verified_block_hash TEXT,
 expires_at TIMESTAMPTZ NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 UNIQUE(chain_id,contract_address,on_chain_id), CHECK(payer<>payee)
);
CREATE INDEX IF NOT EXISTS pay_v1_deals_owners ON pay_v1_deals(payer,payee,created_at);
CREATE TABLE IF NOT EXISTS pay_v1_links (
 slug TEXT PRIMARY KEY, owner TEXT NOT NULL, chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL,
 payee TEXT NOT NULL, token_address TEXT NOT NULL, amount TEXT NOT NULL CHECK(amount ~ '^[1-9][0-9]{0,77}$'), description TEXT,
 max_uses INTEGER NOT NULL CHECK(max_uses>0), current_uses INTEGER NOT NULL DEFAULT 0 CHECK(current_uses>=0 AND current_uses<=max_uses),
 expires_at TIMESTAMPTZ NOT NULL, disabled BOOLEAN NOT NULL DEFAULT false, created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS pay_v1_link_checkouts (
 slug TEXT NOT NULL REFERENCES pay_v1_links(slug), payer TEXT NOT NULL,
 idempotency_key TEXT NOT NULL, intent_id TEXT NOT NULL REFERENCES pay_v1_deals(id),
 PRIMARY KEY(slug,payer,idempotency_key), UNIQUE(intent_id)
);
CREATE TABLE IF NOT EXISTS pay_v1_requests (
 actor TEXT NOT NULL, idempotency_key TEXT NOT NULL, request_hash TEXT NOT NULL,
 response JSONB NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY(actor,idempotency_key)
);
CREATE TABLE IF NOT EXISTS pay_v1_operations (
 id TEXT PRIMARY KEY, deal_id TEXT NOT NULL REFERENCES pay_v1_deals(id), actor TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN ('deposit','release','refund','dispute','resolve-release','resolve-refund')),
 transaction_parameters JSONB NOT NULL, tx_hash TEXT,
 status TEXT NOT NULL DEFAULT 'awaiting_signature' CHECK(status IN ('awaiting_signature','pending','confirmed','failed','verification_required')),
 confirmed_block BIGINT, confirmed_block_hash TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS pay_v1_chain_checkpoints (
 chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL, next_block BIGINT NOT NULL,
 last_block_hash TEXT, healthy BOOLEAN NOT NULL DEFAULT false, last_checked_at TIMESTAMPTZ,
 PRIMARY KEY(chain_id,contract_address)
);
CREATE TABLE IF NOT EXISTS pay_v1_chain_events (
 chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL, block_number BIGINT NOT NULL, block_hash TEXT NOT NULL,
 tx_hash TEXT NOT NULL, log_index BIGINT NOT NULL, payload JSONB NOT NULL,
 PRIMARY KEY(chain_id,contract_address,block_hash,tx_hash,log_index)
);
CREATE TABLE IF NOT EXISTS pay_v1_chain_hints (
 event_id TEXT PRIMARY KEY, tx_hash TEXT NOT NULL, received_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
