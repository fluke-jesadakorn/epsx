-- ================================================================================================
-- EPSX CONSOLIDATED SCHEMA v4 - ROLLBACK
-- ================================================================================================
-- Drops ALL tables, indexes, and constraints created by the consolidated schema.
-- ================================================================================================

SELECT 'Starting EPSX consolidated schema rollback...' AS status_message;

-- Drop functions first
DROP FUNCTION IF EXISTS check_wallet_permissions(VARCHAR, VARCHAR);
DROP FUNCTION IF EXISTS get_wallet_permissions_detailed_working(VARCHAR);

-- Drop read_model schema and its tables
DROP TABLE IF EXISTS read_model.projection_checkpoints;
DROP SCHEMA IF EXISTS read_model CASCADE;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS api_key_usage_logs CASCADE;
DROP TABLE IF EXISTS api_key_module_access CASCADE;
DROP TABLE IF EXISTS api_modules CASCADE;
DROP TABLE IF EXISTS api_keys CASCADE;

DROP TABLE IF EXISTS system_settings CASCADE;

DROP TABLE IF EXISTS aggregate_snapshots CASCADE;
DROP TABLE IF EXISTS outbox_events CASCADE;
DROP TABLE IF EXISTS event_store CASCADE;

DROP TABLE IF EXISTS wallet_activity_logs CASCADE;
DROP TABLE IF EXISTS permission_audit_log CASCADE;

DROP TABLE IF EXISTS notification_subscriptions CASCADE;
DROP TABLE IF EXISTS wallet_notifications CASCADE;

DROP TABLE IF EXISTS route_permissions CASCADE;

DROP TABLE IF EXISTS sessions CASCADE;
DROP TABLE IF EXISTS openid_refresh_tokens CASCADE;
DROP TABLE IF EXISTS web3_auth_nonces CASCADE;

DROP TABLE IF EXISTS wallet_direct_permissions CASCADE;
DROP TABLE IF EXISTS wallet_group_assignments CASCADE;
DROP TABLE IF EXISTS group_permissions CASCADE;
DROP TABLE IF EXISTS groups CASCADE;
DROP TABLE IF EXISTS permissions CASCADE;
DROP TABLE IF EXISTS wallet_users CASCADE;

SELECT 'EPSX consolidated schema rollback completed!' AS completion_message;
