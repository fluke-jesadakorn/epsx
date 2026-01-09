-- Down migration for seed_group_permissions
-- Removes all seeded group_permissions entries

DELETE FROM group_permissions 
WHERE group_id IN (
    SELECT id FROM groups WHERE slug IN ('free', 'starter', 'pro', 'enterprise', 'api-developer')
);
