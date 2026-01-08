-- ================================================================================================
-- EPSX CONSOLIDATED DEVELOPMENT SCHEMA - DIESEL VERSION ROLLBACK
-- ================================================================================================
-- Version: 3.0.0 (Down Migration)
-- Created: 2025-11-26
-- Compatible with: Diesel ORM CLI
--
-- Description: Complete rollback of the consolidated EPSX schema to empty database.
-- This migration drops ALL tables, indexes, and constraints created by
-- the consolidated up migration, providing a clean reset for development.
--
-- Usage: For development environment reset and database cleanup
-- ================================================================================================

SELECT 'Starting EPSX consolidated schema rollback for Diesel...' AS status_message;

-- ================================================================================================
-- SECTION 1: DROP TABLES IN REVERSE DEPENDENCY ORDER
-- ================================================================================================

-- Drop audit and logging tables first (depend on core tables)
DROP TABLE IF EXISTS permission_audit_log CASCADE;

-- Drop notification tables
DROP TABLE IF EXISTS notification_subscriptions CASCADE;
DROP TABLE IF EXISTS wallet_notifications CASCADE;

-- Drop session and auth tables
DROP TABLE IF EXISTS sessions CASCADE;
DROP TABLE IF EXISTS openid_refresh_tokens CASCADE;
DROP TABLE IF EXISTS web3_auth_nonces CASCADE;

-- Drop route permissions
DROP TABLE IF EXISTS route_permissions CASCADE;

-- Drop stock ranking tables
DROP TABLE IF EXISTS assignment_audit_log CASCADE;
DROP TABLE IF EXISTS stock_ranking_assignments CASCADE;

-- Drop permission system tables (drop these last as they have dependencies)
DROP TABLE IF EXISTS wallet_direct_permissions CASCADE;
DROP TABLE IF EXISTS wallet_group_assignments CASCADE;
DROP TABLE IF EXISTS permission_group_memberships CASCADE;
DROP TABLE IF EXISTS permission_groups CASCADE;
DROP TABLE IF EXISTS permissions CASCADE;

-- Drop core wallet users table (this should be last)
DROP TABLE IF EXISTS wallet_users CASCADE;

-- ================================================================================================
-- SECTION 2: DROP INDEXES (Any that might remain)
-- ================================================================================================

-- Core table indexes
DROP INDEX IF EXISTS idx_wallet_users_active;
DROP INDEX IF EXISTS idx_wallet_users_tier;
DROP INDEX IF EXISTS idx_wallet_users_created;
DROP INDEX IF EXISTS idx_wallet_users_updated;
DROP INDEX IF EXISTS idx_wallet_users_last_auth;
DROP INDEX IF EXISTS idx_wallet_users_metadata_gin;

-- Permission table indexes
DROP INDEX IF EXISTS idx_permissions_lookup;
DROP INDEX IF EXISTS idx_permissions_platform;
DROP INDEX IF EXISTS idx_permissions_type;
DROP INDEX IF EXISTS idx_permissions_web3;
DROP INDEX IF EXISTS idx_permissions_audit;
DROP INDEX IF EXISTS idx_permissions_wallet_lookup;
DROP INDEX IF EXISTS idx_permissions_platform_lookup;
DROP INDEX IF EXISTS idx_permissions_source_lookup;
DROP INDEX IF EXISTS idx_permissions_expiry;
DROP INDEX IF EXISTS idx_permissions_active_time;
DROP INDEX IF EXISTS idx_permissions_full_search;
DROP INDEX IF EXISTS idx_permissions_wallet_address;
DROP INDEX IF EXISTS idx_permissions_source;
DROP INDEX IF EXISTS idx_permissions_expires_at;
DROP INDEX IF EXISTS idx_permissions_active;
DROP INDEX IF EXISTS idx_permissions_string_pattern;

-- Permission group indexes
DROP INDEX IF EXISTS idx_permission_groups_active;
DROP INDEX IF EXISTS idx_permission_groups_type;
DROP INDEX IF EXISTS idx_permission_groups_slug;
DROP INDEX IF EXISTS idx_permission_groups_price;
DROP INDEX IF EXISTS idx_permission_groups_created;
DROP INDEX IF EXISTS idx_permission_groups_metadata_gin;
DROP INDEX IF EXISTS idx_permission_groups_assignment_rules_gin;

-- Permission group membership indexes
DROP INDEX IF EXISTS idx_pg_memberships_group;
DROP INDEX IF EXISTS idx_pg_memberships_permission;
DROP INDEX IF EXISTS idx_pg_memberships_audit;
DROP INDEX IF EXISTS idx_pgm_group_fk;
DROP INDEX IF EXISTS idx_pgm_permission_fk;

-- Wallet group assignment indexes
DROP INDEX IF EXISTS idx_wg_assignments_wallet;
DROP INDEX IF EXISTS idx_wg_assignments_group;
DROP INDEX IF EXISTS idx_wg_assignments_expires;
DROP INDEX IF EXISTS idx_wg_assignments_audit;
DROP INDEX IF EXISTS idx_wga_wallet_fk;
DROP INDEX IF EXISTS idx_wga_group_fk;
DROP INDEX IF EXISTS idx_wga_active_lookup;
DROP INDEX IF EXISTS idx_wga_expires_lookup;

-- Wallet direct permission indexes
DROP INDEX IF EXISTS idx_direct_perms_wallet;
DROP INDEX IF EXISTS idx_direct_perms_permission;
DROP INDEX IF EXISTS idx_direct_perms_expires;
DROP INDEX IF EXISTS idx_direct_perms_audit;
DROP INDEX IF EXISTS idx_wdp_wallet_fk;
DROP INDEX IF EXISTS idx_wdp_permission_fk;
DROP INDEX IF EXISTS idx_wdp_active_lookup;
DROP INDEX IF EXISTS idx_wdp_expires_lookup;

