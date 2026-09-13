-- The consolidated v6 baseline owns this materialized view and index; preserve
-- them on rollback of the compatibility migration.
SELECT 1 AS compatibility_rollback_preserved;
