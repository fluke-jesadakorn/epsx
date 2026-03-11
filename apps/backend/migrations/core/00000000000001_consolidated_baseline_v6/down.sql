-- ================================================================================================
-- EPSX CONSOLIDATED SCHEMA baseline (v6) - ROLLBACK
-- ================================================================================================

SELECT 'Starting EPSX consolidated schema baseline rollback...' AS status_message;

-- Drop functions first
DROP FUNCTION IF EXISTS check_wallet_permissions(VARCHAR, VARCHAR);
DROP FUNCTION IF EXISTS get_wallet_permissions_detailed_working(VARCHAR);

-- Drop materialized views
DROP MATERIALIZED VIEW IF EXISTS mv_web3_chain_distribution;

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

-- New tables in v6
DROP TABLE IF EXISTS user_watchlist CASCADE;
DROP TABLE IF EXISTS chat_messages CASCADE;
DROP TABLE IF EXISTS chat_conversations CASCADE;
DROP TABLE IF EXISTS chat_topics CASCADE;
DROP TABLE IF EXISTS news_articles CASCADE;

SELECT 'EPSX consolidated schema baseline rollback completed!' AS completion_message;
