-- ============================================================================
-- PAYMENT CONTEXTS TABLE (V2 Dynamic Payment System)
-- ============================================================================
-- Dynamic payment links supporting context-based payments
-- Enables users to unlock granular features by paying through dynamic links
-- 
-- Context Types:
--   0 = PLAN (subscription plan payment)
--   1 = GROUP (permission group payment)
--   2 = PRODUCT (one-time product purchase)
--   3 = CAMPAIGN (promotional campaign payment)
--   4 = CUSTOM (custom payment link)
-- ============================================================================

CREATE TABLE payment_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Context identification
    context_type VARCHAR(20) NOT NULL
        CHECK (context_type IN ('plan', 'group', 'product', 'campaign', 'custom')),
    context_id UUID,  -- UUID of the linked entity (plan, group, etc.)
    slug VARCHAR(100) UNIQUE NOT NULL,  -- URL-friendly identifier
    
    -- Display information
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Payment details
    amount DECIMAL(20, 8) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USDT',
    
    -- Expiration and usage limits
    expires_at TIMESTAMPTZ,  -- Default: 24 hours from creation
    max_uses INTEGER,  -- NULL = unlimited (multi-use)
    current_uses INTEGER NOT NULL DEFAULT 0,
    
    -- State
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Audit fields
    created_by VARCHAR(42) NOT NULL,  -- Wallet address of creator
    metadata JSONB NOT NULL DEFAULT '{}',
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT positive_amount CHECK (amount > 0),
    CONSTRAINT valid_usage_counts CHECK (
        current_uses >= 0 AND
        (max_uses IS NULL OR max_uses > 0)
    ),
    CONSTRAINT valid_created_by CHECK (
        created_by ~ '^0x[a-fA-F0-9]{40}$'
    )
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Primary lookups
CREATE INDEX idx_payment_contexts_slug ON payment_contexts(slug);
CREATE INDEX idx_payment_contexts_type ON payment_contexts(context_type);
CREATE INDEX idx_payment_contexts_context_id ON payment_contexts(context_id) WHERE context_id IS NOT NULL;

-- Active payment links
CREATE INDEX idx_payment_contexts_active ON payment_contexts(is_active, expires_at)
    WHERE is_active = true;

-- Creator management
CREATE INDEX idx_payment_contexts_created_by ON payment_contexts(created_by);

-- Expiration cleanup
CREATE INDEX idx_payment_contexts_expires ON payment_contexts(expires_at)
    WHERE expires_at IS NOT NULL AND is_active = true;

-- ============================================================================
-- TRIGGER FOR updated_at
-- ============================================================================

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

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE payment_contexts IS 'Dynamic payment links for context-based purchases (V2 payment system)';
COMMENT ON COLUMN payment_contexts.context_type IS 'Type: plan, group, product, campaign, custom';
COMMENT ON COLUMN payment_contexts.context_id IS 'UUID of linked entity (plan_id, group_id, etc.)';
COMMENT ON COLUMN payment_contexts.slug IS 'URL-friendly identifier for payment link';
COMMENT ON COLUMN payment_contexts.max_uses IS 'NULL = unlimited (multi-use), number = max allowed uses';
COMMENT ON COLUMN payment_contexts.expires_at IS 'Default: 24 hours from creation';
