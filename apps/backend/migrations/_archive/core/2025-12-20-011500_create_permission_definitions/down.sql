-- Drop trigger first
DROP TRIGGER IF EXISTS trigger_permission_definitions_updated_at ON permission_definitions;
DROP FUNCTION IF EXISTS update_permission_definitions_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_permission_definitions_platform;
DROP INDEX IF EXISTS idx_permission_definitions_category;
DROP INDEX IF EXISTS idx_permission_definitions_is_active;
DROP INDEX IF EXISTS idx_permission_definitions_permission;

-- Drop the table
DROP TABLE IF EXISTS permission_definitions;
