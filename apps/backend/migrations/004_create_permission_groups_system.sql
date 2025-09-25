-- ============================================================================
-- PERMISSION GROUPS SYSTEM - Create core group-based permission architecture
-- ============================================================================
-- This migration creates the foundational group-based permission system
-- that will be used by the Web3 bridge and admin systems
-- ============================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ============================================================================
-- 1. PERMISSION GROUPS TABLE - Core groups with permissions
-- ============================================================================

CREATE TABLE IF NOT EXISTS permission_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Group Identification
    name VARCHAR(255) NOT NULL UNIQUE,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    
    -- Group Type and Configuration
    group_type VARCHAR(50) DEFAULT 'manual', -- 'manual', 'auto', 'web3', 'temporal'
    is_system_group BOOLEAN DEFAULT FALSE,
    is_web3_managed BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Permissions
    permissions TEXT[] NOT NULL DEFAULT '{}',
    
    -- Group Hierarchy and Priority
    parent_group_id UUID REFERENCES permission_groups(id) ON DELETE SET NULL,
    priority_level INTEGER DEFAULT 0,
    
    -- Group Metadata
    group_metadata JSONB DEFAULT '{}',
    tags TEXT[] DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- 2. USER GROUP MEMBERSHIPS TABLE - Track user assignments to groups
-- ============================================================================

CREATE TABLE IF NOT EXISTS user_group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Membership Details
    user_id UUID NOT NULL, -- References users(id) when available
    group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    
    -- Assignment Details
    assigned_by_admin BOOLEAN DEFAULT FALSE,
    assigned_by_system BOOLEAN DEFAULT TRUE,
    assignment_reason TEXT,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Expiry (for temporary assignments)
    expires_at TIMESTAMPTZ,
    
    -- Metadata
    membership_metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(user_id, group_id),
    CONSTRAINT valid_assignment_type CHECK (assigned_by_admin != assigned_by_system)
);

-- ============================================================================
-- 3. GROUP ASSIGNMENT HISTORY TABLE - Audit trail for group assignments
-- ============================================================================

CREATE TABLE IF NOT EXISTS group_assignment_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Assignment Details
    user_id UUID NOT NULL,
    group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    
    -- Action Details
    action VARCHAR(20) NOT NULL, -- 'assigned', 'removed', 'expired', 'updated'
    reason TEXT,
    
    -- Actor Details
    assigned_by_user_id UUID,
    assigned_by_system VARCHAR(100), -- System/service name
    
    -- Metadata
    previous_membership_data JSONB DEFAULT '{}',
    new_membership_data JSONB DEFAULT '{}',
    
    -- Timestamp
    action_timestamp TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_action CHECK (action IN ('assigned', 'removed', 'expired', 'updated'))
);

-- ============================================================================
-- 4. CREATE INDEXES FOR PERFORMANCE
-- ============================================================================

