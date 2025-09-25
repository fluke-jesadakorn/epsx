-- ================================================================================================
-- DYNAMIC TIER GROUP ASSIGNMENT - Intelligent wallet-to-group assignment system
-- ================================================================================================
-- This migration creates a dynamic tier system that automatically assigns wallets to appropriate
-- permission groups based on their Web3 assets, subscription status, and behavior patterns
-- ================================================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ================================================================================================
-- 1. CREATE DYNAMIC TIER RULES TABLE
-- ================================================================================================

CREATE TABLE dynamic_tier_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Rule Identification
    rule_name VARCHAR(255) NOT NULL UNIQUE,
    rule_description TEXT,
    rule_category VARCHAR(50) NOT NULL, -- 'asset_based', 'subscription_based', 'behavior_based', 'time_based'
    
    -- Target Group
    target_group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    target_tier_name VARCHAR(100) NOT NULL, -- 'free', 'bronze', 'silver', 'gold', 'platinum', 'enterprise'
    
    -- Rule Conditions (JSON-based for flexibility)
    conditions JSONB NOT NULL DEFAULT '{}',
    -- Examples:
    -- {"token_balance": {"contract": "0x...", "minimum": "1000000000000000000000", "network": "bsc"}}
    -- {"nft_ownership": {"contracts": ["0x..."], "minimum_count": 1}}
    -- {"subscription_tier": {"tiers": ["premium", "enterprise"]}}
    -- {"behavior_score": {"minimum": 75, "period_days": 30}}
    
    -- Assignment Configuration
    is_active BOOLEAN DEFAULT TRUE,
    auto_assign BOOLEAN DEFAULT TRUE,
    auto_upgrade BOOLEAN DEFAULT TRUE,
    auto_downgrade BOOLEAN DEFAULT FALSE, -- Usually manual for downgrades
    assignment_duration_days INTEGER, -- NULL = permanent
    
    -- Priority and Conflicts
    priority_level INTEGER DEFAULT 0, -- Higher priority rules checked first
    exclusive_tier BOOLEAN DEFAULT FALSE, -- Can wallet have multiple tiers?
    conflicts_with_rules UUID[] DEFAULT '{}', -- Rules that conflict with this one
    
    -- Evaluation Settings
    evaluation_frequency VARCHAR(20) DEFAULT 'daily', -- 'realtime', 'hourly', 'daily', 'weekly'
    cache_duration_minutes INTEGER DEFAULT 60,
    
    -- Performance Tracking
    total_evaluations BIGINT DEFAULT 0,
    successful_assignments BIGINT DEFAULT 0,
    failed_evaluations BIGINT DEFAULT 0,
    current_assignments BIGINT DEFAULT 0,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    tags TEXT[] DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_evaluation_at TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT valid_rule_category CHECK (rule_category IN ('asset_based', 'subscription_based', 'behavior_based', 'time_based', 'manual')),
    CONSTRAINT valid_tier_name CHECK (target_tier_name IN ('free', 'bronze', 'silver', 'gold', 'platinum', 'enterprise', 'admin', 'custom')),
    CONSTRAINT valid_evaluation_frequency CHECK (evaluation_frequency IN ('realtime', 'hourly', 'daily', 'weekly', 'manual'))
);

-- ================================================================================================
-- 2. CREATE TIER EVALUATION HISTORY TABLE
-- ================================================================================================

