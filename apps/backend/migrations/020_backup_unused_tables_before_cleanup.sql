-- ================================================================================================
-- BACKUP UNUSED TABLES BEFORE CLEANUP MIGRATION  
-- ================================================================================================
-- This migration creates safety backups of all unused tables before they are removed
-- in the next migration. This allows for data recovery if needed.
-- ================================================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ================================================================================================
-- 1. CREATE CLEANUP_BACKUP SCHEMA
-- ================================================================================================

-- Create backup schema for storing removed tables
CREATE SCHEMA IF NOT EXISTS cleanup_backup;

-- Add description
COMMENT ON SCHEMA cleanup_backup IS 'Backup schema containing tables removed during Web3 cleanup migration';

-- ================================================================================================
-- 2. BACKUP LEGACY WALLET SYSTEM TABLES
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE 'Starting backup of legacy wallet system tables...';
    
    -- Backup wallet_identities if it exists
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'wallet_identities') THEN
        EXECUTE 'CREATE TABLE cleanup_backup.wallet_identities AS SELECT * FROM public.wallet_identities';
        RAISE NOTICE 'Backed up wallet_identities table';
    ELSE
        RAISE NOTICE 'wallet_identities table not found in public schema';
    END IF;
    
    -- Backup wallet_group_memberships if it exists
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'wallet_group_memberships') THEN
        EXECUTE 'CREATE TABLE cleanup_backup.wallet_group_memberships AS SELECT * FROM public.wallet_group_memberships';
        RAISE NOTICE 'Backed up wallet_group_memberships table';
    ELSE
        RAISE NOTICE 'wallet_group_memberships table not found in public schema';
    END IF;
    
    -- Backup group_assignment_history if it exists
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'group_assignment_history') THEN
        EXECUTE 'CREATE TABLE cleanup_backup.group_assignment_history AS SELECT * FROM public.group_assignment_history';
        RAISE NOTICE 'Backed up group_assignment_history table';
    ELSE
        RAISE NOTICE 'group_assignment_history table not found in public schema';
    END IF;
    
    -- Backup wallet_permissions if it exists (replaced by wallet_users.permissions)
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'wallet_permissions') THEN
        EXECUTE 'CREATE TABLE cleanup_backup.wallet_permissions AS SELECT * FROM public.wallet_permissions';
        RAISE NOTICE 'Backed up wallet_permissions table';
    ELSE
        RAISE NOTICE 'wallet_permissions table not found in public schema';
    END IF;
END $$;

-- ================================================================================================
-- 3. BACKUP GROUP PERMISSION SYSTEM TABLES
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE 'Starting backup of group permission system tables...';
    
    -- Backup permission_groups if it exists
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'permission_groups') THEN
        EXECUTE 'CREATE TABLE cleanup_backup.permission_groups AS SELECT * FROM public.permission_groups';
        RAISE NOTICE 'Backed up permission_groups table';
    ELSE
        RAISE NOTICE 'permission_groups table not found in public schema';
    END IF;
    
    -- Backup active_wallet_groups if it exists
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'active_wallet_groups') THEN
        EXECUTE 'CREATE TABLE cleanup_backup.active_wallet_groups AS SELECT * FROM public.active_wallet_groups';
        RAISE NOTICE 'Backed up active_wallet_groups table';
    ELSE
        RAISE NOTICE 'active_wallet_groups table not found in public schema';
    END IF;
END $$;

-- ================================================================================================
-- 4. BACKUP WEB3 GROUP SYSTEM TABLES
-- ================================================================================================

DO $$
DECLARE
    table_record RECORD;
BEGIN
    RAISE NOTICE 'Starting backup of Web3 group system tables...';
    
    -- Backup all web3_group_* tables
    FOR table_record IN
        SELECT table_name FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name LIKE 'web3_group_%'
    LOOP
        EXECUTE format('CREATE TABLE cleanup_backup.%I AS SELECT * FROM public.%I', 
                      table_record.table_name, table_record.table_name);
        RAISE NOTICE 'Backed up % table', table_record.table_name;
    END LOOP;