-- Web3 auth nonce indexes
DROP INDEX IF EXISTS idx_web3_auth_nonces_expires_at;

-- OpenID refresh token indexes
DROP INDEX IF EXISTS idx_openid_refresh_tokens_wallet_address;
DROP INDEX IF EXISTS idx_openid_refresh_tokens_expires_at;
DROP INDEX IF EXISTS idx_openid_refresh_tokens_active;

-- Route permission indexes
DROP INDEX IF EXISTS idx_route_permissions_lookup;
DROP INDEX IF EXISTS idx_route_permissions_method;
DROP INDEX IF EXISTS idx_route_permissions_permission;
DROP INDEX IF EXISTS idx_route_permissions_category;
DROP INDEX IF EXISTS idx_route_permissions_patterns;
DROP INDEX IF EXISTS idx_route_permissions_audit;
DROP INDEX IF EXISTS idx_route_permissions_unique_route;

-- Session indexes
DROP INDEX IF EXISTS idx_sessions_wallet_address;
DROP INDEX IF EXISTS idx_sessions_access_token;
DROP INDEX IF EXISTS idx_sessions_refresh_token;
DROP INDEX IF EXISTS idx_sessions_expires_at;
DROP INDEX IF EXISTS idx_sessions_active;
DROP INDEX IF EXISTS idx_sessions_created_at;
DROP INDEX IF EXISTS idx_sessions_last_accessed;
DROP INDEX IF EXISTS idx_sessions_ip_address;

-- Stock ranking indexes
DROP INDEX IF EXISTS idx_stock_ranking_wallet;
DROP INDEX IF EXISTS idx_stock_ranking_package;
DROP INDEX IF EXISTS idx_stock_ranking_active;

-- Assignment audit log indexes
DROP INDEX IF EXISTS idx_audit_assignment;
DROP INDEX IF EXISTS idx_audit_performed_at;

-- Notification indexes (comprehensive cleanup)
DROP INDEX IF EXISTS idx_wallet_notifications_queue_fetch;
DROP INDEX IF EXISTS idx_wallet_notifications_user_query;
DROP INDEX IF EXISTS idx_wallet_notifications_admin_query;
DROP INDEX IF EXISTS idx_wallet_notifications_expiry;
DROP INDEX IF EXISTS idx_wallet_notifications_soft_deleted;
DROP INDEX IF EXISTS idx_wallet_notifications_read_cleanup;
DROP INDEX IF EXISTS idx_wallet_notifications_timestamp_stats;
DROP INDEX IF EXISTS idx_wallet_notifications_type_stats;
DROP INDEX IF EXISTS idx_wallet_notifications_priority_stats;
DROP INDEX IF EXISTS idx_wallet_notifications_read_rate;
DROP INDEX IF EXISTS idx_wallet_notifications_click_rate;
DROP INDEX IF EXISTS idx_wallet_notifications_delivery_rate;
DROP INDEX IF EXISTS idx_wallet_notifications_acknowledgement;
DROP INDEX IF EXISTS idx_wallet_notifications_unread_count;
DROP INDEX IF EXISTS idx_wallet_notifications_wallet;
DROP INDEX IF EXISTS idx_wallet_notifications_timestamp;
DROP INDEX IF EXISTS idx_wallet_notifications_read_at;
DROP INDEX IF EXISTS idx_wallet_notifications_type;
DROP INDEX IF EXISTS idx_wallet_notifications_priority;
DROP INDEX IF EXISTS idx_wallet_notifications_expires;
DROP INDEX IF EXISTS idx_wallet_notifications_wallet_unread;
DROP INDEX IF EXISTS idx_wallet_notifications_undelivered;
DROP INDEX IF EXISTS idx_wallet_notifications_queued;
DROP INDEX IF EXISTS idx_wallet_notifications_cleanup;
DROP INDEX IF EXISTS idx_wallet_notifications_acknowledged;
DROP INDEX IF EXISTS idx_wallet_notifications_active;
DROP INDEX IF EXISTS idx_wallet_notifications_deleted;
DROP INDEX IF EXISTS idx_wallet_notifications_offline_queue;
DROP INDEX IF EXISTS idx_wallet_notifications_unread_active;

-- Notification subscription indexes
DROP INDEX IF EXISTS idx_subscriptions_wallet_active;
DROP INDEX IF EXISTS idx_subscriptions_instance_active;
DROP INDEX IF EXISTS idx_subscriptions_disconnected;
DROP INDEX IF EXISTS idx_subscriptions_stale;

-- Permission audit log indexes
DROP INDEX IF EXISTS idx_audit_log_wallet;
DROP INDEX IF EXISTS idx_audit_log_timestamp;
DROP INDEX IF EXISTS idx_audit_log_event_type;
DROP INDEX IF EXISTS idx_audit_log_permission;
DROP INDEX IF EXISTS idx_audit_log_group;
DROP INDEX IF EXISTS idx_audit_log_performed_by;
DROP INDEX IF EXISTS idx_audit_log_request_id;
DROP INDEX IF EXISTS idx_audit_log_previous_state_gin;
DROP INDEX IF EXISTS idx_audit_log_new_state_gin;
DROP INDEX IF EXISTS idx_audit_log_metadata_gin;
DROP INDEX IF EXISTS idx_audit_log_timestamp_month;

-- Performance indexes
DROP INDEX IF EXISTS idx_pg_active;

SELECT 'EPSX consolidated schema rollback completed for Diesel!' AS completion_message;