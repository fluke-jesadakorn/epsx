-- The consolidated v6 baseline owns the support-chat tables and seed data.
-- Keep rollback non-destructive when this incremental compatibility migration
-- is present after that baseline.
SELECT 1 AS compatibility_rollback_preserved;
