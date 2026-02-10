-- Rollback: Restore Groups from Plans

-- 1. Drop new foreign key constraints
ALTER TABLE wallet_plan_assignments DROP CONSTRAINT IF EXISTS wallet_plan_assignments_plan_id_fkey;
ALTER TABLE plan_permissions DROP CONSTRAINT IF EXISTS plan_permissions_plan_id_fkey;
ALTER TABLE api_key_permissions DROP CONSTRAINT IF EXISTS api_key_permissions_plan_id_fkey;

-- 2. Rename back
ALTER TABLE plans RENAME TO groups;
ALTER TABLE wallet_plan_assignments RENAME TO wallet_group_assignments;
ALTER TABLE wallet_group_assignments RENAME COLUMN plan_id TO group_id;
ALTER TABLE plan_permissions RENAME TO group_permissions;
ALTER TABLE group_permissions RENAME COLUMN plan_id TO group_id;
ALTER TABLE api_key_permissions RENAME COLUMN plan_id TO permission_group_id;

-- 3. Rename indexes back
ALTER INDEX IF EXISTS plans_pkey RENAME TO groups_pkey;
ALTER INDEX IF EXISTS plans_slug_key RENAME TO groups_slug_key;
ALTER INDEX IF EXISTS wallet_plan_assignments_pkey RENAME TO wallet_group_assignments_pkey;
ALTER INDEX IF EXISTS plan_permissions_pkey RENAME TO group_permissions_pkey;

-- 4. Recreate original foreign key constraints
ALTER TABLE wallet_group_assignments ADD CONSTRAINT wallet_group_assignments_group_id_fkey 
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE;

ALTER TABLE group_permissions ADD CONSTRAINT group_permissions_group_id_fkey 
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE;

ALTER TABLE api_key_permissions ADD CONSTRAINT api_key_permissions_permission_group_id_fkey 
    FOREIGN KEY (permission_group_id) REFERENCES groups(id) ON DELETE CASCADE;
