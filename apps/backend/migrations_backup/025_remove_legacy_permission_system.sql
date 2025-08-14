-- Migration: Remove Legacy Permission System Tables (Phase 2)
-- This migration removes the old permission profile system that has been replaced
-- by the modern admin modules system introduced in migration 016

-- Log the start of phase 2 cleanup
DO $$
BEGIN
    RAISE NOTICE 'Starting Phase 2: Legacy Permission System Removal';
    RAISE NOTICE 'Removing permission_profiles, admin_permission_profile_assignments, assignment_audit_log';
    RAISE NOTICE 'Modern admin_modules system will remain as the primary permission system';
END $$;

-- Step 1: Verify modern admin system is in place before removing legacy system
DO $$
DECLARE
    admin_modules_count INT;
    active_admin_users INT;
BEGIN
    SELECT COUNT(*) INTO admin_modules_count FROM admin_modules WHERE is_active = true;
    SELECT COUNT(DISTINCT firebase_uid) INTO active_admin_users FROM user_admin_roles WHERE is_active = true;
    
    IF admin_modules_count = 0 THEN
        RAISE EXCEPTION 'Cannot remove legacy permission system: No admin modules found. Modern system not ready.';
    END IF;
    
    IF active_admin_users = 0 THEN
        RAISE EXCEPTION 'Cannot remove legacy permission system: No admin users assigned. Modern system not ready.';
    END IF;
    
    RAISE NOTICE 'Modern admin system verified: % modules, % admin users', admin_modules_count, active_admin_users;
END $$;

-- Step 2: Drop views that depend on legacy permission tables
DROP VIEW IF EXISTS permission_assignments_view CASCADE;
DROP VIEW IF EXISTS user_permissions_summary CASCADE;
DROP VIEW IF EXISTS profile_usage_stats CASCADE;

-- Step 3: Drop functions that reference legacy tables
DROP FUNCTION IF EXISTS get_user_permission_profiles(UUID) CASCADE;
DROP FUNCTION IF EXISTS assign_permission_profile(UUID, UUID, UUID, TEXT) CASCADE;
DROP FUNCTION IF EXISTS revoke_permission_profile(UUID, UUID, UUID) CASCADE;

-- Step 4: Remove foreign key constraints
ALTER TABLE admin_permission_profile_assignments DROP CONSTRAINT IF EXISTS admin_permission_profile_assignments_user_id_fkey CASCADE;
ALTER TABLE admin_permission_profile_assignments DROP CONSTRAINT IF EXISTS admin_permission_profile_assignments_permission_profile_id_fkey CASCADE;
ALTER TABLE admin_permission_profile_assignments DROP CONSTRAINT IF EXISTS admin_permission_profile_assignments_assigned_by_fkey CASCADE;
ALTER TABLE assignment_audit_log DROP CONSTRAINT IF EXISTS assignment_audit_log_assignment_id_fkey CASCADE;
ALTER TABLE assignment_audit_log DROP CONSTRAINT IF EXISTS assignment_audit_log_performed_by_fkey CASCADE;
ALTER TABLE permission_profiles DROP CONSTRAINT IF EXISTS permission_profiles_created_by_fkey CASCADE;

-- Step 5: Drop indexes for better performance during table removal
DROP INDEX IF EXISTS idx_admin_assignments_user_id;
DROP INDEX IF EXISTS idx_admin_assignments_profile_id;
DROP INDEX IF EXISTS idx_assignment_audit_assignment_id;
DROP INDEX IF EXISTS idx_assignment_audit_timestamp;
DROP INDEX IF EXISTS idx_permission_profiles_category;
DROP INDEX IF EXISTS idx_permission_profiles_status;
DROP INDEX IF EXISTS idx_permission_profiles_name;

-- Step 6: Migrate any remaining critical permission data to admin modules
-- Convert any active permission profile assignments to admin module assignments if needed
INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, granted_reason, is_active, created_at, updated_at)
SELECT DISTINCT
    u.firebase_uid,
    CASE 
        WHEN pp.category = 'admin' THEN 'system_admin'
        WHEN pp.category = 'analytics' THEN 'analytics_specialist'  
        WHEN pp.category = 'user_management' THEN 'user_operations'
        ELSE 'user_operations' -- Default fallback
    END as module_code,
    'legacy_migration_025',
    'Migrated from legacy permission profile: ' || pp.name,
    true,
    NOW(),
    NOW()
FROM admin_permission_profile_assignments appa
JOIN users u ON appa.user_id = u.id
JOIN permission_profiles pp ON appa.permission_profile_id = pp.id
WHERE appa.status = 'active' 
  AND (appa.expires_at IS NULL OR appa.expires_at > NOW())
  AND NOT EXISTS (
      -- Don't duplicate if user already has admin modules
      SELECT 1 FROM user_admin_roles uar 
      WHERE uar.firebase_uid = u.firebase_uid 
      AND uar.is_active = true
  )
ON CONFLICT (firebase_uid, module_code) DO NOTHING;

RAISE NOTICE 'Migrated any remaining active permission assignments to admin modules';

