-- Migration 009: Remove Firebase UID from users table
-- Drop Firebase UID column and related indexes as part of Web3 migration
-- Firebase authentication is being replaced with Web3 wallet-based authentication

-- Drop indexes that reference firebase_uid
DROP INDEX IF EXISTS idx_users_firebase_uid;
DROP INDEX IF EXISTS idx_users_auth_performance;
DROP INDEX IF EXISTS idx_users_firebase_active_login;

-- Drop the firebase_uid column
ALTER TABLE users DROP COLUMN IF EXISTS firebase_uid;

-- Add comment about the migration
COMMENT ON TABLE users IS 'Users table - migrated from Firebase UID to Web3 wallet-based authentication';

-- Ensure all existing users have proper created_at timestamps
UPDATE users SET created_at = NOW() WHERE created_at IS NULL;

-- Update any existing test data to ensure consistency
-- Note: Web3 authentication tables are already created in migration 007