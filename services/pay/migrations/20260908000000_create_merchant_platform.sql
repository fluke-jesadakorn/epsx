-- Additive merchant platform. Legacy Pay and package tables keep their original authority.
CREATE TABLE IF NOT EXISTS pay_merchants (
 id TEXT PRIMARY KEY, owner TEXT NOT NULL UNIQUE, name TEXT NOT NULL,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS pay_merchant_keys (
 id TEXT PRIMARY KEY, merchant_id TEXT NOT NULL REFERENCES pay_merchants(id),
 environment TEXT NOT NULL CHECK(environment IN ('test','live')), name TEXT NOT NULL,
 key_hash TEXT NOT NULL UNIQUE, prefix TEXT NOT NULL, revoked_at TIMESTAMPTZ,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS pay_merchant_links (
 id TEXT PRIMARY KEY, merchant_id TEXT NOT NULL REFERENCES pay_merchants(id),
 environment TEXT NOT NULL CHECK(environment IN ('test','live')),
 mode TEXT NOT NULL CHECK(mode IN ('direct','escrow')), payee TEXT NOT NULL,
 token TEXT NOT NULL, amount TEXT NOT NULL CHECK(amount ~ '^[1-9][0-9]{0,77}$'), description TEXT,
 max_uses INTEGER CHECK(max_uses>0), disabled BOOLEAN NOT NULL DEFAULT false,
 expires_at TIMESTAMPTZ NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS pay_merchant_intents (
 id TEXT PRIMARY KEY, merchant_id TEXT NOT NULL REFERENCES pay_merchants(id),
 environment TEXT NOT NULL CHECK(environment IN ('test','live')), order_reference TEXT NOT NULL,
 link_id TEXT REFERENCES pay_merchant_links(id), mode TEXT NOT NULL CHECK(mode IN ('direct','escrow')),
 chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL, contract_version INTEGER NOT NULL,
 payer TEXT, payee TEXT NOT NULL, token TEXT NOT NULL, token_address TEXT NOT NULL, token_decimals INTEGER NOT NULL,
 amount TEXT NOT NULL CHECK(amount ~ '^[1-9][0-9]{0,77}$'), fee_bps INTEGER NOT NULL,
 fee_amount TEXT NOT NULL DEFAULT '0', description TEXT, metadata JSONB NOT NULL DEFAULT '{}',
 salt TEXT NOT NULL, on_chain_id TEXT, checkout_id TEXT NOT NULL UNIQUE, capability_hash TEXT NOT NULL,
 status TEXT NOT NULL DEFAULT 'awaiting_payment' CHECK(status IN ('awaiting_payment','funded','disputed','succeeded','refunded','expired','verification_required')),
 revision BIGINT NOT NULL DEFAULT 0, tx_hash TEXT, verified_block BIGINT, verified_block_hash TEXT,
 verification_error TEXT, expires_at TIMESTAMPTZ NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 UNIQUE(merchant_id,environment,order_reference), UNIQUE(chain_id,contract_address,on_chain_id)
);
CREATE INDEX IF NOT EXISTS pay_merchant_intents_link ON pay_merchant_intents(link_id,status);
CREATE TABLE IF NOT EXISTS pay_merchant_operations (
 id TEXT PRIMARY KEY, intent_id TEXT NOT NULL REFERENCES pay_merchant_intents(id), actor TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN ('pay','deposit','release','refund','dispute','resolve-release','resolve-refund')),
 transaction_parameters JSONB NOT NULL, approval_transaction JSONB,
 status TEXT NOT NULL DEFAULT 'awaiting_signature' CHECK(status IN ('awaiting_signature','pending','confirmed','failed','verification_required')),
 tx_hash TEXT, verified_block BIGINT, verified_block_hash TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS pay_merchant_requests (
 scope TEXT NOT NULL, key TEXT NOT NULL, request_hash TEXT NOT NULL, response JSONB NOT NULL,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY(scope,key)
);
CREATE TABLE IF NOT EXISTS pay_merchant_checkpoints (
 chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL, next_block BIGINT NOT NULL,
 last_block_hash TEXT, last_block_time TIMESTAMPTZ, healthy BOOLEAN NOT NULL DEFAULT false,
 checked_at TIMESTAMPTZ, PRIMARY KEY(chain_id,contract_address)
);
CREATE TABLE IF NOT EXISTS pay_merchant_chain_events (
 chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL, block_number BIGINT NOT NULL,
 block_hash TEXT NOT NULL, tx_hash TEXT NOT NULL, log_index BIGINT NOT NULL,
 intent_id TEXT NOT NULL REFERENCES pay_merchant_intents(id), status TEXT NOT NULL, fee_amount TEXT NOT NULL,
 payload JSONB NOT NULL, canonical BOOLEAN NOT NULL DEFAULT true,
 PRIMARY KEY(chain_id,contract_address,block_hash,tx_hash,log_index)
);
CREATE TABLE IF NOT EXISTS pay_merchant_endpoints (
 id TEXT PRIMARY KEY, merchant_id TEXT NOT NULL REFERENCES pay_merchants(id),
 environment TEXT NOT NULL CHECK(environment IN ('test','live')), url TEXT NOT NULL,
 secret_version INTEGER NOT NULL DEFAULT 1, previous_version INTEGER, previous_until TIMESTAMPTZ,
 enabled BOOLEAN NOT NULL DEFAULT true, created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS pay_merchant_events (
 id TEXT PRIMARY KEY, merchant_id TEXT NOT NULL REFERENCES pay_merchants(id), environment TEXT NOT NULL,
 intent_id TEXT NOT NULL REFERENCES pay_merchant_intents(id), revision BIGINT NOT NULL,
 event_type TEXT NOT NULL, payload JSONB NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 UNIQUE(intent_id,revision)
);
CREATE TABLE IF NOT EXISTS pay_merchant_deliveries (
 id TEXT PRIMARY KEY, event_id TEXT NOT NULL REFERENCES pay_merchant_events(id),
 endpoint_id TEXT NOT NULL REFERENCES pay_merchant_endpoints(id),
 attempts INTEGER NOT NULL DEFAULT 0, next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 retry_until TIMESTAMPTZ NOT NULL DEFAULT now()+interval '72 hours', leased_until TIMESTAMPTZ,
 status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','delivered','exhausted','disabled')),
 last_status INTEGER, last_error TEXT, delivered_at TIMESTAMPTZ,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(), UNIQUE(event_id,endpoint_id)
);
CREATE INDEX IF NOT EXISTS pay_merchant_deliveries_pending ON pay_merchant_deliveries(next_attempt_at) WHERE status='pending';
CREATE TABLE IF NOT EXISTS pay_merchant_delivery_attempts (
 delivery_id TEXT NOT NULL REFERENCES pay_merchant_deliveries(id), attempt INTEGER NOT NULL,
 http_status INTEGER, error TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 PRIMARY KEY(delivery_id,attempt)
);
CREATE TABLE IF NOT EXISTS pay_merchant_rate_limits (
 scope TEXT NOT NULL, window_start BIGINT NOT NULL, requests INTEGER NOT NULL,
 PRIMARY KEY(scope,window_start)
);
