-- ================================================================================================
-- EMBEDDED PERMISSIONS TO GROUP PERMISSIONS MIGRATION
-- ================================================================================================
-- This migration converts any existing embedded permissions to group-based permissions
-- and ensures the group permission system tables are properly set up
-- ================================================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ================================================================================================
-- 1. ENSURE GROUP PERMISSION SYSTEM TABLES EXIST
-- ================================================================================================

-- Create permission_groups table if it doesn't exist
CREATE TABLE IF NOT EXISTS permission_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    permissions TEXT[] NOT NULL DEFAULT '{}',
    is_system_group BOOLEAN DEFAULT FALSE,
    is_web3_managed BOOLEAN DEFAULT FALSE,
    default_expiry_days INTEGER,
    priority_level INTEGER DEFAULT 0,
    max_members INTEGER,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create user_group_memberships table if it doesn't exist
CREATE TABLE IF NOT EXISTS user_group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    granted_by UUID,
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    assignment_reason TEXT,
    assignment_source VARCHAR(50) DEFAULT 'manual',
    web3_wallet_address VARCHAR(42),
    web3_verification_data JSONB,
    payment_reference VARCHAR(255),
    subscription_tier VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(user_id, group_id)
);

-- Create group_assignment_history table if it doesn't exist
CREATE TABLE IF NOT EXISTS group_assignment_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    group_id UUID NOT NULL,
    action VARCHAR(20) NOT NULL, -- 'granted', 'revoked', 'expired'
    trigger_type VARCHAR(50) NOT NULL, -- 'manual', 'web3', 'payment', 'expiry'
    performed_by UUID,
    reason TEXT,
    old_expires_at TIMESTAMPTZ,
    new_expires_at TIMESTAMPTZ,
    web3_transaction_hash VARCHAR(66),
    payment_reference VARCHAR(255),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- ================================================================================================
-- 2. CREATE VIEW FOR ACTIVE USER GROUPS
-- ================================================================================================

-- Create view for active user group memberships (for performance)
CREATE OR REPLACE VIEW active_user_groups AS
SELECT 
    ugm.user_id,
    ugm.group_id,
    pg.name AS group_name,
    pg.slug,
    pg.permissions,
    ugm.expires_at,
    pg.priority_level,
    ugm.granted_at,
    ugm.assignment_reason,
    ugm.assignment_source
FROM user_group_memberships ugm
JOIN permission_groups pg ON ugm.group_id = pg.id
WHERE ugm.is_active = TRUE
  AND (ugm.expires_at IS NULL OR ugm.expires_at > NOW());

-- ================================================================================================
-- 3. CREATE INDEXES FOR PERFORMANCE
-- ================================================================================================

-- Indexes for permission_groups
CREATE INDEX IF NOT EXISTS idx_permission_groups_slug ON permission_groups(slug);
CREATE INDEX IF NOT EXISTS idx_permission_groups_is_system ON permission_groups(is_system_group);
CREATE INDEX IF NOT EXISTS idx_permission_groups_is_web3 ON permission_groups(is_web3_managed);

