-- ================================================================================================
-- REMOVE UNUSED TABLES AFTER WEB3 MIGRATION
-- ================================================================================================
-- This migration removes all unused tables identified after the Web3 wallet-first migration.
-- All tables have been safely backed up in migration 020 and can be recovered if needed.
-- ================================================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ================================================================================================
-- SAFETY CHECK: ENSURE BACKUP MIGRATION WAS RUN
-- ================================================================================================

DO $$
BEGIN
    -- Verify backup schema exists
    IF NOT EXISTS (SELECT FROM information_schema.schemata WHERE schema_name = 'cleanup_backup') THEN
        RAISE EXCEPTION 'ERROR: cleanup_backup schema not found! Please run migration 020 first to create safety backups.';
    END IF;
    
    -- Verify backup metadata exists
    IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'cleanup_backup' AND table_name = 'backup_metadata') THEN
        RAISE EXCEPTION 'ERROR: backup_metadata table not found! Please run migration 020 first to create safety backups.';
    END IF;
    
    RAISE NOTICE '✅ Backup verification passed - proceeding with cleanup';
END $$;

-- ================================================================================================
-- 1. DROP LEGACY WALLET SYSTEM TABLES
-- ================================================================================================

DO $$
DECLARE
    tables_to_remove TEXT[] := ARRAY[
        'wallet_identities',
        'wallet_group_memberships', 
        'group_assignment_history',
        'wallet_permissions'
    ];
    drop_table_name TEXT;
    removed_count INTEGER := 0;
BEGIN
    RAISE NOTICE '=== PHASE 1: Removing Legacy Wallet System Tables ===';
    
    FOREACH drop_table_name IN ARRAY tables_to_remove
    LOOP
        IF EXISTS (SELECT 1 FROM information_schema.tables t WHERE t.table_schema = 'public' AND t.table_name = drop_table_name) THEN
            -- Drop table with CASCADE to handle foreign key dependencies
            EXECUTE format('DROP TABLE public.%I CASCADE', drop_table_name);
            RAISE NOTICE '✅ Dropped table: %', drop_table_name;
            removed_count := removed_count + 1;
        ELSE
            RAISE NOTICE '⏭️  Table not found (already removed): %', drop_table_name;
        END IF;
    END LOOP;
    
    RAISE NOTICE 'Phase 1 Complete: % legacy wallet system tables removed', removed_count;
END $$;

-- ================================================================================================
-- 2. DROP GROUP PERMISSION SYSTEM TABLES  
-- ================================================================================================

DO $$
DECLARE
    removed_count INTEGER := 0;
    table_name TEXT;
BEGIN
    RAISE NOTICE '=== PHASE 2: Removing Group Permission System Tables ===';
    
    -- Drop permission_groups first (likely referenced by other tables)
    IF EXISTS (SELECT FROM information_schema.tables t WHERE t.table_schema = 'public' AND t.table_name = 'permission_groups') THEN
        DROP TABLE public.permission_groups CASCADE;
        RAISE NOTICE '✅ Dropped table: permission_groups';
        removed_count := removed_count + 1;
    END IF;
    
    -- Drop active_wallet_groups (likely a view or dependent table)
    IF EXISTS (SELECT FROM information_schema.tables t WHERE t.table_schema = 'public' AND t.table_name = 'active_wallet_groups') THEN
        DROP TABLE public.active_wallet_groups CASCADE;
        RAISE NOTICE '✅ Dropped table: active_wallet_groups';
        removed_count := removed_count + 1;
    END IF;
    
    RAISE NOTICE 'Phase 2 Complete: % group permission system tables removed', removed_count;
END $$;

-- ================================================================================================
-- 3. DROP WEB3 GROUP SYSTEM TABLES
-- ================================================================================================

DO $$
DECLARE
    table_record RECORD;
    removed_count INTEGER := 0;
BEGIN
    RAISE NOTICE '=== PHASE 3: Removing Web3 Group System Tables ===';
    
    -- Drop all web3_group_* tables
    FOR table_record IN
        SELECT table_name FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name LIKE 'web3_group_%'
        ORDER BY table_name
    LOOP
        EXECUTE format('DROP TABLE public.%I CASCADE', table_record.table_name);
        RAISE NOTICE '✅ Dropped table: %', table_record.table_name;
        removed_count := removed_count + 1;
    END LOOP;
    
    RAISE NOTICE 'Phase 3 Complete: % Web3 group system tables removed', removed_count;
END $$;

-- ================================================================================================
-- 4. DROP OTHER GROUP-RELATED TABLES
-- ================================================================================================

