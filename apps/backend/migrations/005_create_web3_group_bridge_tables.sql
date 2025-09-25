-- ============================================================================
-- WEB3 GROUP BRIDGE MIGRATION - Web3 to Group Permission Integration
-- ============================================================================
-- This migration creates tables and infrastructure for bridging Web3 permissions
-- (NFT/token ownership) with the group permission system for automatic assignment
-- ============================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ============================================================================
-- 1. WEB3 GROUP RULES TABLE - Rules for automatic group assignment
-- ============================================================================

CREATE TABLE web3_group_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    
    -- Rule Configuration
    rule_name VARCHAR(255) NOT NULL DEFAULT 'Web3 Asset Rule',
    rule_type VARCHAR(50) NOT NULL, -- 'nft_ownership', 'token_balance', 'dao_membership', 'custom'
    rule_description TEXT,
    
    -- Web3 Asset Configuration
    contract_address VARCHAR(42) NOT NULL,
    network VARCHAR(50) NOT NULL DEFAULT 'bsc', -- 'bsc', 'ethereum', 'polygon', etc.
    
    -- Token-specific fields
    minimum_balance DECIMAL(78,0), -- Support large numbers with precision
    token_decimals INTEGER DEFAULT 18,
    token_symbol VARCHAR(20),
    
    -- NFT-specific fields
    specific_token_ids TEXT[] DEFAULT '{}',
    require_specific_tokens BOOLEAN DEFAULT FALSE,
    minimum_nft_count INTEGER DEFAULT 1,
    
    -- DAO-specific fields
    dao_proposal_id VARCHAR(100),
    required_voting_power DECIMAL(78,0),
    
    -- Assignment Configuration
    is_active BOOLEAN DEFAULT TRUE,
    auto_assignment BOOLEAN DEFAULT TRUE,
    assignment_duration_days INTEGER, -- NULL = permanent
    max_assignments INTEGER, -- NULL = unlimited
    current_assignments INTEGER DEFAULT 0,
    
    -- Assignment Conditions
    assignment_conditions JSONB DEFAULT '{}', -- Additional conditions
    verification_frequency VARCHAR(20) DEFAULT 'daily', -- 'realtime', 'hourly', 'daily', 'manual'
    
    -- Caching and Performance
    last_verification_at TIMESTAMPTZ,
    verification_cache_minutes INTEGER DEFAULT 60,
    
    -- Rule Metadata
    priority_order INTEGER DEFAULT 0,
    tags TEXT[] DEFAULT '{}',
    configuration JSONB DEFAULT '{}', -- Additional configuration
    
    -- Rule Statistics
    total_evaluations INTEGER DEFAULT 0,
    successful_assignments INTEGER DEFAULT 0,
    failed_evaluations INTEGER DEFAULT 0,
    success_rate FLOAT DEFAULT 0.0,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_rule_type CHECK (rule_type IN ('nft_ownership', 'token_balance', 'dao_membership', 'custom')),
    CONSTRAINT valid_network CHECK (network IN ('bsc', 'ethereum', 'polygon', 'arbitrum', 'optimism', 'base')),
    CONSTRAINT valid_contract_address CHECK (contract_address ~ '^0x[a-fA-F0-9]{40}$')
);

-- ============================================================================
-- 2. WEB3 GROUP EVALUATIONS TABLE - Track rule evaluations and results
-- ============================================================================