-- Permission groups indexes
CREATE INDEX IF NOT EXISTS idx_permission_groups_type ON permission_groups(group_type);
CREATE INDEX IF NOT EXISTS idx_permission_groups_active ON permission_groups(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_permission_groups_system ON permission_groups(is_system_group) WHERE is_system_group = TRUE;
CREATE INDEX IF NOT EXISTS idx_permission_groups_slug ON permission_groups(slug);

-- User group memberships indexes
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_user ON user_group_memberships(user_id);
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_group ON user_group_memberships(group_id);
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_active ON user_group_memberships(user_id, is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_expires ON user_group_memberships(expires_at) WHERE expires_at IS NOT NULL;

-- Group assignment history indexes
CREATE INDEX IF NOT EXISTS idx_group_assignment_history_user ON group_assignment_history(user_id);
CREATE INDEX IF NOT EXISTS idx_group_assignment_history_group ON group_assignment_history(group_id);
CREATE INDEX IF NOT EXISTS idx_group_assignment_history_action ON group_assignment_history(action, action_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_group_assignment_history_timestamp ON group_assignment_history(action_timestamp DESC);

-- ============================================================================
-- 5. CREATE BASIC FUNCTIONS
-- ============================================================================

-- Function to get user permissions from groups
CREATE OR REPLACE FUNCTION get_user_permissions_from_groups(target_user_id UUID)
RETURNS TEXT[] AS $$
DECLARE
    user_permissions TEXT[] := '{}';
BEGIN
    SELECT ARRAY(
        SELECT DISTINCT unnest(pg.permissions)
        FROM permission_groups pg
        JOIN user_group_memberships ugm ON pg.id = ugm.group_id
        WHERE ugm.user_id = target_user_id 
        AND ugm.is_active = TRUE
        AND pg.is_active = TRUE
        AND (ugm.expires_at IS NULL OR ugm.expires_at > NOW())
        ORDER BY unnest(pg.permissions)
    ) INTO user_permissions;
    
    RETURN user_permissions;
END;
$$ LANGUAGE plpgsql;

-- Function to check if user has specific permission
CREATE OR REPLACE FUNCTION user_has_permission(target_user_id UUID, permission_name TEXT)
RETURNS BOOLEAN AS $$
DECLARE
    has_permission BOOLEAN := FALSE;
BEGIN
    SELECT EXISTS(
        SELECT 1
        FROM permission_groups pg
        JOIN user_group_memberships ugm ON pg.id = ugm.group_id
        WHERE ugm.user_id = target_user_id
        AND ugm.is_active = TRUE
        AND pg.is_active = TRUE
        AND (ugm.expires_at IS NULL OR ugm.expires_at > NOW())
        AND permission_name = ANY(pg.permissions)
    ) INTO has_permission;
    
    RETURN has_permission;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 6. CREATE TRIGGERS FOR AUDIT TRAIL
-- ============================================================================

-- Trigger function to log group membership changes
CREATE OR REPLACE FUNCTION log_group_membership_changes()
RETURNS TRIGGER AS $$
BEGIN
    -- Log assignment
    IF TG_OP = 'INSERT' THEN
        INSERT INTO group_assignment_history (
            user_id, group_id, action, reason,
            assigned_by_system, new_membership_data
        ) VALUES (
            NEW.user_id, NEW.group_id, 'assigned',
            COALESCE(NEW.assignment_reason, 'Group membership assigned'),
            'permission_groups_system',
            to_jsonb(NEW)
        );
        RETURN NEW;
    END IF;
    
    -- Log removal
    IF TG_OP = 'DELETE' THEN
        INSERT INTO group_assignment_history (
            user_id, group_id, action, reason,
            assigned_by_system, previous_membership_data
        ) VALUES (
            OLD.user_id, OLD.group_id, 'removed',
            'Group membership removed',
            'permission_groups_system',
            to_jsonb(OLD)
        );
        RETURN OLD;
    END IF;
    
    -- Log update
    IF TG_OP = 'UPDATE' THEN
        INSERT INTO group_assignment_history (
            user_id, group_id, action, reason,
            assigned_by_system, previous_membership_data, new_membership_data
        ) VALUES (
            NEW.user_id, NEW.group_id, 'updated',
            'Group membership updated',
            'permission_groups_system',
            to_jsonb(OLD), to_jsonb(NEW)
        );
        RETURN NEW;
    END IF;
    
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for group membership audit
CREATE TRIGGER trigger_log_group_membership_changes
    AFTER INSERT OR UPDATE OR DELETE ON user_group_memberships
    FOR EACH ROW
    EXECUTE FUNCTION log_group_membership_changes();

-- ============================================================================
-- 7. INSERT DEFAULT GROUPS
-- ============================================================================

-- Insert basic system groups
INSERT INTO permission_groups (
    name, slug, description, permissions, 
    is_system_group, is_active, priority_level
) VALUES 
(
    'Basic Users',
    'basic-users',
    'Default group for all authenticated users',
    ARRAY['epsx:profile:view', 'epsx:analytics:basic'],
    TRUE, TRUE, 1
),
(
    'Premium Users',
    'premium-users',
    'Users with premium access',
    ARRAY[
        'epsx:analytics:view', 
        'epsx:analytics:export',
        'epsx:profile:manage',
        'epsx:notifications:receive'
    ],
    TRUE, TRUE, 5
),
(
    'Admin Users',
    'admin-users',
    'Administrative users with full access',
    ARRAY[
        'admin:*:*',
        'epsx:*:*',
        'admin:users:manage',
        'admin:permissions:manage',
        'admin:system:manage'
    ],
    TRUE, TRUE, 10
)
ON CONFLICT (slug) DO NOTHING;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE 'Permission Groups System migration completed successfully!';
    RAISE NOTICE 'Created: 3 core tables (permission_groups, user_group_memberships, group_assignment_history)';
    RAISE NOTICE 'Created: 12 indexes, 2 functions, 1 trigger';
    RAISE NOTICE 'Inserted: 3 default system groups';
    RAISE NOTICE 'System ready for group-based permissions!';
END $$;