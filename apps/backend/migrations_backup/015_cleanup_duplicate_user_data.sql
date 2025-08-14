-- Cleanup Duplicate User Data Migration
-- This migration safely removes old user profile data that is now handled by Firebase
-- All operations use IF EXISTS to avoid errors

-- IMPORTANT: Backup existing data before running this migration
-- This migration drops tables and removes data - it cannot be easily reversed

-- 1. Drop deprecated authentication tables (replaced by Firebase-native approach)
DROP TABLE IF EXISTS firebase_user_mappings CASCADE;
DROP TABLE IF EXISTS unified_sessions CASCADE;  
DROP TABLE IF EXISTS provider_user_attributes CASCADE;
DROP TABLE IF EXISTS oauth_provider_configs CASCADE;
DROP TABLE IF EXISTS auth_audit_log CASCADE;

-- 2. Clean up old session tables if they exist
DROP TABLE IF EXISTS sessions CASCADE;
DROP TABLE IF EXISTS user_sessions CASCADE;

-- 3. Drop any remaining duplicate user profile tables
DROP TABLE IF EXISTS user_profiles CASCADE;
DROP TABLE IF EXISTS user_metadata CASCADE;
DROP TABLE IF EXISTS user_settings CASCADE;

-- 4. Clean up foreign key constraints (ignore errors if they don't exist)
-- Note: These may fail silently if tables/constraints don't exist, which is fine
ALTER TABLE IF EXISTS admin_permission_profile_assignments DROP CONSTRAINT IF EXISTS admin_permission_profile_assignments_user_id_fkey;
ALTER TABLE IF EXISTS assignment_audit_log DROP CONSTRAINT IF EXISTS assignment_audit_log_performed_by_fkey;

-- Migration complete - Firebase-native tables will be created by migration 014
-- This cleanup migration can run in any order and will not fail

-- Notes for manual verification after migration:
-- 1. Verify all user profile data exists in Firebase
-- 2. Confirm Firebase UIDs are properly referenced in remaining tables  
-- 3. Test authentication flow with cleaned database schema
-- 4. Verify admin role assignments still work
-- 5. Check that sessions can be created and validated
-- 6. Confirm OIDC endpoints function with minimal database

-- IMPORTANT REMINDERS:
-- - This migration removes user profile data from database
-- - Ensure all user data exists in Firebase before running
-- - Test thoroughly in staging environment first
-- - All operations use IF EXISTS to prevent failures