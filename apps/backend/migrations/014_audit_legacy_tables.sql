-- Migration 014: Audit Legacy Tables
-- Identifies potentially unused legacy tables for cleanup
-- This migration only queries and reports, does not drop anything

-- Create a temporary function to audit database tables
CREATE OR REPLACE FUNCTION audit_legacy_tables()
RETURNS TABLE (
    table_name TEXT,
    row_count BIGINT,
    table_size_mb NUMERIC,
    created_from_migration TEXT,
    legacy_classification TEXT,
    cleanup_recommendation TEXT
) AS $$
DECLARE
    rec RECORD;
    table_row_count BIGINT;
    table_size_bytes BIGINT;
BEGIN
    -- Iterate through all tables in the public schema
    FOR rec IN 
        SELECT schemaname, tablename 
        FROM pg_tables 
        WHERE schemaname = 'public'
        ORDER BY tablename
    LOOP
        -- Get row count
        EXECUTE format('SELECT COUNT(*) FROM %I', rec.tablename) INTO table_row_count;
        
        -- Get table size in bytes
        SELECT pg_total_relation_size(format('%I', rec.tablename)) INTO table_size_bytes;
        
        -- Return analysis for each table
        RETURN QUERY SELECT 
            rec.tablename::TEXT,
            table_row_count,
            ROUND((table_size_bytes / 1024.0 / 1024.0)::numeric, 2) as size_mb,
            CASE 
                -- Map tables to their migration origins
                WHEN rec.tablename IN ('users', 'sessions', 'user_permissions', 'refresh_tokens', 'revoked_tokens', 'pricing_plans', 'promotional_campaigns', 'plan_promotions', 'discount_codes', 'pricing_experiments', 'affiliates', 'referrals', 'commissions', 'affiliate_payouts', 'affiliate_materials', 'affiliate_tiers') THEN 'Migration 001 (Initial Schema)'
                WHEN rec.tablename IN ('permission_hierarchy', 'dynamic_policies', 'policy_evaluations', 'policy_templates', 'policy_approval_queue', 'permission_inheritance_cache', 'policy_performance_metrics') THEN 'Migration 002 (Enhanced Access Control)'
                WHEN rec.tablename LIKE '%rate_limit%' AND rec.tablename NOT LIKE '%comprehensive%' THEN 'Migration 004 (Rate Limiting)'
                WHEN rec.tablename IN ('rate_limit_entries', 'rate_limit_violations', 'rate_limit_tiers', 'client_tier_mappings', 'rate_limit_statistics') THEN 'Migration 005 (Comprehensive Rate Limiting)'
                WHEN rec.tablename IN ('web3_auth_nonces', 'wallet_permissions', 'nft_permission_configs', 'token_permission_configs', 'dao_permission_proposals', 'dao_votes', 'web3_permission_cache', 'wallet_migrations') THEN 'Migration 007 (Web3 Authentication)'
                WHEN rec.tablename IN ('notifications', 'notification_preferences', 'notification_delivery_log') THEN 'Migration 008 (Stateless Notifications)'
                WHEN rec.tablename = 'user_subscription_activations' THEN 'Migration 010 (Subscription Activations)'
                WHEN rec.tablename = 'permission_delegations' THEN 'Migration 011a (Permission Delegations)'
                WHEN rec.tablename = 'permission_templates' THEN 'Migration 011b (Remove Package Tiers)'
                WHEN rec.tablename IN ('api_keys', 'enterprise_teams', 'enterprise_team_members', 'api_requests') THEN 'Migration 012 (API Keys)'
                ELSE 'Unknown/Other'
            END::TEXT,
            CASE
                -- Classify tables as legacy based on Web3-first architecture
                WHEN rec.tablename IN ('sessions', 'refresh_tokens', 'revoked_tokens') THEN 'LEGACY: Traditional auth (Web3 replaced)'
                WHEN rec.tablename = 'user_permissions' THEN 'LEGACY: Replaced by wallet_permissions'
                WHEN rec.tablename IN ('permission_hierarchy', 'dynamic_policies', 'policy_evaluations', 'policy_templates', 'policy_approval_queue', 'permission_inheritance_cache', 'policy_performance_metrics') THEN 'LEGACY: Complex permission system (unused)'
                WHEN rec.tablename IN ('pricing_plans', 'promotional_campaigns', 'plan_promotions', 'discount_codes', 'pricing_experiments', 'affiliates', 'referrals', 'commissions', 'affiliate_payouts', 'affiliate_materials', 'affiliate_tiers') THEN 'POTENTIALLY_LEGACY: Marketing/Affiliate system'
                WHEN rec.tablename IN ('rate_limit_usage', 'rate_limit_violations') AND EXISTS (SELECT 1 FROM pg_tables WHERE tablename = 'rate_limit_entries') THEN 'POTENTIALLY_LEGACY: Duplicate rate limiting (migration 004 vs 005)'
                WHEN rec.tablename IN ('users', 'wallet_permissions', 'web3_auth_nonces', 'notifications', 'api_keys', 'enterprise_teams') THEN 'ACTIVE: Core Web3 system'
                ELSE 'NEEDS_REVIEW: Unknown status'
            END::TEXT,
            CASE
                WHEN rec.tablename IN ('sessions', 'refresh_tokens', 'revoked_tokens', 'user_permissions') AND table_row_count = 0 THEN 'SAFE_TO_DROP: Empty legacy table'
                WHEN rec.tablename IN ('sessions', 'refresh_tokens', 'revoked_tokens', 'user_permissions') AND table_row_count > 0 THEN 'MIGRATE_THEN_DROP: Has data, needs migration to Web3 system'
                WHEN rec.tablename IN ('permission_hierarchy', 'dynamic_policies', 'policy_evaluations', 'policy_templates', 'policy_approval_queue', 'permission_inheritance_cache', 'policy_performance_metrics') THEN 'SAFE_TO_DROP: Complex permission system not used'
                WHEN rec.tablename IN ('pricing_plans', 'promotional_campaigns', 'plan_promotions', 'discount_codes', 'pricing_experiments', 'affiliates', 'referrals', 'commissions', 'affiliate_payouts', 'affiliate_materials', 'affiliate_tiers') THEN 'REVIEW_FIRST: Confirm marketing system not needed'
                WHEN rec.tablename IN ('rate_limit_usage', 'rate_limit_violations') AND EXISTS (SELECT 1 FROM pg_tables WHERE tablename = 'rate_limit_entries') THEN 'CONSOLIDATE: Remove duplicate rate limiting tables'
                ELSE 'KEEP: Active table in current system'
            END::TEXT;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Run the audit and display results
