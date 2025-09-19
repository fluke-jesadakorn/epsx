-- Add missing role column and seed test data
-- This migration adds the role column that the UserRepositoryAdapter expects
-- and creates the test user for development/testing

-- Add role column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS role VARCHAR(50) DEFAULT 'user';

-- Add index for role-based queries
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- Insert test user (info@epsx.io) with proper UUID and Firebase UID
-- This is the test user referenced in CLAUDE.local.md
INSERT INTO users (
    id,
    firebase_uid,
    email,
    display_name,
    name,
    role,
    is_active,
    email_verified,
    created_at,
    updated_at
) VALUES (
    '550e8400-e29b-41d4-a716-446655440000'::UUID,
    'test_user_info_epsx_io',
    'info@epsx.io',
    'EPSX Test User',
    'Test User',
    'admin',
    true,
    true,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
) ON CONFLICT (email) DO UPDATE SET
    firebase_uid = EXCLUDED.firebase_uid,
    display_name = EXCLUDED.display_name,
    name = EXCLUDED.name,
    role = EXCLUDED.role,
    is_active = EXCLUDED.is_active,
    email_verified = EXCLUDED.email_verified,
    updated_at = CURRENT_TIMESTAMP;

-- Grant admin permissions to the test user
INSERT INTO user_permissions (
    user_id,
    permission,
    granted_at,
    granted_by,
    expires_at,
    is_active
) VALUES (
    '550e8400-e29b-41d4-a716-446655440000'::UUID,
    'admin:*:*',
    CURRENT_TIMESTAMP,
    '550e8400-e29b-41d4-a716-446655440000'::UUID, -- Self-granted for bootstrap
    NULL, -- No expiry
    true
) ON CONFLICT DO NOTHING;

-- Insert system user for migrations that reference system@epsx.io
INSERT INTO users (
    id,
    firebase_uid,
    email,
    display_name,
    name,
    role,
    is_active,
    email_verified,
    created_at,
    updated_at
) VALUES (
    '00000000-0000-0000-0000-000000000001'::UUID,
    'system_user_epsx',
    'system@epsx.io',
    'EPSX System',
    'System User',
    'admin',
    true,
    true,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
) ON CONFLICT (email) DO UPDATE SET
    firebase_uid = EXCLUDED.firebase_uid,
    display_name = EXCLUDED.display_name,
    name = EXCLUDED.name,
    role = EXCLUDED.role,
    is_active = EXCLUDED.is_active,
    email_verified = EXCLUDED.email_verified,
    updated_at = CURRENT_TIMESTAMP;

-- Grant system admin permissions
INSERT INTO user_permissions (
    user_id,
    permission,
    granted_at,
    granted_by,
    expires_at,
    is_active
) VALUES (
    '00000000-0000-0000-0000-000000000001'::UUID,
    'admin:*:*',
    CURRENT_TIMESTAMP,
    '00000000-0000-0000-0000-000000000001'::UUID,
    NULL,
    true
) ON CONFLICT DO NOTHING;

-- Add a few more test users for development
INSERT INTO users (
    id,
    firebase_uid,
    email,
    display_name,
    name,
    role,
    is_active,
    email_verified,
    created_at,
    updated_at
) VALUES 
(
    '550e8400-e29b-41d4-a716-446655440001'::UUID,
    'test_user_admin_epsx_io',
    'admin@epsx.io',
    'EPSX Admin',
    'Admin User',
    'admin',
    true,
    true,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
),
(
    '550e8400-e29b-41d4-a716-446655440002'::UUID,
    'test_user_user_epsx_io',
    'user@epsx.io',
    'EPSX User',
    'Regular User',
    'user',
    true,
    true,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
),
(
    '550e8400-e29b-41d4-a716-446655440003'::UUID,
    'test_user_premium_epsx_io',
    'premium@epsx.io',
    'EPSX Premium User',
    'Premium User',
    'premium_user',
    true,
    true,
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
)
ON CONFLICT (email) DO UPDATE SET
    firebase_uid = EXCLUDED.firebase_uid,
    display_name = EXCLUDED.display_name,
    name = EXCLUDED.name,
    role = EXCLUDED.role,
    is_active = EXCLUDED.is_active,
    email_verified = EXCLUDED.email_verified,
    updated_at = CURRENT_TIMESTAMP;

-- Grant permissions to test users
INSERT INTO user_permissions (user_id, permission, granted_at, granted_by, expires_at, is_active) VALUES
-- Admin user gets full admin permissions
('550e8400-e29b-41d4-a716-446655440001'::UUID, 'admin:*:*', CURRENT_TIMESTAMP, '550e8400-e29b-41d4-a716-446655440000'::UUID, NULL, true),

-- Regular user gets basic permissions
('550e8400-e29b-41d4-a716-446655440002'::UUID, 'epsx:basic:read', CURRENT_TIMESTAMP, '550e8400-e29b-41d4-a716-446655440000'::UUID, NULL, true),
('550e8400-e29b-41d4-a716-446655440002'::UUID, 'epsx:rankings:view:1', CURRENT_TIMESTAMP, '550e8400-e29b-41d4-a716-446655440000'::UUID, NULL, true),

-- Premium user gets enhanced permissions
('550e8400-e29b-41d4-a716-446655440003'::UUID, 'epsx:rankings:view:10', CURRENT_TIMESTAMP, '550e8400-e29b-41d4-a716-446655440000'::UUID, NULL, true),
('550e8400-e29b-41d4-a716-446655440003'::UUID, 'epsx:analytics:premium', CURRENT_TIMESTAMP, '550e8400-e29b-41d4-a716-446655440000'::UUID, NULL, true),
('550e8400-e29b-41d4-a716-446655440003'::UUID, 'epsx:export:csv', CURRENT_TIMESTAMP, '550e8400-e29b-41d4-a716-446655440000'::UUID, NULL, true)

ON CONFLICT DO NOTHING;

-- Update existing permission hierarchy entries that failed due to missing system user
UPDATE permission_hierarchy 
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;