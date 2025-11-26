-- ================================================================================================
-- DATABASE INSPECTION SCRIPT - Find Unused Tables After Web3 Migration
-- ================================================================================================
-- Run this script to identify which tables exist in your database
-- and which ones are no longer used by the application
--
-- Usage: psql $DATABASE_URL -f INSPECT_DATABASE.sql
-- ================================================================================================

\echo '================================================================================================'
\echo 'EPSX DATABASE CLEANUP ANALYSIS'
\echo '================================================================================================'
\echo ''

-- ------------------------------------------------------------------------------------------------
-- 1. ALL TABLES IN PUBLIC SCHEMA
-- ------------------------------------------------------------------------------------------------

\echo '1. ALL TABLES IN PUBLIC SCHEMA (with usage status):'
\echo '---------------------------------------------------'

SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size,
    CASE
        -- Active core tables
        WHEN tablename IN (
            'wallet_users', 'permissions', 'permission_groups',
            'permission_group_memberships', 'wallet_group_assignments',
            'wallet_direct_permissions', 'web3_auth_nonces', 'sessions',
            'route_permissions', 'openid_refresh_tokens', 'event_store',
            'outbox_events', 'aggregate_snapshots', 'stock_ranking_assignments',
            'assignment_audit_log', '_sqlx_migrations'
        ) THEN '✅ ACTIVE'

        -- Potential legacy tables
        WHEN tablename IN (
            'users', 'oidc_users', 'user_sessions', 'email_verification',
            'password_resets', 'admin_modules', 'user_permissions',
            'role_permissions', 'user_tiers', 'tier_permissions',
            'tier_limits', 'api_keys', 'subscription_plans',
            'user_subscriptions'
        ) THEN '❌ LEGACY (can remove)'

        -- Backup tables
        WHEN tablename LIKE 'backup_%'
          OR tablename LIKE '%_backup'
          OR tablename LIKE '%_old' THEN '🗑️  BACKUP (can remove)'

        ELSE '⚠️  UNKNOWN (verify)'
    END as status
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY status, tablename;

\echo ''

-- ------------------------------------------------------------------------------------------------
-- 2. READ MODEL SCHEMA TABLES
-- ------------------------------------------------------------------------------------------------

\echo '2. READ MODEL SCHEMA TABLES (CQRS projections):'
\echo '-----------------------------------------------'

SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size,
    '✅ ACTIVE (CQRS)' as status
FROM pg_tables
WHERE schemaname = 'read_model'
ORDER BY tablename;

\echo ''

-- ------------------------------------------------------------------------------------------------
-- 3. MATERIALIZED VIEWS
-- ------------------------------------------------------------------------------------------------

\echo '3. MATERIALIZED VIEWS:'
\echo '---------------------'

SELECT
    schemaname,
    matviewname as viewname,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||matviewname)) as size
FROM pg_matviews
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY schemaname, matviewname;

\echo ''

-- ------------------------------------------------------------------------------------------------
-- 4. TABLES BY SIZE (identify large unused tables)
-- ------------------------------------------------------------------------------------------------

\echo '4. TABLES BY SIZE (largest first):'
\echo '-----------------------------------'

SELECT
    schemaname||'.'||tablename as full_table_name,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as total_size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) as table_size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) as index_size
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
LIMIT 20;

\echo ''

-- ------------------------------------------------------------------------------------------------
-- 5. SPECIFICALLY CHECK FOR KNOWN LEGACY TABLES
-- ------------------------------------------------------------------------------------------------

\echo '5. LEGACY TABLES CHECK (from pre-Web3 migration):'
\echo '-------------------------------------------------'

SELECT
    tablename,
    pg_size_pretty(pg_total_relation_size('public.'||tablename)) as size,
    '❌ CAN BE REMOVED' as recommendation,
    CASE tablename
        WHEN 'users' THEN 'Replaced by wallet_users'
        WHEN 'oidc_users' THEN 'Consolidated into wallet_users'
        WHEN 'admin_modules' THEN 'Replaced by structured permissions'
        WHEN 'tier_permissions' THEN 'Replaced by permission_groups'
        WHEN 'user_permissions' THEN 'Replaced by wallet_direct_permissions'
        WHEN 'user_sessions' THEN 'Replaced by sessions table'
        ELSE 'Legacy table from old system'
    END as reason
FROM pg_tables
WHERE schemaname = 'public'
  AND tablename IN (
    'users', 'oidc_users', 'user_sessions', 'admin_modules',
    'user_permissions', 'role_permissions', 'user_tiers',
    'tier_permissions', 'tier_limits', 'api_keys',
    'subscription_plans', 'user_subscriptions',
    'email_verification', 'password_resets'
  )
ORDER BY tablename;

\echo ''

-- ------------------------------------------------------------------------------------------------
-- 6. BACKUP TABLES CHECK
-- ------------------------------------------------------------------------------------------------

\echo '6. BACKUP TABLES CHECK (from migration 020):'
\echo '--------------------------------------------'

SELECT
    tablename,
    pg_size_pretty(pg_total_relation_size('public.'||tablename)) as size,
    '🗑️  BACKUP - SAFE TO REMOVE' as status
FROM pg_tables
WHERE schemaname = 'public'
  AND (
    tablename LIKE 'backup_%'
    OR tablename LIKE '%_backup'
    OR tablename LIKE '%_old'
    OR tablename LIKE '%_deprecated'
  )
ORDER BY tablename;

\echo ''

-- ------------------------------------------------------------------------------------------------
-- 7. SUMMARY STATISTICS
-- ------------------------------------------------------------------------------------------------

\echo '7. CLEANUP SUMMARY:'
\echo '-------------------'

SELECT
    'Total Tables' as metric,
    COUNT(*) as count
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')

UNION ALL

SELECT
    'Active Tables (in use)' as metric,
    COUNT(*) as count
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
  AND tablename IN (
    'wallet_users', 'permissions', 'permission_groups',
    'permission_group_memberships', 'wallet_group_assignments',
    'wallet_direct_permissions', 'web3_auth_nonces', 'sessions',
    'route_permissions', 'openid_refresh_tokens', 'event_store',
    'outbox_events', 'aggregate_snapshots', 'stock_ranking_assignments',
    'assignment_audit_log', '_sqlx_migrations'
  )

UNION ALL

SELECT
    'Legacy Tables (can remove)' as metric,
    COUNT(*) as count
FROM pg_tables
WHERE schemaname = 'public'
  AND tablename IN (
    'users', 'oidc_users', 'user_sessions', 'admin_modules',
    'user_permissions', 'role_permissions', 'user_tiers',
    'tier_permissions', 'tier_limits', 'api_keys',
    'subscription_plans', 'user_subscriptions'
  )

UNION ALL

SELECT
    'Backup Tables (can remove)' as metric,
    COUNT(*) as count
FROM pg_tables
WHERE schemaname = 'public'
  AND (
    tablename LIKE 'backup_%'
    OR tablename LIKE '%_backup'
    OR tablename LIKE '%_old'
  );

\echo ''
\echo '================================================================================================'
\echo 'NEXT STEPS:'
\echo '1. Review the tables marked as LEGACY or BACKUP above'
\echo '2. If you want to remove them, run: apps/backend/migrations/029_cleanup_unused_tables.sql'
\echo '3. Or manually archive them: CREATE SCHEMA archive; ALTER TABLE <name> SET SCHEMA archive;'
\echo '================================================================================================'
