-- Migration to add database indexes for optimized user search performance
-- These indexes support the advanced user search functionality

-- Index for email search (exact and ILIKE operations)
CREATE INDEX IF NOT EXISTS idx_users_email_search ON users USING gin(to_tsvector('english', email));
CREATE INDEX IF NOT EXISTS idx_users_email_lower ON users (LOWER(email));

-- Index for display name and name search
CREATE INDEX IF NOT EXISTS idx_users_display_name_search ON users USING gin(to_tsvector('english', display_name));
CREATE INDEX IF NOT EXISTS idx_users_name_search ON users USING gin(to_tsvector('english', name));

-- Composite index for common search patterns
CREATE INDEX IF NOT EXISTS idx_users_search_composite ON users (email, display_name, is_active, package_tier);

-- Index for activity status filtering
CREATE INDEX IF NOT EXISTS idx_users_activity ON users (is_active, created_at DESC);

-- Index for date-based filtering
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users (created_at);
CREATE INDEX IF NOT EXISTS idx_users_updated_at ON users (updated_at);
CREATE INDEX IF NOT EXISTS idx_users_last_login ON users (last_login_at) WHERE last_login_at IS NOT NULL;

-- Index for subscription tier filtering
CREATE INDEX IF NOT EXISTS idx_users_package_tier ON users (package_tier);

-- Index for firebase_uid lookups (used for admin role checks)
CREATE INDEX IF NOT EXISTS idx_users_firebase_uid ON users (firebase_uid);

-- Indexes for user admin roles table (supports role-based filtering)
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_firebase_uid ON user_admin_roles (firebase_uid);
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_active ON user_admin_roles (is_active, expires_at) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_module ON user_admin_roles (module_code, is_active) WHERE is_active = true;

-- Composite index for admin role checks
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_lookup ON user_admin_roles (firebase_uid, is_active, expires_at) WHERE is_active = true;

-- Indexes for admin modules table
CREATE INDEX IF NOT EXISTS idx_admin_modules_active ON admin_modules (is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_admin_modules_name ON admin_modules (module_name) WHERE is_active = true;

-- Indexes for user permission assignments (supports permission profile filtering)
CREATE INDEX IF NOT EXISTS idx_user_permission_assignments_user ON user_permission_assignments (user_id, expires_at);
CREATE INDEX IF NOT EXISTS idx_user_permission_assignments_profile ON user_permission_assignments (permission_profile_id, expires_at);

-- Partial indexes for active assignments only
CREATE INDEX IF NOT EXISTS idx_user_permission_assignments_active 
ON user_permission_assignments (user_id, permission_profile_id, assigned_at) 
WHERE expires_at IS NULL OR expires_at > NOW();

-- Indexes for role profiles table
CREATE INDEX IF NOT EXISTS idx_role_profiles_active ON role_profiles (is_active) WHERE is_active = true;

-- Full-text search indexes for comprehensive text search
-- These support advanced search across multiple text fields
CREATE INDEX IF NOT EXISTS idx_users_fulltext_search 
ON users USING gin((
    setweight(to_tsvector('english', COALESCE(email, '')), 'A') ||
    setweight(to_tsvector('english', COALESCE(display_name, '')), 'B') ||
    setweight(to_tsvector('english', COALESCE(name, '')), 'C')
));

-- Optimize queries that join users with admin roles
CREATE INDEX IF NOT EXISTS idx_join_users_admin_roles 
ON user_admin_roles (firebase_uid, is_active, expires_at) 
INCLUDE (module_code) 
WHERE is_active = true;

-- Statistics update to help query planner
-- This should be run after the indexes are created
-- ANALYZE users;
-- ANALYZE user_admin_roles;
-- ANALYZE admin_modules;
-- ANALYZE user_permission_assignments;
-- ANALYZE role_profiles;

-- Create a function to refresh search statistics (optional)
CREATE OR REPLACE FUNCTION refresh_user_search_stats() 
RETURNS void 
LANGUAGE plpgsql 
AS $$
BEGIN
    ANALYZE users;
    ANALYZE user_admin_roles;
    ANALYZE admin_modules;  
    ANALYZE user_permission_assignments;
    ANALYZE role_profiles;
    
    RAISE NOTICE 'User search statistics refreshed';
END;
$$;

-- Add comments for documentation
COMMENT ON INDEX idx_users_email_search IS 'Full-text search index for user email addresses';
COMMENT ON INDEX idx_users_fulltext_search IS 'Comprehensive full-text search across email, display_name, and name';
COMMENT ON INDEX idx_users_search_composite IS 'Composite index for common user search patterns';
COMMENT ON INDEX idx_join_users_admin_roles IS 'Optimized index for user-admin role joins';
COMMENT ON FUNCTION refresh_user_search_stats() IS 'Refreshes statistics for user search query optimization';