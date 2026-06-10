-- Revert to original constraint (will fail if 'system' permissions exist)
ALTER TABLE permissions DROP CONSTRAINT IF EXISTS permissions_type_check;
ALTER TABLE permissions ADD CONSTRAINT permissions_type_check CHECK (
    permission_type IN ('manual', 'nft_gated', 'token_gated', 'dao_governance')
);
