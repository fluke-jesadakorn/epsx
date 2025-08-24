-- Migration: Remove legacy role columns and migrate to permission-based system
-- This migration completes the role-to-permission system migration

-- Step 1: Create admin_modules table if not exists for User entity
CREATE TABLE IF NOT EXISTS user_admin_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    admin_module VARCHAR(50) NOT NULL,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    assigned_by UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, admin_module),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Step 2: Create package_tiers table to manage package tier assignments
CREATE TABLE IF NOT EXISTS user_package_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    package_tier VARCHAR(20) NOT NULL DEFAULT 'FREE',
    expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT valid_package_tier CHECK (package_tier IN ('FREE', 'BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'ENTERPRISE'))
);

-- Step 3: Migrate existing role data to new permission system
-- This migration assumes existing role data needs to be preserved
DO $$
DECLARE
    user_record RECORD;
    old_role TEXT;
BEGIN
    -- Only proceed if role column exists
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'role'
    ) THEN
        
        RAISE NOTICE 'Starting role-to-permission migration...';
        
        -- Iterate through all users with roles
        FOR user_record IN 
            SELECT id, firebase_uid, email, role
            FROM users 
            WHERE role IS NOT NULL
        LOOP
            old_role := user_record.role;
            
            -- Insert package tier based on old role
            INSERT INTO user_package_tiers (user_id, package_tier, is_active)
            VALUES (
                user_record.id,
                CASE 
                    WHEN old_role IN ('super_admin', 'admin') THEN 'ENTERPRISE'
                    WHEN old_role = 'moderator' THEN 'GOLD'
                    WHEN old_role = 'premium' THEN 'SILVER'
                    WHEN old_role = 'user' THEN 'BRONZE'
                    ELSE 'FREE'
                END,
                true
            ) ON CONFLICT (user_id) DO UPDATE SET
                package_tier = EXCLUDED.package_tier,
                updated_at = NOW();
            
            -- Insert admin modules based on old role
            CASE old_role
                WHEN 'super_admin' THEN
                    -- SuperAdmin gets all admin modules
                    INSERT INTO user_admin_modules (user_id, admin_module) VALUES
                        (user_record.id, 'user_operations'),
                        (user_record.id, 'permission_admin'),
                        (user_record.id, 'role_policy_manager'),
                        (user_record.id, 'analytics_specialist'),
                        (user_record.id, 'billing_admin'),
                        (user_record.id, 'system_admin'),
                        (user_record.id, 'developer_relations'),
                        (user_record.id, 'module_coordinator'),
                        (user_record.id, 'compliance_audit'),
                        (user_record.id, 'support_specialist')
                    ON CONFLICT (user_id, admin_module) DO NOTHING;
                    
                WHEN 'admin' THEN
                    -- Admin gets core admin modules
                    INSERT INTO user_admin_modules (user_id, admin_module) VALUES
                        (user_record.id, 'user_operations'),
                        (user_record.id, 'permission_admin'),
                        (user_record.id, 'billing_admin'),
                        (user_record.id, 'system_admin')
                    ON CONFLICT (user_id, admin_module) DO NOTHING;
                    
                WHEN 'moderator' THEN
                    -- Moderator gets limited admin modules
                    INSERT INTO user_admin_modules (user_id, admin_module) VALUES
                        (user_record.id, 'user_operations'),
                        (user_record.id, 'support_specialist')
                    ON CONFLICT (user_id, admin_module) DO NOTHING;
                    
                ELSE
                    -- Regular users get no admin modules
                    NULL;
            END CASE;
            
            RAISE NOTICE 'Migrated user % (%) from role % to permission system', 
                user_record.email, user_record.id, old_role;
        END LOOP;
        
        RAISE NOTICE 'Role-to-permission migration completed successfully';
        
    ELSE
        RAISE NOTICE 'Role column does not exist, skipping role migration';
    END IF;
END$$;

-- Step 4: Add admin_modules and package_tier columns to users table
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS admin_modules TEXT[] DEFAULT '{}',
ADD COLUMN IF NOT EXISTS package_tier VARCHAR(20) DEFAULT 'FREE';

-- Update users table with data from the new tables
UPDATE users SET 
    admin_modules = COALESCE(am.modules, '{}'),
    package_tier = COALESCE(pt.package_tier, 'FREE')
FROM (
    SELECT 
        user_id,
        array_agg(admin_module) as modules
    FROM user_admin_modules 
    GROUP BY user_id
) am
LEFT JOIN user_package_tiers pt ON pt.user_id = am.user_id
WHERE users.id = am.user_id;

-- Update users without admin modules
UPDATE users SET 
    package_tier = COALESCE(pt.package_tier, 'FREE')
FROM user_package_tiers pt
WHERE users.id = pt.user_id 
AND users.admin_modules = '{}';

-- Step 5: Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_admin_modules_user_id ON user_admin_modules(user_id);
CREATE INDEX IF NOT EXISTS idx_user_admin_modules_module ON user_admin_modules(admin_module);
CREATE INDEX IF NOT EXISTS idx_user_package_tiers_user_id ON user_package_tiers(user_id);
CREATE INDEX IF NOT EXISTS idx_user_package_tiers_tier ON user_package_tiers(package_tier);
CREATE INDEX IF NOT EXISTS idx_users_admin_modules ON users USING GIN(admin_modules);
CREATE INDEX IF NOT EXISTS idx_users_package_tier ON users(package_tier);

-- Step 6: Drop role column if it exists (final step)
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'role'
    ) THEN
        ALTER TABLE users DROP COLUMN role;
        RAISE NOTICE 'Dropped legacy role column from users table';
    ELSE
        RAISE NOTICE 'Role column does not exist, nothing to drop';
    END IF;
END$$;

-- Step 7: Add constraints to ensure data integrity
ALTER TABLE users
ADD CONSTRAINT valid_package_tier CHECK (package_tier IN ('FREE', 'BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'ENTERPRISE'));

-- Step 8: Create audit trail for role migration
CREATE TABLE IF NOT EXISTS role_migration_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    old_role VARCHAR(50),
    new_admin_modules TEXT[],
    new_package_tier VARCHAR(20),
    migrated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    migration_status VARCHAR(20) DEFAULT 'success',
    notes TEXT
);

-- Insert migration audit records
INSERT INTO role_migration_audit (user_id, new_admin_modules, new_package_tier, notes)
SELECT 
    id,
    admin_modules,
    package_tier,
    'Migrated from legacy role system to permission-based system'
FROM users;

-- Step 9: Update any remaining role-based queries (cleanup)
COMMENT ON TABLE user_admin_modules IS 'Stores admin module assignments for users - replaces legacy role system';
COMMENT ON TABLE user_package_tiers IS 'Stores package tier assignments for users - replaces legacy subscription tiers';
COMMENT ON COLUMN users.admin_modules IS 'Array of admin module codes assigned to user';
COMMENT ON COLUMN users.package_tier IS 'Package tier determining feature access level';

-- Migration completed
SELECT 'Role-to-Permission migration completed successfully' as result;