-- Migration: Remove Complex Module System Tables (Phase 1)
-- This migration removes the over-engineered module system introduced in migration 005
-- that includes complex sub-modules, API keys, and usage tracking systems

-- Log the start of phase 1 cleanup
DO $$
BEGIN
    RAISE NOTICE 'Starting Phase 1: Complex Module System Removal';
    RAISE NOTICE 'This will remove 5 major tables with complex schemas';
END $$;

-- Step 1: Drop views that depend on the tables we're removing
DROP VIEW IF EXISTS user_module_access CASCADE;
DROP VIEW IF EXISTS api_key_access CASCADE;
DROP VIEW IF EXISTS module_usage_summary CASCADE;

-- Step 2: Drop functions that reference the tables
DROP FUNCTION IF EXISTS get_user_module_permissions(UUID) CASCADE;
DROP FUNCTION IF EXISTS check_api_key_access(VARCHAR, VARCHAR) CASCADE;
DROP FUNCTION IF EXISTS log_module_usage(UUID, UUID, VARCHAR, JSONB) CASCADE;

-- Step 3: Remove foreign key constraints first to avoid dependency issues
ALTER TABLE user_sub_module_assignments DROP CONSTRAINT IF EXISTS user_sub_module_assignments_user_id_fkey CASCADE;
ALTER TABLE user_sub_module_assignments DROP CONSTRAINT IF EXISTS user_sub_module_assignments_sub_module_id_fkey CASCADE;
ALTER TABLE module_usage_logs DROP CONSTRAINT IF EXISTS module_usage_logs_user_id_fkey CASCADE;
ALTER TABLE module_usage_logs DROP CONSTRAINT IF EXISTS module_usage_logs_api_key_id_fkey CASCADE;
ALTER TABLE module_usage_logs DROP CONSTRAINT IF EXISTS module_usage_logs_sub_module_id_fkey CASCADE;
ALTER TABLE module_assignment_audit DROP CONSTRAINT IF EXISTS module_assignment_audit_assignment_id_fkey CASCADE;
ALTER TABLE module_assignment_audit DROP CONSTRAINT IF EXISTS module_assignment_audit_user_id_fkey CASCADE;
ALTER TABLE module_assignment_audit DROP CONSTRAINT IF EXISTS module_assignment_audit_sub_module_id_fkey CASCADE;
ALTER TABLE api_keys DROP CONSTRAINT IF EXISTS api_keys_created_by_fkey CASCADE;
ALTER TABLE api_keys DROP CONSTRAINT IF EXISTS api_keys_managed_by_fkey CASCADE;

-- Step 4: Drop indexes for better performance
DROP INDEX IF EXISTS idx_sub_modules_category;
DROP INDEX IF EXISTS idx_sub_modules_status;
DROP INDEX IF EXISTS idx_sub_modules_created_at;
DROP INDEX IF EXISTS idx_user_assignments_user_id;
DROP INDEX IF EXISTS idx_user_assignments_module_id;
DROP INDEX IF EXISTS idx_user_assignments_status;
DROP INDEX IF EXISTS idx_user_assignments_expires_at;
DROP INDEX IF EXISTS idx_user_assignments_assigned_by;
DROP INDEX IF EXISTS idx_api_keys_status;
DROP INDEX IF EXISTS idx_api_keys_created_by;
DROP INDEX IF EXISTS idx_api_keys_expires_at;
DROP INDEX IF EXISTS idx_api_keys_last_used;
DROP INDEX IF EXISTS idx_usage_logs_user_id_timestamp;
DROP INDEX IF EXISTS idx_usage_logs_api_key_timestamp;
DROP INDEX IF EXISTS idx_usage_logs_module_timestamp;
DROP INDEX IF EXISTS idx_usage_logs_timestamp;
DROP INDEX IF EXISTS idx_usage_logs_billable;
DROP INDEX IF EXISTS idx_assignment_audit_user_id;
DROP INDEX IF EXISTS idx_assignment_audit_timestamp;
DROP INDEX IF EXISTS idx_assignment_audit_performed_by;

-- Step 5: Drop the complex module system tables
-- These tables were introduced in migration 005 but are over-engineered and unused

-- Drop module assignment audit table
DROP TABLE IF EXISTS module_assignment_audit CASCADE;
RAISE NOTICE 'Dropped table: module_assignment_audit';

