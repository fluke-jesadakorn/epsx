-- The consolidated v6 baseline owns this index; preserve it on rollback of
-- the compatibility migration.
SELECT 1 AS compatibility_rollback_preserved;
