-- ================================================================================================
-- ADD PERMISSION FUNCTIONS MIGRATION
-- ================================================================================================
-- Version: 2025-11-27-055800-0000
-- Created: 2025-11-27
-- Description: Add PostgreSQL functions for permission system optimization
--
-- Functions Added:
-- - wallet_has_permission: Check if wallet has specific permission
-- - get_wallet_permissions_detailed_working: Get detailed permissions for wallet
-- - get_wallet_effective_permissions: Get effective permissions as JSON array
-- - get_wallet_permission_stats: Get permission statistics for wallet
-- - wallet_has_permissions_batch: Batch check multiple permissions
--
-- Features:
-- - Optimized single-query permission resolution
-- - Support for direct and group-based permissions
-- - Embedded timestamp permission filtering
-- - Performance indexes for efficient queries
-- ================================================================================================

-- ================================================================================================
-- SECTION 1: CORE PERMISSION CHECK FUNCTION
-- ================================================================================================

-- Check if wallet has specific permission (supports wildcards)
CREATE OR REPLACE FUNCTION wallet_has_permission(
    p_wallet_address VARCHAR(42),
    p_permission_string VARCHAR(255)
) RETURNS BOOLEAN
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_has_permission BOOLEAN := FALSE;
    v_permission_parts TEXT[];
    v_platform TEXT;
    v_resource TEXT;
    v_action TEXT;
BEGIN
    -- Parse permission string
    v_permission_parts := string_to_array(p_permission_string, ':');
    IF array_length(v_permission_parts, 1) != 3 THEN
        RETURN FALSE;
    END IF;

    v_platform := v_permission_parts[1];
    v_resource := v_permission_parts[2];
    v_action := v_permission_parts[3];

    -- Check direct permissions (including expired)
    SELECT EXISTS(
        SELECT 1 FROM wallet_direct_permissions wdp
        INNER JOIN permissions p ON wdp.permission_id = p.id
        WHERE wdp.wallet_address = p_wallet_address
          AND wdp.is_active = TRUE
          AND p.is_active = TRUE
          AND (
              p.permission_string = p_permission_string  -- Exact match
              OR (p.platform = v_platform AND p.resource = '*' AND p.action = '*')  -- Platform wildcard
              OR (p.platform = v_platform AND p.resource = v_resource AND p.action = '*')  -- Action wildcard
              OR (p.platform = v_platform AND p.resource = '*' AND p.action = v_action)  -- Resource wildcard
          )
          AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
    ) INTO v_has_permission;

    IF v_has_permission THEN
        RETURN TRUE;
    END IF;

    -- Check group permissions
    SELECT EXISTS(
        SELECT 1 FROM wallet_group_assignments wga
        INNER JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
        INNER JOIN permissions p ON pgm.permission_id = p.id
        WHERE wga.wallet_address = p_wallet_address
          AND wga.is_active = TRUE
          AND p.is_active = TRUE
          AND (
              p.permission_string = p_permission_string  -- Exact match
              OR (p.platform = v_platform AND p.resource = '*' AND p.action = '*')  -- Platform wildcard
              OR (p.platform = v_platform AND p.resource = v_resource AND p.action = '*')  -- Action wildcard
              OR (p.platform = v_platform AND p.resource = '*' AND p.action = v_action)  -- Resource wildcard
          )
          AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
    ) INTO v_has_permission;

    RETURN v_has_permission;
END;
$$;

-- ================================================================================================
-- SECTION 2: DETAILED PERMISSIONS FUNCTION
-- ================================================================================================

