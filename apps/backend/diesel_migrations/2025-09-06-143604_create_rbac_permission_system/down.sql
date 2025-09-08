-- ============================================================================
-- RBAC Permission System Rollback
-- Removes all RBAC tables, functions, and types
-- ============================================================================

-- Drop functions first
DROP FUNCTION IF EXISTS rbac_user_has_permission(UUID, TEXT, TEXT, TEXT);
DROP FUNCTION IF EXISTS rbac_cleanup_expired_permissions();

-- Drop tables (in reverse dependency order)
DROP TABLE IF EXISTS rbac_permission_audit_log;
DROP TABLE IF EXISTS rbac_user_limits;
DROP TABLE IF EXISTS rbac_user_permissions;
DROP TABLE IF EXISTS rbac_user_roles;
DROP TABLE IF EXISTS rbac_role_permissions;
DROP TABLE IF EXISTS rbac_roles;
DROP TABLE IF EXISTS rbac_permissions;

-- Drop custom types
DROP TYPE IF EXISTS permission_type_enum;
DROP TYPE IF EXISTS audit_operation_enum;
DROP TYPE IF EXISTS limit_type_enum;
