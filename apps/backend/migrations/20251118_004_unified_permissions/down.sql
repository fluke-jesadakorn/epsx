-- ================================================================================================
-- UNIFIED PERMISSIONS TABLE ROLLBACK MIGRATION
-- ================================================================================================
-- This migration safely rolls back the unified permissions table creation.
-- It removes the new table, view, indexes, and helper functions.
--
-- NOTE: This migration assumes no data migration has occurred yet.
-- If data has been migrated from legacy tables, additional steps would be needed
-- to restore the original data structure.
-- ================================================================================================

-- Drop helper functions
DROP FUNCTION IF EXISTS get_permission_stats_by_platform() CASCADE;
DROP FUNCTION IF EXISTS get_wallet_permissions(VARCHAR(42)) CASCADE;
DROP FUNCTION IF EXISTS wallet_has_permission(VARCHAR(42), VARCHAR(42)) CASCADE;
DROP FUNCTION IF EXISTS refresh_wallet_permissions_view() CASCADE;

-- Drop trigger
DROP TRIGGER IF EXISTS update_permissions_updated_at ON permissions;
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

-- Drop materialized view and indexes
DROP MATERIALIZED VIEW IF EXISTS wallet_permissions_view;

-- Drop main permissions table and indexes
DROP TABLE IF EXISTS permissions CASCADE;

-- Remove comments (cleanup)
COMMENT ON TABLE permissions IS NULL;
COMMENT ON MATERIALIZED VIEW wallet_permissions_view IS NULL;
COMMENT ON FUNCTION wallet_has_permission IS NULL;
COMMENT ON FUNCTION get_wallet_permissions IS NULL;
COMMENT ON FUNCTION refresh_wallet_permissions_view IS NULL;