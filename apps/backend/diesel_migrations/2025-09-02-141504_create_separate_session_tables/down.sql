-- Rollback separate session tables migration
-- This undoes the creation of admin_sessions and user_sessions tables

-- Drop cleanup function
DROP FUNCTION IF EXISTS cleanup_expired_sessions();

-- Drop triggers
DROP TRIGGER IF EXISTS admin_sessions_updated_at ON admin_sessions;
DROP TRIGGER IF EXISTS user_sessions_updated_at ON user_sessions;

-- Drop trigger function
DROP FUNCTION IF EXISTS update_session_updated_at();

-- Drop indexes (explicit drop for clarity)
DROP INDEX IF EXISTS idx_admin_sessions_user_id;
DROP INDEX IF EXISTS idx_admin_sessions_session_id;
DROP INDEX IF EXISTS idx_admin_sessions_email;
DROP INDEX IF EXISTS idx_admin_sessions_expires_at;
DROP INDEX IF EXISTS idx_admin_sessions_last_activity;
DROP INDEX IF EXISTS idx_admin_sessions_ip_address;
DROP INDEX IF EXISTS idx_admin_sessions_revoked_at;
DROP INDEX IF EXISTS idx_admin_sessions_permissions_gin;

DROP INDEX IF EXISTS idx_user_sessions_user_id;
DROP INDEX IF EXISTS idx_user_sessions_session_id;
DROP INDEX IF EXISTS idx_user_sessions_email;
DROP INDEX IF EXISTS idx_user_sessions_expires_at;
DROP INDEX IF EXISTS idx_user_sessions_last_activity;
DROP INDEX IF EXISTS idx_user_sessions_package_tier;
DROP INDEX IF EXISTS idx_user_sessions_platform_context;
DROP INDEX IF EXISTS idx_user_sessions_revoked_at;
DROP INDEX IF EXISTS idx_user_sessions_cleanup;
DROP INDEX IF EXISTS idx_user_sessions_permissions_gin;

-- Drop tables (CASCADE to handle dependencies)
DROP TABLE IF EXISTS admin_sessions CASCADE;
DROP TABLE IF EXISTS user_sessions CASCADE;
