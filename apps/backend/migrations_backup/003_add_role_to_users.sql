-- Add role column to users table (if it doesn't exist)
ALTER TABLE users ADD COLUMN IF NOT EXISTS role VARCHAR(50) NOT NULL DEFAULT 'user';

-- Update the promote_admin user to super_admin if they exist
UPDATE users SET role = 'super_admin' WHERE email = 'jesadakorn.kirtnu@gmail.com';