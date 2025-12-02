-- ================================================================================================
-- ROLLBACK PERMISSION FUNCTION BUG FIX
-- ================================================================================================
-- This migration rolls back the fix for the permission function bug
-- It restores the original buggy version of get_wallet_permissions_detailed_working

DROP FUNCTION IF EXISTS get_wallet_permissions_detailed_working(VARCHAR(42));

-- Restore the original buggy function (for rollback purposes)
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

    -- Group permissions (BUGGY VERSION: uses incorrect pg.group_id reference)
    SELECT
        p.permission_string,
        p.id,
        'group'::VARCHAR(20),
        pg.group_id as source_id,  -- BUG: this column doesn't exist
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
