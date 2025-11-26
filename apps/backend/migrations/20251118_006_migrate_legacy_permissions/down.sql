-- ================================================================================================
-- ROLLBACK MIGRATION: UNIFIED PERMISSIONS TO LEGACY TABLES
-- ================================================================================================
-- This migration rolls back the data migration from unified permissions to legacy tables.
-- It restores the original multi-table permission system structure.
--
-- WARNING: This will lose any new permission data added after the migration!
-- ================================================================================================

BEGIN;

-- Step 1: Drop legacy views
DROP VIEW IF EXISTS legacy_wallet_direct_permissions;
DROP VIEW IF EXISTS legacy_wallet_group_memberships;

-- Step 2: Restore legacy tables from backup
DROP TABLE IF EXISTS wallet_direct_permissions;
CREATE TABLE wallet_direct_permissions AS
SELECT * FROM archive.wallet_direct_permissions_backup;

DROP TABLE IF EXISTS wallet_group_memberships;
CREATE TABLE wallet_group_memberships AS
SELECT * FROM archive.wallet_group_memberships_backup;

DROP TABLE IF EXISTS permission_group_memberships;
CREATE TABLE permission_group_memberships AS
SELECT * FROM archive.permission_group_memberships_backup;

-- Step 3: Remove migrated data from unified permissions table
DELETE FROM permissions
WHERE source_type IN ('direct', 'group')
  AND wallet_address IS NOT NULL;

-- Step 4: Drop the new columns from permissions table
ALTER TABLE permissions
DROP COLUMN IF EXISTS wallet_address,
DROP COLUMN IF EXISTS source_type,
DROP COLUMN IF EXISTS source_id,
DROP COLUMN IF EXISTS granted_at,
DROP COLUMN IF EXISTS expires_at,
DROP COLUMN IF EXISTS granted_by,
DROP COLUMN IF EXISTS grant_reason;

-- Step 5: Drop constraints and indexes added in previous migration
ALTER TABLE permissions
DROP CONSTRAINT IF EXISTS permissions_valid_wallet,
DROP CONSTRAINT IF EXISTS permissions_valid_dates,
DROP CONSTRAINT IF EXISTS permissions_active_expires;

DROP INDEX IF EXISTS idx_permissions_wallet_lookup;
DROP INDEX IF EXISTS idx_permissions_platform_lookup;
DROP INDEX IF EXISTS idx_permissions_source_lookup;
DROP INDEX IF EXISTS idx_permissions_expiry;
DROP INDEX IF EXISTS idx_permissions_active_time;
DROP INDEX IF EXISTS idx_permissions_full_search;

-- Step 6: Drop materialized view and related functions
DROP MATERIALIZED VIEW IF EXISTS wallet_permissions_view;
DROP FUNCTION IF EXISTS refresh_wallet_permissions_view();
DROP FUNCTION IF EXISTS wallet_has_permission(VARCHAR(42), VARCHAR(255));
DROP FUNCTION IF EXISTS get_wallet_permissions(VARCHAR(42));
DROP FUNCTION IF EXISTS get_permission_stats_by_platform();

-- Step 7: Update migration status
UPDATE migration_stats
SET last_migration_time = NOW(),
    migration_status = 'rolled_back',
    metadata = json_build_object(
        'legacy_tables_restored', true,
        'unified_data_removed', true,
        'rollback_timestamp', NOW()
    )
WHERE migration_name = 'unified_permissions_migration';

COMMIT;

-- ================================================================================================
-- ROLLBACK COMPLETE
-- ================================================================================================
-- The system has been restored to the original multi-table permission structure.
-- Archive tables remain for reference if needed.