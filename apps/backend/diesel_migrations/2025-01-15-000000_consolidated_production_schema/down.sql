-- ============================================================================
-- EPSX PRODUCTION DATABASE SCHEMA - Rollback
-- ============================================================================
-- Complete rollback for consolidated production schema
-- Drops all tables, views, functions, and types in proper dependency order

-- ============================================================================
-- DROP VIEWS & FUNCTIONS
-- ============================================================================

-- Drop views
DROP VIEW IF EXISTS user_permissions_view;

-- Drop functions
DROP FUNCTION IF EXISTS user_has_admin_module(VARCHAR, admin_module);
DROP FUNCTION IF EXISTS get_user_jwt_claims(VARCHAR);

-- ============================================================================
-- DROP TABLES (In reverse dependency order)
-- ============================================================================

-- Drop dependent tables first
DROP TABLE IF EXISTS audit_logs;
DROP TABLE IF EXISTS security_events;
DROP TABLE IF EXISTS notification_preferences;
DROP TABLE IF EXISTS notifications;
DROP TABLE IF EXISTS eps_growth_analytics;
DROP TABLE IF EXISTS admin_role_audit;
DROP TABLE IF EXISTS user_admin_roles;
DROP TABLE IF EXISTS admin_modules;
DROP TABLE IF EXISTS firebase_sessions;
DROP TABLE IF EXISTS sessions;

-- Drop main user table last
DROP TABLE IF EXISTS users;

-- ============================================================================
-- DROP CUSTOM TYPES
-- ============================================================================

-- Drop enum types
DROP TYPE IF EXISTS notification_priority;
DROP TYPE IF EXISTS notification_type;
DROP TYPE IF EXISTS admin_module;
DROP TYPE IF EXISTS package_tier;

-- ============================================================================
-- NOTE: Extensions are not dropped to avoid affecting other applications
-- ============================================================================
-- Extensions that remain:
-- - uuid-ossp (for UUID generation)
-- These are commonly used by other applications and should remain