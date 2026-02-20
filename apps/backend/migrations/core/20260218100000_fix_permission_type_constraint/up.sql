-- Fix permissions_type_check to include 'system' type
-- The seed_admin_plans seeder uses 'system' for admin permissions (admin:*:*)
-- but the CHECK constraint only allowed manual/nft_gated/token_gated/dao_governance

ALTER TABLE permissions DROP CONSTRAINT IF EXISTS permissions_type_check;
ALTER TABLE permissions ADD CONSTRAINT permissions_type_check CHECK (
    permission_type IN ('manual', 'nft_gated', 'token_gated', 'dao_governance', 'system')
);
