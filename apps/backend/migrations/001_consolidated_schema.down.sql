-- ================================================================================================
-- EPSX CONSOLIDATED DEVELOPMENT SCHEMA - COMPLETE ROLLBACK
-- ================================================================================================
-- Version: 3.0.0 (Down Migration)
-- Created: 2025-11-26
--
-- Description: Complete rollback of the consolidated EPSX schema to empty database.
-- This migration drops ALL tables, views, functions, triggers, and indexes created by
-- the consolidated up migration, providing a clean reset for development.
--
-- Usage: For development environment reset and database cleanup
-- ================================================================================================

-- Set session configuration
SET timezone = 'UTC';
SET client_encoding = 'UTF8';

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'STARTING EPSX CONSOLIDATED SCHEMA ROLLBACK... 🔄';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'This will drop ALL EPSX database objects';
    RAISE NOTICE 'Use only for development environment reset';
    RAISE NOTICE '=================================================================================';
END $$;

-- ================================================================================================
-- SECTION 1: DROP VIEWS (Drop views first to avoid dependency issues)
-- ================================================================================================

DROP VIEW IF EXISTS v_recent_permission_changes CASCADE;
DROP VIEW IF EXISTS v_wallet_permission_history CASCADE;

-- ================================================================================================
-- SECTION 2: DROP MATERIALIZED VIEWS
-- ================================================================================================

DROP MATERIALIZED VIEW IF EXISTS read_model.mv_active_wallets_summary CASCADE;
DROP MATERIALIZED VIEW IF EXISTS mv_wallet_permission_counts CASCADE;
DROP MATERIALIZED VIEW IF EXISTS event_store_stats CASCADE;
DROP MATERIALIZED VIEW IF EXISTS user_effective_permissions CASCADE;
DROP MATERIALIZED VIEW IF EXISTS wallet_permissions_view CASCADE;

-- ================================================================================================
-- SECTION 3: DROP TRIGGERS (Before dropping tables)
-- ================================================================================================

-- Core table triggers
DROP TRIGGER IF EXISTS trigger_wallet_users_updated_at ON wallet_users;
DROP TRIGGER IF EXISTS trigger_permission_groups_updated_at ON permission_groups;
DROP TRIGGER IF EXISTS route_permissions_updated_at_trigger ON route_permissions;
DROP TRIGGER IF EXISTS trigger_sessions_updated_at ON sessions;
DROP TRIGGER IF EXISTS update_wallet_notifications_updated_at ON wallet_notifications;
DROP TRIGGER IF EXISTS update_permissions_updated_at ON permissions;

-- Audit triggers (if they exist)
DROP TRIGGER IF EXISTS trg_audit_wallet_direct_permissions ON wallet_direct_permissions;
DROP TRIGGER IF EXISTS trg_audit_wallet_group_assignments ON wallet_group_assignments;

-- ================================================================================================
-- SECTION 4: DROP TABLES IN REVERSE DEPENDENCY ORDER
-- ================================================================================================

-- Drop audit and logging tables first (depend on core tables)
DROP TABLE IF EXISTS permission_audit_log CASCADE;

-- Drop notification tables
DROP TABLE IF EXISTS notification_subscriptions CASCADE;
DROP TABLE IF EXISTS wallet_notifications CASCADE;

-- Drop read model tables
DROP TABLE IF EXISTS read_model.projection_checkpoints CASCADE;
DROP TABLE IF EXISTS read_model.analytics_rankings CASCADE;
DROP TABLE IF EXISTS read_model.permission_summary CASCADE;
DROP TABLE IF EXISTS read_model.wallet_details CASCADE;

-- Drop event sourcing tables
DROP TABLE IF EXISTS outbox_events CASCADE;
DROP TABLE IF EXISTS event_store CASCADE;
DROP TABLE IF EXISTS aggregate_snapshots CASCADE;

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
-- SECTION 5: DROP FUNCTIONS
-- ================================================================================================

