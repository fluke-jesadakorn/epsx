-- ================================================================================================
-- EPSX Schema Repair Migration
-- ================================================================================================
-- This migration fixes the existing database schema to match the consolidated target state
-- Handles hybrid state where both user-based and wallet-based tables coexist
-- ================================================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ================================================================================================
-- 1. UPDATE EXISTING WALLET_GROUP_MEMBERSHIPS TO MATCH TARGET SCHEMA
-- ================================================================================================

-- Add missing columns to wallet_group_memberships if they don't exist
DO $$
BEGIN
    -- Add granted_by_wallet column (rename from assigned_by)
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'wallet_group_memberships' AND column_name = 'assigned_by')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns 
                       WHERE table_name = 'wallet_group_memberships' AND column_name = 'granted_by_wallet')
    THEN
        ALTER TABLE wallet_group_memberships RENAME COLUMN assigned_by TO granted_by_wallet;
    END IF;
    
    -- Add granted_at column (rename from assigned_at)
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'wallet_group_memberships' AND column_name = 'assigned_at')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns 
                       WHERE table_name = 'wallet_group_memberships' AND column_name = 'granted_at')
    THEN
        ALTER TABLE wallet_group_memberships RENAME COLUMN assigned_at TO granted_at;
    END IF;
    
    -- Add missing Web3-specific columns
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_group_memberships' AND column_name = 'assignment_reason') THEN
        ALTER TABLE wallet_group_memberships ADD COLUMN assignment_reason TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_group_memberships' AND column_name = 'assignment_source') THEN
        ALTER TABLE wallet_group_memberships ADD COLUMN assignment_source VARCHAR(50);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_group_memberships' AND column_name = 'web3_verification_tx') THEN
        ALTER TABLE wallet_group_memberships ADD COLUMN web3_verification_tx VARCHAR(66);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_group_memberships' AND column_name = 'web3_verification_block') THEN
        ALTER TABLE wallet_group_memberships ADD COLUMN web3_verification_block BIGINT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_group_memberships' AND column_name = 'web3_verification_data') THEN
        ALTER TABLE wallet_group_memberships ADD COLUMN web3_verification_data JSONB;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_group_memberships' AND column_name = 'payment_reference') THEN
        ALTER TABLE wallet_group_memberships ADD COLUMN payment_reference VARCHAR(100);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_group_memberships' AND column_name = 'subscription_tier') THEN
        ALTER TABLE wallet_group_memberships ADD COLUMN subscription_tier VARCHAR(50);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_group_memberships' AND column_name = 'metadata') THEN
        ALTER TABLE wallet_group_memberships ADD COLUMN metadata JSONB DEFAULT '{}';
    END IF;
END $$;

-- ================================================================================================
-- 2. UPDATE WALLET_IDENTITIES TO MATCH TARGET SCHEMA  
-- ================================================================================================

-- Add missing columns to wallet_identities
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'avatar_url') THEN
        ALTER TABLE wallet_identities ADD COLUMN avatar_url TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'bio') THEN
        ALTER TABLE wallet_identities ADD COLUMN bio TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'website') THEN
        ALTER TABLE wallet_identities ADD COLUMN website VARCHAR(500);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'twitter_handle') THEN
        ALTER TABLE wallet_identities ADD COLUMN twitter_handle VARCHAR(100);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'discord_handle') THEN
        ALTER TABLE wallet_identities ADD COLUMN discord_handle VARCHAR(100);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'telegram_handle') THEN
        ALTER TABLE wallet_identities ADD COLUMN telegram_handle VARCHAR(100);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'total_requests') THEN
        ALTER TABLE wallet_identities ADD COLUMN total_requests BIGINT DEFAULT 0;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'verification_method') THEN
        ALTER TABLE wallet_identities ADD COLUMN verification_method VARCHAR(50);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'verification_data') THEN
        ALTER TABLE wallet_identities ADD COLUMN verification_data JSONB DEFAULT '{}';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'settings') THEN
        ALTER TABLE wallet_identities ADD COLUMN settings JSONB DEFAULT '{}';
    END IF;
