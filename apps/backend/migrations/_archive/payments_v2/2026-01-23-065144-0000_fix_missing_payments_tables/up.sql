
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_reference VARCHAR(255) UNIQUE NOT NULL,
    transaction_hash VARCHAR(66) UNIQUE,
    wallet_address VARCHAR(42) NOT NULL,
    amount DECIMAL(20, 6) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    method VARCHAR(50) NOT NULL DEFAULT 'crypto',
    status VARCHAR(20) NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'pending', 'confirmed', 'failed', 'refunded', 'expired')),
    plan_id UUID NOT NULL,
    contract_address VARCHAR(42),
    token_address VARCHAR(42),
    block_number BIGINT,
    confirmations INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    last_checked_at TIMESTAMPTZ,
    error_message TEXT,
    network VARCHAR(50),
    
    CONSTRAINT valid_wallet_address CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$'
    ),
    CONSTRAINT valid_transaction_hash CHECK (
        transaction_hash IS NULL OR
        transaction_hash ~ '^0x[a-fA-F0-9]{64}$'
    ),
    CONSTRAINT positive_amount CHECK (amount > 0)
);

CREATE INDEX IF NOT EXISTS idx_payments_wallet ON payments(wallet_address);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);
CREATE INDEX IF NOT EXISTS idx_payments_plan ON payments(plan_id);
CREATE INDEX IF NOT EXISTS idx_payments_tx_hash ON payments(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_payments_created ON payments(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_payments_expires ON payments(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_payments_status_pending ON payments(status) WHERE status = 'pending';

CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    plan_id UUID NOT NULL,
    payment_id UUID REFERENCES payments(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'expired', 'cancelled', 'suspended')),
    started_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    cancelled_at TIMESTAMPTZ,
    auto_renew BOOLEAN DEFAULT false,
    metadata JSONB DEFAULT '{}',
    
    CONSTRAINT unique_active_subscription
        UNIQUE (wallet_address, plan_id)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT valid_subscription_dates CHECK (
        started_at < expires_at
    ),
    CONSTRAINT valid_wallet_address_sub CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$'
    )
);

CREATE INDEX IF NOT EXISTS idx_subs_wallet ON subscriptions(wallet_address);
CREATE INDEX IF NOT EXISTS idx_subs_plan ON subscriptions(plan_id);
CREATE INDEX IF NOT EXISTS idx_subs_status ON subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_subs_payment ON subscriptions(payment_id);
CREATE INDEX IF NOT EXISTS idx_subs_expires ON subscriptions(expires_at);
CREATE INDEX IF NOT EXISTS idx_subs_active ON subscriptions(wallet_address, plan_id) WHERE status = 'active';

CREATE TABLE IF NOT EXISTS stock_ranking_assignments (
    assignment_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    package_id VARCHAR(255) NOT NULL,
    package_name VARCHAR(255) NOT NULL,
    rank_access_level INTEGER NOT NULL DEFAULT 1000,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    assignment_source VARCHAR(50) NOT NULL,
    auto_renew BOOLEAN NOT NULL DEFAULT false,
    payment_reference VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_stock_wallet ON stock_ranking_assignments(wallet_address);
CREATE INDEX IF NOT EXISTS idx_stock_package ON stock_ranking_assignments(package_id);
CREATE INDEX IF NOT EXISTS idx_stock_active ON stock_ranking_assignments(is_active, expires_at);

CREATE TABLE IF NOT EXISTS payment_contexts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    context_type VARCHAR(20) NOT NULL
        CHECK (context_type IN ('plan', 'group', 'product', 'campaign', 'custom')),
    context_id UUID,
    slug VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    amount DECIMAL(20, 8) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USDT',
    expires_at TIMESTAMPTZ,
    max_uses INTEGER,
    current_uses INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by VARCHAR(42) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT positive_context_amount CHECK (amount > 0),
    CONSTRAINT valid_usage_counts CHECK (
        current_uses >= 0 AND
        (max_uses IS NULL OR max_uses > 0)
    ),
    CONSTRAINT valid_context_created_by CHECK (
        created_by ~ '^0x[a-fA-F0-9]{40}$'
    )
);

CREATE INDEX IF NOT EXISTS idx_payment_contexts_slug ON payment_contexts(slug);
CREATE INDEX IF NOT EXISTS idx_payment_contexts_type ON payment_contexts(context_type);
CREATE INDEX IF NOT EXISTS idx_payment_contexts_context_id ON payment_contexts(context_id) WHERE context_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_payment_contexts_active ON payment_contexts(is_active, expires_at) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_payment_contexts_created_by ON payment_contexts(created_by);
CREATE INDEX IF NOT EXISTS idx_payment_contexts_expires ON payment_contexts(expires_at) WHERE expires_at IS NOT NULL AND is_active = true;