CREATE TABLE web3_group_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES web3_group_rules(id) ON DELETE CASCADE,
    user_id UUID, -- NULL if user doesn't exist yet
    wallet_address VARCHAR(42) NOT NULL,
    
    -- Evaluation Results
    evaluation_result BOOLEAN NOT NULL, -- Did the rule match?
    assignment_made BOOLEAN DEFAULT FALSE, -- Was user actually assigned to group?
    assignment_skipped_reason TEXT, -- Why assignment was skipped (if applicable)
    
    -- Verification Data
    verification_data JSONB DEFAULT '{}', -- Data used for verification
    verification_method VARCHAR(50), -- How verification was performed
    verification_time_ms INTEGER, -- Time taken for verification
    
    -- Asset Data (at time of evaluation)
    asset_balance DECIMAL(78,0), -- Token balance or NFT count
    asset_metadata JSONB DEFAULT '{}', -- Additional asset information
    
    -- Network Data
    network_block_number BIGINT, -- Block number at verification time
    network_timestamp TIMESTAMPTZ, -- Network timestamp
    transaction_hash VARCHAR(66), -- If related to a specific transaction
    
    -- Caching
    cached_result BOOLEAN DEFAULT FALSE,
    cache_expires_at TIMESTAMPTZ,
    
    -- Timestamps
    evaluated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- 3. WEB3 ASSIGNMENT QUEUE TABLE - Queue for processing assignments
-- ============================================================================

CREATE TABLE web3_assignment_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Queue Item Configuration
    wallet_address VARCHAR(42) NOT NULL,
    rule_ids UUID[] NOT NULL DEFAULT '{}', -- Specific rules to evaluate, empty = all rules
    priority INTEGER DEFAULT 0, -- Higher = processed first
    
    -- Queue Status
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed', 'cancelled'
    processing_started_at TIMESTAMPTZ,
    processing_completed_at TIMESTAMPTZ,
    
    -- Processing Results
    groups_assigned INTEGER DEFAULT 0,
    groups_removed INTEGER DEFAULT 0,
    verification_errors INTEGER DEFAULT 0,
    processing_time_ms INTEGER,
    
    -- Error Handling
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    
    -- Assignment Metadata
    trigger_source VARCHAR(50) DEFAULT 'manual', -- 'manual', 'webhook', 'scheduled', 'user_action'
    trigger_metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_status CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')),
    CONSTRAINT valid_priority CHECK (priority >= 0),
    CONSTRAINT valid_retry_count CHECK (retry_count <= max_retries)
);

-- ============================================================================
-- 4. WEB3 NETWORK CACHE TABLE - Cache Web3 network data for performance
-- ============================================================================

CREATE TABLE web3_network_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Cache Key Components
    cache_key VARCHAR(255) NOT NULL UNIQUE, -- Composite key: wallet:contract:network:method
    wallet_address VARCHAR(42) NOT NULL,
    contract_address VARCHAR(42) NOT NULL,
    network VARCHAR(50) NOT NULL,
    method_type VARCHAR(50) NOT NULL, -- 'balance', 'ownership', 'dao_voting'
    
    -- Cached Data
    cached_result JSONB NOT NULL,
    cached_value DECIMAL(78,0), -- Primary numeric result (balance, count, etc.)
    cached_boolean BOOLEAN, -- Primary boolean result
    
    -- Network State
    block_number BIGINT NOT NULL,
    block_timestamp TIMESTAMPTZ NOT NULL,
    
    -- Cache Metadata
    cache_hit_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Expiry
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- 5. EXTEND EXISTING TABLES - Add Web3 bridge support
-- ============================================================================

