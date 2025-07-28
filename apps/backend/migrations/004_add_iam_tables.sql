-- IAM (Identity and Access Management) tables

-- IAM Roles table
CREATE TABLE IF NOT EXISTS iam_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    permissions JSONB NOT NULL DEFAULT '[]',
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- IAM Policies table
CREATE TABLE IF NOT EXISTS iam_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    effect VARCHAR(50) NOT NULL DEFAULT 'Allow', -- Allow or Deny
    actions JSONB NOT NULL DEFAULT '[]',
    resources JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User roles junction table
CREATE TABLE IF NOT EXISTS user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES iam_roles(id) ON DELETE CASCADE,
    assigned_by UUID REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(user_id, role_id)
);

-- Role policies junction table
CREATE TABLE IF NOT EXISTS role_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES iam_roles(id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES iam_policies(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(role_id, policy_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_iam_roles_name ON iam_roles(name);
CREATE INDEX IF NOT EXISTS idx_iam_roles_is_system ON iam_roles(is_system);
CREATE INDEX IF NOT EXISTS idx_iam_policies_name ON iam_policies(name);
CREATE INDEX IF NOT EXISTS idx_iam_policies_effect ON iam_policies(effect);
CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_expires_at ON user_roles(expires_at);
CREATE INDEX IF NOT EXISTS idx_role_policies_role_id ON role_policies(role_id);
CREATE INDEX IF NOT EXISTS idx_role_policies_policy_id ON role_policies(policy_id);

-- Insert default system roles
INSERT INTO iam_roles (name, description, permissions, is_system) VALUES
('admin', 'System administrator with full access', '["*"]', true),
('user', 'Regular user with basic permissions', '["read:profile", "update:profile"]', true),
('moderator', 'Content moderator with limited admin access', '["read:users", "moderate:content"]', true)
ON CONFLICT (name) DO NOTHING;

-- Insert default policies
INSERT INTO iam_policies (name, description, effect, actions, resources) VALUES
('FullAccess', 'Complete system access', 'Allow', '["*"]', '["*"]'),
('ReadOnlyAccess', 'Read-only access to all resources', 'Allow', '["read:*"]', '["*"]'),
('UserSelfManagement', 'Users can manage their own profile', 'Allow', '["read:profile", "update:profile", "delete:profile"]', '["user:self"]'),
('AdminUserManagement', 'Admin can manage all users', 'Allow', '["create:user", "read:user", "update:user", "delete:user"]', '["user:*"]')
ON CONFLICT (name) DO NOTHING;