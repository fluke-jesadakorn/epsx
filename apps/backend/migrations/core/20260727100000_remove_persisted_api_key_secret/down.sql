-- Secret values cannot be recovered after the forward migration. Recreate
-- only the nullable compatibility column if a rollback is explicitly needed.
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS full_key VARCHAR(128);