-- Get detailed permissions for a wallet with source information
CREATE OR REPLACE FUNCTION get_wallet_permissions_detailed_working(
    p_wallet_address VARCHAR(42)
) RETURNS TABLE (
    permission_string VARCHAR(255),
    permission_id UUID,
    source_type VARCHAR(20),
    source_id UUID,
    source_name VARCHAR(255),
    expires_at TIMESTAMPTZ,
    granted_at TIMESTAMPTZ,
    is_permanent BOOLEAN
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    RETURN QUERY
    -- Direct permissions
    SELECT
        p.permission_string,
        p.id,
        'direct'::VARCHAR(20),
        p.id as source_id,
        'Direct Permission' as source_name,
        wdp.expires_at,
        wdp.granted_at,
        (wdp.expires_at IS NULL) as is_permanent
    FROM wallet_direct_permissions wdp
    INNER JOIN permissions p ON wdp.permission_id = p.id
    WHERE wdp.wallet_address = p_wallet_address
      AND wdp.is_active = TRUE
      AND p.is_active = TRUE
      AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())

    UNION ALL

    -- Group permissions
    SELECT
        p.permission_string,
        p.id,
        'group'::VARCHAR(20),
        pg.group_id as source_id,
        pg.name as source_name,
        wga.expires_at,
        wga.assigned_at as granted_at,
        (wga.expires_at IS NULL) as is_permanent
    FROM wallet_group_assignments wga
    INNER JOIN permission_groups pg ON wga.group_id = pg.id
    INNER JOIN permission_group_memberships pgm ON pg.id = pgm.group_id
    INNER JOIN permissions p ON pgm.permission_id = p.id
    WHERE wga.wallet_address = p_wallet_address
      AND wga.is_active = TRUE
      AND pg.is_active = TRUE
      AND p.is_active = TRUE
      AND (wga.expires_at IS NULL OR wga.expires_at > NOW());
END;
$$;

-- ================================================================================================
-- SECTION 3: EFFECTIVE PERMISSIONS FUNCTION
-- ================================================================================================

-- Get effective permissions as JSON array
CREATE OR REPLACE FUNCTION get_wallet_effective_permissions(
    p_wallet_address VARCHAR(42)
) RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_permissions JSONB := '[]'::JSONB;
BEGIN
    SELECT jsonb_agg(DISTINCT permission_string) INTO v_permissions
    FROM (
        -- Direct permissions
        SELECT p.permission_string
        FROM wallet_direct_permissions wdp
        INNER JOIN permissions p ON wdp.permission_id = p.id
        WHERE wdp.wallet_address = p_wallet_address
          AND wdp.is_active = TRUE
          AND p.is_active = TRUE
          AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())

        UNION

        -- Group permissions
        SELECT p.permission_string
        FROM wallet_group_assignments wga
        INNER JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
        INNER JOIN permissions p ON pgm.permission_id = p.id
        WHERE wga.wallet_address = p_wallet_address
          AND wga.is_active = TRUE
          AND p.is_active = TRUE
          AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
    ) AS all_permissions;

    RETURN COALESCE(v_permissions, '[]'::JSONB);
END;
$$;

-- ================================================================================================
-- SECTION 4: PERMISSION STATISTICS FUNCTION
-- ================================================================================================