-- Indexes for user_group_memberships
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_user_id ON user_group_memberships(user_id);
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_group_id ON user_group_memberships(group_id);
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_active ON user_group_memberships(is_active);
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_expires ON user_group_memberships(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_group_memberships_wallet ON user_group_memberships(web3_wallet_address);

-- Indexes for group_assignment_history
CREATE INDEX IF NOT EXISTS idx_group_assignment_history_user_id ON group_assignment_history(user_id);
CREATE INDEX IF NOT EXISTS idx_group_assignment_history_group_id ON group_assignment_history(group_id);
CREATE INDEX IF NOT EXISTS idx_group_assignment_history_timestamp ON group_assignment_history(timestamp);

-- ================================================================================================
-- 4. MIGRATE EMBEDDED PERMISSIONS TO GROUP PERMISSIONS
-- ================================================================================================

-- Function to migrate embedded permissions from user permissions (if they exist)
DO $$
DECLARE
    user_record RECORD;
    permission_record RECORD;
    group_id UUID;
    user_count INTEGER := 0;
    permission_count INTEGER := 0;
BEGIN
    -- Check if users table exists and has permissions
    IF EXISTS (
        SELECT 1 FROM information_schema.tables 
        WHERE table_name = 'users'
    ) THEN
        -- Look for users with embedded timestamp permissions (format: permission:timestamp)
        FOR user_record IN 
            SELECT DISTINCT id, permissions
            FROM users 
            WHERE permissions IS NOT NULL 
              AND array_length(permissions, 1) > 0
        LOOP
            user_count := user_count + 1;
            
            -- Process each permission for this user
            FOR permission_record IN 
                SELECT UNNEST(user_record.permissions) AS permission_str
            LOOP
                -- Check if this is an embedded timestamp permission (contains more than 3 colons)
                IF (LENGTH(permission_record.permission_str) - LENGTH(REPLACE(permission_record.permission_str, ':', ''))) >= 3 THEN
                    -- This looks like an embedded permission: platform:resource:action:timestamp
                    DECLARE
                        parts TEXT[];
                        base_permission TEXT;
                        expiry_timestamp BIGINT;
                        expiry_time TIMESTAMPTZ;
                        group_name TEXT;
                        group_slug TEXT;
                    BEGIN
                        parts := string_to_array(permission_record.permission_str, ':');
                        
                        -- If last part is numeric (timestamp), it's an embedded permission
                        IF parts[array_length(parts, 1)] ~ '^[0-9]+$' THEN
                            -- Extract base permission (all parts except last)
                            base_permission := array_to_string(parts[1:array_length(parts, 1)-1], ':');
                            expiry_timestamp := parts[array_length(parts, 1)]::BIGINT;
                            
                            -- Convert Unix timestamp to PostgreSQL timestamp
                            expiry_time := to_timestamp(expiry_timestamp);
                            
                            -- Only migrate if permission hasn't expired
                            IF expiry_time > NOW() THEN
                                -- Create temporary group name
                                group_name := 'migrated-' || replace(base_permission, ':', '-') || '-' || user_record.id;
                                group_slug := lower(group_name);
                                
                                -- Create permission group for this embedded permission
                                INSERT INTO permission_groups (
                                    name, slug, description, permissions, is_system_group, 
                                    is_web3_managed, priority_level, max_members
                                ) VALUES (
                                    group_name,
                                    group_slug,
                                    'Migrated from embedded timestamp permission: ' || base_permission,
                                    ARRAY[base_permission],
                                    FALSE,
                                    FALSE,
                                    1, -- low priority
                                    1  -- single user
                                ) 
                                ON CONFLICT (slug) DO NOTHING
                                RETURNING id INTO group_id;
                                
                                -- If group already exists, get its ID
                                IF group_id IS NULL THEN
                                    SELECT id INTO group_id FROM permission_groups WHERE slug = group_slug;
                                END IF;
                                
                                -- Assign user to the group
                                INSERT INTO user_group_memberships (
                                    user_id, group_id, expires_at, assignment_reason, assignment_source
                                ) VALUES (
                                    user_record.id,
                                    group_id,
                                    expiry_time,
                                    'Migrated from embedded timestamp permission',
                                    'migration'
                                ) ON CONFLICT (user_id, group_id) DO NOTHING;
                                
                                -- Log the migration in history
                                INSERT INTO group_assignment_history (
                                    user_id, group_id, action, trigger_type, reason, new_expires_at
                                ) VALUES (
                                    user_record.id,
                                    group_id,
                                    'granted',
                                    'migration',
                                    'Migrated from embedded permission: ' || permission_record.permission_str,
                                    expiry_time
                                );
                                
                                permission_count := permission_count + 1;
                            END IF;
                        END IF;
                    END;
                END IF;
            END LOOP;
        END LOOP;
        
        RAISE NOTICE 'Migration completed: processed % users and migrated % embedded permissions to group permissions', user_count, permission_count;
    ELSE
        RAISE NOTICE 'No users table found, skipping embedded permission migration';
    END IF;
END $$;

-- ================================================================================================
-- 5. CREATE DEFAULT SYSTEM GROUPS
-- ================================================================================================

-- Create default system groups for common permissions
INSERT INTO permission_groups (name, slug, description, permissions, is_system_group, priority_level) VALUES
    ('Admin Full Access', 'admin-full', 'Full administrative access to all features', ARRAY['admin:*:*'], TRUE, 10),
    ('EPSX Analytics View', 'epsx-analytics-view', 'View access to EPSX analytics', ARRAY['epsx:analytics:view'], TRUE, 5),
    ('EPSX Analytics Manage', 'epsx-analytics-manage', 'Manage access to EPSX analytics', ARRAY['epsx:analytics:manage'], TRUE, 7),
    ('EPSX Trading View', 'epsx-trading-view', 'View access to trading features', ARRAY['epsx:trading:view'], TRUE, 5),
    ('EPSX Profile Manage', 'epsx-profile-manage', 'Manage user profile settings', ARRAY['epsx:profile:manage'], TRUE, 3)
ON CONFLICT (slug) DO NOTHING;

-- ================================================================================================
-- 6. CLEAN UP LEGACY PERMISSION DATA (Optional)
-- ================================================================================================

-- NOTE: We don't automatically remove the old permissions array from users table
-- This allows for rollback if needed. Uncomment the following if you want to clean up:

/*
-- Remove permissions array from users table after successful migration
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'permissions'
    ) THEN
        -- Clear the permissions array but don't drop the column (for safety)
        UPDATE users SET permissions = '{}' WHERE permissions IS NOT NULL;
        RAISE NOTICE 'Cleared permissions array from users table (column preserved for rollback)';
    END IF;
END $$;
*/

-- ================================================================================================
-- 7. REFRESH MATERIALIZED VIEWS (if any exist)
-- ================================================================================================

-- Update any materialized views that might depend on permission data
-- (Add specific view refreshes here if needed)

RAISE NOTICE 'Embedded permissions to group permissions migration completed successfully';