-- Update timestamp functions
DROP FUNCTION IF EXISTS trigger_update_timestamp() CASCADE;
DROP FUNCTION IF EXISTS trigger_update_wallet_users_timestamp() CASCADE;
DROP FUNCTION IF EXISTS update_route_permissions_updated_at() CASCADE;
DROP FUNCTION IF EXISTS trigger_update_sessions_timestamp() CASCADE;
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

-- Materialized view refresh functions
DROP FUNCTION IF EXISTS refresh_user_effective_permissions() CASCADE;
DROP FUNCTION IF EXISTS refresh_event_store_stats() CASCADE;
DROP FUNCTION IF EXISTS refresh_wallet_permission_counts() CASCADE;
DROP FUNCTION IF EXISTS refresh_wallet_permissions_view() CASCADE;

-- Enhanced permission functions
DROP FUNCTION IF EXISTS wallet_has_permission(VARCHAR(42), VARCHAR(255)) CASCADE;
DROP FUNCTION IF EXISTS get_wallet_permissions(VARCHAR(42)) CASCADE;
DROP FUNCTION IF EXISTS get_permission_stats_by_platform() CASCADE;

-- Read model helper functions
DROP FUNCTION IF EXISTS read_model.get_wallet_full_details(VARCHAR(42)) CASCADE;

-- Permission optimization functions
DROP FUNCTION IF EXISTS get_wallet_permissions_detailed(VARCHAR) CASCADE;
DROP FUNCTION IF EXISTS get_wallet_effective_permissions(VARCHAR) CASCADE;
DROP FUNCTION IF EXISTS get_wallet_permission_stats(VARCHAR) CASCADE;
DROP FUNCTION IF EXISTS get_wallet_permission_cache_key(VARCHAR) CASCADE;
DROP FUNCTION IF EXISTS wallet_has_permissions_batch(VARCHAR, VARCHAR[]) CASCADE;
DROP FUNCTION IF EXISTS get_expiring_permissions(INTEGER) CASCADE;

-- Audit logging functions
DROP FUNCTION IF EXISTS log_permission_granted(VARCHAR, VARCHAR, UUID, VARCHAR, TEXT, TIMESTAMPTZ, JSONB) CASCADE;
DROP FUNCTION IF EXISTS log_permission_revoked(VARCHAR, VARCHAR, UUID, VARCHAR, TEXT, JSONB) CASCADE;
DROP FUNCTION IF EXISTS log_group_assigned(VARCHAR, UUID, VARCHAR, VARCHAR, TEXT, TIMESTAMPTZ, JSONB) CASCADE;
DROP FUNCTION IF EXISTS log_group_removed(VARCHAR, UUID, VARCHAR, VARCHAR, TEXT, JSONB) CASCADE;

-- Audit trigger functions (if they exist)
DROP FUNCTION IF EXISTS audit_wallet_direct_permissions() CASCADE;
DROP FUNCTION IF EXISTS audit_wallet_group_assignments() CASCADE;

-- ================================================================================================
-- SECTION 6: DROP SCHEMA
-- ================================================================================================

DROP SCHEMA IF EXISTS read_model CASCADE;

-- ================================================================================================
-- SECTION 7: DROP INDEXES (Any that might remain)
-- ================================================================================================

-- Drop any remaining indexes that weren't automatically dropped with tables
-- This is a safety net to ensure clean database

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

-- Event store indexes
DROP INDEX IF EXISTS idx_event_store_aggregate;
DROP INDEX IF EXISTS idx_event_store_type_time;
DROP INDEX IF EXISTS idx_event_store_correlation;
DROP INDEX IF EXISTS idx_event_store_aggregate_type;
DROP INDEX IF EXISTS idx_event_store_occurred_at;