END $$;

-- ================================================================================================
-- 3. UPDATE REQUEST_NONCES TO MATCH TARGET SCHEMA
-- ================================================================================================

-- Add missing columns to request_nonces
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'request_nonces' AND column_name = 'id') THEN
        ALTER TABLE request_nonces ADD COLUMN id UUID DEFAULT gen_random_uuid();
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'request_nonces' AND column_name = 'http_method') THEN
        ALTER TABLE request_nonces ADD COLUMN http_method VARCHAR(10) NOT NULL DEFAULT 'GET';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'request_nonces' AND column_name = 'request_timestamp') THEN
        ALTER TABLE request_nonces ADD COLUMN request_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW();
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'request_nonces' AND column_name = 'client_ip') THEN
        ALTER TABLE request_nonces ADD COLUMN client_ip INET;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'request_nonces' AND column_name = 'user_agent') THEN
        ALTER TABLE request_nonces ADD COLUMN user_agent TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'request_nonces' AND column_name = 'is_used') THEN
        ALTER TABLE request_nonces ADD COLUMN is_used BOOLEAN DEFAULT FALSE;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'request_nonces' AND column_name = 'signature_hash') THEN
        ALTER TABLE request_nonces ADD COLUMN signature_hash VARCHAR(132);
    END IF;
END $$;

-- ================================================================================================
-- 4. CREATE MISSING TABLES
-- ================================================================================================

-- Create missing tables that should exist but don't
CREATE TABLE IF NOT EXISTS wallet_auth_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) NOT NULL,
    auth_method VARCHAR(20) NOT NULL,
    signature_hash VARCHAR(132) NOT NULL,
    message_hash VARCHAR(66) NOT NULL,
    endpoint_path VARCHAR(500),
    http_method VARCHAR(10),
    success BOOLEAN NOT NULL,
    failure_reason TEXT,
    client_ip INET,
    user_agent TEXT,
    chain_id INTEGER,
    verification_time_ms INTEGER,
    blockchain_calls INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT fk_auth_log_wallet FOREIGN KEY (wallet_address) 
        REFERENCES wallet_identities(wallet_address) ON DELETE CASCADE
);

-- ================================================================================================
-- 5. CREATE MISSING INDEXES
-- ================================================================================================

-- Add missing indexes that should exist
CREATE INDEX IF NOT EXISTS idx_wallet_identities_verification ON wallet_identities(is_verified, verification_method);
CREATE INDEX IF NOT EXISTS idx_request_nonces_used ON request_nonces(is_used, expires_at);
CREATE INDEX IF NOT EXISTS idx_request_nonces_signature ON request_nonces(signature_hash);

-- Wallet group memberships indexes with correct column names
CREATE INDEX IF NOT EXISTS idx_wallet_groups_source ON wallet_group_memberships(assignment_source);
CREATE INDEX IF NOT EXISTS idx_wallet_groups_granted_by ON wallet_group_memberships(granted_by_wallet);

-- Group assignment history indexes (fix column references)
CREATE INDEX IF NOT EXISTS idx_gah_wallet_performed_at ON group_assignment_history(user_id, performed_at);

-- ================================================================================================
-- 6. FIX UTILITY FUNCTIONS  
-- ================================================================================================

-- Fix the cleanup function with correct syntax
CREATE OR REPLACE FUNCTION cleanup_wallet_auth_data()
RETURNS INTEGER AS $$
DECLARE
    cleaned_count INTEGER := 0;
    temp_count INTEGER;