-- Add Web3 bridge tracking to user_group_memberships
ALTER TABLE user_group_memberships 
ADD COLUMN IF NOT EXISTS web3_rule_id UUID REFERENCES web3_group_rules(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS web3_last_verified_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS web3_verification_status VARCHAR(20) DEFAULT 'pending';

-- Add Web3 bridge tracking to group_assignment_history
ALTER TABLE group_assignment_history
ADD COLUMN IF NOT EXISTS web3_rule_id UUID,
ADD COLUMN IF NOT EXISTS web3_verification_data JSONB DEFAULT '{}';

-- ============================================================================
-- 6. INDEXES FOR PERFORMANCE
-- ============================================================================

-- Web3 Group Rules indexes
CREATE INDEX idx_web3_group_rules_group_id ON web3_group_rules(group_id);
CREATE INDEX idx_web3_group_rules_active ON web3_group_rules(is_active, auto_assignment);
CREATE INDEX idx_web3_group_rules_contract ON web3_group_rules(contract_address, network);
CREATE INDEX idx_web3_group_rules_priority ON web3_group_rules(priority_order DESC) WHERE is_active = TRUE;

-- Web3 Group Evaluations indexes
CREATE INDEX idx_web3_evaluations_rule_wallet ON web3_group_evaluations(rule_id, wallet_address);
CREATE INDEX idx_web3_evaluations_result ON web3_group_evaluations(evaluation_result, assignment_made);
CREATE INDEX idx_web3_evaluations_time ON web3_group_evaluations(evaluated_at DESC);

-- Web3 Assignment Queue indexes
CREATE INDEX idx_web3_queue_status ON web3_assignment_queue(status, priority DESC);
CREATE INDEX idx_web3_queue_wallet ON web3_assignment_queue(wallet_address);
CREATE INDEX idx_web3_queue_created ON web3_assignment_queue(created_at DESC);

-- Web3 Network Cache indexes (already defined above)

-- Enhanced permission_groups indexes
-- Web3 Network Cache indexes
CREATE INDEX idx_web3_cache_key ON web3_network_cache(cache_key);
CREATE INDEX idx_web3_cache_wallet_contract ON web3_network_cache(wallet_address, contract_address, network);
CREATE INDEX idx_web3_cache_expires ON web3_network_cache(expires_at) WHERE expires_at > NOW();

-- Enhanced permission_groups indexes
CREATE INDEX IF NOT EXISTS idx_permission_groups_web3_managed ON permission_groups(is_web3_managed) WHERE is_web3_managed = TRUE;

-- Enhanced user_group_memberships indexes
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_web3_rule ON user_group_memberships(web3_rule_id) WHERE web3_rule_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_web3_status ON user_group_memberships(web3_verification_status, web3_last_verified_at);

-- ============================================================================
-- 7. FUNCTIONS FOR WEB3 GROUP OPERATIONS
-- ============================================================================

-- Function to clean up expired cache entries
CREATE OR REPLACE FUNCTION cleanup_web3_cache()
RETURNS INTEGER AS $$
DECLARE
    cleaned_count INTEGER;
BEGIN
    DELETE FROM web3_network_cache WHERE expires_at <= NOW();
    GET DIAGNOSTICS cleaned_count = ROW_COUNT;
    
    RAISE NOTICE 'Cleaned up % expired Web3 cache entries', cleaned_count;
    RETURN cleaned_count;
END;
$$ LANGUAGE plpgsql;

-- Function to update rule statistics
CREATE OR REPLACE FUNCTION update_web3_rule_stats(rule_uuid UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE web3_group_rules
    SET 
        total_evaluations = (
            SELECT COUNT(*) FROM web3_group_evaluations WHERE rule_id = rule_uuid
        ),
        successful_assignments = (
            SELECT COUNT(*) FROM web3_group_evaluations 
            WHERE rule_id = rule_uuid AND assignment_made = TRUE
        ),
        failed_evaluations = (
            SELECT COUNT(*) FROM web3_group_evaluations 
            WHERE rule_id = rule_uuid AND evaluation_result = FALSE
        ),
        success_rate = (
            SELECT 
                CASE 
                    WHEN COUNT(*) = 0 THEN 0.0
                    ELSE ROUND(
                        (COUNT(*) FILTER (WHERE assignment_made = TRUE))::FLOAT / COUNT(*)::FLOAT, 
                        3
                    )
                END
            FROM web3_group_evaluations 
            WHERE rule_id = rule_uuid
        ),
        updated_at = NOW()
    WHERE id = rule_uuid;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 8. TRIGGERS FOR AUTOMATIC UPDATES
-- ============================================================================

-- Trigger to update rule statistics when evaluations change
CREATE OR REPLACE FUNCTION trigger_update_web3_rule_stats()
RETURNS TRIGGER AS $$
BEGIN
    -- Update stats for the affected rule
    PERFORM update_web3_rule_stats(COALESCE(NEW.rule_id, OLD.rule_id));
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_web3_evaluation_stats
    AFTER INSERT OR UPDATE OR DELETE ON web3_group_evaluations
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_web3_rule_stats();

-- Trigger to update cache access statistics
CREATE OR REPLACE FUNCTION trigger_update_cache_access()
RETURNS TRIGGER AS $$
BEGIN
    NEW.cache_hit_count = OLD.cache_hit_count + 1;
    NEW.last_accessed_at = NOW();
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_web3_cache_access
    BEFORE UPDATE ON web3_network_cache
    FOR EACH ROW
    WHEN (OLD.cached_result IS NOT DISTINCT FROM NEW.cached_result)
    EXECUTE FUNCTION trigger_update_cache_access();

-- ============================================================================
-- 9. INITIAL DATA - Default Web3 Group Rules
-- ============================================================================

-- Insert default Web3-managed groups if they don't exist
INSERT INTO permission_groups (
    name, slug, description, permissions, 
    is_system_group, is_web3_managed, priority_level
) VALUES 
(
    'BSC CAKE Holders',
    'bsc-cake-holders',
    'Users holding CAKE tokens on BSC network',
    ARRAY['epsx:defi:basic', 'epsx:trading:pancakeswap'],
    TRUE, TRUE, 5
),
(
    'BSC NFT Collectors',
    'bsc-nft-collectors',
    'Users holding specific NFTs on BSC network',
    ARRAY['epsx:nft:view', 'epsx:collectibles:access'],
    TRUE, TRUE, 4
),
(
    'High Value BSC Holders',
    'high-value-bsc-holders',
    'Users with significant BNB or token holdings',
    ARRAY['epsx:premium:access', 'epsx:analytics:advanced'],
    TRUE, TRUE, 8
)
ON CONFLICT (slug) DO NOTHING;

-- Insert default Web3 group rules
DO $$
DECLARE
    cake_group_id UUID;
    nft_group_id UUID;
    high_value_group_id UUID;
BEGIN
    -- Get group IDs
    SELECT id INTO cake_group_id FROM permission_groups WHERE slug = 'bsc-cake-holders';
    SELECT id INTO nft_group_id FROM permission_groups WHERE slug = 'bsc-nft-collectors';
    SELECT id INTO high_value_group_id FROM permission_groups WHERE slug = 'high-value-bsc-holders';
    
    -- Insert CAKE holder rule
    IF cake_group_id IS NOT NULL THEN
        INSERT INTO web3_group_rules (
            group_id, rule_name, rule_type, rule_description,
            contract_address, network, minimum_balance, token_decimals, token_symbol,
            is_active, auto_assignment, assignment_duration_days,
            verification_frequency, priority_order
        ) VALUES (
            cake_group_id,
            'CAKE Token Holder (100+ CAKE)',
            'token_balance',
            'Users holding at least 100 CAKE tokens',
            '0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82', -- CAKE token contract
            'bsc',
            100000000000000000000, -- 100 CAKE with 18 decimals
            18,
            'CAKE',
            TRUE, TRUE, NULL, -- Permanent assignment
            'daily', 1
        ) ON CONFLICT DO NOTHING;
    END IF;
    
    -- Insert high-value BNB rule
    IF high_value_group_id IS NOT NULL THEN
        INSERT INTO web3_group_rules (
            group_id, rule_name, rule_type, rule_description,
            contract_address, network, minimum_balance, token_decimals,
            is_active, auto_assignment, assignment_duration_days,
            verification_frequency, priority_order
        ) VALUES (
            high_value_group_id,
            'High-Value BNB Holder (10+ BNB)',
            'token_balance',
            'Users holding at least 10 BNB',
            '0x0000000000000000000000000000000000000000', -- Native BNB
            'bsc',
            10000000000000000000, -- 10 BNB with 18 decimals
            18,
            TRUE, TRUE, 30, -- 30-day assignments
            'daily', 10
        ) ON CONFLICT DO NOTHING;
    END IF;
END $$;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE 'Web3 Group Bridge migration completed successfully!';
    RAISE NOTICE 'Created: 4 new tables, 15 indexes, 2 functions, 2 triggers';
    RAISE NOTICE 'Added: Web3 bridge support to existing tables';
    RAISE NOTICE 'System ready for Web3-to-Group automatic assignment!';
END $$;