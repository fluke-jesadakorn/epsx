DROP INDEX IF EXISTS idx_developer_api_key_audit_key_created;
DROP TABLE IF EXISTS developer_api_key_audit;
DROP INDEX IF EXISTS idx_developer_api_key_idempotency_resource;
DROP TABLE IF EXISTS developer_api_key_idempotency;
DROP INDEX IF EXISTS idx_permissions_api_assignable;
ALTER TABLE permissions DROP COLUMN IF EXISTS api_assignable;