END $$;

-- ================================================================================================
-- 5. BACKUP OTHER GROUP-RELATED TABLES
-- ================================================================================================

DO $$
DECLARE
    tables_to_backup TEXT[] := ARRAY[
        'dynamic_group_rules',
        'group_analytics', 
        'group_compositions',
        'group_contexts',
        'group_hierarchies',
        'group_hierarchy_flattened',
        'group_membership_stats',
        'group_templates'
    ];
    backup_table_name TEXT;
BEGIN
    RAISE NOTICE 'Starting backup of other group-related tables...';
    
    FOREACH backup_table_name IN ARRAY tables_to_backup
    LOOP
        IF EXISTS (SELECT 1 FROM information_schema.tables t WHERE t.table_schema = 'public' AND t.table_name = backup_table_name) THEN
            EXECUTE format('CREATE TABLE cleanup_backup.%I AS SELECT * FROM public.%I', backup_table_name, backup_table_name);
            RAISE NOTICE 'Backed up % table', backup_table_name;
        ELSE
            RAISE NOTICE '% table not found in public schema', backup_table_name;
        END IF;
    END LOOP;
END $$;

-- ================================================================================================
-- 6. CREATE BACKUP METADATA TABLE
-- ================================================================================================

-- Create metadata table to track what was backed up
CREATE TABLE cleanup_backup.backup_metadata (
    backup_date TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    backup_reason TEXT DEFAULT 'Web3 wallet-first migration cleanup' NOT NULL,
    tables_backed_up TEXT[] NOT NULL,
    total_tables_backed_up INTEGER NOT NULL,
    migration_version VARCHAR(50) DEFAULT '020_backup_unused_tables_before_cleanup' NOT NULL,
    notes TEXT
);

-- Insert backup metadata
DO $$
DECLARE
    backed_up_tables TEXT[];
    table_count INTEGER := 0;
BEGIN
    -- Get list of all tables in cleanup_backup schema (excluding metadata)
    SELECT array_agg(table_name) INTO backed_up_tables
    FROM information_schema.tables 
    WHERE table_schema = 'cleanup_backup' 
    AND table_name != 'backup_metadata';
    
    -- Count tables
    SELECT array_length(backed_up_tables, 1) INTO table_count;
    table_count := COALESCE(table_count, 0);
    
    -- Insert metadata
    INSERT INTO cleanup_backup.backup_metadata (
        backup_date,
        backup_reason,
        tables_backed_up,
        total_tables_backed_up,
        migration_version,
        notes
    ) VALUES (
        NOW(),
        'Pre-cleanup backup for Web3 wallet-first migration - removed unused group permission system and legacy wallet tables',
        COALESCE(backed_up_tables, ARRAY[]::TEXT[]),
        table_count,
        '020_backup_unused_tables_before_cleanup',
        'These tables were identified as unused after the Web3 wallet-first migration. The wallet_users table now handles all user and permission management directly.'
    );
    
    RAISE NOTICE 'Created backup metadata with % tables backed up', table_count;
END $$;

-- ================================================================================================
-- 7. CREATE RECOVERY FUNCTIONS
-- ================================================================================================

-- Function to restore a backed up table if needed
CREATE OR REPLACE FUNCTION restore_table_from_backup(
    table_name_to_restore TEXT,
    overwrite_existing BOOLEAN DEFAULT FALSE
) RETURNS BOOLEAN AS $$
DECLARE
    backup_exists BOOLEAN := FALSE;
    public_exists BOOLEAN := FALSE;
