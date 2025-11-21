-- ================================================================================================
-- UPDATE EXISTING PERMISSIONS TABLE TO UNIFIED STRUCTURE
-- ================================================================================================
-- This migration transforms the existing permissions table to match the new unified structure
-- that combines multiple permission storage mechanisms into a single optimized table.
--
-- This migration:
-- 1. Adds new columns to the existing permissions table for unified structure
-- 2. Creates indexes for optimized query performance
-- 3. Creates the materialized view for wallet permissions
-- 4. Sets up helper functions for common operations
-- ================================================================================================

-- Step 1: Add new columns to existing permissions table
ALTER TABLE permissions
ADD COLUMN IF NOT EXISTS wallet_address VARCHAR(42),
ADD COLUMN IF NOT EXISTS source_type VARCHAR(20) CHECK (source_type IN ('direct', 'group', 'route')),
ADD COLUMN IF NOT EXISTS source_id UUID,
ADD COLUMN IF NOT EXISTS granted_at TIMESTAMPTZ DEFAULT NOW(),
ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS granted_by VARCHAR(255),
ADD COLUMN IF NOT EXISTS grant_reason TEXT;

-- Step 2: Create constraints for the unified structure
ALTER TABLE permissions
ADD CONSTRAINT IF NOT EXISTS permissions_valid_wallet
CHECK (wallet_address IS NULL OR wallet_address ~ '^0x[a-fA-F0-9]{40}$'),
ADD CONSTRAINT IF NOT EXISTS permissions_valid_dates
CHECK (expires_at IS NULL OR expires_at > granted_at),
ADD CONSTRAINT IF NOT EXISTS permissions_active_expires
CHECK (is_active = false OR expires_at IS NULL OR expires_at > NOW());

-- Step 3: Create strategic indexes for Diesel DSL optimization

-- Primary permission lookup (most common query)
-- Used by: validate_wallet_permission(), get_wallet_permissions()
CREATE INDEX IF NOT EXISTS idx_permissions_wallet_lookup
ON permissions (wallet_address, is_active, expires_at)
WHERE wallet_address IS NOT NULL;

-- Platform-specific queries (admin dashboard analytics)
-- Used by: permission_statistics(), platform_analytics()
CREATE INDEX IF NOT EXISTS idx_permissions_platform_lookup
ON permissions (platform, resource, action, is_active)
WHERE is_active = true;

-- Source-based queries (group management)
-- Used by: get_group_permissions(), find_group_permissions()
CREATE INDEX IF NOT EXISTS idx_permissions_source_lookup
ON permissions (source_type, source_id, is_active)
WHERE is_active = true AND source_type IS NOT NULL;

-- Expiration management (cleanup jobs)
-- Used by: cleanup_expired_permissions(), permission_audits()
CREATE INDEX IF NOT EXISTS idx_permissions_expiry
ON permissions (expires_at, is_active)
WHERE expires_at IS NOT NULL;

-- Active permission enumeration (admin lists)
-- Used by: list_all_permissions(), permission_reporting()
CREATE INDEX IF NOT EXISTS idx_permissions_active_time
ON permissions (is_active, granted_at, expires_at)
WHERE is_active = true;

-- Full permission search (admin search functionality)
-- Used by: search_permissions(), admin_permission_search()
CREATE INDEX IF NOT EXISTS idx_permissions_full_search
ON permissions (permission_string, wallet_address, is_active)
WHERE is_active = true;

-- Step 4: Create materialized view for complex wallet-permission queries
-- This view pre-aggregates permissions for wallet listing operations
DROP MATERIALIZED VIEW IF EXISTS wallet_permissions_view;
CREATE MATERIALIZED VIEW wallet_permissions_view AS
SELECT
    -- Wallet information
    wu.wallet_address,
    wu.is_active as wallet_active,
    wu.wallet_metadata,
    wu.created_at as wallet_created,
    wu.updated_at as wallet_updated,
    wu.last_auth_at,

    -- Permission aggregation (JSON)
    COALESCE(
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'id', p.id,
                'permission_string', p.permission_string,
                'platform', p.platform,
                'resource', p.resource,
                'action', p.action,
                'source_type', p.source_type,
                'source_id', p.source_id,
                'granted_at', COALESCE(p.granted_at, p.created_at),
                'expires_at', p.expires_at,
                'granted_by', p.granted_by,
                'grant_reason', p.grant_reason
            )
        ) FILTER (WHERE p.id IS NOT NULL AND p.wallet_address IS NOT NULL),
        '[]'::json
    ) as permissions,

    -- Permission counts for quick stats
    COUNT(p.id) FILTER (WHERE p.id IS NOT NULL AND p.wallet_address IS NOT NULL) as total_permissions,
    COUNT(p.id) FILTER (WHERE p.source_type = 'direct' AND p.wallet_address IS NOT NULL) as direct_permissions,
    COUNT(p.id) FILTER (WHERE p.source_type = 'group' AND p.wallet_address IS NOT NULL) as group_permissions,
    COUNT(p.id) FILTER (WHERE p.expires_at IS NOT NULL AND p.wallet_address IS NOT NULL) as temporary_permissions,

    -- Wallet activity metrics
    CASE
        WHEN wu.last_auth_at IS NOT NULL THEN 'active'
        WHEN wu.created_at > NOW() - INTERVAL '30 days' THEN 'new'
        ELSE 'inactive'
    END as activity_status,

    -- Creation timestamp (for view refresh)
    NOW() as view_refreshed_at

