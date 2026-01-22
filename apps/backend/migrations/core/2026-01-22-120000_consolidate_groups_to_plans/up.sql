-- Migration: Consolidate Groups into Plans
-- This migration renames 'groups' to 'plans' and 'wallet_group_assignments' to 'wallet_plan_assignments'
-- to unify the permission/subscription system under a single "Plan" concept

-- 1. Drop foreign key constraints first
ALTER TABLE wallet_group_assignments DROP CONSTRAINT IF EXISTS wallet_group_assignments_group_id_fkey;
ALTER TABLE group_permissions DROP CONSTRAINT IF EXISTS group_permissions_group_id_fkey;
ALTER TABLE api_key_permissions DROP CONSTRAINT IF EXISTS api_key_permissions_permission_group_id_fkey;

-- 2. Rename groups table to plans
ALTER TABLE groups RENAME TO plans;

-- 3. Rename wallet_group_assignments to wallet_plan_assignments
ALTER TABLE wallet_group_assignments RENAME TO wallet_plan_assignments;
ALTER TABLE wallet_plan_assignments RENAME COLUMN group_id TO plan_id;

-- 4. Rename group_permissions to plan_permissions
ALTER TABLE group_permissions RENAME TO plan_permissions;
ALTER TABLE plan_permissions RENAME COLUMN group_id TO plan_id;

-- 5. Update api_key_permissions column name
ALTER TABLE api_key_permissions RENAME COLUMN permission_group_id TO plan_id;

-- 6. Rename indexes
ALTER INDEX IF EXISTS groups_pkey RENAME TO plans_pkey;
ALTER INDEX IF EXISTS groups_slug_key RENAME TO plans_slug_key;
ALTER INDEX IF EXISTS wallet_group_assignments_pkey RENAME TO wallet_plan_assignments_pkey;
ALTER INDEX IF EXISTS group_permissions_pkey RENAME TO plan_permissions_pkey;

-- 7. Recreate foreign key constraints with new names
ALTER TABLE wallet_plan_assignments ADD CONSTRAINT wallet_plan_assignments_plan_id_fkey 
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE;

ALTER TABLE plan_permissions ADD CONSTRAINT plan_permissions_plan_id_fkey 
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE;

ALTER TABLE api_key_permissions ADD CONSTRAINT api_key_permissions_plan_id_fkey 
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE;

-- 8. Update any sequences if needed
-- (Diesel typically uses UUIDs, so no sequence updates needed)

-- 9. Add comment for clarity
COMMENT ON TABLE plans IS 'Unified subscription plans with permissions, pricing, and rate limits (formerly groups)';
COMMENT ON TABLE wallet_plan_assignments IS 'Assigns wallets to subscription plans (formerly wallet_group_assignments)';
COMMENT ON TABLE plan_permissions IS 'Maps permissions to plans (formerly group_permissions)';