-- Outbox event indexes
DROP INDEX IF EXISTS idx_outbox_unprocessed;
DROP INDEX IF EXISTS idx_outbox_aggregate;
DROP INDEX IF EXISTS idx_outbox_retry;
DROP INDEX IF EXISTS idx_outbox_created_at;

-- Aggregate snapshot indexes
DROP INDEX IF EXISTS idx_snapshots_type_version;
DROP INDEX IF EXISTS idx_snapshots_created_at;

-- Read model indexes
DROP INDEX IF EXISTS idx_wallet_details_active;
DROP INDEX IF EXISTS idx_wallet_details_tier;
DROP INDEX IF EXISTS idx_wallet_details_permissions_gin;
DROP INDEX IF EXISTS idx_wallet_details_groups_gin;
DROP INDEX IF EXISTS idx_wallet_details_activity;
DROP INDEX IF EXISTS idx_wallet_details_engagement;
DROP INDEX IF EXISTS idx_wallet_details_created_at;

DROP INDEX IF EXISTS idx_permission_wallet;
DROP INDEX IF EXISTS idx_permission_active;
DROP INDEX IF EXISTS idx_permission_source;
DROP INDEX IF EXISTS idx_permission_string;
DROP INDEX IF EXISTS idx_permission_expires_at;

DROP INDEX IF EXISTS idx_analytics_rank;
DROP INDEX IF EXISTS idx_analytics_sector;
DROP INDEX IF EXISTS idx_analytics_score;
DROP INDEX IF EXISTS idx_analytics_updated_at;

DROP INDEX IF EXISTS idx_checkpoint_health;
DROP INDEX IF EXISTS idx_checkpoint_processed_at;

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

-- Materialized view indexes (might remain if views were dropped separately)
DROP INDEX IF EXISTS idx_user_eff_perms_unique;
DROP INDEX IF EXISTS idx_user_eff_perms_wallet;
DROP INDEX IF EXISTS idx_user_eff_perms_permission;
DROP INDEX IF EXISTS idx_user_eff_perms_platform;
DROP INDEX IF EXISTS idx_event_store_stats_pk;
DROP INDEX IF EXISTS idx_mv_active_wallets_summary;
DROP INDEX IF EXISTS idx_mv_wallet_perm_counts_wallet;
DROP INDEX IF EXISTS idx_mv_wallet_perm_counts_tier;
DROP INDEX IF EXISTS idx_wallet_permissions_view_address;
DROP INDEX IF EXISTS idx_wallet_permissions_view_activity;
DROP INDEX IF EXISTS idx_wallet_permissions_view_stats;

-- Performance indexes
DROP INDEX IF EXISTS idx_pg_active;

-- ================================================================================================
-- SECTION 8: DROP EXTENSIONS (Optional - comment out if you want to keep them)
-- ================================================================================================

-- Uncomment these lines if you want to completely remove extensions
-- Note: This will affect other applications that might use these extensions
-- DROP EXTENSION IF EXISTS btree_gist CASCADE;
-- DROP EXTENSION IF EXISTS pg_trgm CASCADE;
-- DROP EXTENSION IF EXISTS "uuid-ossp" CASCADE;

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'EPSX CONSOLIDATED SCHEMA ROLLBACK COMPLETED! ✅';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Dropped Objects:';
    RAISE NOTICE '  ✅ All EPSX tables (15+ tables)';
    RAISE NOTICE '  ✅ All materialized views (5 views)';
    RAISE NOTICE '  ✅ All regular views (2 views)';
    RAISE NOTICE '  ✅ All functions (20+ functions)';
    RAISE NOTICE '  ✅ All triggers (7+ triggers)';
    RAISE NOTICE '  ✅ All indexes (80+ indexes)';
    RAISE NOTICE '  ✅ Read model schema';
    RAISE NOTICE '';
    RAISE NOTICE 'Database is now clean and ready for fresh EPSX schema deployment';
    RAISE NOTICE 'Run the consolidated up migration to recreate the schema';
    RAISE NOTICE '=================================================================================';
END $$;