-- Step 7: Remove legacy permission system tables
-- These tables are replaced by the modern admin_modules system

-- Drop assignment audit log table (legacy audit)
DROP TABLE IF EXISTS assignment_audit_log CASCADE;
RAISE NOTICE 'Dropped table: assignment_audit_log';

-- Drop admin permission profile assignments table (legacy assignments)
DROP TABLE IF EXISTS admin_permission_profile_assignments CASCADE;
RAISE NOTICE 'Dropped table: admin_permission_profile_assignments';

-- Drop permission profiles table (legacy profiles)
DROP TABLE IF EXISTS permission_profiles CASCADE;
RAISE NOTICE 'Dropped table: permission_profiles';

-- Step 8: Clean up any remaining references in audit logs
UPDATE audit_logs 
SET details = details - 'permission_profile_id' - 'assignment_id' - 'profile_data'
WHERE details ? 'permission_profile_id' OR details ? 'assignment_id' OR details ? 'profile_data';

UPDATE audit_logs
SET metadata = metadata - 'permission_profile' - 'assignment_data'
WHERE metadata ? 'permission_profile' OR metadata ? 'assignment_data';

RAISE NOTICE 'Cleaned up legacy permission references in audit_logs';

-- Step 9: Remove legacy Casbin policies related to old permission system  
DELETE FROM casbin_rule 
WHERE v1 LIKE '%permission-profile%' 
   OR v1 LIKE '%admin-permission%'
   OR v2 LIKE '%permission-profile%' 
   OR v2 LIKE '%admin-permission%'
   OR v1 IN ('user-basic-001', 'user-premium-002', 'moderator-standard-003', 'admin-full-004');

RAISE NOTICE 'Cleaned up legacy Casbin rules';

-- Step 10: Update sessions table to remove any legacy permission references
DO $$ 
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'permission_profile_id'
    ) THEN
        ALTER TABLE sessions DROP COLUMN permission_profile_id;
        RAISE NOTICE 'Dropped permission_profile_id column from sessions';
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'legacy_permissions'
    ) THEN
        ALTER TABLE sessions DROP COLUMN legacy_permissions;
        RAISE NOTICE 'Dropped legacy_permissions column from sessions';
    END IF;
END $$;

-- Step 11: Create a summary view of the modern admin system
CREATE OR REPLACE VIEW admin_system_summary AS
SELECT 
    'admin_modules' as component,
    COUNT(*) as count,
    'Core admin functional modules' as description
FROM admin_modules WHERE is_active = true
UNION ALL
SELECT 
    'user_admin_roles' as component,
    COUNT(*) as count,
    'Admin role assignments' as description
FROM user_admin_roles WHERE is_active = true
UNION ALL
SELECT 
    'admin_module_permissions' as component,
    COUNT(*) as count,
    'Module permission definitions' as description
FROM admin_module_permissions
UNION ALL
SELECT 
    'admin_role_audit' as component,
    COUNT(*) as count,
    'Admin role audit entries' as description
FROM admin_role_audit;

-- Step 12: Verify cleanup and provide summary
DO $$
DECLARE
    removed_tables INT;
    admin_users_count INT;
    admin_modules_count INT;
    permission_definitions INT;
BEGIN
    -- Check that legacy tables are removed
    SELECT COUNT(*) INTO removed_tables
    FROM information_schema.tables 
    WHERE table_name IN ('permission_profiles', 'admin_permission_profile_assignments', 'assignment_audit_log')
    AND table_schema = 'public';
    
    -- Count active admin users and modules
    SELECT COUNT(DISTINCT firebase_uid) INTO admin_users_count FROM user_admin_roles WHERE is_active = true;
    SELECT COUNT(*) INTO admin_modules_count FROM admin_modules WHERE is_active = true;
    SELECT COUNT(*) INTO permission_definitions FROM admin_module_permissions;
    
    RAISE NOTICE 'Phase 2 Legacy Permission System Removal completed:';
    RAISE NOTICE '- Legacy permission tables removed: % (should be 0)', removed_tables;
    RAISE NOTICE '- Active admin users: %', admin_users_count;
    RAISE NOTICE '- Active admin modules: %', admin_modules_count;
    RAISE NOTICE '- Permission definitions: %', permission_definitions;
    RAISE NOTICE '- Modern admin system is now the sole permission authority';
    
    IF removed_tables = 0 THEN
        RAISE NOTICE '✓ Phase 2 cleanup successful - legacy permission system removed';
    ELSE
        RAISE NOTICE '⚠ Warning: Some legacy tables remain, check dependencies';
    END IF;
END $$;

-- Step 13: Record the migration
INSERT INTO schema_migrations (version, description, executed_at) 
VALUES ('025', 'Phase 2: Remove legacy permission system - permission_profiles, admin_permission_profile_assignments, assignment_audit_log', NOW())
ON CONFLICT (version) DO NOTHING;

-- Final notice
RAISE NOTICE 'Migration 025 completed: Legacy permission system removed, modern admin modules system is now primary';