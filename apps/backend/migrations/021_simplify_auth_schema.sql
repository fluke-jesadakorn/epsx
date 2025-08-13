-- Migration: Simplify Authentication Schema for Modern JWT System
-- This migration removes complex IAM tables and Casbin references that are no longer needed
-- with our Auth.js v5 + JWT-based permission system

-- Step 1: Remove complex IAM tables (replaced by JWT claims)
-- These tables are no longer needed since permissions come from JWT tokens
DROP TABLE IF EXISTS role_policies CASCADE;
DROP TABLE IF EXISTS user_roles CASCADE;
DROP TABLE IF EXISTS iam_policies CASCADE;
DROP TABLE IF EXISTS iam_roles CASCADE;

-- Step 2: Clean up any remaining Casbin references from previous migrations
-- Remove any leftover Casbin rules since we've completely removed Casbin
DROP TABLE IF EXISTS casbin_rule CASCADE;
DROP TABLE IF EXISTS casbin_policy CASCADE;

-- Step 3: Simplify users table to only essential columns for JWT auth
-- Ensure users table is minimal and focused on Firebase integration
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS package_tier VARCHAR(50) DEFAULT 'FREE',
ADD COLUMN IF NOT EXISTS permissions TEXT[] DEFAULT '{}',
ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT true,
ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ;

-- Add index for package tier filtering
CREATE INDEX IF NOT EXISTS idx_users_package_tier ON users(package_tier);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active) WHERE is_active = true;

-- Step 4: Update admin modules system to be the primary permission system
-- Ensure admin_modules table has all the fields needed for modern auth
ALTER TABLE admin_modules 
ADD COLUMN IF NOT EXISTS jwt_claims TEXT[] DEFAULT '{}',
ADD COLUMN IF NOT EXISTS package_requirements TEXT[] DEFAULT '{}';

-- Update admin module permissions to include JWT-compatible claims
UPDATE admin_module_permissions SET 
permissions = ARRAY[
    'admin:' || module_code,
    'manage:' || CASE 
        WHEN module_code = 'system_admin' THEN 'all'
        WHEN module_code = 'user_operations' THEN 'user'
        WHEN module_code = 'analytics_specialist' THEN 'analytics'
        WHEN module_code = 'billing_admin' THEN 'payment'
        WHEN module_code = 'permission_admin' THEN 'permissions'
        WHEN module_code = 'developer_relations' THEN 'system'
        WHEN module_code = 'module_coordinator' THEN 'module'
        WHEN module_code = 'compliance_audit' THEN 'audit'
        WHEN module_code = 'support_specialist' THEN 'support'
        ELSE 'basic'
    END
]
WHERE permissions IS NOT NULL;

-- Step 5: Add session table enhancements for Auth.js integration
ALTER TABLE sessions 
ADD COLUMN IF NOT EXISTS provider VARCHAR(50) DEFAULT 'credentials',
ADD COLUMN IF NOT EXISTS provider_account_id TEXT,
ADD COLUMN IF NOT EXISTS session_token TEXT UNIQUE,
ADD COLUMN IF NOT EXISTS jwt_token TEXT;

-- Add indexes for Auth.js session management
CREATE INDEX IF NOT EXISTS idx_sessions_session_token ON sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_sessions_provider ON sessions(provider);

-- Step 6: Create a simple user permissions view for JWT generation
CREATE OR REPLACE VIEW user_permissions_view AS
SELECT 
    u.id,
    u.firebase_uid,
    u.email,
    u.package_tier,
    u.permissions as base_permissions,
    COALESCE(
        ARRAY_AGG(DISTINCT uar.module_code) FILTER (WHERE uar.module_code IS NOT NULL),
        '{}'
    ) as admin_modules,
    COALESCE(
        ARRAY_AGG(DISTINCT amp.permissions) FILTER (WHERE amp.permissions IS NOT NULL),
        '{}'
    ) as module_permissions,
    u.is_active,
    u.created_at,
    u.last_login_at
FROM users u
LEFT JOIN user_admin_roles uar ON u.firebase_uid = uar.firebase_uid 
    AND uar.is_active = true 
    AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
LEFT JOIN admin_module_permissions amp ON uar.module_code = amp.module_code
WHERE u.is_active = true
GROUP BY u.id, u.firebase_uid, u.email, u.package_tier, u.permissions, u.is_active, u.created_at, u.last_login_at;