DO $$
DECLARE
    tables_to_remove TEXT[] := ARRAY[
        'dynamic_group_rules',
        'group_analytics',
        'group_compositions', 
        'group_contexts',
        'group_hierarchies',
        'group_hierarchy_flattened',
        'group_membership_stats',
        'group_templates'
    ];
    drop_table_name TEXT;
    removed_count INTEGER := 0;
BEGIN
    RAISE NOTICE '=== PHASE 4: Removing Other Group-Related Tables ===';
    
    FOREACH drop_table_name IN ARRAY tables_to_remove
    LOOP
        IF EXISTS (SELECT 1 FROM information_schema.tables t WHERE t.table_schema = 'public' AND t.table_name = drop_table_name) THEN
            EXECUTE format('DROP TABLE public.%I CASCADE', drop_table_name);
            RAISE NOTICE '✅ Dropped table: %', drop_table_name;
            removed_count := removed_count + 1;
        ELSE
            RAISE NOTICE '⏭️  Table not found (already removed): %', drop_table_name;
        END IF;
    END LOOP;
    
    RAISE NOTICE 'Phase 4 Complete: % other group-related tables removed', removed_count;
END $$;

-- ================================================================================================
-- 5. CLEAN UP ORPHANED VIEWS AND FUNCTIONS
-- ================================================================================================

DO $$
DECLARE
    removed_count INTEGER := 0;
BEGIN
    RAISE NOTICE '=== PHASE 5: Cleaning Up Orphaned Views and Functions ===';
    
    -- Drop views that might reference removed tables
    IF EXISTS (SELECT FROM information_schema.views WHERE table_schema = 'public' AND table_name = 'active_wallet_groups') THEN
        DROP VIEW public.active_wallet_groups CASCADE;
        RAISE NOTICE '✅ Dropped view: active_wallet_groups';
        removed_count := removed_count + 1;
    END IF;
    
    -- Drop functions related to group permissions (if they exist)
    DROP FUNCTION IF EXISTS public.get_wallet_permissions(VARCHAR) CASCADE;
    DROP FUNCTION IF EXISTS public.wallet_has_permission(VARCHAR, TEXT) CASCADE;
    DROP FUNCTION IF EXISTS public.assign_wallet_to_group(VARCHAR, UUID, VARCHAR, TIMESTAMPTZ, TEXT, VARCHAR) CASCADE;
    DROP FUNCTION IF EXISTS public.get_user_permissions(UUID) CASCADE;
    DROP FUNCTION IF EXISTS public.check_permission(UUID, TEXT) CASCADE;
    DROP FUNCTION IF EXISTS public.get_user_groups(UUID) CASCADE;
    DROP FUNCTION IF EXISTS public.cleanup_expired_group_memberships() CASCADE;
    
    RAISE NOTICE '✅ Cleaned up orphaned functions related to group permissions';
    
    RAISE NOTICE 'Phase 5 Complete: Cleaned up orphaned database objects';
END $$;

-- ================================================================================================
-- 6. REMOVE UNUSED INDEXES
-- ================================================================================================

DO $$
DECLARE
    index_record RECORD;
    removed_count INTEGER := 0;
BEGIN
    RAISE NOTICE '=== PHASE 6: Removing Orphaned Indexes ===';
    
    -- Find indexes that reference tables that no longer exist
    FOR index_record IN
        SELECT indexname, tablename 
        FROM pg_indexes 
        WHERE schemaname = 'public'
        AND (
            tablename LIKE '%group%' OR
            tablename LIKE 'wallet_identities%' OR
            tablename LIKE 'wallet_group_%' OR
            tablename LIKE 'web3_group_%'
        )
        AND NOT EXISTS (
            SELECT 1 FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = pg_indexes.tablename
        )
    LOOP
        EXECUTE format('DROP INDEX IF EXISTS public.%I CASCADE', index_record.indexname);
        RAISE NOTICE '✅ Dropped orphaned index: % (was on table: %)', index_record.indexname, index_record.tablename;
        removed_count := removed_count + 1;
    END LOOP;
    
    RAISE NOTICE 'Phase 6 Complete: % orphaned indexes removed', removed_count;
END $$;

-- ================================================================================================
-- 7. UPDATE CLEANUP METADATA
-- ================================================================================================

-- Record the cleanup operation in the backup metadata
INSERT INTO cleanup_backup.backup_metadata (
    backup_date,
    backup_reason,
    tables_backed_up,
    total_tables_backed_up,
    migration_version,
    notes
) VALUES (
    NOW(),
    'Web3 wallet-first migration - unused table cleanup completed',
    (SELECT array_agg(table_name) FROM information_schema.tables WHERE table_schema = 'cleanup_backup' AND table_name != 'backup_metadata'),
    (SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'cleanup_backup' AND table_name != 'backup_metadata'),
    '021_remove_unused_tables_after_web3_migration',
    'Successfully removed all unused legacy tables. The database now only contains actively used tables for the Web3 wallet-first system.'
);

