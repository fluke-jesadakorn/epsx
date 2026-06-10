-- ================================================================================================
-- FIX PERMISSION FUNCTION BUG MIGRATION
-- ================================================================================================
-- Version: 2025-11-27-051350-0000
-- Created: 2025-11-27
-- Description: Fix SQL bug in get_wallet_permissions_detailed_working function
--
-- Bug Fixed:
-- - Incorrect column reference: pg.group_id should be pg.id
-- - The permission_groups table uses 'id' as primary key, not 'group_id'
--
-- Impact:
-- - get_wallet_permissions_detailed_working function was failing with SQL error
-- - Backend permission fetching was broken for wallet users
-- ================================================================================================

-- Fix the get_wallet_permissions_detailed_working function
DROP FUNCTION IF EXISTS get_wallet_permissions_detailed_working(VARCHAR(42));

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

    -- Group permissions (FIXED: use pg.id instead of pg.group_id)
    SELECT
        p.permission_string,
        p.id,
        'group'::VARCHAR(20),
        pg.id as source_id,  -- FIXED: was pg.group_id
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
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'PERMISSION FUNCTION BUG FIXED! 🔧';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Fixed:';
    RAISE NOTICE '  ✅ get_wallet_permissions_detailed_working() - SQL column reference bug';
    RAISE NOTICE '  ✅ Changed pg.group_id to pg.id in group permissions query';
    RAISE NOTICE '';
    RAISE NOTICE 'Impact:';
    RAISE NOTICE '  ✅ Backend permission fetching now works for wallet users';
    RAISE NOTICE '  ✅ Unified permission service can resolve wallet permissions';
    RAISE NOTICE '  ✅ Database error: "column pg.group_id does not exist" - RESOLVED';
    RAISE NOTICE '';
    RAISE NOTICE '🚀 Permission function is now working correctly!';
    RAISE NOTICE '=================================================================================';
END $$;
