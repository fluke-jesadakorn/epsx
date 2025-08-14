-- Migration: Backup important data before schema cleanup
-- This migration creates backup tables for critical data from tables that will be removed
-- to ensure data preservation during the cleanup process

-- Create backup table for any critical permission profile data
CREATE TABLE IF NOT EXISTS permission_profiles_backup AS
SELECT 
    id,
    name,
    description,
    category,
    status,
    profile_data,
    created_at,
    created_by
FROM permission_profiles
WHERE status = 'active' OR profile_data IS NOT NULL;

-- Create backup table for any critical admin permission assignments
CREATE TABLE IF NOT EXISTS admin_permission_assignments_backup AS  
SELECT
    id,
    user_id,
    permission_profile_id,
    assigned_by,
    assignment_type,
    assignment_reason,
    status,
    created_at
FROM admin_permission_profile_assignments
WHERE status = 'active' AND expires_at IS NULL OR expires_at > NOW();

-- Create backup table for sub-modules data in case it contains important configuration
CREATE TABLE IF NOT EXISTS sub_modules_backup AS
SELECT
    id,
    name,
    display_name,
    description,
    category,
    status,
    api_endpoints,
    ui_components,
    access_levels,
    created_at
FROM sub_modules  
WHERE status = 'active';

-- Create backup table for any active user module assignments
CREATE TABLE IF NOT EXISTS user_module_assignments_backup AS
SELECT
    id,
    user_id,
    sub_module_id,
    access_level,
    assignment_reason,
    status,
    created_at
FROM user_sub_module_assignments
WHERE status = 'active';

-- Create backup table for API keys that might still be active
CREATE TABLE IF NOT EXISTS api_keys_backup AS
SELECT
    id,
    key_prefix,
    client_name,
    client_description,
    allowed_modules,
    status,
    created_at,
    last_used_at,
    total_requests
FROM api_keys
WHERE status = 'active' AND (last_used_at IS NULL OR last_used_at > NOW() - INTERVAL '90 days');

-- Create backup for assignment audit logs with important data
CREATE TABLE IF NOT EXISTS assignment_audit_backup AS
SELECT
    id,
    assignment_id,
    action,
    performed_by,
    details,
    timestamp
FROM assignment_audit_log
WHERE timestamp > NOW() - INTERVAL '1 year';

-- Add indexes to backup tables for potential future queries
CREATE INDEX IF NOT EXISTS idx_permission_profiles_backup_name ON permission_profiles_backup(name);
CREATE INDEX IF NOT EXISTS idx_admin_assignments_backup_user ON admin_permission_assignments_backup(user_id);
CREATE INDEX IF NOT EXISTS idx_sub_modules_backup_name ON sub_modules_backup(name);
CREATE INDEX IF NOT EXISTS idx_user_assignments_backup_user ON user_module_assignments_backup(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_backup_client ON api_keys_backup(client_name);
CREATE INDEX IF NOT EXISTS idx_audit_backup_timestamp ON assignment_audit_backup(timestamp);

-- Add comments explaining backup purpose
COMMENT ON TABLE permission_profiles_backup IS 'Backup of active permission profiles before schema cleanup - migration 023';
COMMENT ON TABLE admin_permission_assignments_backup IS 'Backup of active admin permission assignments before schema cleanup - migration 023';
COMMENT ON TABLE sub_modules_backup IS 'Backup of active sub-modules before schema cleanup - migration 023';
COMMENT ON TABLE user_module_assignments_backup IS 'Backup of user module assignments before schema cleanup - migration 023';
COMMENT ON TABLE api_keys_backup IS 'Backup of active API keys before schema cleanup - migration 023';
COMMENT ON TABLE assignment_audit_backup IS 'Backup of assignment audit logs before schema cleanup - migration 023';

-- Create a summary view of what was backed up
CREATE OR REPLACE VIEW backup_summary AS
SELECT 
    'permission_profiles' as table_name,
    COUNT(*) as backed_up_records,
    NOW() as backup_timestamp
FROM permission_profiles_backup
UNION ALL
SELECT 
    'admin_permission_assignments' as table_name,
    COUNT(*) as backed_up_records,
    NOW() as backup_timestamp
FROM admin_permission_assignments_backup
UNION ALL
SELECT 
    'sub_modules' as table_name,
    COUNT(*) as backed_up_records,
    NOW() as backup_timestamp
FROM sub_modules_backup
UNION ALL
SELECT 
    'user_module_assignments' as table_name,
    COUNT(*) as backed_up_records,
    NOW() as backup_timestamp
FROM user_module_assignments_backup
UNION ALL
SELECT 
    'api_keys' as table_name,
    COUNT(*) as backed_up_records,
    NOW() as backup_timestamp
FROM api_keys_backup
UNION ALL
SELECT 
    'assignment_audit' as table_name,
    COUNT(*) as backed_up_records,
    NOW() as backup_timestamp
FROM assignment_audit_backup;

-- Log completion
DO $$
DECLARE
    total_backed_up INT;
BEGIN
    SELECT SUM(backed_up_records) INTO total_backed_up FROM backup_summary;
    
    RAISE NOTICE 'Schema cleanup backup completed:';
    RAISE NOTICE '- Total records backed up: %', total_backed_up;
    RAISE NOTICE '- Backup tables created with _backup suffix';
    RAISE NOTICE '- Backup summary available in backup_summary view';
    RAISE NOTICE '- All backup tables include data preservation indexes';
END $$;

-- Record this migration
INSERT INTO schema_migrations (version, description, executed_at) 
VALUES ('023', 'Backup data before schema cleanup - preserve important data from tables to be removed', NOW())
ON CONFLICT (version) DO NOTHING;