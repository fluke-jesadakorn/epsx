-- Migration: Rename permission_groups → groups
-- This migration renames the tables and updates references for cleaner terminology

-- Step 1: Rename main permission_groups table to groups
ALTER TABLE permission_groups RENAME TO groups;

-- Step 2: Rename permission_group_memberships → group_permissions (relates permissions to groups)
ALTER TABLE permission_group_memberships RENAME TO group_permissions;

-- Step 3: Update indexes (PostgreSQL auto-renames indexes when table is renamed, but we should rename custom ones)
-- The indexes will auto-update with their table names

-- Step 4: Add comments for documentation
COMMENT ON TABLE groups IS 'Groups of permissions that can be assigned to wallets';
COMMENT ON TABLE group_permissions IS 'Junction table mapping which permissions belong to which groups';

-- Note: Foreign key constraints and column references remain unchanged
-- - wallet_group_assignments.group_id still references groups(id)
-- - payments.plan_id still references groups(id)
-- - subscriptions.plan_id still references groups(id)
