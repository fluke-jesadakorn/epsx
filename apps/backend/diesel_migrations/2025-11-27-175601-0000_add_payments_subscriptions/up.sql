-- Add comprehensive payments and subscriptions system
-- This migration creates tables for payment processing, subscription management,
-- and audit logging to support the EPSX payment validation system.

-- Payments table: Core payment tracking and validation
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Payment identification
    payment_reference VARCHAR(255) UNIQUE NOT NULL,
    transaction_hash VARCHAR(66) UNIQUE, -- Blockchain transaction hash

    -- Payment details
    wallet_address VARCHAR(42) NOT NULL,
    amount DECIMAL(20, 6) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    method VARCHAR(50) NOT NULL DEFAULT 'crypto',

    -- Status tracking
    status VARCHAR(20) NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'pending', 'confirmed', 'failed', 'refunded', 'expired')),

    -- Plan association
    plan_id UUID NOT NULL REFERENCES permission_groups(id),

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

-- Subscriptions table: Active subscription management
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Subscription identification
    wallet_address VARCHAR(42) NOT NULL,
    plan_id UUID NOT NULL REFERENCES permission_groups(id),
    payment_id UUID NOT NULL REFERENCES payments(id),

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

-- Payment audit log: Comprehensive audit trail for compliance
CREATE TABLE payment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Payment association
    payment_id UUID NOT NULL REFERENCES payments(id),

    -- Action details
    action VARCHAR(50) NOT NULL,
    old_status VARCHAR(20),
    new_status VARCHAR(20) NOT NULL,
    reason TEXT,

    -- Who performed the action
    performed_by VARCHAR(42), -- Wallet address of administrator/system
    performed_at TIMESTAMPTZ DEFAULT NOW(),

    -- Additional context
    metadata JSONB DEFAULT '{}',

    -- Constraints
    CONSTRAINT valid_performed_by CHECK (
        performed_by IS NULL OR
        performed_by ~ '^0x[a-fA-F0-9]{40}$'
    )
);

-- Indexes for performance optimization

-- Payments table indexes
CREATE INDEX idx_payments_wallet_address ON payments(wallet_address);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_plan_id ON payments(plan_id);
CREATE INDEX idx_payments_transaction_hash ON payments(transaction_hash);
CREATE INDEX idx_payments_created_at ON payments(created_at);
CREATE INDEX idx_payments_expires_at ON payments(expires_at) WHERE expires_at IS NOT NULL;

-- Subscriptions table indexes
CREATE INDEX idx_subscriptions_wallet_address ON subscriptions(wallet_address);
CREATE INDEX idx_subscriptions_plan_id ON subscriptions(plan_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_payment_id ON subscriptions(payment_id);
CREATE INDEX idx_subscriptions_expires_at ON subscriptions(expires_at);
CREATE INDEX idx_subscriptions_active ON subscriptions(wallet_address, plan_id)
    WHERE status = 'active';

-- Audit log indexes
CREATE INDEX idx_payment_audit_log_payment_id ON payment_audit_log(payment_id);
CREATE INDEX idx_payment_audit_log_action ON payment_audit_log(action);
CREATE INDEX idx_payment_audit_log_performed_at ON payment_audit_log(performed_at);
CREATE INDEX idx_payment_audit_log_performed_by ON payment_audit_log(performed_by);

-- Trigger functions for automated timestamp updates

-- Update updated_at timestamp for payments
CREATE OR REPLACE FUNCTION update_payment_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update payment status changes in audit log
CREATE OR REPLACE FUNCTION log_payment_status_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        INSERT INTO payment_audit_log (
            payment_id,
            action,
            old_status,
            new_status,
            reason,
            performed_by,
            metadata
        ) VALUES (
            NEW.id,
            'status_change',
            OLD.status,
            NEW.status,
            'Automatic status change',
            'system',
            jsonb_build_object(
                'old_record', to_jsonb(OLD),
                'new_record', to_jsonb(NEW)
            )
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers
CREATE TRIGGER payment_updated_at
    BEFORE UPDATE ON payments
    FOR EACH ROW
    EXECUTE FUNCTION update_payment_updated_at();

CREATE TRIGGER payment_audit_trigger
    AFTER UPDATE ON payments
    FOR EACH ROW
    WHEN (OLD.status IS DISTINCT FROM NEW.status)
    EXECUTE FUNCTION log_payment_status_changes();

-- Row Level Security (RLS) for payment data protection
ALTER TABLE payments ENABLE ROW LEVEL SECURITY;
ALTER TABLE subscriptions ENABLE ROW LEVEL SECURITY;
ALTER TABLE payment_audit_log ENABLE ROW LEVEL SECURITY;

-- RLS Policies

-- Users can only see their own payments and subscriptions
CREATE POLICY users_view_own_payments ON payments
    FOR SELECT USING (
        wallet_address = current_setting('app.current_wallet_address', true) OR
        current_setting('app.is_admin', true)::boolean
    );

CREATE POLICY users_view_own_subscriptions ON subscriptions
    FOR SELECT USING (
        wallet_address = current_setting('app.current_wallet_address', true) OR
        current_setting('app.is_admin', true)::boolean
    );

-- Admin policies for full access
CREATE POLICY admin_full_access_payments ON payments
    FOR ALL USING (current_setting('app.is_admin', true)::boolean);

CREATE POLICY admin_full_access_subscriptions ON subscriptions
    FOR ALL USING (current_setting('app.is_admin', true)::boolean);

CREATE POLICY admin_full_access_audit_log ON payment_audit_log
    FOR ALL USING (current_setting('app.is_admin', true)::boolean);

-- Grant permissions
GRANT SELECT, INSERT, UPDATE ON payments TO epsx_user;
GRANT SELECT, INSERT, UPDATE ON subscriptions TO epsx_user;
GRANT SELECT, INSERT ON payment_audit_log TO epsx_user;

GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO epsx_user;

-- Add comments for documentation
COMMENT ON TABLE payments IS 'Core payment tracking table for all payment transactions and validation status';
COMMENT ON TABLE subscriptions IS 'Active subscription management with lifecycle tracking';
COMMENT ON TABLE payment_audit_log IS 'Comprehensive audit trail for all payment status changes and actions';

COMMENT ON COLUMN payments.payment_reference IS 'Unique payment reference for tracking and support';
COMMENT ON COLUMN payments.transaction_hash IS 'Blockchain transaction hash for cryptocurrency payments';
COMMENT ON COLUMN payments.status IS 'Current payment status: created, pending, confirmed, failed, refunded, expired';
COMMENT ON COLUMN payments.metadata IS 'Additional payment data in JSON format (gas fees, exchange rates, etc.)';

COMMENT ON COLUMN subscriptions.status IS 'Subscription status: active, expired, cancelled, suspended';
COMMENT ON COLUMN subscriptions.auto_renew IS 'Whether subscription should automatically renew';
COMMENT ON COLUMN subscriptions.metadata IS 'Subscription metadata (features, limits, usage stats)';

COMMENT ON COLUMN payment_audit_log.action IS 'Action performed: status_change, refund, manual_review, etc.';
COMMENT ON COLUMN payment_audit_log.performed_by IS 'Wallet address of who performed the action (system or admin)';
COMMENT ON COLUMN payment_audit_log.metadata IS 'Additional context about the action performed';