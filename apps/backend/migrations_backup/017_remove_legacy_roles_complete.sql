-- Complete Legacy Role Removal and Admin Module Migration
-- This migration completely removes all legacy role-based authentication
-- and ensures all users have proper admin module assignments

-- Step 1: Ensure jesadakorn.kirtnu@gmail.com has all admin modules
INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, granted_reason, is_active)
SELECT 
    'KLiZ6jiuzchxUppd60IdBD5WS4U2',
    am.module_code,
    'system',
    'Admin module migration - full access for primary admin',
    true
FROM admin_modules am
ON CONFLICT (firebase_uid, module_code) DO UPDATE SET
    granted_by = 'system',
    granted_reason = 'Admin module migration - full access for primary admin',
    is_active = true,
    updated_at = NOW();

-- Step 2: Migrate any remaining users with legacy admin roles to proper admin modules
-- Find users who might have admin access but no admin modules assigned
INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, granted_reason, is_active)
SELECT DISTINCT
    u.firebase_uid,
    'user_operations', -- Give basic user management access
    'legacy_migration',
    'Migrated from legacy role system',
    true
FROM users u
WHERE u.firebase_uid NOT IN (
    SELECT DISTINCT firebase_uid FROM user_admin_roles WHERE is_active = true
)
-- Only include users that might have had admin access (customize this condition based on your data)
AND (
    u.email LIKE '%@admin.%' OR 
    u.email LIKE '%admin%' OR
    u.created_at < '2024-01-01' -- Early users might have been admins
);

-- Step 3: Remove any legacy role columns from users table
-- First check if role column exists and remove it
DO $$ 
BEGIN
    -- Check if role column exists before dropping it
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'role'
    ) THEN
        ALTER TABLE users DROP COLUMN role;
        RAISE NOTICE 'Dropped role column from users table';
    END IF;

    -- Check if legacy_role column exists and remove it  
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'legacy_role'
    ) THEN
        ALTER TABLE users DROP COLUMN legacy_role;
        RAISE NOTICE 'Dropped legacy_role column from users table';
    END IF;

    -- Check if user_role column exists and remove it
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'user_role'
    ) THEN
        ALTER TABLE users DROP COLUMN user_role;
        RAISE NOTICE 'Dropped user_role column from users table';
    END IF;
END $$;

-- Step 4: Remove any legacy role-based Casbin policies
-- Clean up any policies that reference legacy roles
DELETE FROM casbin_rule 
WHERE (v1 LIKE '%admin-full-004%' 
    OR v1 LIKE '%moderator-standard-003%' 
    OR v1 LIKE '%user-basic-001%'
    OR v1 LIKE '%user-premium-002%'
    OR v2 LIKE '%admin-full-004%'
    OR v2 LIKE '%moderator-standard-003%'
    OR v2 LIKE '%user-basic-001%'
    OR v2 LIKE '%user-premium-002%');

-- Step 5: Create modern admin module-based Casbin policies
-- Add policies for admin module system
INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5) VALUES
-- System admin can access everything
('p', 'system_admin', '*', '*', '', '', ''),
-- User operations module permissions
('p', 'user_operations', 'users', 'read', '', '', ''),
('p', 'user_operations', 'users', 'write', '', '', ''),
-- Analytics module permissions
('p', 'analytics_specialist', 'analytics', 'read', '', '', ''),
-- Billing module permissions
('p', 'billing_admin', 'billing', 'read', '', '', ''),
('p', 'billing_admin', 'billing', 'write', '', '', ''),
-- Permission admin module
('p', 'permission_admin', 'permissions', 'read', '', '', ''),
('p', 'permission_admin', 'permissions', 'write', '', '', '')
ON CONFLICT DO NOTHING;

-- Step 6: Update session table to remove legacy role references
-- Remove any role-based session data if it exists
DO $$ 
BEGIN
    -- Check if sessions table has role column
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'role'
    ) THEN
        ALTER TABLE sessions DROP COLUMN role;
        RAISE NOTICE 'Dropped role column from sessions table';
    END IF;
END $$;

-- Step 7: Verify migration completed successfully
-- Count users with admin modules
DO $$
DECLARE
    admin_users_count INT;
    total_modules_assigned INT;
BEGIN
    SELECT COUNT(DISTINCT firebase_uid) INTO admin_users_count 
    FROM user_admin_roles WHERE is_active = true;
    
    SELECT COUNT(*) INTO total_modules_assigned 
    FROM user_admin_roles WHERE is_active = true;
    
    RAISE NOTICE 'Legacy role migration completed:';
    RAISE NOTICE '- Users with admin modules: %', admin_users_count;
    RAISE NOTICE '- Total admin modules assigned: %', total_modules_assigned;
    
    -- Verify primary admin has all modules
    IF EXISTS (
        SELECT 1 FROM user_admin_roles 
        WHERE firebase_uid = 'KLiZ6jiuzchxUppd60IdBD5WS4U2' 
        AND module_code = 'system_admin'
        AND is_active = true
    ) THEN
        RAISE NOTICE '- Primary admin (jesadakorn.kirtnu@gmail.com) has system_admin module: YES';
    ELSE
        RAISE NOTICE '- Primary admin (jesadakorn.kirtnu@gmail.com) has system_admin module: NO';
    END IF;
END $$;

-- Create index for better performance on admin module queries
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_user_module_active 
ON user_admin_roles (firebase_uid, module_code, is_active) WHERE is_active = true;

-- Migration 017: Completely removes legacy role-based authentication system and migrates all data to modern admin module system. 
-- Ensures primary admin has full access and cleans up all legacy database structures.