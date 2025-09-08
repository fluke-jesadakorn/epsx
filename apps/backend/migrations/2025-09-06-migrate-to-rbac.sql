-- ============================================================================
-- RBAC PERMISSION MIGRATION SCRIPT
-- ============================================================================
-- This script migrates existing string-based permissions to the new RBAC system
-- Run this after the RBAC tables have been created

-- Create system permissions
INSERT INTO rbac_permissions (id, name, platform, resource, action, description, is_system_permission, created_at, updated_at)
VALUES 
  -- Admin permissions
  (gen_random_uuid(), 'admin:*:*', 'admin', '*', '*', 'Full admin access', true, NOW(), NOW()),
  (gen_random_uuid(), 'admin:users:view', 'admin', 'users', 'view', 'View users', true, NOW(), NOW()),
  (gen_random_uuid(), 'admin:users:manage', 'admin', 'users', 'manage', 'Manage users', true, NOW(), NOW()),
  (gen_random_uuid(), 'admin:users:create', 'admin', 'users', 'create', 'Create users', true, NOW(), NOW()),
  (gen_random_uuid(), 'admin:users:edit', 'admin', 'users', 'edit', 'Edit users', true, NOW(), NOW()),
  (gen_random_uuid(), 'admin:users:delete', 'admin', 'users', 'delete', 'Delete users', true, NOW(), NOW()),
  (gen_random_uuid(), 'admin:system:manage', 'admin', 'system', 'manage', 'Manage system', true, NOW(), NOW()),
  (gen_random_uuid(), 'admin:audit:read', 'admin', 'audit', 'read', 'View audit logs', true, NOW(), NOW()),
  (gen_random_uuid(), 'admin:permissions:manage', 'admin', 'permissions', 'manage', 'Manage permissions', true, NOW(), NOW()),
  (gen_random_uuid(), 'admin:roles:manage', 'admin', 'roles', 'manage', 'Manage roles', true, NOW(), NOW()),
  
  -- EPSX platform permissions
  (gen_random_uuid(), 'epsx:analytics:view', 'epsx', 'analytics', 'view', 'View analytics', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx:analytics:export', 'epsx', 'analytics', 'export', 'Export analytics data', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx:analytics:advanced', 'epsx', 'analytics', 'advanced', 'Advanced analytics features', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx:realtime:access', 'epsx', 'realtime', 'access', 'Access real-time data', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx:profile:manage', 'epsx', 'profile', 'manage', 'Manage user profile', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx:profile:edit', 'epsx', 'profile', 'edit', 'Edit user profile', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx:notifications:receive', 'epsx', 'notifications', 'receive', 'Receive notifications', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx:billing:manage', 'epsx', 'billing', 'manage', 'Manage billing', true, NOW(), NOW()),
  
  -- EPSX Pay permissions
  (gen_random_uuid(), 'epsx-pay:payments:view', 'epsx-pay', 'payments', 'view', 'View payments', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx-pay:payments:process', 'epsx-pay', 'payments', 'process', 'Process payments', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx-pay:payments:refund', 'epsx-pay', 'payments', 'refund', 'Refund payments', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx-pay:transactions:view', 'epsx-pay', 'transactions', 'view', 'View transactions', true, NOW(), NOW()),
  
  -- EPSX Token permissions
  (gen_random_uuid(), 'epsx-token:tokens:view', 'epsx-token', 'tokens', 'view', 'View tokens', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx-token:governance:vote', 'epsx-token', 'governance', 'vote', 'Vote in governance', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx-token:governance:propose', 'epsx-token', 'governance', 'propose', 'Create proposals', true, NOW(), NOW()),
  (gen_random_uuid(), 'epsx-token:tokens:stake', 'epsx-token', 'tokens', 'stake', 'Stake tokens', true, NOW(), NOW())
ON CONFLICT (name) DO NOTHING;

-- Create system roles
WITH role_insertions AS (
  INSERT INTO rbac_roles (id, name, description, is_system_role, created_at, updated_at)
  VALUES 
    (gen_random_uuid(), 'super_admin', 'Super administrator with full system access', true, NOW(), NOW()),
    (gen_random_uuid(), 'user_admin', 'User administrator with user management capabilities', true, NOW(), NOW()),
    (gen_random_uuid(), 'system_admin', 'System administrator with system management capabilities', true, NOW(), NOW()),
    (gen_random_uuid(), 'analytics_admin', 'Analytics administrator with full analytics access', true, NOW(), NOW()),
    (gen_random_uuid(), 'analytics_user', 'Analytics user with advanced analytics features', true, NOW(), NOW()),
    (gen_random_uuid(), 'premium_user', 'Premium user with enhanced features', true, NOW(), NOW()),
    (gen_random_uuid(), 'epsx_user', 'Basic EPSX user with core platform access', true, NOW(), NOW()),
    (gen_random_uuid(), 'guest_user', 'Guest user with limited access', true, NOW(), NOW())
  ON CONFLICT (name) DO NOTHING
  RETURNING id, name
)
SELECT 'Roles created: ' || COUNT(*) FROM role_insertions;

-- Assign permissions to roles
-- Super Admin gets all admin permissions
INSERT INTO rbac_role_permissions (role_id, permission_id, created_at, updated_at)
SELECT r.id, p.id, NOW(), NOW()
FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'super_admin' AND p.name = 'admin:*:*'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- User Admin gets user management permissions
INSERT INTO rbac_role_permissions (role_id, permission_id, created_at, updated_at)
SELECT r.id, p.id, NOW(), NOW()
FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'user_admin' AND p.name IN (
  'admin:users:view',
  'admin:users:manage',
  'admin:users:create',
  'admin:users:edit',
  'admin:audit:read'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- System Admin gets system management permissions
INSERT INTO rbac_role_permissions (role_id, permission_id, created_at, updated_at)
SELECT r.id, p.id, NOW(), NOW()
FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'system_admin' AND p.name IN (
  'admin:system:manage',
  'admin:audit:read'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Analytics Admin gets full analytics access
INSERT INTO rbac_role_permissions (role_id, permission_id, created_at, updated_at)
SELECT r.id, p.id, NOW(), NOW()
FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'analytics_admin' AND p.name IN (
  'epsx:analytics:view',
  'epsx:analytics:export',
  'epsx:analytics:advanced',
  'epsx:realtime:access',
  'epsx:profile:manage',
  'epsx:notifications:receive'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Analytics User gets advanced analytics features
INSERT INTO rbac_role_permissions (role_id, permission_id, created_at, updated_at)
SELECT r.id, p.id, NOW(), NOW()
FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'analytics_user' AND p.name IN (
  'epsx:analytics:view',
  'epsx:analytics:export',
  'epsx:analytics:advanced',
  'epsx:realtime:access',
  'epsx:profile:manage',
  'epsx:notifications:receive'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Premium User gets enhanced features
INSERT INTO rbac_role_permissions (role_id, permission_id, created_at, updated_at)
SELECT r.id, p.id, NOW(), NOW()
FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'premium_user' AND p.name IN (
  'epsx:analytics:view',
  'epsx:analytics:export',
  'epsx:profile:manage',
  'epsx:notifications:receive',
  'epsx:billing:manage'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- EPSX User gets basic platform access
INSERT INTO rbac_role_permissions (role_id, permission_id, created_at, updated_at)
SELECT r.id, p.id, NOW(), NOW()
FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'epsx_user' AND p.name IN (
  'epsx:analytics:view',
  'epsx:profile:manage',
  'epsx:notifications:receive'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Guest User gets minimal access
INSERT INTO rbac_role_permissions (role_id, permission_id, created_at, updated_at)
SELECT r.id, p.id, NOW(), NOW()
FROM rbac_roles r, rbac_permissions p
WHERE r.name = 'guest_user' AND p.name IN (
  'epsx:profile:manage'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ============================================================================
-- MIGRATE EXISTING USER PERMISSIONS TO RBAC
-- ============================================================================

-- Create a temporary mapping table for migration
CREATE TEMP TABLE permission_migration_map AS
SELECT DISTINCT permission as old_permission,
  CASE 
    -- Admin permissions
    WHEN permission LIKE '%admin%' OR permission IN ('user_management', 'manage_users') THEN 'admin:users:manage'
    WHEN permission = 'system_management' OR permission = 'manage_system' THEN 'admin:system:manage'
    
    -- Analytics permissions
    WHEN permission IN ('view_eps', 'eps_view') THEN 'epsx:analytics:view'
    WHEN permission IN ('export_data', 'data_export') THEN 'epsx:analytics:export'
    WHEN permission IN ('advanced_filters', 'filters_advanced') THEN 'epsx:analytics:advanced'
    
    -- Profile permissions  
    WHEN permission IN ('profile_manage', 'manage_profile', 'profile_edit', 'edit_profile') THEN 'epsx:profile:manage'
    
    -- Notification permissions
    WHEN permission IN ('notifications', 'receive_notifications') THEN 'epsx:notifications:receive'
    
    -- Real-time permissions
    WHEN permission IN ('realtime', 'realtime_access') THEN 'epsx:realtime:access'
    
    -- Billing permissions
    WHEN permission IN ('billing', 'billing_manage') THEN 'epsx:billing:manage'
    
    -- Already structured permissions - keep as is
    WHEN permission ~ '^[a-z-]+:[a-z-]+:[a-z]+$' THEN permission
    
    -- Default mapping to basic view
    ELSE 'epsx:analytics:view'
  END as new_permission
FROM user_permissions 
WHERE permission IS NOT NULL AND permission != '';

-- Show the mapping for review
DO $$
BEGIN
  RAISE NOTICE 'Permission Migration Mapping:';
END $$;

SELECT 'Mapping: ' || old_permission || ' -> ' || new_permission as mapping
FROM permission_migration_map
ORDER BY old_permission;

-- Migrate user permissions to role assignments based on their permissions
-- First, assign roles based on permission patterns
WITH user_permission_analysis AS (
  SELECT 
    up.user_id,
    ARRAY_AGG(DISTINCT pm.new_permission) as new_permissions,
    ARRAY_AGG(DISTINCT up.permission) as old_permissions,
    -- Determine best role based on permissions
    CASE 
      WHEN 'admin:*:*' = ANY(ARRAY_AGG(DISTINCT pm.new_permission)) OR 
           'admin:users:manage' = ANY(ARRAY_AGG(DISTINCT pm.new_permission)) THEN 'super_admin'
      WHEN ARRAY_LENGTH(ARRAY(SELECT UNNEST(ARRAY_AGG(DISTINCT pm.new_permission)) WHERE UNNEST LIKE 'epsx:analytics:%'), 1) >= 3 THEN 'analytics_user'
      WHEN 'epsx:billing:manage' = ANY(ARRAY_AGG(DISTINCT pm.new_permission)) OR 
           'epsx:analytics:export' = ANY(ARRAY_AGG(DISTINCT pm.new_permission)) THEN 'premium_user'
      WHEN 'epsx:analytics:view' = ANY(ARRAY_AGG(DISTINCT pm.new_permission)) THEN 'epsx_user'
      ELSE 'guest_user'
    END as suggested_role
  FROM user_permissions up
  JOIN permission_migration_map pm ON up.permission = pm.old_permission
  GROUP BY up.user_id
)
INSERT INTO rbac_user_roles (user_id, role_id, created_at, updated_at)
SELECT 
  upa.user_id,
  r.id,
  NOW(),
  NOW()
FROM user_permission_analysis upa
JOIN rbac_roles r ON r.name = upa.suggested_role
ON CONFLICT (user_id, role_id) DO NOTHING;

-- Also create direct permission assignments for permissions not covered by roles
INSERT INTO rbac_user_permissions (user_id, permission_id, permission_type, created_at, updated_at)
SELECT DISTINCT
  up.user_id,
  p.id,
  'grant',
  NOW(),
  NOW()
FROM user_permissions up
JOIN permission_migration_map pm ON up.permission = pm.old_permission
JOIN rbac_permissions p ON p.name = pm.new_permission
ON CONFLICT (user_id, permission_id) DO NOTHING;

-- Update materialized view
REFRESH MATERIALIZED VIEW rbac_user_permissions_with_roles;

-- Show migration statistics
SELECT 'Migration Statistics:' as info;

SELECT 
  'Total Permissions Created: ' || COUNT(*) as stat
FROM rbac_permissions;

SELECT 
  'Total Roles Created: ' || COUNT(*) as stat  
FROM rbac_roles;

SELECT 
  'User Role Assignments: ' || COUNT(*) as stat
FROM rbac_user_roles;

SELECT 
  'User Permission Assignments: ' || COUNT(*) as stat
FROM rbac_user_permissions;

-- Show role distribution
SELECT 
  r.name as role_name,
  COUNT(ur.user_id) as user_count
FROM rbac_roles r
LEFT JOIN rbac_user_roles ur ON r.id = ur.role_id
GROUP BY r.name
ORDER BY user_count DESC;

-- Clean up temp table
DROP TABLE permission_migration_map;

-- Optional: Comment out the old user_permissions table to avoid confusion
-- (Uncomment the line below if you want to rename the old table)
-- ALTER TABLE user_permissions RENAME TO user_permissions_legacy_backup;

COMMIT;