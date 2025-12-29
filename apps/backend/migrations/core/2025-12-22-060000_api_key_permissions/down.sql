-- Drop API key permissions table
DROP INDEX IF EXISTS idx_api_key_permissions_group;
DROP INDEX IF EXISTS idx_api_key_permissions_key;
DROP TABLE IF EXISTS api_key_permissions;
