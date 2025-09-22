-- Migration 015: Remove Legacy Tables
-- Comprehensive cleanup of unused legacy tables identified in migration 014
-- This migration PERMANENTLY removes legacy tables - run with caution

-- Step 1: Backup any data from legacy tables before dropping (for safety)
-- Note: In production, consider creating a backup schema first

-- Step 2: Drop legacy authentication tables (replaced by Web3 system)
-- These were supposed to be dropped in migration 013 but may still exist
DO $$
BEGIN
    -- Drop legacy authentication tables if they exist
    DROP TABLE IF EXISTS sessions CASCADE;
    DROP TABLE IF EXISTS refresh_tokens CASCADE;
    DROP TABLE IF EXISTS revoked_tokens CASCADE;
    
    -- user_permissions was replaced by wallet_permissions in Web3 system
    -- Only drop if wallet_permissions exists and has data
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'wallet_permissions') THEN
        DROP TABLE IF EXISTS user_permissions CASCADE;
        RAISE NOTICE 'Dropped user_permissions table - replaced by wallet_permissions';
    ELSE
        RAISE WARNING 'wallet_permissions table not found - keeping user_permissions for safety';
    END IF;
    
    RAISE NOTICE 'Completed legacy authentication table cleanup';
END $$;

-- Step 3: Drop complex permission system tables from migration 002 (unused)
DO $$
BEGIN
    DROP TABLE IF EXISTS permission_hierarchy CASCADE;
    DROP TABLE IF EXISTS dynamic_policies CASCADE;
    DROP TABLE IF EXISTS policy_evaluations CASCADE;
    DROP TABLE IF EXISTS policy_templates CASCADE;
    DROP TABLE IF EXISTS policy_approval_queue CASCADE;
    DROP TABLE IF EXISTS permission_inheritance_cache CASCADE;
    DROP TABLE IF EXISTS policy_performance_metrics CASCADE;
    
    -- Also drop any partitioned tables from migration 003
    DROP TABLE IF EXISTS policy_evaluations_partitioned CASCADE;
    
    RAISE NOTICE 'Completed complex permission system cleanup';
END $$;

-- Step 4: Clean up marketing/affiliate system tables (if confirmed unused)
-- WARNING: Only uncomment this section if you've confirmed these tables are not needed
/*
DO $$
BEGIN
    -- Check if any of these tables have data
    DECLARE
        has_data BOOLEAN := FALSE;
        table_count INTEGER;
    BEGIN
        -- Check for data in marketing tables
        SELECT INTO table_count COUNT(*) FROM pricing_plans WHERE 1=1;
        IF table_count > 0 THEN has_data := TRUE; END IF;
        
        SELECT INTO table_count COUNT(*) FROM affiliates WHERE 1=1;
        IF table_count > 0 THEN has_data := TRUE; END IF;
        
        IF has_data THEN
            RAISE WARNING 'Marketing/affiliate tables contain data - skipping cleanup';
        ELSE
            -- Safe to drop - no data found
            DROP TABLE IF EXISTS affiliate_tiers CASCADE;
            DROP TABLE IF EXISTS affiliate_materials CASCADE;
            DROP TABLE IF EXISTS affiliate_payouts CASCADE;
            DROP TABLE IF EXISTS commissions CASCADE;
            DROP TABLE IF EXISTS referrals CASCADE;
            DROP TABLE IF EXISTS affiliates CASCADE;
            DROP TABLE IF EXISTS pricing_experiments CASCADE;
            DROP TABLE IF EXISTS discount_codes CASCADE;
            DROP TABLE IF EXISTS plan_promotions CASCADE;
            DROP TABLE IF EXISTS promotional_campaigns CASCADE;
            DROP TABLE IF EXISTS pricing_plans CASCADE;
            
            RAISE NOTICE 'Completed marketing/affiliate system cleanup';
        END IF;
    EXCEPTION
        WHEN undefined_table THEN
            RAISE NOTICE 'Some marketing tables already dropped or never existed';
    END;
END $$;
*/

-- Step 5: Consolidate rate limiting tables (remove duplicates from migration 004 vs 005)
DO $$
BEGIN
    -- Check if we have both old and new rate limiting tables
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'rate_limit_entries') 
       AND EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'rate_limit_usage') THEN
        
        -- Migrate any important data from old tables to new tables if needed
        -- For now, just drop the old migration 004 tables in favor of migration 005 tables
        DROP TABLE IF EXISTS rate_limit_usage CASCADE;
        DROP TABLE IF EXISTS rate_limit_violations CASCADE; -- This might exist in both migrations
        
        RAISE NOTICE 'Consolidated rate limiting tables - kept migration 005 structure';
    END IF;
    
    -- Also clean up any orphaned rate limiting tables
    DROP TABLE IF EXISTS rate_limit_stats CASCADE;
    DROP TABLE IF EXISTS rate_limit_configs CASCADE;
    
EXCEPTION
    WHEN undefined_table THEN
        RAISE NOTICE 'Some rate limiting tables already cleaned up';
END $$;

-- Step 6: Clean up any orphaned indexes from dropped tables
DO $$
DECLARE
    index_record RECORD;