SELECT 
    table_name,
    row_count,
    table_size_mb,
    created_from_migration,
    legacy_classification,
    cleanup_recommendation
FROM audit_legacy_tables()
ORDER BY 
    CASE 
        WHEN legacy_classification LIKE 'LEGACY:%' THEN 1
        WHEN legacy_classification LIKE 'POTENTIALLY_LEGACY:%' THEN 2
        WHEN legacy_classification LIKE 'NEEDS_REVIEW:%' THEN 3
        ELSE 4
    END,
    table_name;

-- Create summary statistics
SELECT 
    legacy_classification,
    COUNT(*) as table_count,
    SUM(row_count) as total_rows,
    ROUND(SUM(table_size_mb), 2) as total_size_mb
FROM audit_legacy_tables()
GROUP BY legacy_classification
ORDER BY 
    CASE 
        WHEN legacy_classification LIKE 'LEGACY:%' THEN 1
        WHEN legacy_classification LIKE 'POTENTIALLY_LEGACY:%' THEN 2
        WHEN legacy_classification LIKE 'NEEDS_REVIEW:%' THEN 3
        ELSE 4
    END;

-- Generate cleanup recommendations summary
SELECT 
    cleanup_recommendation,
    COUNT(*) as table_count,
    string_agg(table_name, ', ' ORDER BY table_name) as table_names
FROM audit_legacy_tables()
WHERE cleanup_recommendation != 'KEEP: Active table in current system'
GROUP BY cleanup_recommendation
ORDER BY table_count DESC;

-- Drop the temporary function
DROP FUNCTION audit_legacy_tables();

-- Log the audit completion
SELECT 'Legacy table audit completed - review results above for cleanup recommendations' as audit_status;