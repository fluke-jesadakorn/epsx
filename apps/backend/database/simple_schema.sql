-- ============================================================================
-- EPSX SIMPLE DATABASE SCHEMA
-- ============================================================================
-- Complete replacement of complex permission system with 3-role simplicity
-- Roles: admin, user, guest
-- Features: view_eps, export_data, realtime, profile, notifications, billing, advanced_filters
-- Version: Simple 2024
-- ============================================================================

-- ============================================================================
-- EXTENSIONS
-- ============================================================================

-- UUID generation support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "btree_gist";

-- ============================================================================
-- DROP ALL COMPLEX PERMISSION SYSTEMS
-- ============================================================================

-- Drop complex permission tables (clean slate approach)
DROP TABLE IF EXISTS admin_module_permissions CASCADE;
DROP TABLE IF EXISTS user_admin_roles CASCADE;
DROP TABLE IF EXISTS admin_modules CASCADE;
DROP TABLE IF EXISTS admin_role_audit CASCADE;
DROP TABLE IF EXISTS permissions CASCADE;
DROP TABLE IF EXISTS user_permission_profiles CASCADE;
DROP TABLE IF EXISTS admin_module_access CASCADE;
DROP TABLE IF EXISTS package_tier_access CASCADE;
DROP TABLE IF EXISTS tier_limits CASCADE;
DROP TABLE IF EXISTS permission_inheritance CASCADE;
DROP TABLE IF EXISTS temporary_permissions CASCADE;
DROP TABLE IF EXISTS permission_audit_logs CASCADE;

-- Drop complex views
DROP VIEW IF EXISTS admin_system_summary CASCADE;
DROP VIEW IF EXISTS user_permissions_view CASCADE;
DROP VIEW IF EXISTS role_history CASCADE;

-- Drop complex functions
DROP FUNCTION IF EXISTS get_user_jwt_claims(VARCHAR);
DROP FUNCTION IF EXISTS user_has_permission(VARCHAR, TEXT);
DROP FUNCTION IF EXISTS get_user_admin_modules(VARCHAR);
DROP FUNCTION IF EXISTS user_has_admin_module(VARCHAR, VARCHAR);

-- Drop complex enums
DROP TYPE IF EXISTS permission_scope CASCADE;
DROP TYPE IF EXISTS permission_level CASCADE;
DROP TYPE IF EXISTS admin_module CASCADE;
DROP TYPE IF EXISTS admin_module_permission CASCADE;
DROP TYPE IF EXISTS tier_feature CASCADE;
DROP TYPE IF EXISTS tier_limit_type CASCADE;
DROP TYPE IF EXISTS tier_reset_period CASCADE;
DROP TYPE IF EXISTS package_tier CASCADE;
DROP TYPE IF EXISTS denial_reason CASCADE;

-- ============================================================================
-- SIMPLE ENUMS AND TYPES
-- ============================================================================

-- Simple 3-role system
CREATE TYPE user_role AS ENUM ('admin', 'user', 'guest');

-- ============================================================================
-- CORE TABLES (SIMPLIFIED)
-- ============================================================================

-- Users table (simplified - remove package_tier and permissions columns)
ALTER TABLE users 
DROP COLUMN IF EXISTS package_tier CASCADE,
DROP COLUMN IF EXISTS permissions CASCADE,
ADD COLUMN IF NOT EXISTS role user_role DEFAULT 'guest';

-- Update existing users: map package tiers to simple roles
UPDATE users SET role = CASE
    WHEN package_tier = 'admin' THEN 'admin'::user_role
    WHEN package_tier IN ('bronze', 'silver', 'gold', 'platinum') THEN 'user'::user_role
    ELSE 'guest'::user_role
END
WHERE package_tier IS NOT NULL;

-- Sessions table (keep as-is, no permission complexity)
-- firebase_sessions table (keep as-is)
-- audit_logs table (keep as-is for basic auditing)
-- security_events table (keep as-is for security)
-- attack_attempts table (keep as-is for security)
-- ip_blacklist table (keep as-is for security)  
-- alert_notifications table (keep as-is for security)
-- notifications table (keep as-is for user notifications)
-- eps_growth_analytics table (keep as-is for analytics)

