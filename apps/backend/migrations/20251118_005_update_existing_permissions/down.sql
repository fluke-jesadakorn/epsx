-- ================================================================================================
-- ROLLBACK UNIFIED PERMISSIONS TABLE UPDATES
-- ================================================================================================
-- This migration rolls back the updates to the existing permissions table
-- and removes the materialized view and helper functions.
-- ================================================================================================

-- Drop helper functions
DROP FUNCTION IF EXISTS get_permission_stats_by_platform() CASCADE;
DROP FUNCTION IF EXISTS get_wallet_permissions(VARCHAR(42)) CASCADE;
DROP FUNCTION IF EXISTS wallet_has_permission(VARCHAR(42), VARCHAR(255)) CASCADE;
DROP FUNCTION IF EXISTS refresh_wallet_permissions_view() CASCADE;

-- Drop materialized view and indexes
DROP MATERIALIZED VIEW IF EXISTS wallet_permissions_view;

-- Drop trigger and function
DROP TRIGGER IF EXISTS update_permissions_updated_at ON permissions;
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

-- Drop indexes
DROP INDEX IF EXISTS idx_permissions_wallet_lookup;
DROP INDEX IF EXISTS idx_permissions_platform_lookup;
DROP INDEX IF EXISTS idx_permissions_source_lookup;
DROP INDEX IF EXISTS idx_permissions_expiry;
DROP INDEX IF EXISTS idx_permissions_active_time;
DROP INDEX IF EXISTS idx_permissions_full_search;

-- Drop constraints
ALTER TABLE permissions DROP CONSTRAINT IF EXISTS permissions_valid_wallet;
ALTER TABLE permissions DROP CONSTRAINT IF EXISTS permissions_valid_dates;
ALTER TABLE permissions DROP CONSTRAINT IF EXISTS permissions_active_expires;

-- Drop columns (this will preserve data if needed, otherwise use DROP COLUMN CASCADE)
ALTER TABLE permissions DROP COLUMN IF EXISTS wallet_address;
ALTER TABLE permissions DROP COLUMN IF EXISTS source_type;
ALTER TABLE permissions DROP COLUMN IF EXISTS source_id;
ALTER TABLE permissions DROP COLUMN IF EXISTS granted_at;
ALTER TABLE permissions DROP COLUMN IF EXISTS expires_at;
ALTER TABLE permissions DROP COLUMN IF EXISTS granted_by;
ALTER TABLE permissions DROP COLUMN IF EXISTS grant_reason;

-- Remove comments
COMMENT ON COLUMN permissions.wallet_address IS NULL;
COMMENT ON COLUMN permissions.source_type IS NULL;
COMMENT ON COLUMN permissions.source_id IS NULL;
COMMENT ON COLUMN permissions.granted_at IS NULL;
COMMENT ON COLUMN permissions.expires_at IS NULL;
COMMENT ON COLUMN permissions.granted_by IS NULL;
COMMENT ON COLUMN permissions.grant_reason IS NULL;