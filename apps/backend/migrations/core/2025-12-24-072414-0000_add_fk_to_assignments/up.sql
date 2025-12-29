-- Clean up any assignments that reference non-existent packages (if any exist)
-- We assume package_id references groups.slug
DELETE FROM stock_ranking_assignments
WHERE package_id NOT IN (SELECT slug FROM groups);

-- Add Foreign Key constraint
ALTER TABLE stock_ranking_assignments
    ADD CONSTRAINT fk_ranking_package
    FOREIGN KEY (package_id)
    REFERENCES groups (slug)
    ON DELETE RESTRICT
    ON UPDATE CASCADE;
