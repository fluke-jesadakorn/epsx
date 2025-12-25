-- Remove selected_permissions column from api_keys table
DROP INDEX IF EXISTS idx_api_keys_selected_permissions;
ALTER TABLE api_keys DROP COLUMN IF EXISTS selected_permissions;
