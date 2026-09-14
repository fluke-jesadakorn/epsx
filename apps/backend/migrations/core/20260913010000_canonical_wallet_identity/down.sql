-- Address casing is not an identity change; do not invent prior display casing
-- or recreate accounts when rolling back this schema guard.
ALTER TABLE public.wallet_users DROP CONSTRAINT IF EXISTS wallet_users_canonical_address;
