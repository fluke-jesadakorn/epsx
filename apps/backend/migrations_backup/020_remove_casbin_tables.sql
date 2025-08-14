-- Migration: Remove Casbin tables
-- This migration removes all Casbin-related database tables as part of 
-- modernizing to Auth.js v5 with JWT-based permissions

-- Drop Casbin policy tables
DROP TABLE IF EXISTS casbin_policy CASCADE;
DROP TABLE IF EXISTS casbin_rule CASCADE;

-- Drop any Casbin-related indexes if they exist
DROP INDEX IF EXISTS idx_casbin_policy_subject;
DROP INDEX IF EXISTS idx_casbin_policy_object;
DROP INDEX IF EXISTS idx_casbin_policy_action;
DROP INDEX IF EXISTS idx_casbin_rule_ptype;

-- Remove any Casbin-related functions if they exist
DROP FUNCTION IF EXISTS casbin_policy_trigger() CASCADE;

-- Log the migration
INSERT INTO schema_migrations (version, description, executed_at) 
VALUES ('020', 'Remove Casbin tables - modernize to JWT auth', NOW())
ON CONFLICT (version) DO NOTHING;