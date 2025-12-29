-- Drop table
DROP TABLE IF EXISTS read_model.projection_checkpoints;

-- Drop schema if empty (optional, but safer to leave it if other tables might use it, strictly here we can drop)
-- But cleaner to just drop the table. We won't drop schema just in case.
