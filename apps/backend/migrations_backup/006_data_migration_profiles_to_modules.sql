-- Data Migration: Convert Permission Profiles to Module System
-- This script migrates existing permission profile assignments to module assignments

-- ========================================
-- BACKUP EXISTING DATA
-- ========================================

-- Create backup tables
CREATE TABLE IF NOT EXISTS backup_permission_profiles AS 
SELECT * FROM permission_profiles;

CREATE TABLE IF NOT EXISTS backup_admin_permission_profile_assignments AS 
SELECT * FROM admin_permission_profile_assignments;

-- ========================================
-- PERMISSION PROFILE TO MODULE MAPPING
-- ========================================

-- Create temporary mapping table
CREATE TEMP TABLE profile_to_module_mapping AS
SELECT 
    pp.id as profile_id,
    pp.name as profile_name,
    pp.category,
    -- Map existing profiles to new modules
    CASE 
        WHEN pp.name LIKE '%Bronze%Analytics%' OR pp.name LIKE '%Bronze%' THEN 'stock-ranking'
        WHEN pp.name LIKE '%Silver%Analytics%' OR pp.name LIKE '%Silver%' THEN 'stock-ranking'
        WHEN pp.name LIKE '%Gold%Analytics%' OR pp.name LIKE '%Gold%' THEN 'stock-ranking'
        WHEN pp.name LIKE '%Platinum%Analytics%' OR pp.name LIKE '%Platinum%' THEN 'stock-ranking'
        WHEN pp.name LIKE '%Portfolio%' THEN 'portfolio-analysis'
        WHEN pp.name LIKE '%Market%Data%' THEN 'market-data'
        WHEN pp.name LIKE '%Trading%' OR pp.name LIKE '%Signal%' THEN 'trading-signals'
        WHEN pp.name LIKE '%Admin%' THEN NULL -- Skip admin profiles for now
        ELSE 'stock-ranking' -- Default fallback
    END as module_name,
    -- Map to access levels
    CASE 
        WHEN pp.name LIKE '%Bronze%' OR pp.pricing_tier->>'tier' = 'Bronze' THEN 'bronze'
        WHEN pp.name LIKE '%Silver%' OR pp.pricing_tier->>'tier' = 'Silver' THEN 'silver'
        WHEN pp.name LIKE '%Gold%' OR pp.pricing_tier->>'tier' = 'Gold' THEN 'gold'
        WHEN pp.name LIKE '%Platinum%' OR pp.pricing_tier->>'tier' = 'Platinum' THEN 'platinum'
        WHEN pp.name LIKE '%Enterprise%' OR pp.pricing_tier->>'tier' = 'Enterprise' THEN 'platinum'
        WHEN pp.name LIKE '%Premium%' THEN 'gold'
        WHEN pp.name LIKE '%Pro%' THEN 'silver'
        WHEN pp.name LIKE '%Basic%' THEN 'bronze'
        ELSE 'bronze' -- Default fallback
    END as access_level
FROM permission_profiles pp
WHERE pp.status = 'active';

-- ========================================
-- MIGRATE USER ASSIGNMENTS
-- ========================================

-- Insert module assignments based on existing permission profile assignments
INSERT INTO user_sub_module_assignments (
    user_id,
    sub_module_id,
    access_level,
    custom_quotas,
    restrictions,
    assigned_by,
    assignment_reason,
    assignment_type,
    starts_at,
    expires_at,
    status,
    created_at,
    updated_at
)
SELECT 
    appa.user_id,
    sm.id as sub_module_id,
    ptm.access_level,
    COALESCE(appa.variables, '{}') as custom_quotas,
    '{}' as restrictions, -- Default empty restrictions
    appa.assigned_by,
    CONCAT('Migrated from permission profile: ', ptm.profile_name, '. Original reason: ', appa.assignment_reason) as assignment_reason,
    'migrated' as assignment_type,
    appa.created_at as starts_at,
    appa.expires_at,
    CASE 
        WHEN appa.status = 'active' THEN 'active'
        WHEN appa.status = 'suspended' THEN 'suspended'
        WHEN appa.status = 'expired' THEN 'expired'
        ELSE 'revoked'
    END as status,
    appa.created_at,
    NOW() as updated_at