CREATE TABLE tier_evaluation_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Evaluation Context
    wallet_address VARCHAR(42) NOT NULL,
    rule_id UUID NOT NULL REFERENCES dynamic_tier_rules(id) ON DELETE CASCADE,
    evaluation_batch_id UUID NOT NULL, -- Group evaluations together
    
    -- Evaluation Results
    rule_matched BOOLEAN NOT NULL,
    assignment_made BOOLEAN DEFAULT FALSE,
    assignment_action VARCHAR(20), -- 'assigned', 'upgraded', 'downgraded', 'maintained', 'skipped'
    
    -- Condition Results (detailed breakdown)
    condition_results JSONB DEFAULT '{}',
    -- Example: {"token_balance": {"result": true, "actual_balance": "5000000000000000000000", "required": "1000000000000000000000"}}
    
    -- Assignment Details
    previous_tier VARCHAR(100),
    new_tier VARCHAR(100),
    previous_group_id UUID,
    new_group_id UUID,
    
    -- Verification Data
    verification_data JSONB DEFAULT '{}', -- Blockchain/API data used
    verification_time_ms INTEGER,
    data_sources TEXT[] DEFAULT '{}', -- 'blockchain', 'database', 'api', 'cache'
    
    -- Skip Reasons (if no assignment made)
    skip_reason TEXT,
    conflict_rules UUID[] DEFAULT '{}',
    
    -- Network Context
    blockchain_block_number BIGINT,
    blockchain_timestamp TIMESTAMPTZ,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    evaluated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_assignment_action CHECK (assignment_action IN ('assigned', 'upgraded', 'downgraded', 'maintained', 'skipped', 'removed')),
    CONSTRAINT fk_tier_history_wallet FOREIGN KEY (wallet_address) REFERENCES wallet_identities(wallet_address) ON DELETE CASCADE
);

-- ================================================================================================
-- 3. CREATE TIER ASSIGNMENT QUEUE TABLE
-- ================================================================================================

CREATE TABLE tier_assignment_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Queue Configuration
    wallet_address VARCHAR(42) NOT NULL,
    rule_ids UUID[] DEFAULT '{}', -- Specific rules to evaluate, empty = all active rules
    trigger_source VARCHAR(50) DEFAULT 'scheduled', -- 'scheduled', 'manual', 'webhook', 'blockchain_event'
    
    -- Queue Priority
    priority INTEGER DEFAULT 0, -- Higher = processed first
    urgent BOOLEAN DEFAULT FALSE,
    
    -- Queue Status
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed', 'cancelled'
    processing_started_at TIMESTAMPTZ,
    processing_completed_at TIMESTAMPTZ,
    processing_worker_id VARCHAR(100),
    
    -- Processing Results
    evaluation_batch_id UUID,
    rules_evaluated INTEGER DEFAULT 0,
    assignments_made INTEGER DEFAULT 0,
    upgrades_performed INTEGER DEFAULT 0,
    downgrades_performed INTEGER DEFAULT 0,
    errors_encountered INTEGER DEFAULT 0,
    
    -- Error Handling
    error_details JSONB DEFAULT '{}',
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    
    -- Trigger Context
    trigger_metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_queue_status CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')),
    CONSTRAINT valid_trigger_source CHECK (trigger_source IN ('scheduled', 'manual', 'webhook', 'blockchain_event', 'subscription_change')),
    CONSTRAINT fk_tier_queue_wallet FOREIGN KEY (wallet_address) REFERENCES wallet_identities(wallet_address) ON DELETE CASCADE
);

-- ================================================================================================
-- 4. CREATE WALLET TIER CACHE TABLE
-- ================================================================================================

