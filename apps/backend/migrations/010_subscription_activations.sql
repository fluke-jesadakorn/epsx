-- Migration: Add subscription activation tracking table
-- This table tracks payment confirmations and subscription activations

-- User Subscription Activations table
CREATE TABLE user_subscription_activations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id INTEGER NOT NULL REFERENCES pricing_plans(id),
    payment_id VARCHAR NOT NULL, -- Payment aggregate ID
    transaction_hash VARCHAR NOT NULL, -- Blockchain transaction hash
    activated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ, -- When the subscription expires
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure unique activation per payment
    UNIQUE(payment_id, transaction_hash),
    -- Ensure we don't duplicate activations for the same user/plan/transaction
    UNIQUE(user_id, transaction_hash)
);

-- Indexes for performance
CREATE INDEX idx_user_subscription_activations_user_id ON user_subscription_activations(user_id);
CREATE INDEX idx_user_subscription_activations_plan_id ON user_subscription_activations(plan_id);
CREATE INDEX idx_user_subscription_activations_payment_id ON user_subscription_activations(payment_id);
CREATE INDEX idx_user_subscription_activations_transaction_hash ON user_subscription_activations(transaction_hash);
CREATE INDEX idx_user_subscription_activations_activated_at ON user_subscription_activations(activated_at);
CREATE INDEX idx_user_subscription_activations_expires_at ON user_subscription_activations(expires_at) WHERE expires_at IS NOT NULL;

-- Add updated_at trigger
CREATE TRIGGER update_user_subscription_activations_updated_at 
    BEFORE UPDATE ON user_subscription_activations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();