-- Grant admin permissions to info@epsx.io
-- This script grants full admin access to the specified user

-- First, check if the user exists
SELECT id, email, firebase_uid FROM users WHERE email = 'info@epsx.io';

-- Grant admin:*:* permission to info@epsx.io user
INSERT INTO user_permissions (user_id, permission, granted_at, is_active)
SELECT 
    u.id,
    'admin:*:*',
    CURRENT_TIMESTAMP,
    true
FROM users u 
WHERE u.email = 'info@epsx.io'
AND NOT EXISTS (
    SELECT 1 FROM user_permissions up 
    WHERE up.user_id = u.id 
    AND up.permission = 'admin:*:*' 
    AND up.is_active = true
);

-- Also grant some basic permissions for the analytics platform
INSERT INTO user_permissions (user_id, permission, granted_at, is_active)
SELECT 
    u.id,
    permission,
    CURRENT_TIMESTAMP,
    true
FROM users u 
CROSS JOIN (VALUES 
    ('epsx:*:*'),
    ('epsx:analytics:*'),
    ('epsx:dashboard:*'),
    ('admin:users:*'),
    ('admin:permissions:*'),
    ('admin:analytics:*')
) AS perms(permission)
WHERE u.email = 'info@epsx.io'
AND NOT EXISTS (
    SELECT 1 FROM user_permissions up 
    WHERE up.user_id = u.id 
    AND up.permission = perms.permission 
    AND up.is_active = true
);

-- Verify permissions were granted
SELECT u.email, up.permission, up.granted_at, up.is_active 
FROM users u 
JOIN user_permissions up ON u.id = up.user_id 
WHERE u.email = 'info@epsx.io' 
AND up.is_active = true
ORDER BY up.granted_at DESC;