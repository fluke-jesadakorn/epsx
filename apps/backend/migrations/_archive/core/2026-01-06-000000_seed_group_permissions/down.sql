-- Rollback: Remove all seeded group_permissions entries
-- This is a safe operation as it only removes the entries we seeded

DELETE FROM group_permissions WHERE created_at >= '2026-01-06';

-- Alternative: Remove specific entries by group slug
-- DO $$
-- DECLARE
--     group_ids UUID[];
-- BEGIN
--     SELECT ARRAY(SELECT id FROM groups WHERE slug IN ('free', 'starter', 'pro', 'enterprise', 'api-developer'))
--     INTO group_ids;
--     
--     DELETE FROM group_permissions WHERE group_id = ANY(group_ids);
-- END $$;
