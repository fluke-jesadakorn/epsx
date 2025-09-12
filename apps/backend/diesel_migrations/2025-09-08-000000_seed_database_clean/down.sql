-- ============================================================================
-- EPSX Database Seed Migration Rollback
-- Drops all tables, functions, types, and indexes created by seed migration
-- ============================================================================

-- Drop functions first
DROP FUNCTION IF EXISTS rbac_user_has_permission(UUID, TEXT, TEXT, TEXT);
DROP FUNCTION IF EXISTS rbac_cleanup_expired_permissions();
DROP FUNCTION IF EXISTS cleanup_expired_tokens(INTEGER);

-- Drop triggers
DROP TRIGGER IF EXISTS set_updated_at ON users;
DROP TRIGGER IF EXISTS set_updated_at ON sessions;
DROP TRIGGER IF EXISTS set_updated_at ON refresh_tokens;
DROP TRIGGER IF EXISTS set_updated_at ON rbac_permissions;
DROP TRIGGER IF EXISTS set_updated_at ON rbac_roles;
DROP TRIGGER IF EXISTS set_updated_at ON rbac_user_roles;
DROP TRIGGER IF EXISTS set_updated_at ON rbac_user_permissions;
DROP TRIGGER IF EXISTS set_updated_at ON rbac_user_limits;
DROP TRIGGER IF EXISTS set_updated_at ON user_permissions;
DROP TRIGGER IF EXISTS set_updated_at ON user_dynamic_limits;
DROP TRIGGER IF EXISTS set_updated_at ON user_notification_preferences;
DROP TRIGGER IF EXISTS set_updated_at ON fcm_tokens;
DROP TRIGGER IF EXISTS set_updated_at ON fcm_topics;
DROP TRIGGER IF EXISTS set_updated_at ON eps_growth_analytics;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS eps_growth_analytics;
DROP TABLE IF EXISTS audit_logs;
DROP TABLE IF EXISTS security_events;
DROP TABLE IF EXISTS fcm_topics;
DROP TABLE IF EXISTS fcm_tokens;
DROP TABLE IF EXISTS user_notification_preferences;
DROP TABLE IF EXISTS notification_deliveries;
DROP TABLE IF EXISTS user_notifications;
DROP TABLE IF EXISTS notifications;
DROP TABLE IF EXISTS user_dynamic_limits;
DROP TABLE IF EXISTS user_permissions;
DROP TABLE IF EXISTS rbac_user_limits;
DROP TABLE IF EXISTS rbac_permission_audit_log;
DROP TABLE IF EXISTS rbac_user_permissions;
DROP TABLE IF EXISTS rbac_user_roles;
DROP TABLE IF EXISTS rbac_role_permissions;
DROP TABLE IF EXISTS rbac_roles;
DROP TABLE IF EXISTS rbac_permissions;
DROP TABLE IF EXISTS revoked_tokens;
DROP TABLE IF EXISTS refresh_tokens;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS users;

-- Drop custom types
DROP TYPE IF EXISTS limit_type_enum;
DROP TYPE IF EXISTS audit_operation_enum;
DROP TYPE IF EXISTS permission_type_enum;
DROP TYPE IF EXISTS delivery_status;
DROP TYPE IF EXISTS delivery_channel;
DROP TYPE IF EXISTS notification_priority;
DROP TYPE IF EXISTS notification_type;

-- Remove schema comment
COMMENT ON SCHEMA public IS NULL;