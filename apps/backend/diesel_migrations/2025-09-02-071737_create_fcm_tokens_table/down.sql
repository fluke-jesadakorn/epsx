-- Drop trigger first
DROP TRIGGER IF EXISTS trigger_fcm_tokens_updated_at ON fcm_tokens;

-- Drop function
DROP FUNCTION IF EXISTS update_fcm_tokens_updated_at();

-- Drop indexes (will be dropped automatically with table, but explicit for clarity)
DROP INDEX IF EXISTS idx_fcm_tokens_updated_at;
DROP INDEX IF EXISTS idx_fcm_tokens_platform;
DROP INDEX IF EXISTS idx_fcm_tokens_active;
DROP INDEX IF EXISTS idx_fcm_tokens_user_id;

-- Drop table
DROP TABLE IF EXISTS fcm_tokens;

-- Drop enum type
DROP TYPE IF EXISTS device_platform;