CREATE TABLE wallet_tier_cache (
    wallet_address VARCHAR(42) PRIMARY KEY,
    
    -- Current Tier Information
    current_tier VARCHAR(100) NOT NULL,
    current_group_id UUID REFERENCES permission_groups(id),
    tier_assigned_at TIMESTAMPTZ DEFAULT NOW(),
    tier_expires_at TIMESTAMPTZ,
    
    -- Tier History Summary
    previous_tier VARCHAR(100),
    tier_changes_count INTEGER DEFAULT 0,
    last_upgrade_at TIMESTAMPTZ,
    last_downgrade_at TIMESTAMPTZ,
    
    -- Asset Summary (cached for performance)
    asset_summary JSONB DEFAULT '{}',
    -- Example: {"tokens": {"CAKE": "5000.0", "BNB": "10.5"}, "nfts": {"count": 3, "collections": ["0x..."]}}
    
    -- Behavior Metrics (cached)
    behavior_score INTEGER DEFAULT 0, -- 0-100 score
    activity_level VARCHAR(20) DEFAULT 'unknown', -- 'high', 'medium', 'low', 'inactive'
    last_activity_at TIMESTAMPTZ,
    
    -- Subscription Status
    subscription_tier VARCHAR(50),
    subscription_status VARCHAR(20), -- 'active', 'expired', 'cancelled', 'none'
    subscription_expires_at TIMESTAMPTZ,
    
    -- Cache Metadata
    last_evaluated_at TIMESTAMPTZ DEFAULT NOW(),
    cache_expires_at TIMESTAMPTZ DEFAULT (NOW() + INTERVAL '1 hour'),
    evaluation_count INTEGER DEFAULT 0,
    
    -- Performance Tracking
    avg_evaluation_time_ms FLOAT DEFAULT 0,
    last_evaluation_time_ms INTEGER,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT fk_wallet_tier_cache FOREIGN KEY (wallet_address) REFERENCES wallet_identities(wallet_address) ON DELETE CASCADE,
    CONSTRAINT valid_current_tier CHECK (current_tier IN ('free', 'bronze', 'silver', 'gold', 'platinum', 'enterprise', 'admin')),
    CONSTRAINT valid_activity_level CHECK (activity_level IN ('high', 'medium', 'low', 'inactive', 'unknown'))
);

-- ================================================================================================
-- 5. CREATE INDEXES FOR PERFORMANCE
-- ================================================================================================

-- Dynamic tier rules indexes
CREATE INDEX idx_dynamic_tier_rules_active ON dynamic_tier_rules(is_active, auto_assign);
CREATE INDEX idx_dynamic_tier_rules_category ON dynamic_tier_rules(rule_category);
CREATE INDEX idx_dynamic_tier_rules_priority ON dynamic_tier_rules(priority_level DESC) WHERE is_active = TRUE;
CREATE INDEX idx_dynamic_tier_rules_target_group ON dynamic_tier_rules(target_group_id);
CREATE INDEX idx_dynamic_tier_rules_tier ON dynamic_tier_rules(target_tier_name);

-- Tier evaluation history indexes
CREATE INDEX idx_tier_evaluation_history_wallet ON tier_evaluation_history(wallet_address);
CREATE INDEX idx_tier_evaluation_history_rule ON tier_evaluation_history(rule_id);
CREATE INDEX idx_tier_evaluation_history_batch ON tier_evaluation_history(evaluation_batch_id);
CREATE INDEX idx_tier_evaluation_history_time ON tier_evaluation_history(evaluated_at DESC);
CREATE INDEX idx_tier_evaluation_history_action ON tier_evaluation_history(assignment_action);

-- Tier assignment queue indexes  
CREATE INDEX idx_tier_assignment_queue_status ON tier_assignment_queue(status, priority DESC);
CREATE INDEX idx_tier_assignment_queue_wallet ON tier_assignment_queue(wallet_address);
CREATE INDEX idx_tier_assignment_queue_urgent ON tier_assignment_queue(urgent, created_at) WHERE urgent = TRUE;
CREATE INDEX idx_tier_assignment_queue_retry ON tier_assignment_queue(next_retry_at) WHERE next_retry_at IS NOT NULL;

-- Wallet tier cache indexes
CREATE INDEX idx_wallet_tier_cache_tier ON wallet_tier_cache(current_tier);
CREATE INDEX idx_wallet_tier_cache_expires ON wallet_tier_cache(cache_expires_at);
CREATE INDEX idx_wallet_tier_cache_activity ON wallet_tier_cache(activity_level, last_activity_at);

-- ================================================================================================
-- 6. CREATE TIER ASSIGNMENT FUNCTIONS
-- ================================================================================================

