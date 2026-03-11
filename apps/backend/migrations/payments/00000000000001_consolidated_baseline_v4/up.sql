-- ============================================================================
-- EPSX PAYMENTS CONSOLIDATED SCHEMA v4
-- ============================================================================
-- Version: 4.0.0 (Consolidated March 2026)
-- Description: Complete payments schema for blockchain transactions.
--              Includes credit wallet system and payment tracking.
--
-- Tables:
--   - payments: Core payment tracking
--   - subscriptions: Active subscription management  
--   - stock_ranking_assignments: Stock ranking package assignments
--   - payment_contexts: Dynamic payment links
--   - payment_audit_log: Payment event audit trail
--   - wallet_credits: User credit balances
--   - credit_transactions: Credit transaction history
-- ============================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

-- ============================================================================
-- PAYMENTS TABLE
-- ============================================================================

CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_reference VARCHAR(255) UNIQUE NOT NULL,
    transaction_hash VARCHAR(66),
    wallet_address VARCHAR(42) NOT NULL,
    amount DECIMAL(20, 6) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    method VARCHAR(50) NOT NULL DEFAULT 'crypto',
    status VARCHAR(20) NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'pending', 'confirming', 'confirmed', 'failed', 'refunded', 'expired')),
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

CREATE INDEX idx_payments_wallet ON payments(wallet_address);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_plan ON payments(plan_id);
CREATE INDEX idx_payments_tx_hash ON payments(transaction_hash);
CREATE INDEX idx_payments_created ON payments(created_at DESC);
CREATE INDEX idx_payments_expires ON payments(expires_at) WHERE expires_at IS NOT NULL;

-- Unique transaction hash (excluding NULLs for credit-only payments)
CREATE UNIQUE INDEX idx_payments_unique_tx_hash ON payments (transaction_hash) WHERE transaction_hash IS NOT NULL;

-- ============================================================================
-- SUBSCRIPTIONS TABLE
-- ============================================================================

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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

CREATE INDEX idx_subs_wallet ON subscriptions(wallet_address);
CREATE INDEX idx_subs_status ON subscriptions(status);
CREATE INDEX idx_subs_expires ON subscriptions(expires_at);

-- ============================================================================
-- STOCK RANKING ASSIGNMENTS TABLE
-- ============================================================================

CREATE TABLE stock_ranking_assignments (
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

CREATE INDEX idx_stock_wallet ON stock_ranking_assignments(wallet_address);
CREATE INDEX idx_stock_active ON stock_ranking_assignments(is_active, expires_at);

-- ============================================================================
-- PAYMENT CONTEXTS TABLE
-- ============================================================================

CREATE TABLE payment_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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
    CONSTRAINT valid_context_created_by CHECK (
        created_by ~ '^0x[a-fA-F0-9]{40}$'
    )
);

CREATE INDEX idx_payment_contexts_slug ON payment_contexts(slug);
CREATE INDEX idx_payment_contexts_active ON payment_contexts(is_active, expires_at) WHERE is_active = true;

-- ============================================================================
-- CREDIT WALLET SYSTEM
-- ============================================================================

CREATE TABLE wallet_credits (
    wallet_address VARCHAR(42) PRIMARY KEY,
    balance NUMERIC(10,2) NOT NULL DEFAULT 0,
    pending_balance NUMERIC(10,2) NOT NULL DEFAULT 0,
    lifetime_earned NUMERIC(10,2) NOT NULL DEFAULT 0,
    lifetime_spent NUMERIC(10,2) NOT NULL DEFAULT 0,
    last_transaction_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_wallet_address_credits CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$'
    ),
    CONSTRAINT non_negative_balance CHECK (balance >= 0)
);

CREATE INDEX idx_wallet_credits_balance ON wallet_credits(balance) WHERE balance > 0;

CREATE TABLE credit_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) NOT NULL,
    amount NUMERIC(10,2) NOT NULL,
    balance_after NUMERIC(10,2) NOT NULL,
    tx_type VARCHAR(30) NOT NULL
        CHECK (tx_type IN ('grant', 'revoke', 'payment_debit', 'proration_credit', 'refund', 'expiry', 'adjustment')),
    reference_id UUID,
    reference_type VARCHAR(30),
    reason TEXT,
    granted_by VARCHAR(42),
    expires_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_wallet_address_tx CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$'
    )
);

CREATE INDEX idx_credit_tx_wallet ON credit_transactions(wallet_address);
CREATE INDEX idx_credit_tx_created ON credit_transactions(created_at DESC);

