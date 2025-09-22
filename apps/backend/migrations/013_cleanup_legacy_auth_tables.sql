-- Migration 013: Clean up legacy authentication tables
-- Remove tables that are not used with Web3 wallet-first authentication
-- The Web3 system uses wallet_permissions instead of user_permissions
-- and Web3 authentication doesn't use traditional sessions

-- Drop legacy user permissions table
-- Web3 system uses wallet_permissions instead
DROP TABLE IF EXISTS user_permissions CASCADE;

-- Drop legacy sessions table 
-- Web3 authentication doesn't use traditional session storage
DROP TABLE IF EXISTS sessions CASCADE;

-- Drop any remaining legacy authentication tables if they exist
DROP TABLE IF EXISTS refresh_tokens CASCADE;
DROP TABLE IF EXISTS revoked_tokens CASCADE;

-- Clean up any orphaned indexes
DROP INDEX IF EXISTS idx_users_firebase_uid;
DROP INDEX IF EXISTS idx_sessions_access_token;
DROP INDEX IF EXISTS idx_refresh_tokens_user_id;
DROP INDEX IF EXISTS idx_refresh_tokens_token_hash;
DROP INDEX IF EXISTS idx_refresh_tokens_family_id;
DROP INDEX IF EXISTS idx_refresh_tokens_expires_at;
DROP INDEX IF EXISTS idx_refresh_tokens_revoked;
DROP INDEX IF EXISTS idx_revoked_tokens_jti;
DROP INDEX IF EXISTS idx_revoked_tokens_user_id;
DROP INDEX IF EXISTS idx_revoked_tokens_expires_at;

-- Add comment about the cleanup
COMMENT ON DATABASE epsx_db IS 'EPSX database - cleaned up legacy authentication tables for Web3 wallet-first system';

-- Show summary of what was cleaned up
SELECT 'Legacy authentication tables cleaned up successfully' as status;