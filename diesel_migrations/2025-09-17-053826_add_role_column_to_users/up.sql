-- Add role column to users table for legacy compatibility
ALTER TABLE users ADD COLUMN role VARCHAR(50) DEFAULT 'user';

-- Update existing users with appropriate roles based on their RBAC role assignments
UPDATE users SET role = 'admin' 
WHERE id IN (
    SELECT DISTINCT ur.user_id 
    FROM rbac_user_roles ur
    JOIN rbac_roles r ON ur.role_id = r.id
    WHERE r.name IN ('admin', 'super_admin') 
    AND ur.is_active = true
    AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
);

-- Create index for performance
CREATE INDEX idx_users_role ON users(role);