-- ============================================================================
-- PAYMENT AUDIT LOG
-- ============================================================================

CREATE TABLE payment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL REFERENCES payments(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    old_status VARCHAR(20),
    new_status VARCHAR(20),
    tx_hash TEXT,
    reason TEXT,
    performed_by VARCHAR(42),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payment_audit_payment ON payment_audit_log(payment_id);
CREATE INDEX idx_payment_audit_tx ON payment_audit_log(tx_hash);
CREATE INDEX idx_payment_audit_created ON payment_audit_log(created_at DESC);

-- ============================================================================
-- TRIGGERS & FUNCTIONS
-- ============================================================================

CREATE OR REPLACE FUNCTION update_timestamp_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER payments_updated_at BEFORE UPDATE ON payments FOR EACH ROW EXECUTE FUNCTION update_timestamp_column();
CREATE TRIGGER payment_contexts_updated_at BEFORE UPDATE ON payment_contexts FOR EACH ROW EXECUTE FUNCTION update_timestamp_column();
CREATE TRIGGER wallet_credits_updated_at BEFORE UPDATE ON wallet_credits FOR EACH ROW EXECUTE FUNCTION update_timestamp_column();

-- Function to safely add credit transaction and update balance
CREATE OR REPLACE FUNCTION add_credit_transaction(
    p_wallet_address VARCHAR(42),
    p_amount NUMERIC(10,2),
    p_tx_type VARCHAR(30),
    p_reference_id UUID DEFAULT NULL,
    p_reference_type VARCHAR(30) DEFAULT NULL,
    p_reason TEXT DEFAULT NULL,
    p_granted_by VARCHAR(42) DEFAULT NULL,
    p_expires_at TIMESTAMPTZ DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'
)
RETURNS UUID AS $$
DECLARE
    v_current_balance NUMERIC(10,2);
    v_new_balance NUMERIC(10,2);
    v_transaction_id UUID;
BEGIN
    INSERT INTO wallet_credits (wallet_address, balance)
    VALUES (p_wallet_address, 0)
    ON CONFLICT (wallet_address) DO NOTHING;

    SELECT balance INTO v_current_balance FROM wallet_credits WHERE wallet_address = p_wallet_address FOR UPDATE;
    v_new_balance := v_current_balance + p_amount;

    IF v_new_balance < 0 THEN
        RAISE EXCEPTION 'Insufficient credit balance';
    END IF;

    INSERT INTO credit_transactions (
        wallet_address, amount, balance_after, tx_type,
        reference_id, reference_type, reason, granted_by, expires_at, metadata
    ) VALUES (
        p_wallet_address, p_amount, v_new_balance, p_tx_type,
        p_reference_id, p_reference_type, p_reason, p_granted_by, p_expires_at, p_metadata
    ) RETURNING id INTO v_transaction_id;

    UPDATE wallet_credits
    SET
        balance = v_new_balance,
        last_transaction_at = NOW(),
        lifetime_earned = CASE WHEN p_amount > 0 THEN lifetime_earned + p_amount ELSE lifetime_earned END,
        lifetime_spent = CASE WHEN p_amount < 0 THEN lifetime_spent + ABS(p_amount) ELSE lifetime_spent END
    WHERE wallet_address = p_wallet_address;

    RETURN v_transaction_id;
END;
$$ LANGUAGE plpgsql;

-- Payment status change audit trigger
CREATE OR REPLACE FUNCTION log_payment_status_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        INSERT INTO payment_audit_log (
            payment_id, action, old_status, new_status, tx_hash, reason, performed_by, metadata
        ) VALUES (
            NEW.id, 'status_change', OLD.status, NEW.status, NEW.transaction_hash,
            'Automatic status change', NULL,
            jsonb_build_object('old_record', to_jsonb(OLD), 'new_record', to_jsonb(NEW))
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER payment_status_audit
    AFTER UPDATE ON payments
    FOR EACH ROW
    EXECUTE FUNCTION log_payment_status_changes();

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE payments IS 'Core payment tracking for blockchain transactions';
COMMENT ON TABLE subscriptions IS 'Active subscription management with lifecycle tracking';
COMMENT ON TABLE wallet_credits IS 'User credit balances for payment system';
COMMENT ON TABLE credit_transactions IS 'Credit transaction history with full audit trail';

SELECT 'EPSX PAYMENTS CONSOLIDATED SCHEMA v4 CREATED SUCCESSFULLY! 🎉' AS success_message;
