-- Drop all views
DROP VIEW IF EXISTS role_history;
DROP VIEW IF EXISTS user_permissions_view;
DROP VIEW IF EXISTS admin_system_summary;

-- Drop all functions
DROP FUNCTION IF EXISTS cleanup_expired_firebase_sessions();
DROP FUNCTION IF EXISTS user_has_admin_module(VARCHAR, VARCHAR);
DROP FUNCTION IF EXISTS get_user_admin_modules(VARCHAR);
DROP FUNCTION IF EXISTS user_has_permission(VARCHAR, TEXT);
DROP FUNCTION IF EXISTS get_user_jwt_claims(VARCHAR);

-- Drop all tables (in reverse dependency order)
DROP TABLE IF EXISTS alert_notifications;
DROP TABLE IF EXISTS permission_audit_logs;
DROP TABLE IF EXISTS permission_inheritance;
DROP TABLE IF EXISTS tier_limits;
DROP TABLE IF EXISTS package_tier_access;
DROP TABLE IF EXISTS admin_module_access;
DROP TABLE IF EXISTS user_permission_profiles;
DROP TABLE IF EXISTS permissions;
DROP TABLE IF EXISTS temporary_permissions;
DROP TABLE IF EXISTS admin_role_audit;
DROP TABLE IF EXISTS admin_module_permissions;
DROP TABLE IF EXISTS user_admin_roles;
DROP TABLE IF EXISTS admin_modules;
DROP TABLE IF EXISTS notifications;
DROP TABLE IF EXISTS eps_growth_analytics;
DROP TABLE IF EXISTS audit_logs;
DROP TABLE IF EXISTS ip_blacklist;
DROP TABLE IF EXISTS attack_attempts;
DROP TABLE IF EXISTS security_alert_rules;
DROP TABLE IF EXISTS security_events;
DROP TABLE IF EXISTS firebase_sessions;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS users;

-- Drop all custom types
DROP TYPE IF EXISTS denial_reason;
DROP TYPE IF EXISTS package_tier;
DROP TYPE IF EXISTS tier_reset_period;
DROP TYPE IF EXISTS tier_limit_type;
DROP TYPE IF EXISTS tier_feature;
DROP TYPE IF EXISTS admin_module_permission;
DROP TYPE IF EXISTS admin_module;
DROP TYPE IF EXISTS permission_level;
DROP TYPE IF EXISTS permission_scope;

-- Drop extensions (only if they were created by this migration)
-- Note: We don't drop these as they might be used by other applications
-- DROP EXTENSION IF EXISTS "btree_gist";
-- DROP EXTENSION IF EXISTS "uuid-ossp";