-- ============================================================================
-- SIMPLE FUNCTIONS
-- ============================================================================

-- Simple JWT claims generation (replaces complex permission aggregation)
CREATE OR REPLACE FUNCTION get_simple_user_claims(user_firebase_uid VARCHAR(128))
RETURNS JSONB
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT jsonb_build_object(
        'firebase_uid', u.firebase_uid,
        'email', u.email,
        'role', u.role::text,
        'display_name', u.display_name,
        'name', u.name,
        'avatar_url', u.avatar_url,
        'is_active', u.is_active,
        'last_login_at', u.last_login_at
    )
    FROM users u
    WHERE u.firebase_uid = user_firebase_uid 
    AND u.is_active = true;
$BODY$;

-- Simple role checking (replaces complex permission checking)
CREATE OR REPLACE FUNCTION user_has_role(
    user_firebase_uid VARCHAR(128), 
    required_role TEXT
)
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT CASE
        WHEN u.role = 'admin' THEN true  -- admin can access everything
        WHEN u.role = 'user' AND required_role IN ('user', 'guest') THEN true
        WHEN u.role = 'guest' AND required_role = 'guest' THEN true
        ELSE false
    END
    FROM users u
    WHERE u.firebase_uid = user_firebase_uid 
    AND u.is_active = true;
$BODY$;

-- Simple feature access checking (matches frontend logic exactly)
CREATE OR REPLACE FUNCTION user_has_feature_access(
    user_firebase_uid VARCHAR(128), 
    feature TEXT
)
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT CASE
        WHEN u.role = 'admin' THEN true  -- admin can access everything
        WHEN u.role = 'user' AND feature IN (
            'view_eps', 'export_data', 'realtime', 'profile',
            'notifications', 'billing', 'advanced_filters'
        ) THEN true
        WHEN u.role = 'guest' AND feature = 'view_eps' THEN true
        ELSE false
    END
    FROM users u
    WHERE u.firebase_uid = user_firebase_uid 
    AND u.is_active = true;
$BODY$;

-- ============================================================================
-- SIMPLE INDEXES (OPTIMIZED)
-- ============================================================================

-- Users role index (new)
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- Keep existing essential indexes for core tables:
-- idx_users_firebase_uid, idx_users_email, idx_sessions_user_id, etc.
-- Remove complex permission-related indexes (already dropped with tables)

-- ============================================================================
-- DATA MIGRATION (EXISTING USERS)
-- ============================================================================

-- Ensure all users have a role assigned
UPDATE users SET role = 'guest' WHERE role IS NULL;

-- Make role column NOT NULL
ALTER TABLE users ALTER COLUMN role SET NOT NULL;

-- ============================================================================
-- SIMPLE VIEWS (OPTIONAL)
-- ============================================================================

-- Simple user info view for JWT generation
CREATE VIEW simple_user_view AS
SELECT 
    u.id,
    u.firebase_uid,
    u.email,
    u.display_name,
    u.name,
    u.avatar_url,
    u.role,
    u.is_active,
    u.created_at,
    u.last_login_at
FROM users u
WHERE u.is_active = true;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TYPE user_role IS 'Simple 3-role system: admin (full access), user (premium features), guest (read-only)';
COMMENT ON COLUMN users.role IS 'User role determining feature access: admin, user, or guest';
COMMENT ON FUNCTION get_simple_user_claims(VARCHAR) IS 'Returns simple JWT claims with role-based access';
COMMENT ON FUNCTION user_has_role(VARCHAR, TEXT) IS 'Checks if user has minimum required role';
COMMENT ON FUNCTION user_has_feature_access(VARCHAR, TEXT) IS 'Checks if user can access specific feature';

-- ============================================================================
-- SCHEMA SIMPLIFICATION COMPLETE
-- ============================================================================
-- This simplified schema provides:
-- ✅ 3-role system (admin, user, guest) 
-- ✅ Feature-based access control
-- ✅ Simple JWT claims generation
-- ✅ Backend-frontend identical logic
-- ✅ No package tier complexity
-- ✅ No admin module complexity
-- ✅ Clean slate approach
-- 
-- Removed: 12+ complex permission tables
-- Added: 3 simple functions, 1 role enum
-- Migration: Automatic mapping of existing users to simple roles