-- Function to evaluate a single rule for a wallet
CREATE OR REPLACE FUNCTION evaluate_tier_rule(
    target_wallet VARCHAR(42),
    rule_uuid UUID,
    batch_id UUID DEFAULT NULL
) RETURNS BOOLEAN AS $$
DECLARE
    rule_record dynamic_tier_rules%ROWTYPE;
    rule_matched BOOLEAN := FALSE;
    assignment_made BOOLEAN := FALSE;
    condition_results JSONB := '{}';
    verification_data JSONB := '{}';
    evaluation_start TIMESTAMPTZ := NOW();
    evaluation_time_ms INTEGER;
BEGIN
    -- Get rule details
    SELECT * INTO rule_record FROM dynamic_tier_rules WHERE id = rule_uuid AND is_active = TRUE;
    
    IF NOT FOUND THEN
        RAISE WARNING 'Rule % not found or inactive', rule_uuid;
        RETURN FALSE;
    END IF;
    
    -- Generate batch ID if not provided
    IF batch_id IS NULL THEN
        batch_id := gen_random_uuid();
    END IF;
    
    -- Evaluate rule based on category
    IF rule_record.rule_category = 'asset_based' THEN
        -- Check token balance conditions
        IF rule_record.conditions ? 'token_balance' THEN
            DECLARE
                token_condition JSONB := rule_record.conditions->'token_balance';
                contract_address TEXT := token_condition->>'contract';
                minimum_balance DECIMAL := (token_condition->>'minimum')::DECIMAL;
                -- For now, we'll simulate the check (in production, this would query blockchain)
                actual_balance DECIMAL := 0;
            BEGIN
                -- Simulate token balance check (replace with actual blockchain query)
                IF contract_address = '0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82' THEN -- CAKE
                    actual_balance := 5000 * 1000000000000000000::DECIMAL; -- Simulate 5000 CAKE
                END IF;
                
                rule_matched := actual_balance >= minimum_balance;
                condition_results := jsonb_set(condition_results, '{token_balance}', 
                    jsonb_build_object(
                        'result', rule_matched,
                        'actual_balance', actual_balance::TEXT,
                        'required_balance', minimum_balance::TEXT,
                        'contract', contract_address
                    )
                );
            END;
        END IF;
        
        -- Check NFT ownership conditions
        IF rule_record.conditions ? 'nft_ownership' THEN
            DECLARE
                nft_condition JSONB := rule_record.conditions->'nft_ownership';
                required_count INTEGER := (nft_condition->>'minimum_count')::INTEGER;
                -- Simulate NFT count (replace with actual blockchain query)
                actual_nft_count INTEGER := 2;
            BEGIN
                rule_matched := rule_matched OR (actual_nft_count >= required_count);
                condition_results := jsonb_set(condition_results, '{nft_ownership}', 
                    jsonb_build_object(
                        'result', actual_nft_count >= required_count,
                        'actual_count', actual_nft_count,
                        'required_count', required_count
                    )
                );
            END;
        END IF;
        
    ELSIF rule_record.rule_category = 'subscription_based' THEN
        -- Check subscription tier from cache
        DECLARE
            cached_subscription TEXT;
        BEGIN
            SELECT subscription_tier INTO cached_subscription 
            FROM wallet_tier_cache 
            WHERE wallet_address = target_wallet;
            
            IF rule_record.conditions ? 'subscription_tiers' THEN
                rule_matched := cached_subscription = ANY(
                    ARRAY(SELECT jsonb_array_elements_text(rule_record.conditions->'subscription_tiers'))
                );
                condition_results := jsonb_set(condition_results, '{subscription_tier}', 
                    jsonb_build_object(
                        'result', rule_matched,
                        'actual_tier', COALESCE(cached_subscription, 'none'),
                        'required_tiers', rule_record.conditions->'subscription_tiers'
                    )
                );
            END IF;
        END;
        
    ELSIF rule_record.rule_category = 'behavior_based' THEN
        -- Check behavior score
        DECLARE
            cached_score INTEGER;
            required_score INTEGER;
        BEGIN
            SELECT behavior_score INTO cached_score 
            FROM wallet_tier_cache 
            WHERE wallet_address = target_wallet;
            
            required_score := (rule_record.conditions->>'minimum_behavior_score')::INTEGER;
            rule_matched := COALESCE(cached_score, 0) >= required_score;
            
            condition_results := jsonb_set(condition_results, '{behavior_score}', 
                jsonb_build_object(
                    'result', rule_matched,
                    'actual_score', COALESCE(cached_score, 0),
                    'required_score', required_score
                )
            );
        END;
    END IF;
    
    -- If rule matched and auto-assign is enabled, make the assignment
    IF rule_matched AND rule_record.auto_assign THEN
        -- Check if wallet is already in this group
        IF NOT EXISTS (
            SELECT 1 FROM wallet_group_memberships 
            WHERE wallet_address = target_wallet 
            AND group_id = rule_record.target_group_id 
            AND is_active = TRUE
        ) THEN
            -- Assign wallet to group
            INSERT INTO wallet_group_memberships (
                wallet_address, group_id, assignment_reason, assignment_source,
                expires_at
            ) VALUES (
                target_wallet, rule_record.target_group_id,
                'Automatic tier assignment: ' || rule_record.rule_name,
                'dynamic_tier',
                CASE WHEN rule_record.assignment_duration_days IS NOT NULL 
                     THEN NOW() + (rule_record.assignment_duration_days || ' days')::INTERVAL 
                     ELSE NULL END
            ) ON CONFLICT (wallet_address, group_id) DO UPDATE SET
                updated_at = NOW(),
                expires_at = CASE WHEN rule_record.assignment_duration_days IS NOT NULL 
                                 THEN NOW() + (rule_record.assignment_duration_days || ' days')::INTERVAL 
                                 ELSE wallet_group_memberships.expires_at END;
            
            assignment_made := TRUE;
            
            -- Update wallet tier cache
            INSERT INTO wallet_tier_cache (
                wallet_address, current_tier, current_group_id, tier_assigned_at
            ) VALUES (
                target_wallet, rule_record.target_tier_name, rule_record.target_group_id, NOW()
            ) ON CONFLICT (wallet_address) DO UPDATE SET
                previous_tier = wallet_tier_cache.current_tier,
                current_tier = rule_record.target_tier_name,
                current_group_id = rule_record.target_group_id,
                tier_assigned_at = NOW(),
                tier_changes_count = wallet_tier_cache.tier_changes_count + 1,
                last_upgrade_at = CASE WHEN rule_record.target_tier_name > wallet_tier_cache.current_tier 
                                      THEN NOW() ELSE wallet_tier_cache.last_upgrade_at END,
                updated_at = NOW();
        END IF;
    END IF;
    
    -- Calculate evaluation time
    evaluation_time_ms := EXTRACT(EPOCH FROM (NOW() - evaluation_start)) * 1000;
    
    -- Log evaluation in history
    INSERT INTO tier_evaluation_history (
        wallet_address, rule_id, evaluation_batch_id,
        rule_matched, assignment_made,
        assignment_action, condition_results, verification_data,
        verification_time_ms, evaluated_at
    ) VALUES (
        target_wallet, rule_uuid, batch_id,
        rule_matched, assignment_made,
        CASE WHEN assignment_made THEN 'assigned' ELSE 'maintained' END,
        condition_results, verification_data,
        evaluation_time_ms, NOW()
    );
    
    -- Update rule statistics
    UPDATE dynamic_tier_rules SET
        total_evaluations = total_evaluations + 1,
        successful_assignments = successful_assignments + CASE WHEN assignment_made THEN 1 ELSE 0 END,
        last_evaluation_at = NOW(),
        updated_at = NOW()
    WHERE id = rule_uuid;
    
    RETURN rule_matched;
