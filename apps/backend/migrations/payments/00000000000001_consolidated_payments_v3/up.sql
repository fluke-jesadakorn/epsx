-- ============================================================================
-- EPSX PAYMENTS CONSOLIDATED SCHEMA v3
-- ============================================================================
-- Version: 3.0.0 (Consolidated February 2026)
-- Description: Complete payments schema for blockchain transactions.
--              Merges consolidated_payments_v2 + fix_missing_payments_tables.
--
-- Tables:
--   - payments: Core payment tracking
--   - subscriptions: Active subscription management  
--   - stock_ranking_assignments: Stock ranking package assignments
--   - payment_contexts: Dynamic payment links (V2)
--   - payment_audit_log: Payment event audit trail
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

CREATE INDEX idx_payments_wallet ON payments(wallet_address);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_plan ON payments(plan_id);
CREATE INDEX idx_payments_tx_hash ON payments(transaction_hash);
CREATE INDEX idx_payments_created ON payments(created_at DESC);
CREATE INDEX idx_payments_expires ON payments(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_payments_status_pending ON payments(status) WHERE status = 'pending';

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
CREATE INDEX idx_subs_plan ON subscriptions(plan_id);
CREATE INDEX idx_subs_status ON subscriptions(status);
CREATE INDEX idx_subs_payment ON subscriptions(payment_id);
CREATE INDEX idx_subs_expires ON subscriptions(expires_at);
CREATE INDEX idx_subs_active ON subscriptions(wallet_address, plan_id) WHERE status = 'active';

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
CREATE INDEX idx_stock_package ON stock_ranking_assignments(package_id);
CREATE INDEX idx_stock_active ON stock_ranking_assignments(is_active, expires_at);

-- ============================================================================
-- PAYMENT CONTEXTS TABLE (V2 Dynamic Payment System)
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
    CONSTRAINT valid_usage_counts CHECK (
        current_uses >= 0 AND
        (max_uses IS NULL OR max_uses > 0)
    ),
    CONSTRAINT valid_context_created_by CHECK (
        created_by ~ '^0x[a-fA-F0-9]{40}$'
    )
);

CREATE INDEX idx_payment_contexts_slug ON payment_contexts(slug);
CREATE INDEX idx_payment_contexts_type ON payment_contexts(context_type);
CREATE INDEX idx_payment_contexts_context_id ON payment_contexts(context_id) WHERE context_id IS NOT NULL;
CREATE INDEX idx_payment_contexts_active ON payment_contexts(is_active, expires_at) WHERE is_active = true;
CREATE INDEX idx_payment_contexts_created_by ON payment_contexts(created_by);
CREATE INDEX idx_payment_contexts_expires ON payment_contexts(expires_at) WHERE expires_at IS NOT NULL AND is_active = true;

-- ============================================================================
-- PAYMENT AUDIT LOG
-- ============================================================================

CREATE TABLE payment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL REFERENCES payments(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    old_status VARCHAR(20),
    new_status VARCHAR(20),
    reason TEXT,
    performed_by VARCHAR(42),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT valid_audit_performed_by CHECK (
        performed_by IS NULL OR performed_by ~ '^0x[a-fA-F0-9]{40}$'
    )
);

CREATE INDEX idx_payment_audit_payment ON payment_audit_log(payment_id);
CREATE INDEX idx_payment_audit_action ON payment_audit_log(action);
CREATE INDEX idx_payment_audit_created ON payment_audit_log(created_at DESC);

-- ============================================================================
-- TRIGGERS
-- ============================================================================

CREATE OR REPLACE FUNCTION update_payment_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER payment_updated_at
    BEFORE UPDATE ON payments
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_updated_at();

CREATE OR REPLACE FUNCTION update_payment_context_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    NEW.version = OLD.version + 1;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER payment_context_updated_at
    BEFORE UPDATE ON payment_contexts
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_context_updated_at();

-- Payment status change audit trigger
CREATE OR REPLACE FUNCTION log_payment_status_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        INSERT INTO payment_audit_log (
            payment_id, action, old_status, new_status, reason, performed_by, metadata
        ) VALUES (
            NEW.id, 'status_change', OLD.status, NEW.status,
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
COMMENT ON TABLE stock_ranking_assignments IS 'Stock ranking package assignments with access levels';
COMMENT ON TABLE payment_contexts IS 'Dynamic payment links for context-based purchases (V2 payment system)';
COMMENT ON TABLE payment_audit_log IS 'Audit trail of payment status changes';

SELECT 'EPSX PAYMENTS CONSOLIDATED SCHEMA v3 CREATED SUCCESSFULLY! 🎉' AS success_message;