-- Drop module usage logs table  
DROP TABLE IF EXISTS module_usage_logs CASCADE;
RAISE NOTICE 'Dropped table: module_usage_logs';

-- Drop API keys table (complex third-party access system)
DROP TABLE IF EXISTS api_keys CASCADE;
RAISE NOTICE 'Dropped table: api_keys';

-- Drop user module assignments table
DROP TABLE IF EXISTS user_sub_module_assignments CASCADE;  
RAISE NOTICE 'Dropped table: user_sub_module_assignments';

-- Drop sub-modules table (overly complex module definitions)
DROP TABLE IF EXISTS sub_modules CASCADE;
RAISE NOTICE 'Dropped table: sub_modules';

-- Step 6: Clean up any remaining references in audit logs
-- Remove JSONB fields that might reference the removed tables
UPDATE audit_logs 
SET details = details - 'sub_module_id' - 'api_key_id' - 'module_assignment_id'
WHERE details ? 'sub_module_id' OR details ? 'api_key_id' OR details ? 'module_assignment_id';

UPDATE audit_logs
SET metadata = metadata - 'sub_module_data' - 'api_key_data' - 'module_usage_data'
WHERE metadata ? 'sub_module_data' OR metadata ? 'api_key_data' OR metadata ? 'module_usage_data';

RAISE NOTICE 'Cleaned up references in audit_logs table';

-- Step 7: Remove any Casbin policies that might reference the removed tables
DELETE FROM casbin_rule 
WHERE v1 LIKE '%sub_module%' 
   OR v1 LIKE '%api_key%' 
   OR v1 LIKE '%module_usage%'
   OR v2 LIKE '%sub_module%' 
   OR v2 LIKE '%api_key%' 
   OR v2 LIKE '%module_usage%';

RAISE NOTICE 'Cleaned up Casbin rules referencing removed tables';

-- Step 8: Update users table to remove any module-related fields if they exist
DO $$ 
BEGIN
    -- Remove module-related columns from users table if they exist
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'assigned_modules'
    ) THEN
        ALTER TABLE users DROP COLUMN assigned_modules;
        RAISE NOTICE 'Dropped assigned_modules column from users table';
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'module_access_level'
    ) THEN
        ALTER TABLE users DROP COLUMN module_access_level;
        RAISE NOTICE 'Dropped module_access_level column from users table';
    END IF;
END $$;

-- Step 9: Verify cleanup and provide summary
DO $$
DECLARE
    remaining_tables INT;
    total_users INT;
    admin_modules INT;
BEGIN
    -- Count remaining module-related tables
    SELECT COUNT(*) INTO remaining_tables
    FROM information_schema.tables 
    WHERE table_name IN ('sub_modules', 'user_sub_module_assignments', 'api_keys', 'module_usage_logs', 'module_assignment_audit')
    AND table_schema = 'public';
    
    -- Get current user count
    SELECT COUNT(*) INTO total_users FROM users;
    
    -- Get admin modules count (should remain)
    SELECT COUNT(*) INTO admin_modules FROM admin_modules WHERE is_active = true;
    
    RAISE NOTICE 'Phase 1 Complex Module System Removal completed:';
    RAISE NOTICE '- Removed complex module tables: % (should be 0)', remaining_tables;
    RAISE NOTICE '- Total users preserved: %', total_users;
    RAISE NOTICE '- Admin modules preserved: %', admin_modules;
    RAISE NOTICE '- Modern admin module system remains active';
    RAISE NOTICE '- All data backed up in migration 023';
    
    IF remaining_tables = 0 THEN
        RAISE NOTICE '✓ Phase 1 cleanup successful - all complex module tables removed';
    ELSE
        RAISE NOTICE '⚠ Warning: Some tables were not removed, check dependencies';
    END IF;
END $$;

-- Step 10: Record the migration
INSERT INTO schema_migrations (version, description, executed_at) 
VALUES ('024', 'Phase 1: Remove complex module system tables - sub_modules, user_sub_module_assignments, api_keys, module_usage_logs, module_assignment_audit', NOW())
ON CONFLICT (version) DO NOTHING;

-- Final notice
RAISE NOTICE 'Migration 024 completed: Complex module system removed, modern admin system preserved';