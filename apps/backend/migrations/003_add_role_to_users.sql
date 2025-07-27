-- Add role column to users table
ALTER TABLE users ADD COLUMN role VARCHAR(50) NOT NULL DEFAULT 'user';

-- Update the promote_admin user to super_admin if they exist
UPDATE users SET role = 'super_admin' WHERE email = 'jesadakorn.kirtnu@gmail.com';