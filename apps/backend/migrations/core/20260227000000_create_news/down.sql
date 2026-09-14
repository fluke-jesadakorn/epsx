-- The consolidated v6 baseline owns news_articles and its existing rows.
-- Preserve them on rollback of this compatibility migration.
SELECT 1 AS compatibility_rollback_preserved;