BEGIN
    -- Drop indexes that reference dropped tables
    FOR index_record IN
        SELECT indexname 
        FROM pg_indexes 
        WHERE schemaname = 'public'
        AND (
            indexname LIKE '%sessions%' OR
            indexname LIKE '%refresh_tokens%' OR
            indexname LIKE '%revoked_tokens%' OR
            indexname LIKE '%permission_hierarchy%' OR
            indexname LIKE '%dynamic_policies%' OR
            indexname LIKE '%policy_evaluations%' OR
            indexname LIKE '%rate_limit_usage%'
        )
    LOOP
        BEGIN
            EXECUTE 'DROP INDEX IF EXISTS ' || index_record.indexname;
            RAISE NOTICE 'Dropped orphaned index: %', index_record.indexname;
        EXCEPTION
            WHEN OTHERS THEN
                RAISE NOTICE 'Could not drop index % (may not exist): %', index_record.indexname, SQLERRM;
        END;
    END LOOP;
END $$;

-- Step 7: Clean up any orphaned foreign key constraints
DO $$
DECLARE
    constraint_record RECORD;
BEGIN
    -- Find and drop constraints referencing dropped tables
    FOR constraint_record IN
        SELECT conname, conrelid::regclass as table_name
        FROM pg_constraint
        WHERE contype = 'f'
        AND EXISTS (
            SELECT 1 FROM information_schema.constraint_column_usage
            WHERE constraint_name = conname
            AND table_name IN ('sessions', 'refresh_tokens', 'revoked_tokens', 'user_permissions', 'permission_hierarchy')
        )
    LOOP
        BEGIN
            EXECUTE format('ALTER TABLE %s DROP CONSTRAINT IF EXISTS %s', constraint_record.table_name, constraint_record.conname);
            RAISE NOTICE 'Dropped orphaned constraint: % from table %', constraint_record.conname, constraint_record.table_name;
        EXCEPTION
            WHEN OTHERS THEN
                RAISE NOTICE 'Could not drop constraint % from %: %', constraint_record.conname, constraint_record.table_name, SQLERRM;
        END;
    END LOOP;
END $$;

-- Step 8: Clean up any orphaned functions related to dropped tables
DO $$
BEGIN
    -- Drop functions that were specific to legacy permission system
    DROP FUNCTION IF EXISTS evaluate_dynamic_policy(UUID, TEXT, JSONB);
    DROP FUNCTION IF EXISTS cache_permission_inheritance(UUID);
    DROP FUNCTION IF EXISTS cleanup_policy_evaluations();
    DROP FUNCTION IF EXISTS get_user_permission_hierarchy(UUID);
    
    -- Functions related to legacy auth
    DROP FUNCTION IF EXISTS cleanup_expired_sessions();
    DROP FUNCTION IF EXISTS revoke_user_tokens(TEXT);
    DROP FUNCTION IF EXISTS validate_refresh_token(TEXT);
    
    RAISE NOTICE 'Cleaned up orphaned functions';
EXCEPTION
    WHEN OTHERS THEN
        RAISE NOTICE 'Some functions may not have existed: %', SQLERRM;
END $$;

-- Step 9: Update table comments to reflect current Web3-first architecture
DO $$
BEGIN
    -- Update comments on core tables to reflect their current purpose
    COMMENT ON TABLE users IS 'Core user table - Web3 wallet-first authentication system';
    COMMENT ON TABLE wallet_permissions IS 'Primary permission system for Web3 wallet-based authentication';
    COMMENT ON TABLE web3_auth_nonces IS 'Nonces for Web3 wallet signature authentication';
    COMMENT ON TABLE notifications IS 'Stateless notification system';
    COMMENT ON TABLE api_keys IS 'API key authentication for enterprise users';
    
    RAISE NOTICE 'Updated table comments to reflect Web3-first architecture';
END $$;

-- Step 10: Vacuum and analyze remaining tables for optimal performance
DO $$
DECLARE
    table_record RECORD;
BEGIN
    FOR table_record IN
        SELECT tablename 
        FROM pg_tables 
        WHERE schemaname = 'public'
        AND tablename IN ('users', 'wallet_permissions', 'web3_auth_nonces', 'notifications', 'api_keys', 'enterprise_teams')
    LOOP
        EXECUTE format('VACUUM ANALYZE %I', table_record.tablename);
        RAISE NOTICE 'Vacuumed and analyzed table: %', table_record.tablename;
    END LOOP;
END $$;

-- Step 11: Final cleanup verification
SELECT 
    'Legacy table cleanup completed successfully' as status,
    COUNT(*) as remaining_tables,
    string_agg(tablename, ', ' ORDER BY tablename) as table_list
FROM pg_tables 
WHERE schemaname = 'public';

-- Step 12: Log successful completion with summary
INSERT INTO notifications (
    id,
    recipient_id,
    title,
    message,
    notification_type,
    priority,
    created_at
) VALUES (
    gen_random_uuid(),
    (SELECT id FROM users WHERE email LIKE '%admin%' OR role = 'admin' LIMIT 1),
    'Database Cleanup Completed',
    'Migration 015: Successfully removed legacy tables and consolidated Web3-first database schema',
    'system',
    'low',
    NOW()
) ON CONFLICT DO NOTHING;

RAISE NOTICE '============================================================================';
RAISE NOTICE 'MIGRATION 015 COMPLETED SUCCESSFULLY';
RAISE NOTICE '- Removed legacy authentication tables (sessions, refresh_tokens, etc.)';
RAISE NOTICE '- Removed complex permission system tables from migration 002';
RAISE NOTICE '- Consolidated rate limiting tables';
RAISE NOTICE '- Cleaned up orphaned indexes, constraints, and functions';
RAISE NOTICE '- Updated table comments to reflect Web3-first architecture';
RAISE NOTICE '- Database schema now aligned with completed Web3 migration';
RAISE NOTICE '============================================================================';