FROM wallet_users wu
LEFT JOIN permissions p ON wu.wallet_address = p.wallet_address
    AND p.is_active = true
    AND (p.expires_at IS NULL OR p.expires_at > NOW())
GROUP BY
    wu.wallet_address, wu.is_active, wu.wallet_metadata,
    wu.created_at, wu.updated_at, wu.last_auth_at;

-- Create indexes for the materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_wallet_permissions_view_address
ON wallet_permissions_view (wallet_address);

CREATE INDEX IF NOT EXISTS idx_wallet_permissions_view_activity
ON wallet_permissions_view (activity_status, wallet_created DESC);

CREATE INDEX IF NOT EXISTS idx_wallet_permissions_view_stats
ON wallet_permissions_view (total_permissions DESC, direct_permissions DESC);

-- Step 5: Create helper functions for common permission operations

-- Function: Check if wallet has specific permission
CREATE OR REPLACE FUNCTION wallet_has_permission(
    p_wallet_address VARCHAR(42),
    p_permission_string VARCHAR(255)
) RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM permissions
        WHERE wallet_address = p_wallet_address
          AND permission_string = p_permission_string
          AND is_active = true
          AND (expires_at IS NULL OR expires_at > NOW())
    );
END;
$$ LANGUAGE plpgsql;

-- Function: Get all permissions for wallet
CREATE OR REPLACE FUNCTION get_wallet_permissions(
    p_wallet_address VARCHAR(42)
) RETURNS JSON AS $$
BEGIN
    RETURN COALESCE(
        (
            SELECT JSON_AGG(
                JSON_BUILD_OBJECT(
                    'permission_string', permission_string,
                    'platform', platform,
                    'resource', resource,
                    'action', action,
                    'source_type', source_type,
                    'granted_at', COALESCE(granted_at, created_at),
                    'expires_at', expires_at
                )
            )
            FROM permissions
            WHERE wallet_address = p_wallet_address
              AND is_active = true
              AND (expires_at IS NULL OR expires_at > NOW())
        ),
        '[]'::json
    );
END;
$$ LANGUAGE plpgsql;

-- Function: Count active permissions by platform
CREATE OR REPLACE FUNCTION get_permission_stats_by_platform()
RETURNS TABLE(platform VARCHAR(50), permission_count BIGINT, wallet_count BIGINT) AS $$
BEGIN
    RETURN QUERY
    SELECT
        p.platform,
        COUNT(*)::BIGINT as permission_count,
        COUNT(DISTINCT p.wallet_address)::BIGINT as wallet_count
    FROM permissions p
    WHERE p.is_active = true
      AND p.wallet_address IS NOT NULL
      AND (p.expires_at IS NULL OR p.expires_at > NOW())
    GROUP BY p.platform
    ORDER BY permission_count DESC;
END;
$$ LANGUAGE plpgsql;

-- Function: Refresh the materialized view
CREATE OR REPLACE FUNCTION refresh_wallet_permissions_view()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY wallet_permissions_view;
END;
$$ LANGUAGE plpgsql;

-- Step 6: Create trigger to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_permissions_updated_at ON permissions;
CREATE TRIGGER update_permissions_updated_at
    BEFORE UPDATE ON permissions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Step 7: Grant permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON permissions TO epsx_app;
GRANT SELECT ON wallet_permissions_view TO epsx_app;
GRANT EXECUTE ON FUNCTION refresh_wallet_permissions_view() TO epsx_app;
GRANT EXECUTE ON FUNCTION wallet_has_permission(VARCHAR(42), VARCHAR(255)) TO epsx_app;
GRANT EXECUTE ON FUNCTION get_wallet_permissions(VARCHAR(42)) TO epsx_app;
GRANT EXECUTE ON FUNCTION get_permission_stats_by_platform() TO epsx_app;

-- Step 8: Add comments for documentation
COMMENT ON COLUMN permissions.wallet_address IS 'Wallet address that has this permission (for unified permissions table)';
COMMENT ON COLUMN permissions.source_type IS 'Source type: direct, group, or route (for unified permissions table)';
COMMENT ON COLUMN permissions.source_id IS 'Source ID for group-based permissions (for unified permissions table)';
COMMENT ON COLUMN permissions.granted_at IS 'When this permission was granted (for unified permissions table)';
COMMENT ON COLUMN permissions.expires_at IS 'When this permission expires (NULL for permanent)';
COMMENT ON COLUMN permissions.granted_by IS 'Who granted this permission (admin wallet address)';
COMMENT ON COLUMN permissions.grant_reason IS 'Reason for granting this permission';
COMMENT ON MATERIALIZED VIEW wallet_permissions_view IS 'Pre-aggregated wallet permissions for optimized dashboard queries';

-- Refresh the materialized view to populate it with current data
SELECT refresh_wallet_permissions_view();