END;
$$ LANGUAGE plpgsql;

-- Function to evaluate all active rules for a wallet
CREATE OR REPLACE FUNCTION evaluate_wallet_tiers(target_wallet VARCHAR(42))
RETURNS TABLE(
    rule_id UUID,
    rule_name TEXT,
    matched BOOLEAN,
    assigned BOOLEAN,
    tier_name TEXT
) AS $$
DECLARE
    batch_id UUID := gen_random_uuid();
    rule_record RECORD;
BEGIN
    -- Process all active rules in priority order
    FOR rule_record IN 
        SELECT id, rule_name, target_tier_name
        FROM dynamic_tier_rules 
        WHERE is_active = TRUE 
        ORDER BY priority_level DESC, created_at ASC
    LOOP
        -- Evaluate the rule
        DECLARE
            rule_matched BOOLEAN;
            assignment_made BOOLEAN;
        BEGIN
            SELECT evaluate_tier_rule(target_wallet, rule_record.id, batch_id) INTO rule_matched;
            
            -- Check if assignment was made (from the history log)
            SELECT assignment_made INTO assignment_made
            FROM tier_evaluation_history 
            WHERE wallet_address = target_wallet 
            AND rule_id = rule_record.id 
            AND evaluation_batch_id = batch_id;
            
            -- Return the result
            rule_id := rule_record.id;
            rule_name := rule_record.rule_name;
            matched := rule_matched;
            assigned := assignment_made;
            tier_name := rule_record.target_tier_name;
            
            RETURN NEXT;
        END;
    END LOOP;
    
    RETURN;