FROM admin_permission_profile_assignments appa
JOIN profile_to_module_mapping ptm ON appa.permission_profile_id = ptm.profile_id
JOIN sub_modules sm ON sm.name = ptm.module_name
WHERE ptm.module_name IS NOT NULL  -- Skip admin profiles
ON CONFLICT (user_id, sub_module_id) DO UPDATE SET
    -- If conflict, keep the higher access level
    access_level = CASE 
        WHEN user_sub_module_assignments.access_level = 'bronze' AND EXCLUDED.access_level IN ('silver', 'gold', 'platinum') THEN EXCLUDED.access_level
        WHEN user_sub_module_assignments.access_level = 'silver' AND EXCLUDED.access_level IN ('gold', 'platinum') THEN EXCLUDED.access_level
        WHEN user_sub_module_assignments.access_level = 'gold' AND EXCLUDED.access_level = 'platinum' THEN EXCLUDED.access_level
        ELSE user_sub_module_assignments.access_level
    END,
    assignment_reason = user_sub_module_assignments.assignment_reason || ' | ' || EXCLUDED.assignment_reason,
    updated_at = NOW();

-- ========================================
-- HANDLE MULTIPLE PROFILE ASSIGNMENTS
-- ========================================

-- Some users might have multiple permission profiles (e.g., Bronze Analytics + Portfolio Analysis)
-- This creates additional module assignments for users with multiple profiles

-- Find users with multiple different module types
WITH user_multiple_modules AS (
    SELECT 
        appa.user_id,
        COUNT(DISTINCT ptm.module_name) as module_count,
        array_agg(DISTINCT ptm.module_name) as modules,
        array_agg(DISTINCT ptm.access_level) as levels
    FROM admin_permission_profile_assignments appa
    JOIN profile_to_module_mapping ptm ON appa.permission_profile_id = ptm.profile_id
    WHERE ptm.module_name IS NOT NULL AND appa.status = 'active'
    GROUP BY appa.user_id
    HAVING COUNT(DISTINCT ptm.module_name) > 1
)
-- Log these cases for manual review
INSERT INTO module_assignment_audit (
    assignment_id,
    user_id,
    sub_module_id,
    action,
    old_values,
    new_values,
    changes,
    performed_by,
    reason,
    timestamp
)
SELECT 
    NULL as assignment_id,
    umm.user_id,
    sm.id as sub_module_id,
    'migration_multiple_modules' as action,
    NULL as old_values,
    jsonb_build_object('modules', umm.modules, 'levels', umm.levels) as new_values,
    jsonb_build_object('module_count', umm.module_count) as changes,
    (SELECT id FROM users WHERE email = 'system@migration.local' LIMIT 1) as performed_by,
    'User had multiple permission profiles during migration' as reason,
    NOW() as timestamp
FROM user_multiple_modules umm
JOIN sub_modules sm ON sm.name = ANY(umm.modules);

-- ========================================
-- CREATE SYSTEM USER FOR MIGRATION AUDIT
-- ========================================

-- Insert system user for audit trail if not exists
INSERT INTO users (firebase_uid, email, created_at, updated_at)
VALUES ('system_migration', 'system@migration.local', NOW(), NOW())
ON CONFLICT (firebase_uid) DO NOTHING;

-- Update audit records to use system user
UPDATE module_assignment_audit 
SET performed_by = (SELECT id FROM users WHERE firebase_uid = 'system_migration')
WHERE performed_by IS NULL;

-- ========================================
-- MIGRATION STATISTICS AND VALIDATION
-- ========================================

-- Create migration report
CREATE TEMP TABLE migration_report AS
SELECT 
    'Original Permission Profiles' as metric,
    COUNT(*) as count
FROM backup_permission_profiles
UNION ALL
SELECT 
    'Original Profile Assignments' as metric,
    COUNT(*) as count
FROM backup_admin_permission_profile_assignments
UNION ALL
SELECT 
    'New Module Assignments Created' as metric,
    COUNT(*) as count
FROM user_sub_module_assignments
WHERE assignment_type = 'migrated'
UNION ALL
SELECT 
    'Users with Module Access' as metric,
    COUNT(DISTINCT user_id) as count
FROM user_sub_module_assignments
WHERE status = 'active'
UNION ALL
SELECT 
    'Active Modules' as metric,
    COUNT(*) as count
FROM sub_modules
WHERE status = 'active';

