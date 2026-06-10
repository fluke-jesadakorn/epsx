-- ============================================================================
-- EPSX Payments Database Schema
-- ============================================================================
-- Separate database for financial transactions
-- Enables PCI-DSS compliance isolation and dedicated transaction handling
-- 
-- Tables:
--   - payments
--   - subscriptions
--   - stock_ranking_assignments
-- ============================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

-- ============================================================================
-- PAYMENTS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Payment identification
    payment_reference VARCHAR(255) UNIQUE NOT NULL,
    transaction_hash VARCHAR(66) UNIQUE,

    -- Payment details
    wallet_address VARCHAR(42) NOT NULL,
    amount DECIMAL(20, 6) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    method VARCHAR(50) NOT NULL DEFAULT 'crypto',

    -- Status tracking
    status VARCHAR(20) NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'pending', 'confirmed', 'failed', 'refunded', 'expired')),

    -- Plan association (reference only, no FK - cross-database)
    plan_id UUID NOT NULL,

    -- Blockchain details
    contract_address VARCHAR(42),
    token_address VARCHAR(42),
    block_number BIGINT,
    confirmations INTEGER DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    -- Additional metadata
    metadata JSONB DEFAULT '{}',

    -- Constraints
    CONSTRAINT valid_wallet_address CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$'
    ),
    CONSTRAINT valid_transaction_hash CHECK (
        transaction_hash IS NULL OR
        transaction_hash ~ '^0x[a-fA-F0-9]{64}$'
    ),
    CONSTRAINT positive_amount CHECK (amount > 0)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_payments_wallet ON payments(wallet_address);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);
CREATE INDEX IF NOT EXISTS idx_payments_plan ON payments(plan_id);
CREATE INDEX IF NOT EXISTS idx_payments_tx_hash ON payments(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_payments_created ON payments(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_payments_expires ON payments(expires_at) WHERE expires_at IS NOT NULL;

-- ============================================================================
-- SUBSCRIPTIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Subscription identification
    wallet_address VARCHAR(42) NOT NULL,
    plan_id UUID NOT NULL,  -- Reference only, no FK
    payment_id UUID REFERENCES payments(id),

    -- Status and lifecycle
    status VARCHAR(20) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'expired', 'cancelled', 'suspended')),

    -- Timestamps
    started_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    cancelled_at TIMESTAMPTZ,

    -- Subscription settings
    auto_renew BOOLEAN DEFAULT false,

    -- Additional metadata
    metadata JSONB DEFAULT '{}',

    -- Constraints
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

-- Indexes
CREATE INDEX IF NOT EXISTS idx_subs_wallet ON subscriptions(wallet_address);
CREATE INDEX IF NOT EXISTS idx_subs_plan ON subscriptions(plan_id);
CREATE INDEX IF NOT EXISTS idx_subs_status ON subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_subs_payment ON subscriptions(payment_id);
CREATE INDEX IF NOT EXISTS idx_subs_expires ON subscriptions(expires_at);
CREATE INDEX IF NOT EXISTS idx_subs_active ON subscriptions(wallet_address, plan_id) WHERE status = 'active';

-- ============================================================================
-- STOCK RANKING ASSIGNMENTS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS stock_ranking_assignments (
    assignment_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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

-- Indexes
CREATE INDEX IF NOT EXISTS idx_stock_wallet ON stock_ranking_assignments(wallet_address);
CREATE INDEX IF NOT EXISTS idx_stock_package ON stock_ranking_assignments(package_id);
CREATE INDEX IF NOT EXISTS idx_stock_active ON stock_ranking_assignments(is_active, expires_at);

-- ============================================================================
-- TRIGGER FOR updated_at
-- ============================================================================

CREATE OR REPLACE FUNCTION update_payment_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'payment_updated_at') THEN
        CREATE TRIGGER payment_updated_at
            BEFORE UPDATE ON payments
            FOR EACH ROW
            EXECUTE FUNCTION update_payment_updated_at();
    END IF;
END $$;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE payments IS 'Core payment tracking for blockchain transactions';
COMMENT ON TABLE subscriptions IS 'Active subscription management with lifecycle tracking';
COMMENT ON TABLE stock_ranking_assignments IS 'Stock ranking package assignments with access levels';
