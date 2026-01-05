-- Revert get_wallet_permissions_detailed_working function to use old table names
-- Warning: This will fail if tables permission_groups/permission_group_memberships truly don't exist anymore
-- But strictly speaking, down migration should reverse up migration logic.

CREATE OR REPLACE FUNCTION public.get_wallet_permissions_detailed_working(p_wallet_address character varying)
 RETURNS TABLE(permission_string character varying, permission_id uuid, source_type character varying, source_id uuid, source_name character varying, expires_at timestamp with time zone, granted_at timestamp with time zone, is_permanent boolean)
 LANGUAGE plpgsql
 SECURITY DEFINER
AS $function$
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
    WHERE LOWER(wdp.wallet_address) = LOWER(p_wallet_address)
      AND wdp.is_active = TRUE
      AND p.is_active = TRUE
      AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())

    UNION ALL

    -- Group permissions (Old table names)
    SELECT
        p.permission_string,
        p.id,
        'group'::VARCHAR(20),
        pg.id as source_id,
        pg.name as source_name,
        wga.expires_at,
        wga.assigned_at as granted_at,
        (wga.expires_at IS NULL) as is_permanent
    FROM wallet_group_assignments wga
    INNER JOIN permission_groups pg ON wga.group_id = pg.id
    INNER JOIN permission_group_memberships pgm ON pg.id = pgm.group_id
    INNER JOIN permissions p ON pgm.permission_id = p.id
    WHERE LOWER(wga.wallet_address) = LOWER(p_wallet_address)
      AND wga.is_active = TRUE
      AND pg.is_active = TRUE
      AND p.is_active = TRUE
      AND (wga.expires_at IS NULL OR wga.expires_at > NOW());
END;
$function$;