BEGIN
    -- Clean up expired nonces
    DELETE FROM request_nonces 
    WHERE expires_at < NOW() OR (is_used = TRUE AND created_at < NOW() - INTERVAL '1 hour');
    
    GET DIAGNOSTICS temp_count = ROW_COUNT;
    cleaned_count := temp_count;
    
    -- Clean up expired permission cache
    DELETE FROM wallet_permission_cache 
    WHERE expires_at < NOW();
    
    GET DIAGNOSTICS temp_count = ROW_COUNT;
    cleaned_count := cleaned_count + temp_count;
    
    -- Clean up expired Web3 verification cache
    DELETE FROM web3_verification_cache 
    WHERE expires_at < NOW();
    
    GET DIAGNOSTICS temp_count = ROW_COUNT;
    cleaned_count := cleaned_count + temp_count;
    
    -- Clean up old auth logs (keep 30 days)
    DELETE FROM wallet_auth_log 
    WHERE created_at < NOW() - INTERVAL '30 days';
    
    GET DIAGNOSTICS temp_count = ROW_COUNT;
    cleaned_count := cleaned_count + temp_count;
    
    RETURN cleaned_count;
END;
$$ LANGUAGE plpgsql;

-- ================================================================================================
-- 7. CREATE/FIX VIEWS WITH CORRECT COLUMN REFERENCES
-- ================================================================================================

-- Drop existing views that have wrong column references
DROP VIEW IF EXISTS active_wallet_groups;
DROP VIEW IF EXISTS group_membership_stats;

-- Create views with correct column references
CREATE VIEW active_wallet_groups AS
SELECT 
    wgm.wallet_address,
    pg.id as group_id,
    pg.name as group_name,
    pg.slug,
    pg.permissions,
    pg.priority_level,
    wgm.granted_at,
    wgm.expires_at,
    wgm.assignment_reason,
    wgm.assignment_source
FROM wallet_group_memberships wgm
INNER JOIN permission_groups pg ON wgm.group_id = pg.id
WHERE wgm.is_active = true 
  AND (wgm.expires_at IS NULL OR wgm.expires_at > NOW())
ORDER BY wgm.wallet_address, pg.priority_level DESC;

-- Create group membership stats with correct column references
CREATE VIEW group_membership_stats AS
SELECT 
    pg.id,
    pg.name,
    pg.slug,
    COUNT(CASE WHEN wgm.is_active = true AND (wgm.expires_at IS NULL OR wgm.expires_at > NOW()) THEN 1 END) as active_members,
    COUNT(wgm.id) as total_assignments,
    COUNT(CASE WHEN wgm.expires_at IS NOT NULL AND wgm.expires_at <= NOW() THEN 1 END) as expired_members,
    MAX(wgm.granted_at) as last_assignment,
    pg.max_members,
    pg.is_system_group,
    pg.is_web3_managed
FROM permission_groups pg
LEFT JOIN wallet_group_memberships wgm ON pg.id = wgm.group_id
GROUP BY pg.id, pg.name, pg.slug, pg.max_members, pg.is_system_group, pg.is_web3_managed;

-- ================================================================================================
-- 8. INSERT SEED DATA (WITH CONFLICT HANDLING)
-- ================================================================================================

-- Insert default permission groups if they don't exist
INSERT INTO permission_groups (name, slug, description, permissions, is_system_group, priority_level) 
SELECT * FROM (VALUES
    ('Free Template', 'free-template', 'Basic free tier with essential features', 
     ARRAY['epsx:rankings:view:3', 'epsx:trading:basic', 'epsx:portfolio:view'], 
     true, 0),
    ('Bronze Template', 'bronze-template', 'Enhanced access with basic features',
     ARRAY['epsx:rankings:view:5', 'epsx:trading:basic', 'epsx:portfolio:view', 'epsx:portfolio:history'],
     true, 1),
    ('Silver Template', 'silver-template', 'Premium access with advanced analytics',
     ARRAY['epsx:rankings:view:25', 'epsx:trading:basic', 'epsx:trading:advanced', 'epsx:portfolio:view', 'epsx:analytics:basic'],
     true, 2),
    ('Gold Template', 'gold-template', 'Professional access with premium tools',
     ARRAY['epsx:rankings:view:50', 'epsx:trading:premium', 'epsx:portfolio:tools', 'epsx:analytics:advanced'],
     true, 3),
    ('Platinum Template', 'platinum-template', 'VIP access with advanced features',
     ARRAY['epsx:rankings:view:100', 'epsx:trading:premium', 'epsx:analytics:premium', 'epsx:research:reports', 'epsx:dashboards:custom'],
     true, 4),
    ('Enterprise Template', 'enterprise-template', 'Unlimited access with all platform features',
     ARRAY['epsx:rankings:view:unlimited', 'epsx:*:*', 'epsx-pay:*:*', 'epsx-token:*:*'],
     true, 5),
    ('Admin Template', 'admin-template', 'Full administrative access',
     ARRAY['admin:*:*', 'epsx:*:*', 'epsx-pay:*:*', 'epsx-token:*:*'],
     true, 6)
) AS t(name, slug, description, permissions, is_system_group, priority_level)
WHERE NOT EXISTS (
    SELECT 1 FROM permission_groups pg WHERE pg.name = t.name
);