-- Step 7: Clean up temporary permissions table to work with JWT
-- Ensure temporary permissions integrate well with JWT claims
ALTER TABLE temporary_permissions 
ADD COLUMN IF NOT EXISTS jwt_claims TEXT[] DEFAULT '{}',
ADD COLUMN IF NOT EXISTS package_tier_override VARCHAR(50);

-- Step 8: Remove any unnecessary audit/logging tables that reference removed IAM system
-- Clean up references to removed IAM tables in audit logs
UPDATE audit_logs 
SET details = details - 'iam_role_id' - 'policy_id' - 'casbin_rule'
WHERE details ? 'iam_role_id' OR details ? 'policy_id' OR details ? 'casbin_rule';

-- Step 9: Create modern auth functions for JWT integration
-- Function to get user JWT claims
CREATE OR REPLACE FUNCTION get_user_jwt_claims(user_firebase_uid VARCHAR(128))
RETURNS JSONB
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT jsonb_build_object(
        'firebase_uid', upv.firebase_uid,
        'email', upv.email,
        'package_tier', upv.package_tier,
        'permissions', upv.base_permissions,
        'admin_modules', upv.admin_modules,
        'role', CASE 
            WHEN 'system_admin' = ANY(upv.admin_modules) THEN 'super_admin'
            WHEN array_length(upv.admin_modules, 1) > 0 THEN 'admin'
            ELSE 'user'
        END,
        'is_active', upv.is_active,
        'last_login_at', upv.last_login_at
    )
    FROM user_permissions_view upv
    WHERE upv.firebase_uid = user_firebase_uid;
$BODY$;

-- Function to check if user has specific permission (for backend validation)
CREATE OR REPLACE FUNCTION user_has_permission(
    user_firebase_uid VARCHAR(128), 
    required_permission TEXT
)
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT EXISTS(
        SELECT 1 FROM user_permissions_view upv
        WHERE upv.firebase_uid = user_firebase_uid
        AND (
            required_permission = ANY(upv.base_permissions)
            OR required_permission = ANY(upv.module_permissions)
            OR 'system_admin' = ANY(upv.admin_modules) -- System admin has all permissions
            OR '*' = ANY(upv.base_permissions) -- Wildcard permission
        )
    );
$BODY$;

-- Step 10: Update any remaining tables to remove IAM references
-- Clean up permission_profiles table to work with new system
ALTER TABLE permission_profiles 
ADD COLUMN IF NOT EXISTS jwt_permissions TEXT[] DEFAULT '{}',
ADD COLUMN IF NOT EXISTS required_package_tier VARCHAR(50) DEFAULT 'FREE';

-- Step 11: Add comments for documentation
COMMENT ON VIEW user_permissions_view IS 'Modern view for JWT token generation with user permissions';
COMMENT ON FUNCTION get_user_jwt_claims(VARCHAR) IS 'Returns JWT claims for a user based on their permissions and admin modules';
COMMENT ON FUNCTION user_has_permission(VARCHAR, TEXT) IS 'Checks if user has a specific permission for backend validation';

-- Step 12: Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_temporary_permissions_firebase_uid ON temporary_permissions(firebase_uid) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_permission_profiles_tier ON permission_profiles(required_package_tier);

-- Step 13: Log the migration completion
INSERT INTO schema_migrations (version, description, executed_at) 
VALUES ('021', 'Simplify auth schema for modern JWT system - remove IAM tables', NOW())
ON CONFLICT (version) DO NOTHING;

-- Final verification
DO $$
DECLARE
    iam_tables_count INT;
    admin_modules_count INT;
    active_users_count INT;
BEGIN
    -- Check that IAM tables are removed
    SELECT COUNT(*) INTO iam_tables_count
    FROM information_schema.tables 
    WHERE table_name IN ('iam_roles', 'iam_policies', 'user_roles', 'role_policies', 'casbin_rule', 'casbin_policy')
    AND table_schema = 'public';
    
    -- Count active admin modules
    SELECT COUNT(*) INTO admin_modules_count
    FROM admin_modules WHERE is_active = true;
    
    -- Count active users
    SELECT COUNT(*) INTO active_users_count
    FROM users WHERE is_active = true;
    
    RAISE NOTICE 'Auth schema simplification completed:';
    RAISE NOTICE '- Removed IAM tables: % (should be 0)', iam_tables_count;
    RAISE NOTICE '- Active admin modules: %', admin_modules_count;
    RAISE NOTICE '- Active users: %', active_users_count;
    RAISE NOTICE '- JWT-based permission system is now active';
END $$;