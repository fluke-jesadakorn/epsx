-- REVERTING THIS MIGRATION IS NOT RECOMMENDED
-- These legacy admin tables were replaced by the structured permissions system
-- The system has successfully migrated to user_permissions table

-- Reverting would bring back deprecated tables without proper integration
-- The new structured permissions system (user_permissions) is the replacement

DO $$
BEGIN
    RAISE WARNING 'Reverting drop_legacy_admin_tables migration is not recommended. These tables were replaced by structured permissions system.';
    RAISE WARNING 'Use user_permissions table instead of admin_modules system.';
END $$;