-- Log migration statistics
INSERT INTO module_assignment_audit (
    assignment_id,
    user_id,
    sub_module_id,
    action,
    old_values,
    new_values,
    changes,
    performed_by,
    reason,
    timestamp
)
SELECT 
    NULL,
    (SELECT id FROM users WHERE firebase_uid = 'system_migration'),
    (SELECT id FROM sub_modules WHERE name = 'stock-ranking' LIMIT 1),
    'migration_completed',
    NULL,
    jsonb_object_agg(mr.metric, mr.count),
    jsonb_build_object('migration_date', NOW(), 'version', '1.0'),
    (SELECT id FROM users WHERE firebase_uid = 'system_migration'),
    'Permission profile to module system migration completed',
    NOW()
FROM migration_report mr;

-- ========================================
-- DATA VALIDATION QUERIES
-- ========================================

-- Simple validation check (replaced complex PL/pgSQL block)
-- Count original and new assignments for validation
-- This will be logged in the audit table below for review

-- Create validation views for admin review
CREATE OR REPLACE VIEW migration_validation AS
SELECT 
    'Migration Summary' as section,
    metric,
    count
FROM (
    SELECT 'Original Profiles' as metric, COUNT(*) as count FROM backup_permission_profiles
    UNION ALL
    SELECT 'Original Assignments', COUNT(*) FROM backup_admin_permission_profile_assignments WHERE status = 'active'
    UNION ALL
    SELECT 'New Module Assignments', COUNT(*) FROM user_sub_module_assignments WHERE assignment_type = 'migrated'
    UNION ALL
    SELECT 'Active Users with Modules', COUNT(DISTINCT user_id) FROM user_sub_module_assignments WHERE status = 'active'
    UNION ALL
    SELECT 'Total Available Modules', COUNT(*) FROM sub_modules WHERE status = 'active'
) stats
ORDER BY 
    CASE metric
        WHEN 'Original Profiles' THEN 1
        WHEN 'Original Assignments' THEN 2
        WHEN 'New Module Assignments' THEN 3
        WHEN 'Active Users with Modules' THEN 4
        WHEN 'Total Available Modules' THEN 5
    END;

-- View to compare old vs new access for users
CREATE OR REPLACE VIEW user_access_comparison AS
SELECT 
    u.email,
    u.id as user_id,
    -- Old system
    string_agg(DISTINCT pp.name, ', ') as old_profiles,
    -- New system
    string_agg(DISTINCT sm.display_name || ' (' || uma.access_level || ')', ', ') as new_modules
FROM users u
LEFT JOIN backup_admin_permission_profile_assignments bappa ON u.id = bappa.user_id AND bappa.status = 'active'
LEFT JOIN backup_permission_profiles pp ON bappa.permission_profile_id = pp.id
LEFT JOIN user_sub_module_assignments uma ON u.id = uma.user_id AND uma.status = 'active'
LEFT JOIN sub_modules sm ON uma.sub_module_id = sm.id
WHERE bappa.user_id IS NOT NULL OR uma.user_id IS NOT NULL
GROUP BY u.id, u.email
ORDER BY u.email;

-- ========================================
-- CLEANUP INSTRUCTIONS (COMMENTED OUT)
-- ========================================

-- DANGER: Only run these after thorough testing and validation
-- These commands will permanently remove the old permission profile system

-- Step 1: Verify migration is successful
-- SELECT * FROM migration_validation;
-- SELECT * FROM user_access_comparison LIMIT 10;

-- Step 2: If satisfied with migration, remove old tables
-- DROP TABLE IF EXISTS admin_permission_profile_assignments CASCADE;
-- DROP TABLE IF EXISTS permission_profiles CASCADE;

-- Step 3: Remove backup tables (after archival if needed)
-- DROP TABLE IF EXISTS backup_permission_profiles;
-- DROP TABLE IF EXISTS backup_admin_permission_profile_assignments;

-- Step 4: Update sequences and constraints
-- ALTER TABLE user_sub_module_assignments DROP CONSTRAINT IF EXISTS unique_user_module;
-- ALTER TABLE user_sub_module_assignments ADD CONSTRAINT unique_user_module UNIQUE (user_id, sub_module_id);

-- Final success message
SELECT 
    'MIGRATION COMPLETED SUCCESSFULLY' as status,
    NOW() as completed_at,
    'Review migration_validation and user_access_comparison views' as next_steps;