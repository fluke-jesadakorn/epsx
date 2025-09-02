-- OIDC Database Schema Rollback
-- Removes all OIDC-related tables, functions, and triggers

-- Drop views first (they depend on tables)
DROP VIEW IF EXISTS oidc_token_stats;
DROP VIEW IF EXISTS active_oidc_tokens;

-- Drop functions
DROP FUNCTION IF EXISTS revoke_user_oidc_tokens(VARCHAR(255), TEXT);
DROP FUNCTION IF EXISTS cleanup_expired_oidc_tokens(INTEGER);

-- Drop triggers
DROP TRIGGER IF EXISTS oidc_refresh_tokens_audit_trigger ON oidc_refresh_tokens;

-- Drop audit function
DROP FUNCTION IF EXISTS audit_oidc_token_changes();

-- Drop tables (audit table first to avoid foreign key constraints)
DROP TABLE IF EXISTS oidc_token_audit;
DROP TABLE IF EXISTS oidc_refresh_tokens;