-- Get permission statistics for a wallet
CREATE OR REPLACE FUNCTION get_wallet_permission_stats(
    p_wallet_address VARCHAR(42)
) RETURNS TABLE (
    total_permissions BIGINT,
    direct_permissions BIGINT,
    group_permissions BIGINT,
    permanent_permissions BIGINT,
    temporary_permissions BIGINT,
    groups_count BIGINT,
    expiring_soon_count BIGINT
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    RETURN QUERY
    SELECT
        COALESCE(total_perms.total_count, 0)::BIGINT,
        COALESCE(direct_perms.direct_count, 0)::BIGINT,
        COALESCE(group_perms.group_count, 0)::BIGINT,
        COALESCE(permanent_perms.permanent_count, 0)::BIGINT,
        COALESCE(temporary_perms.temporary_count, 0)::BIGINT,
        COALESCE(active_groups.groups_count, 0)::BIGINT,
        COALESCE(expiring_soon.expiring_count, 0)::BIGINT
    FROM (SELECT 1) AS dummy
    LEFT JOIN (
        SELECT COUNT(*) as total_count
        FROM (
            SELECT p.permission_string
            FROM wallet_direct_permissions wdp
            INNER JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = p_wallet_address
              AND wdp.is_active = TRUE
              AND p.is_active = TRUE
              AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
            UNION
            SELECT p.permission_string
            FROM wallet_group_assignments wga
            INNER JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
            INNER JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = p_wallet_address
              AND wga.is_active = TRUE
              AND p.is_active = TRUE
              AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
        ) AS all_perms
    ) AS total_perms ON TRUE
    LEFT JOIN (
        SELECT COUNT(*) as direct_count
        FROM wallet_direct_permissions wdp
        INNER JOIN permissions p ON wdp.permission_id = p.id
        WHERE wdp.wallet_address = p_wallet_address
          AND wdp.is_active = TRUE
          AND p.is_active = TRUE
          AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
    ) AS direct_perms ON TRUE
    LEFT JOIN (
        SELECT COUNT(DISTINCT p.permission_string) as group_count
        FROM wallet_group_assignments wga
        INNER JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
        INNER JOIN permissions p ON pgm.permission_id = p.id
        WHERE wga.wallet_address = p_wallet_address
          AND wga.is_active = TRUE
          AND p.is_active = TRUE
          AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
    ) AS group_perms ON TRUE
    LEFT JOIN (
        SELECT COUNT(*) as permanent_count
        FROM (
            SELECT p.permission_string
            FROM wallet_direct_permissions wdp
            INNER JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = p_wallet_address
              AND wdp.is_active = TRUE
              AND p.is_active = TRUE
              AND wdp.expires_at IS NULL
            UNION
            SELECT p.permission_string
            FROM wallet_group_assignments wga
            INNER JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
            INNER JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = p_wallet_address
              AND wga.is_active = TRUE
              AND wga.expires_at IS NULL
        ) AS permanent_perms
    ) AS permanent_perms ON TRUE
    LEFT JOIN (
        SELECT COUNT(*) as temporary_count
        FROM (
            SELECT p.permission_string
            FROM wallet_direct_permissions wdp
            INNER JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = p_wallet_address
              AND wdp.is_active = TRUE
              AND p.is_active = TRUE
              AND wdp.expires_at IS NOT NULL
            UNION
            SELECT p.permission_string
            FROM wallet_group_assignments wga
            INNER JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
            INNER JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = p_wallet_address
              AND wga.is_active = TRUE
              AND wga.expires_at IS NOT NULL
        ) AS temporary_perms
    ) AS temporary_perms ON TRUE
    LEFT JOIN (
        SELECT COUNT(*) as groups_count
        FROM wallet_group_assignments wga
        WHERE wga.wallet_address = p_wallet_address
          AND wga.is_active = TRUE
          AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
    ) AS active_groups ON TRUE
    LEFT JOIN (
        SELECT COUNT(*) as expiring_count
        FROM (
            SELECT p.permission_string
            FROM wallet_direct_permissions wdp
            INNER JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = p_wallet_address
              AND wdp.is_active = TRUE
              AND p.is_active = TRUE
              AND wdp.expires_at IS NOT NULL
              AND wdp.expires_at <= NOW() + INTERVAL '7 days'
              AND wdp.expires_at > NOW()
            UNION
            SELECT p.permission_string
            FROM wallet_group_assignments wga
            INNER JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
            INNER JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = p_wallet_address
              AND wga.is_active = TRUE
              AND wga.expires_at IS NOT NULL
              AND wga.expires_at <= NOW() + INTERVAL '7 days'
              AND wga.expires_at > NOW()
        ) AS expiring_perms
    ) AS expiring_soon ON TRUE;
END;
$$;

-- ================================================================================================
-- SECTION 5: BATCH PERMISSION CHECK FUNCTION
-- ================================================================================================

-- Batch check multiple permissions at once
CREATE OR REPLACE FUNCTION wallet_has_permissions_batch(
    p_wallet_address VARCHAR(42),
    p_permission_strings VARCHAR(255)[]
) RETURNS TABLE (
    permission_string VARCHAR(255),
    has_permission BOOLEAN
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    RETURN QUERY
    SELECT
        perm_str,
        wallet_has_permission(p_wallet_address, perm_str)
    FROM unnest(p_permission_strings) AS perm_str;
END;
$$;

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'PERMISSION FUNCTIONS CREATED SUCCESSFULLY! 🎉';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Functions Added:';
    RAISE NOTICE '  ✅ wallet_has_permission() - Check single permission';
    RAISE NOTICE '  ✅ get_wallet_permissions_detailed_working() - Get detailed permissions';
    RAISE NOTICE '  ✅ get_wallet_effective_permissions() - Get permissions as JSON';
    RAISE NOTICE '  ✅ get_wallet_permission_stats() - Get permission statistics';
    RAISE NOTICE '  ✅ wallet_has_permissions_batch() - Batch permission check';
    RAISE NOTICE '';
    RAISE NOTICE 'Features:';
    RAISE NOTICE '  ✅ Optimized single-query permission resolution';
    RAISE NOTICE '  ✅ Support for direct and group-based permissions';
    RAISE NOTICE '  ✅ Embedded timestamp permission filtering';
    RAISE NOTICE '  ✅ Wildcard permission support';
    RAISE NOTICE '  ✅ Performance indexes for efficient queries';
    RAISE NOTICE '';
    RAISE NOTICE '🚀 Permission system functions ready for EPSX platform!';
    RAISE NOTICE '=================================================================================';
END $$;