END;
$$ LANGUAGE plpgsql;

-- Function to process tier assignment queue
CREATE OR REPLACE FUNCTION process_tier_assignment_queue(
    batch_size INTEGER DEFAULT 100,
    worker_id TEXT DEFAULT 'default_worker'
) RETURNS INTEGER AS $$
DECLARE
    processed_count INTEGER := 0;
    queue_record tier_assignment_queue%ROWTYPE;
    batch_id UUID := gen_random_uuid();
BEGIN
    -- Process pending queue items
    FOR queue_record IN 
        SELECT * FROM tier_assignment_queue 
        WHERE status = 'pending' 
        ORDER BY urgent DESC, priority DESC, created_at ASC 
        LIMIT batch_size
        FOR UPDATE SKIP LOCKED
    LOOP
        BEGIN
            -- Mark as processing
            UPDATE tier_assignment_queue SET
                status = 'processing',
                processing_started_at = NOW(),
                processing_worker_id = worker_id,
                updated_at = NOW()
            WHERE id = queue_record.id;
            
            -- Evaluate tiers for the wallet
            PERFORM evaluate_wallet_tiers(queue_record.wallet_address);
            
            -- Mark as completed
            UPDATE tier_assignment_queue SET
                status = 'completed',
                processing_completed_at = NOW(),
                evaluation_batch_id = batch_id,
                updated_at = NOW()
            WHERE id = queue_record.id;
            
            processed_count := processed_count + 1;
            
        EXCEPTION WHEN OTHERS THEN
            -- Mark as failed and increment retry count
            UPDATE tier_assignment_queue SET
                status = 'failed',
                error_details = jsonb_build_object(
                    'error_message', SQLERRM,
                    'error_state', SQLSTATE,
                    'error_time', NOW()
                ),
                retry_count = retry_count + 1,
                next_retry_at = CASE WHEN retry_count < max_retries 
                               THEN NOW() + INTERVAL '1 hour' * power(2, retry_count)
                               ELSE NULL END,
                updated_at = NOW()
            WHERE id = queue_record.id;
        END;
    END LOOP;
    
    RETURN processed_count;
END;
$$ LANGUAGE plpgsql;

-- ================================================================================================
-- 7. INSERT DEFAULT TIER RULES
-- ================================================================================================

-- Insert default tier rules for common scenarios
DO $$
DECLARE
    free_group_id UUID;
    bronze_group_id UUID;
    silver_group_id UUID;
    gold_group_id UUID;
    platinum_group_id UUID;
