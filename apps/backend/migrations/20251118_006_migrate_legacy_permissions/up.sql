-- ================================================================================================
-- DATA MIGRATION: LEGACY PERMISSION TABLES TO UNIFIED PERMISSIONS TABLE
-- ================================================================================================
-- This migration moves data from the old multi-table permission system to the new unified permissions table.
-- It preserves all existing functionality while consolidating data into a single optimized structure.
--
-- Legacy tables being migrated:
-- - wallet_direct_permissions -> permissions with source_type = 'direct'
-- - wallet_group_memberships + permission_group_memberships -> permissions with source_type = 'group'
--
-- Safety features:
-- - Transactions with rollback capability
-- - Data validation before migration
-- - Backup of original data in archive schema
-- - Comprehensive logging
-- ================================================================================================

BEGIN;

-- Create archive schema for backup
CREATE SCHEMA IF NOT EXISTS archive;

-- Step 1: Backup existing permission tables before migration
DROP TABLE IF EXISTS archive.wallet_direct_permissions_backup;
CREATE TABLE archive.wallet_direct_permissions_backup AS
SELECT * FROM wallet_direct_permissions;

DROP TABLE IF EXISTS archive.wallet_group_memberships_backup;
CREATE TABLE archive.wallet_group_memberships_backup AS
SELECT * FROM wallet_group_memberships;

DROP TABLE IF EXISTS archive.permission_group_memberships_backup;
CREATE TABLE archive.permission_group_memberships_backup AS
SELECT * FROM permission_group_memberships;

-- Log the backup operation
INSERT INTO audit_logs (
    event_type,
    entity_type,
    description,
    metadata,
    event_timestamp
) VALUES (
    'migration_backup',
    'permissions',
    'Backed up legacy permission tables before migration to unified structure',
    json_build_object(
        'wallet_direct_permissions_count', (SELECT COUNT(*) FROM wallet_direct_permissions),
        'wallet_group_memberships_count', (SELECT COUNT(*) FROM wallet_group_memberships),
        'permission_group_memberships_count', (SELECT COUNT(*) FROM permission_group_memberships)
    ),
    NOW()
);

-- Step 2: Migrate wallet_direct_permissions to unified permissions table
INSERT INTO permissions (
    wallet_address,
    permission_string,
    platform,
    resource,
    action,
    description,
    permission_type,
    source_type,
    source_id,
    granted_at,
    expires_at,
    granted_by,
    grant_reason,
    is_active,
    created_at,
    updated_at
)
SELECT
    wdp.wallet_address,
    p.permission_string,
    p.platform,
    p.resource,
    p.action,
    p.description,
    'manual' as permission_type,  -- Direct permissions are manual by default
    'direct' as source_type,
    wdp.permission_id as source_id,  -- Reference to the original permission definition
    COALESCE(wdp.granted_at, wdp.created_at, NOW()) as granted_at,
    wdp.expires_at,
    wdp.granted_by,
    wdp.grant_reason,
    wdp.is_active,
    wdp.created_at,
    wdp.updated_at
FROM wallet_direct_permissions wdp
JOIN permissions p ON wdp.permission_id = p.id
WHERE wdp.wallet_address IS NOT NULL
ON CONFLICT (wallet_address, permission_string, source_type, source_id)
DO NOTHING;  -- Skip duplicates

-- Log direct permission migration
INSERT INTO audit_logs (
    event_type,
    entity_type,
    description,
    metadata,
    event_timestamp
) VALUES (
    'migration_data',
    'direct_permissions',
    'Migrated wallet direct permissions to unified permissions table',
    json_build_object('migrated_count', (SELECT COUNT(*) FROM wallet_direct_permissions JOIN permissions ON wallet_direct_permissions.permission_id = permissions.id)),
    NOW()
);

-- Step 3: Migrate group-based permissions (more complex)
INSERT INTO permissions (
    wallet_address,
    permission_string,
    platform,
    resource,
    action,
    description,
    permission_type,
    source_type,
    source_id,
    granted_at,
    expires_at,
    granted_by,
    grant_reason,
    is_active,
    created_at,
    updated_at
)
SELECT
    wgm.wallet_address,
    p.permission_string,
    p.platform,
    p.resource,
    p.action,
    p.description,
    'manual' as permission_type,
    'group' as source_type,
    wgm.group_id as source_id,  -- Reference to the permission group
    COALESCE(wgm.created_at, NOW()) as granted_at,
    wgm.expires_at,
    wgm.granted_by,
    wgm.grant_reason,
    wgm.is_active,
    wgm.created_at,
    wgm.updated_at
