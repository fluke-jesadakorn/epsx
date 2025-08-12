-- Fix Admin Module Assignment
-- This migration ensures jesadakorn.kirtnu@gmail.com has all necessary admin modules
-- and resolves the legacy role migration issues

-- Step 1: Ensure jesadakorn.kirtnu@gmail.com has all admin modules with proper permissions
INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, granted_reason, is_active)
SELECT 
    'KLiZ6jiuzchxUppd60IdBD5WS4U2',
    am.module_code,
    'system_migration',
    'Fix admin module assignment - ensure full access for primary admin',
    true
FROM admin_modules am
ON CONFLICT (firebase_uid, module_code) DO UPDATE SET
    granted_by = 'system_migration',
    granted_reason = 'Fix admin module assignment - ensure full access for primary admin',
    is_active = true,
    updated_at = NOW();

-- Step 2: Clean up any legacy role references that might still exist
DELETE FROM casbin_rule 
WHERE (v0 LIKE '%admin-full-004%' 
    OR v0 LIKE '%moderator-standard-003%' 
    OR v0 LIKE '%user-basic-001%'
    OR v0 LIKE '%user-premium-002%'
    OR v1 LIKE '%admin-full-004%' 
    OR v1 LIKE '%moderator-standard-003%' 
    OR v1 LIKE '%user-basic-001%'
    OR v1 LIKE '%user-premium-002%'
    OR v2 LIKE '%admin-full-004%'
    OR v2 LIKE '%moderator-standard-003%'
    OR v2 LIKE '%user-basic-001%'
    OR v2 LIKE '%user-premium-002%');

-- Step 3: Ensure modern admin module-based Casbin policies exist
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

-- Step 4: Verify the primary admin user has proper access
DO $$
DECLARE
    admin_modules_count INT;
    total_modules_count INT;
BEGIN
    -- Count admin modules assigned to primary admin
    SELECT COUNT(*) INTO admin_modules_count 
    FROM user_admin_roles 
    WHERE firebase_uid = 'KLiZ6jiuzchxUppd60IdBD5WS4U2' 
    AND is_active = true;
    
    -- Count total available admin modules
    SELECT COUNT(*) INTO total_modules_count 
    FROM admin_modules 
    WHERE is_active = true;
    
    RAISE NOTICE 'Admin module assignment verification:';
    RAISE NOTICE '- Primary admin (jesadakorn.kirtnu@gmail.com) has % admin modules assigned', admin_modules_count;
    RAISE NOTICE '- Total available admin modules: %', total_modules_count;
    
    -- Verify system_admin module specifically
    IF EXISTS (
        SELECT 1 FROM user_admin_roles 
        WHERE firebase_uid = 'KLiZ6jiuzchxUppd60IdBD5WS4U2' 
        AND module_code = 'system_admin'
        AND is_active = true
    ) THEN
        RAISE NOTICE '- Primary admin has system_admin module: YES';
    ELSE
        RAISE NOTICE '- Primary admin has system_admin module: NO - attempting to add';
        
        -- Force add system_admin if it doesn't exist
        INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, granted_reason, is_active)
        VALUES ('KLiZ6jiuzchxUppd60IdBD5WS4U2', 'system_admin', 'system_fix', 'Emergency system_admin assignment', true)
        ON CONFLICT (firebase_uid, module_code) DO UPDATE SET
            granted_by = 'system_fix',
            granted_reason = 'Emergency system_admin assignment',
            is_active = true,
            updated_at = NOW();
            
        RAISE NOTICE '- System admin module assignment completed';
    END IF;
END $$;

-- Step 5: Create performance index for admin module queries if it doesn't exist
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_firebase_uid_active 
ON user_admin_roles (firebase_uid, is_active) WHERE is_active = true;

-- Migration 018: Fixes admin module assignment issues and ensures primary admin has full system access. 
-- Resolves migration conflicts from 017.