BEGIN
    -- Check if backup exists
    SELECT EXISTS(
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'cleanup_backup' AND table_name = table_name_to_restore
    ) INTO backup_exists;
    
    IF NOT backup_exists THEN
        RAISE EXCEPTION 'No backup found for table: %', table_name_to_restore;
    END IF;
    
    -- Check if table already exists in public schema
    SELECT EXISTS(
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'public' AND table_name = table_name_to_restore
    ) INTO public_exists;
    
    IF public_exists AND NOT overwrite_existing THEN
        RAISE EXCEPTION 'Table % already exists in public schema. Use overwrite_existing=TRUE to replace it.', table_name_to_restore;
    END IF;
    
    -- Drop existing table if overwrite is enabled
    IF public_exists AND overwrite_existing THEN
        EXECUTE format('DROP TABLE public.%I CASCADE', table_name_to_restore);
        RAISE NOTICE 'Dropped existing table: public.%', table_name_to_restore;
    END IF;
    
    -- Restore table from backup
    EXECUTE format('CREATE TABLE public.%I AS SELECT * FROM cleanup_backup.%I', 
                   table_name_to_restore, table_name_to_restore);
    
    RAISE NOTICE 'Successfully restored table: % from backup', table_name_to_restore;
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Function to list all available backups
CREATE OR REPLACE FUNCTION list_backup_tables() RETURNS TABLE(
    table_name TEXT,
    row_count BIGINT,
    table_size TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        t.table_name::TEXT,
        0::BIGINT as row_count, -- We'll update this in a separate query if needed
        pg_size_pretty(pg_total_relation_size(('cleanup_backup.' || t.table_name)::regclass))::TEXT as table_size
    FROM information_schema.tables t
    WHERE t.table_schema = 'cleanup_backup'
    AND t.table_name != 'backup_metadata'
    ORDER BY t.table_name;
END;
$$ LANGUAGE plpgsql;

-- ================================================================================================
-- 8. SET PERMISSIONS AND ANALYZE
-- ================================================================================================

-- Grant appropriate permissions to the cleanup_backup schema
-- (Adjust based on your user roles)
GRANT USAGE ON SCHEMA cleanup_backup TO postgres;
GRANT SELECT ON ALL TABLES IN SCHEMA cleanup_backup TO postgres;

-- Analyze all backed up tables for performance
DO $$
DECLARE
    backup_table RECORD;
BEGIN
    FOR backup_table IN 
        SELECT table_name FROM information_schema.tables 
        WHERE table_schema = 'cleanup_backup'
    LOOP
        EXECUTE format('ANALYZE cleanup_backup.%I', backup_table.table_name);
    END LOOP;
END $$;

-- ================================================================================================
-- 9. VERIFICATION AND COMPLETION MESSAGE
-- ================================================================================================

DO $$
DECLARE
    backup_count INTEGER;
    backup_tables TEXT[];
BEGIN
    -- Count backed up tables
    SELECT COUNT(*) INTO backup_count 
    FROM information_schema.tables 
    WHERE table_schema = 'cleanup_backup'
    AND table_name != 'backup_metadata';
    
    -- Get list of backed up tables
    SELECT array_agg(table_name ORDER BY table_name) INTO backup_tables
    FROM information_schema.tables 
    WHERE table_schema = 'cleanup_backup'
    AND table_name != 'backup_metadata';
    
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'BACKUP MIGRATION COMPLETED SUCCESSFULLY! 🛡️';  
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Backup Summary:';
    RAISE NOTICE '• Tables backed up: %', backup_count;
    RAISE NOTICE '• Backup schema: cleanup_backup';
    RAISE NOTICE '• Backup date: %', NOW();
    RAISE NOTICE '';
    RAISE NOTICE 'Backed up tables:';
    
    -- Show backed up tables
    IF backup_tables IS NOT NULL THEN
        FOR i IN 1..array_length(backup_tables, 1) LOOP
            RAISE NOTICE '  • %', backup_tables[i];
        END LOOP;
    ELSE
        RAISE NOTICE '  • No tables were backed up (all tables may have already been removed)';
    END IF;
    
    RAISE NOTICE '';
    RAISE NOTICE 'Recovery Options:';
    RAISE NOTICE '• Use restore_table_from_backup(''table_name'') to restore individual tables';
    RAISE NOTICE '• Use list_backup_tables() to see all available backups';
    RAISE NOTICE '• All backup data preserved in cleanup_backup schema';
    RAISE NOTICE '';
    RAISE NOTICE '✅ Ready for cleanup migration 021!';
    RAISE NOTICE '=================================================================================';
END $$;