-- Revert changes
ALTER TABLE stock_ranking_assignments
    DROP CONSTRAINT IF EXISTS fk_ranking_package;
