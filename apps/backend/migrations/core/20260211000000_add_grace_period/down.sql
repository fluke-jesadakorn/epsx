-- Remove grace_period_hours column
ALTER TABLE plans DROP COLUMN IF EXISTS grace_period_hours;

-- Restore original permission function without grace period
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

    -- Plan permissions
    SELECT
        p.permission_string,
        p.id,
        'group'::VARCHAR(20),
        pl.id as source_id,
        pl.name as source_name,
        wpa.expires_at,
        wpa.assigned_at as granted_at,
        (wpa.expires_at IS NULL) as is_permanent
    FROM wallet_plan_assignments wpa
    INNER JOIN plans pl ON wpa.plan_id = pl.id
    INNER JOIN plan_permissions pp ON pl.id = pp.plan_id
    INNER JOIN permissions p ON pp.permission_id = p.id
    WHERE LOWER(wpa.wallet_address) = LOWER(p_wallet_address)
      AND wpa.is_active = TRUE
      AND pl.is_active = TRUE
      AND p.is_active = TRUE
      AND (wpa.expires_at IS NULL OR wpa.expires_at > NOW());
END;
$function$;