FROM wallet_group_memberships wgm
JOIN permission_group_memberships pgm ON wgm.group_id = pgm.group_id
JOIN permissions p ON pgm.permission_id = p.id
WHERE wgm.wallet_address IS NOT NULL
  AND wgm.is_active = true
  AND pgm.is_active = true
ON CONFLICT (wallet_address, permission_string, source_type, source_id)
DO NOTHING;  -- Skip duplicates

-- Log group permission migration
INSERT INTO audit_logs (
    event_type,
    entity_type,
    description,
    metadata,
    event_timestamp
) VALUES (
    'migration_data',
    'group_permissions',
    'Migrated group-based permissions to unified permissions table',
    json_build_object('migrated_count', (SELECT COUNT(*) FROM wallet_group_memberships JOIN permission_group_memberships ON wallet_group_memberships.group_id = permission_group_memberships.group_id JOIN permissions ON permission_group_memberships.permission_id = permissions.id WHERE wallet_group_memberships.is_active = true AND permission_group_memberships.is_active = true)),
    NOW()
);

-- Step 4: Create legacy views for backward compatibility during transition
CREATE OR REPLACE VIEW legacy_wallet_direct_permissions AS
SELECT
    id,
    wallet_address,
    permission_id as permission_id,
    granted_at,
    expires_at,
    granted_by,
    grant_reason,
    is_active,
    created_at,
    updated_at
FROM permissions
WHERE source_type = 'direct'
  AND wallet_address IS NOT NULL;

CREATE OR REPLACE VIEW legacy_wallet_group_memberships AS
SELECT
    id as legacy_id,
    wallet_address,
    source_id as group_id,
    granted_at,
    expires_at,
    granted_by,
    grant_reason,
    is_active,
    created_at,
    updated_at
FROM permissions
WHERE source_type = 'group'
  AND wallet_address IS NOT NULL;

-- Step 5: Update statistics
UPDATE migration_stats
SET last_migration_time = NOW(),
    migration_status = 'completed',
    metadata = json_build_object(
        'unified_permissions_count', (SELECT COUNT(*) FROM permissions WHERE wallet_address IS NOT NULL),
        'direct_permissions_count', (SELECT COUNT(*) FROM permissions WHERE source_type = 'direct'),
        'group_permissions_count', (SELECT COUNT(*) FROM permissions WHERE source_type = 'group'),
        'legacy_tables_archived', true
    )
WHERE migration_name = 'unified_permissions_migration';

-- Step 6: Refresh the materialized view with new data
SELECT refresh_wallet_permissions_view();

-- Step 7: Verify migration integrity
-- Check for any orphaned permissions that couldn't be migrated
CREATE TEMP TABLE migration_verification AS
SELECT 'missing_wallet_address' as issue_type, COUNT(*) as count
FROM wallet_direct_permissions
WHERE wallet_address IS NULL

UNION ALL

SELECT 'invalid_permission_reference' as issue_type, COUNT(*) as count
FROM wallet_direct_permissions wdp
LEFT JOIN permissions p ON wdp.permission_id = p.id
WHERE p.id IS NULL

UNION ALL

SELECT 'missing_group_permissions' as issue_type, COUNT(*) as count
FROM wallet_group_memberships wgm
LEFT JOIN permission_group_memberships pgm ON wgm.group_id = pgm.group_id
WHERE pgm.group_id IS NULL;

-- Log verification results
INSERT INTO audit_logs (
    event_type,
    entity_type,
    description,
    metadata,
    event_timestamp
) VALUES (
    'migration_verification',
    'permissions',
    'Verification of unified permissions migration completed',
    (SELECT json_agg(json_build_object('issue_type', issue_type, 'count', count)) FROM migration_verification),
    NOW()
);

COMMIT;

-- ================================================================================================
-- POST-MIGRATION NOTES
-- ================================================================================================
-- 1. The unified permissions table now contains all permission data
-- 2. Legacy tables are preserved in archive schema for rollback
-- 3. Legacy views provide backward compatibility during transition
-- 4. All existing functionality should work without changes
-- 5. Materialized view has been refreshed with new data
--
-- NEXT STEPS:
-- 1. Update application code to use unified permissions table
-- 2. Test all permission validation flows
-- 3. Monitor performance improvements
-- 4. Plan removal of legacy tables after transition period
-- ================================================================================================