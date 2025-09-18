-- Remove role column from users table
DROP INDEX IF EXISTS idx_users_role;
ALTER TABLE users DROP COLUMN role;