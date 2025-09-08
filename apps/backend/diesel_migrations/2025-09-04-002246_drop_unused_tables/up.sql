-- Drop unused tables created but never properly integrated
-- These tables exist in migrations but were never added to schema.rs or implemented in repositories

-- Drop admin_sessions table (created in 2025-09-02-141504 but never used)
DROP TABLE IF EXISTS admin_sessions CASCADE;

-- Drop user_sessions table (created in 2025-09-02-141504 but never used)  
DROP TABLE IF EXISTS user_sessions CASCADE;

-- Drop oidc_refresh_tokens table (in schema but completely unused - using refresh_tokens instead)
DROP TABLE IF EXISTS oidc_refresh_tokens CASCADE;

-- Drop oidc_token_audit table (in schema but no repository implementation)
DROP TABLE IF EXISTS oidc_token_audit CASCADE;

-- Clean up any functions created for the dropped session tables
DROP FUNCTION IF EXISTS update_session_updated_at();
DROP FUNCTION IF EXISTS cleanup_expired_sessions();

-- Add comment for future reference
COMMENT ON SCHEMA public IS 'Cleaned up unused tables: admin_sessions, user_sessions, oidc_refresh_tokens, oidc_token_audit - 2025-09-04';
