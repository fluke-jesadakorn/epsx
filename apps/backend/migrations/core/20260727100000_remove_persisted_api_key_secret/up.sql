-- API-key secrets must be returned once at creation and never persisted.
-- Existing rows are not otherwise modified; the column itself is removed as
-- the required structural security change.
ALTER TABLE api_keys DROP COLUMN IF EXISTS full_key;