-- Insert tier pricing if doesn't exist
INSERT INTO tier_pricing (tier, billing_type, base_cost_usd, included_features, usage_limits) 
SELECT * FROM (VALUES
    ('Starter', 'monthly', 29.00::DECIMAL, 
     '["Basic Web3 Authentication", "Standard API Rate Limits", "Community Support"]'::jsonb,
     '{"max_api_calls_per_month": 10000, "max_authentication_requests": 1000, "max_data_transfer_gb": 1.0, "max_storage_gb": 0.1, "rate_limit_per_minute": 60}'::jsonb),
    ('Business', 'monthly', 99.00::DECIMAL,
     '["Enhanced Web3 Authentication", "Compliance Features", "Priority Support", "Advanced Analytics"]'::jsonb,
     '{"max_api_calls_per_month": 100000, "max_authentication_requests": 10000, "max_data_transfer_gb": 10.0, "max_storage_gb": 1.0, "rate_limit_per_minute": 300}'::jsonb),
    ('Enterprise', 'monthly', 299.00::DECIMAL,
     '["Full Authentication Suite", "Advanced Security Monitoring", "Custom Integrations", "Dedicated Account Manager"]'::jsonb,
     '{"max_api_calls_per_month": 1000000, "max_authentication_requests": 100000, "max_data_transfer_gb": 100.0, "max_storage_gb": 10.0, "rate_limit_per_minute": 1000}'::jsonb),
    ('Whale', 'monthly', 999.00::DECIMAL,
     '["Premium Features", "Custom Development", "White-label Solutions", "24/7 Premium Support"]'::jsonb,
     '{"max_api_calls_per_month": null, "max_authentication_requests": null, "max_data_transfer_gb": null, "max_storage_gb": null, "rate_limit_per_minute": null}'::jsonb)
) AS t(tier, billing_type, base_cost_usd, included_features, usage_limits)
WHERE NOT EXISTS (
    SELECT 1 FROM tier_pricing tp WHERE tp.tier = t.tier AND tp.billing_type = t.billing_type
);

-- ================================================================================================
-- 9. ADD FOREIGN KEY CONSTRAINTS THAT MIGHT BE MISSING
-- ================================================================================================

-- Add foreign key constraint from wallet_group_memberships to wallet_identities for granted_by_wallet
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'fk_wallet_membership_granter'
    ) THEN
        ALTER TABLE wallet_group_memberships 
        ADD CONSTRAINT fk_wallet_membership_granter 
        FOREIGN KEY (granted_by_wallet) REFERENCES wallet_identities(wallet_address) ON DELETE SET NULL;
    END IF;
END $$;

-- ================================================================================================
-- 10. SUCCESS MESSAGE
-- ================================================================================================

DO $$ 
BEGIN 
    RAISE NOTICE '✅ EPSX Schema Repair Migration Complete!';
    RAISE NOTICE '📊 Schema Updates:';
    RAISE NOTICE '  - Updated wallet_group_memberships with missing columns';
    RAISE NOTICE '  - Updated wallet_identities with profile fields';
    RAISE NOTICE '  - Updated request_nonces with security fields';
    RAISE NOTICE '  - Fixed utility functions and views';
    RAISE NOTICE '  - Added missing indexes and constraints';
    RAISE NOTICE '  - Inserted seed data with conflict handling';
    RAISE NOTICE '🚀 EPSX database schema is now aligned with consolidated target state!';
END $$;