-- ================================================================================================
-- EPSX CONSOLIDATED SCHEMA v5 - ROLLBACK
-- ================================================================================================

SELECT 'Starting EPSX consolidated schema v5 rollback...' AS status_message;

-- Drop functions first
DROP FUNCTION IF EXISTS check_wallet_permissions(VARCHAR, VARCHAR);
DROP FUNCTION IF EXISTS get_wallet_permissions_detailed_working(VARCHAR);

-- Drop read_model schema and its tables
DROP TABLE IF EXISTS read_model.projection_checkpoints;
DROP SCHEMA IF EXISTS read_model CASCADE;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS api_key_permissions CASCADE;
DROP TABLE IF EXISTS api_key_module_access CASCADE;
DROP TABLE IF EXISTS api_modules CASCADE;
DROP TABLE IF EXISTS api_keys CASCADE;

DROP TABLE IF EXISTS system_settings CASCADE;

DROP TABLE IF EXISTS route_permissions CASCADE;

DROP TABLE IF EXISTS openid_refresh_tokens CASCADE;
DROP TABLE IF EXISTS web3_auth_nonces CASCADE;

DROP TABLE IF EXISTS wallet_direct_permissions CASCADE;
DROP TABLE IF EXISTS wallet_plan_assignments CASCADE;
DROP TABLE IF EXISTS plan_permissions CASCADE;
DROP TABLE IF EXISTS plans CASCADE;
DROP TABLE IF EXISTS permissions CASCADE;
DROP TABLE IF EXISTS wallet_users CASCADE;

SELECT 'EPSX consolidated schema v5 rollback completed!' AS completion_message;
