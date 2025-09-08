-- Drop legacy admin system tables replaced by structured permissions
-- These tables were deprecated after successful migration to user_permissions system

-- Drop legacy admin tables (in reverse FK dependency order)
DROP TABLE IF EXISTS user_admin_roles CASCADE;
DROP TABLE IF EXISTS admin_role_audit CASCADE;
DROP TABLE IF EXISTS admin_modules CASCADE;

-- Drop firebase_sessions table (replaced by main sessions table)
DROP TABLE IF EXISTS firebase_sessions CASCADE;

-- Drop the AdminModule enum type that was used by the admin tables
DROP TYPE IF EXISTS admin_module CASCADE;

-- Add migration comment
COMMENT ON SCHEMA public IS 'Dropped legacy admin system tables: admin_modules, admin_role_audit, user_admin_roles, firebase_sessions - replaced by structured permissions - 2025-09-04';
