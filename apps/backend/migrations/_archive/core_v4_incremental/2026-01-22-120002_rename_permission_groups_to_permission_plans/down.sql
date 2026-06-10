-- Rollback: Rename permission_plans back to permission_groups
ALTER TABLE wallet_users RENAME COLUMN permission_plans TO permission_groups;
