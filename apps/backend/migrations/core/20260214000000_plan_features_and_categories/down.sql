-- Revert plan features & categories
DROP TABLE IF EXISTS plan_features;
DROP TABLE IF EXISTS features;
-- plan_category, its constraint, and idx_plans_category belong to the
-- consolidated v6 baseline and must survive rollback of this incremental
-- compatibility migration.
