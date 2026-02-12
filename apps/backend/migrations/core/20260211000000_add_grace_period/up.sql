-- Add grace_period_hours to plans table
-- Allows plans to maintain access for N hours after expiry before deactivation
ALTER TABLE plans ADD COLUMN grace_period_hours INTEGER NOT NULL DEFAULT 0;

-- Update permission function to include grace period window
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

    -- Plan permissions (with grace period support)
    SELECT
        p.permission_string,
        p.id,
        'plan'::VARCHAR(20),
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
      AND (wpa.expires_at IS NULL
           OR wpa.expires_at > NOW()
           OR (wpa.expires_at + (pl.grace_period_hours || ' hours')::INTERVAL) > NOW());
END;
$function$;
