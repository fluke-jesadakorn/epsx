CREATE TABLE IF NOT EXISTS pay_purchase_orders (
 id UUID PRIMARY KEY, wallet_address TEXT NOT NULL, plan_id UUID NOT NULL,
 request_key TEXT NOT NULL, request_hash TEXT NOT NULL,
 merchant_id TEXT NOT NULL, environment TEXT NOT NULL,
 chain_id BIGINT NOT NULL, contract_address TEXT NOT NULL, payee TEXT NOT NULL,
 token TEXT NOT NULL, token_address TEXT NOT NULL, token_decimals INTEGER NOT NULL,
 amount TEXT NOT NULL, duration_days BIGINT, description TEXT NOT NULL,
 pay_intent_id TEXT UNIQUE, checkout_url TEXT,
 status TEXT NOT NULL DEFAULT 'pending', payment_revision BIGINT NOT NULL DEFAULT 0,
 first_paid_at TIMESTAMPTZ, created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 UNIQUE(wallet_address,request_key)
);
CREATE TABLE IF NOT EXISTS pay_fulfillment_inbox (
 event_id TEXT PRIMARY KEY, order_id UUID NOT NULL REFERENCES pay_purchase_orders(id),
 payment_revision BIGINT NOT NULL, received_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- Existing assignments are captured once; each subsequent purchase has its own revocable grant.
CREATE TABLE IF NOT EXISTS pay_assignment_baselines (
 wallet_address TEXT NOT NULL, plan_id UUID NOT NULL, expires_at TIMESTAMPTZ,
 was_active BOOLEAN NOT NULL, assigned_at TIMESTAMPTZ NOT NULL,
 PRIMARY KEY(wallet_address,plan_id)
);
CREATE TABLE IF NOT EXISTS pay_purchase_grants (
 reference TEXT PRIMARY KEY, wallet_address TEXT NOT NULL, plan_id UUID NOT NULL,
 granted_at TIMESTAMPTZ NOT NULL, duration_days BIGINT, active BOOLEAN NOT NULL,
 source TEXT NOT NULL, UNIQUE(wallet_address,plan_id,reference)
);
