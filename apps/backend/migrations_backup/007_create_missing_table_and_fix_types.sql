-- Create user_permission_profile_assignments table if not exists
CREATE TABLE IF NOT EXISTS user_permission_profile_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission_profile_id UUID NOT NULL REFERENCES permission_profiles(id) ON DELETE CASCADE,
    assigned_by UUID NOT NULL REFERENCES users(id),
    assignment_type VARCHAR(50) NOT NULL DEFAULT 'admin',
    assignment_source VARCHAR(50) NOT NULL DEFAULT 'admin_dashboard',
    assignment_reason TEXT,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    deactivated_at TIMESTAMPTZ,
    deactivation_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, permission_profile_id)
);

-- Performance indexes for user_permission_profile_assignments
CREATE INDEX IF NOT EXISTS idx_user_permission_assignments_user_id ON user_permission_profile_assignments(user_id);
CREATE INDEX IF NOT EXISTS idx_user_permission_assignments_profile_id ON user_permission_profile_assignments(permission_profile_id);
CREATE INDEX IF NOT EXISTS idx_user_permission_assignments_status ON user_permission_profile_assignments(status);
CREATE INDEX IF NOT EXISTS idx_user_permission_assignments_expires_at ON user_permission_profile_assignments(expires_at);

-- Fix client_ip column type issues by ensuring proper handling of NULL values
UPDATE audit_logs 
SET client_ip = NULL 
WHERE client_ip IS NOT NULL 
AND client_ip::text = '';