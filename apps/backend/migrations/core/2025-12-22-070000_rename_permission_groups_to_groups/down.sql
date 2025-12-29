-- Rollback: Revert groups → permission_groups

-- Step 1: Rename groups table back to permission_groups
ALTER TABLE groups RENAME TO permission_groups;

-- Step 2: Rename group_permissions back to permission_group_memberships
ALTER TABLE group_permissions RENAME TO permission_group_memberships;

-- Restore original comments
COMMENT ON TABLE permission_groups IS 'Enhanced permission group definitions with pay-per-use billing support';
COMMENT ON TABLE permission_group_memberships IS 'Junction table for permission group to permission mapping';