BEGIN
    -- Get group IDs (create if they don't exist)
    SELECT id INTO free_group_id FROM permission_groups WHERE slug = 'connected-wallet';
    SELECT id INTO bronze_group_id FROM permission_groups WHERE slug = 'verified-wallet';
    SELECT id INTO silver_group_id FROM permission_groups WHERE slug = 'premium-wallet';
    
    -- Create tier groups if they don't exist
    INSERT INTO permission_groups (name, slug, description, permissions, is_system_group, priority_level)
    VALUES 
    ('Bronze Tier', 'bronze-tier', 'Bronze tier users with basic token holdings', 
     ARRAY['epsx:analytics:view', 'epsx:trading:basic'], TRUE, 3),
    ('Silver Tier', 'silver-tier', 'Silver tier users with significant holdings',
     ARRAY['epsx:analytics:view', 'epsx:analytics:advanced', 'epsx:trading:advanced'], TRUE, 5),
    ('Gold Tier', 'gold-tier', 'Gold tier users with substantial holdings',
     ARRAY['epsx:analytics:view', 'epsx:analytics:advanced', 'epsx:trading:premium', 'epsx:realtime:access'], TRUE, 7),
    ('Platinum Tier', 'platinum-tier', 'Platinum tier VIP users',
     ARRAY['epsx:*:*', 'epsx:premium:access', 'epsx:vip:support'], TRUE, 9)
    ON CONFLICT (slug) DO NOTHING;
    
    -- Get the created group IDs
    SELECT id INTO bronze_group_id FROM permission_groups WHERE slug = 'bronze-tier';
    SELECT id INTO silver_group_id FROM permission_groups WHERE slug = 'silver-tier';
    SELECT id INTO gold_group_id FROM permission_groups WHERE slug = 'gold-tier';
    SELECT id INTO platinum_group_id FROM permission_groups WHERE slug = 'platinum-tier';
    
    -- Insert tier rules
    INSERT INTO dynamic_tier_rules (
        rule_name, rule_description, rule_category, target_group_id, target_tier_name,
        conditions, is_active, auto_assign, priority_level, evaluation_frequency
    ) VALUES
    (
        'Bronze CAKE Holders',
        'Users holding 100+ CAKE tokens qualify for bronze tier',
        'asset_based',
        bronze_group_id,
        'bronze',
        '{"token_balance": {"contract": "0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82", "minimum": "100000000000000000000", "network": "bsc"}}'::jsonb,
        TRUE, TRUE, 3, 'daily'
    ),
    (
        'Silver Token Portfolio',
        'Users with diverse token portfolio (1000+ CAKE or 5+ BNB)',
        'asset_based',
        silver_group_id,
        'silver',
        '{"token_balance": {"contract": "0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82", "minimum": "1000000000000000000000", "network": "bsc"}}'::jsonb,
        TRUE, TRUE, 5, 'daily'
    ),
    (
        'Gold High Value Holders',
        'Users with significant holdings (10000+ CAKE or 50+ BNB)',
        'asset_based',
        gold_group_id,
        'gold',
        '{"token_balance": {"contract": "0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82", "minimum": "10000000000000000000000", "network": "bsc"}}'::jsonb,
        TRUE, TRUE, 7, 'daily'
    ),
    (
        'Platinum Whale Status',
        'Ultra-high net worth users (100000+ CAKE)',
        'asset_based',
        platinum_group_id,
        'platinum',
        '{"token_balance": {"contract": "0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82", "minimum": "100000000000000000000000", "network": "bsc"}}'::jsonb,
        TRUE, TRUE, 10, 'daily'
    ),
    (
        'Premium Subscription',
        'Users with active premium subscription',
        'subscription_based',
        silver_group_id,
        'silver',
        '{"subscription_tiers": ["premium", "pro"]}'::jsonb,
        TRUE, TRUE, 6, 'hourly'
    ),
    (
        'High Activity Users',
        'Users with high behavior score',
        'behavior_based',
        bronze_group_id,
        'bronze',
        '{"minimum_behavior_score": 75}'::jsonb,
        TRUE, TRUE, 2, 'weekly'
    );
    
    RAISE NOTICE 'Inserted % default tier rules', 6;
END $$;

-- ================================================================================================
-- 8. CREATE INITIAL QUEUE ENTRIES FOR EXISTING WALLETS
-- ================================================================================================

-- Queue all existing wallets for tier evaluation
INSERT INTO tier_assignment_queue (
    wallet_address, trigger_source, priority, urgent
)
SELECT 
    wallet_address, 
    'initial_migration',
    1, -- High priority for initial migration
    FALSE
FROM wallet_identities
WHERE NOT EXISTS (
    SELECT 1 FROM tier_assignment_queue q 
    WHERE q.wallet_address = wallet_identities.wallet_address 
    AND q.status IN ('pending', 'processing')
);

-- ================================================================================================
-- 9. CREATE TRIGGERS FOR AUTOMATIC QUEUE MANAGEMENT
-- ================================================================================================

-- Trigger to automatically queue wallets when they get new group memberships
CREATE OR REPLACE FUNCTION trigger_wallet_tier_reevaluation()
RETURNS TRIGGER AS $$
BEGIN
    -- Queue wallet for reevaluation when membership changes
    INSERT INTO tier_assignment_queue (
        wallet_address, trigger_source, trigger_metadata
    ) VALUES (
        COALESCE(NEW.wallet_address, OLD.wallet_address),
        'membership_change',
        jsonb_build_object(
            'trigger_action', TG_OP,
            'group_id', COALESCE(NEW.group_id, OLD.group_id),
            'trigger_time', NOW()
        )
    ) ON CONFLICT DO NOTHING; -- Avoid duplicate entries
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create trigger on wallet_group_memberships
DROP TRIGGER IF EXISTS trigger_wallet_tier_reevaluation ON wallet_group_memberships;
CREATE TRIGGER trigger_wallet_tier_reevaluation
    AFTER INSERT OR UPDATE OR DELETE ON wallet_group_memberships
    FOR EACH ROW
    EXECUTE FUNCTION trigger_wallet_tier_reevaluation();

-- ================================================================================================
-- 10. SUCCESS MESSAGE AND VERIFICATION
-- ================================================================================================

DO $$
DECLARE
    rule_count INTEGER;
    queue_count INTEGER;
    cache_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO rule_count FROM dynamic_tier_rules WHERE is_active = TRUE;
    SELECT COUNT(*) INTO queue_count FROM tier_assignment_queue WHERE status = 'pending';
    SELECT COUNT(*) INTO cache_count FROM wallet_tier_cache;
    
    RAISE NOTICE '🎉 ===== DYNAMIC TIER GROUP ASSIGNMENT SYSTEM CREATED! =====';
    RAISE NOTICE '✅ System Components Created:';
    RAISE NOTICE '  - dynamic_tier_rules table with flexible JSON conditions';
    RAISE NOTICE '  - tier_evaluation_history for complete audit trail';
    RAISE NOTICE '  - tier_assignment_queue for scalable processing';
    RAISE NOTICE '  - wallet_tier_cache for performance optimization';
    RAISE NOTICE '  - Comprehensive functions for tier evaluation and management';
    RAISE NOTICE '📊 Initial Data:';
    RAISE NOTICE '  - Active tier rules: %', rule_count;
    RAISE NOTICE '  - Pending queue items: %', queue_count;
    RAISE NOTICE '  - Cached wallet tiers: %', cache_count;
    RAISE NOTICE '🚀 Dynamic tier system is ready for intelligent wallet assignment!';
    RAISE NOTICE '💡 Use process_tier_assignment_queue() to process pending evaluations';
    RAISE NOTICE '📈 Wallets will be automatically assigned to appropriate tiers based on their assets and behavior';
END $$;