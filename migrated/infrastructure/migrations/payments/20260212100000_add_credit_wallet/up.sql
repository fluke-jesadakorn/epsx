-- ============================================================================
-- CREDIT WALLET SYSTEM
-- ============================================================================
-- Migration: 20260212100000_add_credit_wallet
-- Description: Adds credit wallet system for storing and tracking user credits
--
-- Tables:
--   - wallet_credits: User credit balances
--   - credit_transactions: Credit transaction history
-- ============================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- ============================================================================
-- WALLET CREDITS TABLE
-- ============================================================================
-- Stores the current credit balance for each wallet
-- Separate from core.wallet_users for better separation of concerns

CREATE TABLE wallet_credits (
    wallet_address VARCHAR(42) PRIMARY KEY,
    balance NUMERIC(10,2) NOT NULL DEFAULT 0,
    pending_balance NUMERIC(10,2) NOT NULL DEFAULT 0,
    lifetime_earned NUMERIC(10,2) NOT NULL DEFAULT 0,
    lifetime_spent NUMERIC(10,2) NOT NULL DEFAULT 0,
    last_transaction_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_wallet_address CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$'
    ),
    CONSTRAINT non_negative_balance CHECK (balance >= 0),
    CONSTRAINT non_negative_pending CHECK (pending_balance >= 0),
    CONSTRAINT non_negative_lifetime_earned CHECK (lifetime_earned >= 0),
    CONSTRAINT non_negative_lifetime_spent CHECK (lifetime_spent >= 0)
);

CREATE INDEX idx_wallet_credits_balance ON wallet_credits(balance) WHERE balance > 0;
CREATE INDEX idx_wallet_credits_updated ON wallet_credits(updated_at DESC);

-- ============================================================================
-- CREDIT TRANSACTIONS TABLE
-- ============================================================================
-- Tracks all credit movements (grants, debits, refunds, etc.)

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
    ),
    CONSTRAINT valid_granted_by CHECK (
        granted_by IS NULL OR granted_by ~ '^0x[a-fA-F0-9]{40}$'
    ),
    CONSTRAINT valid_reference_type CHECK (
        reference_type IS NULL OR reference_type IN ('payment', 'subscription', 'refund', 'admin_action')
    )
);

CREATE INDEX idx_credit_tx_wallet ON credit_transactions(wallet_address);
CREATE INDEX idx_credit_tx_type ON credit_transactions(tx_type);
CREATE INDEX idx_credit_tx_created ON credit_transactions(created_at DESC);
CREATE INDEX idx_credit_tx_reference ON credit_transactions(reference_id) WHERE reference_id IS NOT NULL;
CREATE INDEX idx_credit_tx_expires ON credit_transactions(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_credit_tx_granted_by ON credit_transactions(granted_by) WHERE granted_by IS NOT NULL;

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Update wallet_credits.updated_at on any change
CREATE OR REPLACE FUNCTION update_wallet_credits_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER wallet_credits_updated_at
    BEFORE UPDATE ON wallet_credits
    FOR EACH ROW
    EXECUTE FUNCTION update_wallet_credits_timestamp();

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

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
    -- Get or create wallet_credits record
    INSERT INTO wallet_credits (wallet_address, balance)
    VALUES (p_wallet_address, 0)
    ON CONFLICT (wallet_address) DO NOTHING;

    -- Get current balance with row lock
    SELECT balance INTO v_current_balance
    FROM wallet_credits
    WHERE wallet_address = p_wallet_address
    FOR UPDATE;

    -- Calculate new balance
    v_new_balance := v_current_balance + p_amount;

    -- Ensure balance doesn't go negative
    IF v_new_balance < 0 THEN
        RAISE EXCEPTION 'Insufficient credit balance. Current: %, Requested: %', v_current_balance, p_amount;
    END IF;

    -- Insert transaction
    INSERT INTO credit_transactions (
        wallet_address, amount, balance_after, tx_type,
        reference_id, reference_type, reason, granted_by, expires_at, metadata
    ) VALUES (
        p_wallet_address, p_amount, v_new_balance, p_tx_type,
        p_reference_id, p_reference_type, p_reason, p_granted_by, p_expires_at, p_metadata
    ) RETURNING id INTO v_transaction_id;

    -- Update wallet balance and stats
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

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE wallet_credits IS 'User credit balances for payment system';
COMMENT ON TABLE credit_transactions IS 'Credit transaction history with full audit trail';
COMMENT ON FUNCTION add_credit_transaction IS 'Safely add credit transaction with balance update in single atomic operation';

SELECT 'CREDIT WALLET SYSTEM CREATED SUCCESSFULLY! 💰' AS success_message;
