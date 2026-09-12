-- Administrative assignment changes supersede the entitlement projection they replace.
-- Financial purchases/events are retained and future purchases receive separate grants.
ALTER TABLE pay_assignment_baselines ADD COLUMN IF NOT EXISTS projection JSONB;
ALTER TABLE pay_purchase_grants ADD COLUMN IF NOT EXISTS superseded BOOLEAN NOT NULL DEFAULT false;
UPDATE pay_assignment_baselines b
SET projection = coalesce((SELECT jsonb_build_object('expires_at',a.expires_at,'is_active',a.is_active)
 FROM wallet_plan_assignments a WHERE lower(a.wallet_address)=lower(b.wallet_address) AND a.plan_id=b.plan_id LIMIT 1),'null'::jsonb)
WHERE projection IS NULL;