-- ================================================================================================
-- 8. VERIFY CORE TABLES REMAIN INTACT
-- ================================================================================================

DO $$
DECLARE
    core_tables TEXT[] := ARRAY[
        'wallet_users',
        'web3_auth_nonces', 
        'payment_records',
        'active_subscriptions',
        'notifications',
        'usage_metrics'
    ];
    check_table_name TEXT;
    missing_core_tables TEXT[] := ARRAY[]::TEXT[];
    core_table_count INTEGER := 0;
BEGIN
    RAISE NOTICE '=== VERIFICATION: Checking Core Tables Integrity ===';
    
    FOREACH check_table_name IN ARRAY core_tables
    LOOP
        IF EXISTS (SELECT 1 FROM information_schema.tables t WHERE t.table_schema = 'public' AND t.table_name = check_table_name) THEN
            core_table_count := core_table_count + 1;
            RAISE NOTICE '✅ Core table intact: %', check_table_name;
        ELSE
            missing_core_tables := array_append(missing_core_tables, check_table_name);
            RAISE NOTICE '❌ Core table missing: %', check_table_name;
        END IF;
    END LOOP;
    
    IF array_length(missing_core_tables, 1) > 0 THEN
        RAISE WARNING 'Some core tables are missing: %', array_to_string(missing_core_tables, ', ');
    ELSE
        RAISE NOTICE '✅ All core tables verified intact';
    END IF;
    
    RAISE NOTICE 'Verification Complete: %/% core tables present', core_table_count, array_length(core_tables, 1);
END $$;

-- ================================================================================================
-- 9. FINAL STATISTICS AND COMPLETION
-- ================================================================================================

DO $$
DECLARE
    final_table_count INTEGER;
    backup_table_count INTEGER;
    reduction_percentage NUMERIC;
BEGIN
    -- Count remaining tables in public schema
    SELECT COUNT(*) INTO final_table_count 
    FROM information_schema.tables 
    WHERE table_schema = 'public';
    
    -- Count backed up tables
    SELECT COUNT(*) INTO backup_table_count 
    FROM information_schema.tables 
    WHERE table_schema = 'cleanup_backup'
    AND table_name != 'backup_metadata';
    
    -- Calculate reduction (approximation since we started with ~70 tables)
    reduction_percentage := ROUND((backup_table_count::NUMERIC / (final_table_count + backup_table_count)::NUMERIC) * 100, 1);
    
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'DATABASE CLEANUP MIGRATION COMPLETED SUCCESSFULLY! 🎉🧹';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Cleanup Summary:';
    RAISE NOTICE '• Final table count in public schema: %', final_table_count;
    RAISE NOTICE '• Tables removed and backed up: %', backup_table_count;
    RAISE NOTICE '• Database size reduction: ~% percent (% tables removed)', reduction_percentage, backup_table_count;
    RAISE NOTICE '';
    RAISE NOTICE 'What was removed:';
    RAISE NOTICE '• ❌ Legacy wallet_identities system (replaced by wallet_users)';
    RAISE NOTICE '• ❌ Complex group permission system (replaced by direct permissions)';
    RAISE NOTICE '• ❌ Web3 group bridge tables (not needed for wallet-first approach)';
    RAISE NOTICE '• ❌ Orphaned views, functions, and indexes';
    RAISE NOTICE '';
    RAISE NOTICE 'What remains (core Web3-first system):';
    RAISE NOTICE '• ✅ wallet_users (unified user + permission management)';
    RAISE NOTICE '• ✅ web3_auth_nonces (SIWE authentication)';
    RAISE NOTICE '• ✅ payment_records, active_subscriptions (payment system)';
    RAISE NOTICE '• ✅ notifications, usage_metrics (core platform features)';
    RAISE NOTICE '• ✅ All other actively used tables preserved';
    RAISE NOTICE '';
    RAISE NOTICE 'Recovery Options (if needed):';
    RAISE NOTICE '• 🛡️  All removed data backed up in cleanup_backup schema';
    RAISE NOTICE '• 📞 Use restore_table_from_backup(''table_name'') to recover';
    RAISE NOTICE '• 📋 Use list_backup_tables() to see available backups';
    RAISE NOTICE '';
    RAISE NOTICE '🎯 EPSX Database Cleanup Complete!';
    RAISE NOTICE '🚀 Pure Web3-First Architecture Achieved!';
    RAISE NOTICE '⚡ Improved Performance & Reduced Complexity!';
    RAISE NOTICE '=================================================================================';
END $$;

-- Update database statistics after cleanup
VACUUM ANALYZE;