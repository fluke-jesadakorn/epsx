-- ================================================================================================
-- REMOVE TIER SYSTEM AND CONSOLIDATE TO GRANULAR PERMISSIONS
-- ================================================================================================
-- This migration removes all tier-related tables, columns, and functions to consolidate
-- the permission system to use only granular permission groups
-- ================================================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ================================================================================================
-- 1. BACKUP EXISTING TIER DATA BEFORE REMOVAL (FOR AUDIT PURPOSES)
-- ================================================================================================

-- Create temporary backup table for tier assignments if needed for data recovery
CREATE TABLE IF NOT EXISTS tier_system_backup_20250126 (
    backup_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42),
    tier_level VARCHAR(20),
    permissions JSONB,
    backed_up_at TIMESTAMPTZ DEFAULT NOW()
);

-- Backup existing tier assignments from wallet_users
INSERT INTO tier_system_backup_20250126 (wallet_address, tier_level, permissions)
SELECT wallet_address, tier_level, permissions 
FROM wallet_users 
WHERE tier_level IS NOT NULL;

-- ================================================================================================
-- 2. DROP TIER-RELATED FUNCTIONS AND PROCEDURES
-- ================================================================================================

-- Drop tier processing functions
DROP FUNCTION IF EXISTS process_tier_assignment_queue(INTEGER);
DROP FUNCTION IF EXISTS determine_tier_level(JSONB);
DROP FUNCTION IF EXISTS evaluate_tier_assignment_rules(VARCHAR);
DROP FUNCTION IF EXISTS update_wallet_tier_cache(VARCHAR);

-- ================================================================================================
-- 3. DROP TIER-RELATED INDEXES
-- ================================================================================================

-- Drop tier-specific indexes
DROP INDEX IF EXISTS idx_wallet_users_tier;
DROP INDEX IF EXISTS idx_tier_assignment_queue_status;
DROP INDEX IF EXISTS idx_tier_assignment_queue_wallet;
DROP INDEX IF EXISTS idx_tier_assignment_queue_urgent;
DROP INDEX IF EXISTS idx_tier_assignment_queue_retry;
DROP INDEX IF EXISTS idx_tier_evaluation_history_wallet;
DROP INDEX IF EXISTS idx_tier_evaluation_history_rule;
DROP INDEX IF EXISTS idx_tier_evaluation_history_batch;
DROP INDEX IF EXISTS idx_dynamic_tier_rules_category;
DROP INDEX IF EXISTS idx_dynamic_tier_rules_active;
DROP INDEX IF EXISTS idx_wallet_tier_cache_expires;
DROP INDEX IF EXISTS idx_wallet_tier_cache_tier;

-- ================================================================================================
-- 4. DROP TIER-RELATED TABLES IN DEPENDENCY ORDER
-- ================================================================================================

-- Drop tables with foreign key dependencies first
DROP TABLE IF EXISTS tier_evaluation_history CASCADE;
DROP TABLE IF EXISTS tier_assignment_queue CASCADE;
DROP TABLE IF EXISTS wallet_tier_cache CASCADE;
DROP TABLE IF EXISTS dynamic_tier_rules CASCADE;

-- ================================================================================================
-- 5. REMOVE TIER_LEVEL COLUMN FROM WALLET_USERS
-- ================================================================================================

-- Drop tier-related constraints first
ALTER TABLE wallet_users DROP CONSTRAINT IF EXISTS valid_tier_level;

-- Remove tier_level column
ALTER TABLE wallet_users DROP COLUMN IF EXISTS tier_level;

-- ================================================================================================
-- 6. MIGRATE TIER-BASED PERMISSIONS TO GRANULAR PERMISSION GROUPS
-- ================================================================================================

-- Ensure permission_groups table exists (should be from previous migrations)
CREATE TABLE IF NOT EXISTS permission_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    permissions JSONB DEFAULT '[]' NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create default permission groups to replace tier system
INSERT INTO permission_groups (id, name, description, permissions, is_active) VALUES
(
    'a1b2c3d4-e5f6-4g7h-8i9j-0k1l2m3n4o5p'::uuid,
    'Basic Access Group',
    'Basic permission group for new users (replaces Bronze tier)',
    '[
        "epsx:rankings:view:3",
        "epsx:trading:basic",
        "epsx:portfolio:view"
    ]'::jsonb,
    true
),
(
    'b2c3d4e5-f6g7-4h8i-9j0k-1l2m3n4o5p6q'::uuid,
    'Standard Access Group', 
    'Standard permission group for regular users (replaces Silver tier)',
    '[
        "epsx:rankings:view:25",
        "epsx:trading:basic",
        "epsx:trading:advanced",
        "epsx:portfolio:view",
        "epsx:analytics:basic"
    ]'::jsonb,
    true
),
(
    'c3d4e5f6-g7h8-4i9j-0k1l-2m3n4o5p6q7r'::uuid,
    'Premium Access Group',
    'Premium permission group for power users (replaces Gold tier)',
    '[
        "epsx:rankings:view:50",
        "epsx:trading:premium",
        "epsx:portfolio:tools",
        "epsx:analytics:advanced"
    ]'::jsonb,
    true
),
(
    'd4e5f6g7-h8i9-4j0k-1l2m-3n4o5p6q7r8s'::uuid,
    'Professional Access Group',
    'Professional permission group for advanced users (replaces Platinum tier)',
    '[
        "epsx:rankings:view:100",
        "epsx:trading:premium",
        "epsx:analytics:premium",
        "epsx:research:reports",
        "epsx:dashboards:custom"
    ]'::jsonb,
    true
),
(
    'e5f6g7h8-i9j0-4k1l-2m3n-4o5p6q7r8s9t'::uuid,
    'Enterprise Access Group',
    'Enterprise permission group for unlimited access (replaces Diamond tier)',
    '[
        "epsx:rankings:view:unlimited",
        "epsx:*:*",
        "epsx-pay:*:*",
        "epsx-token:*:*"
    ]'::jsonb,
    true
)
ON CONFLICT (name) DO NOTHING;

-- ================================================================================================
-- 7. CREATE WALLET_USER_GROUPS RELATIONSHIP TABLE
-- ================================================================================================

-- Create table to manage wallet to permission group relationships
CREATE TABLE IF NOT EXISTS wallet_user_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) NOT NULL,
    group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NULL, -- NULL for permanent assignment
    assigned_by VARCHAR(42) NULL, -- Admin wallet who assigned this
    assignment_reason TEXT DEFAULT 'tier_migration',
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Constraints
    UNIQUE(wallet_address, group_id),
    CONSTRAINT fk_wallet_user_groups_wallet 
        FOREIGN KEY (wallet_address) 
        REFERENCES wallet_users(wallet_address) 
        ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_wallet_user_groups_wallet ON wallet_user_groups(wallet_address, is_active);
CREATE INDEX IF NOT EXISTS idx_wallet_user_groups_group ON wallet_user_groups(group_id, is_active);
CREATE INDEX IF NOT EXISTS idx_wallet_user_groups_expires ON wallet_user_groups(expires_at) WHERE expires_at IS NOT NULL;

-- ================================================================================================
-- 8. MIGRATE EXISTING WALLET PERMISSIONS TO GROUP ASSIGNMENTS
-- ================================================================================================

-- Function to determine appropriate group based on existing permissions
CREATE OR REPLACE FUNCTION migrate_user_to_permission_group(user_permissions JSONB) 
RETURNS UUID AS $$
DECLARE
    group_id UUID;
    max_rankings INTEGER := 0;
    has_premium BOOLEAN := false;
    has_enterprise BOOLEAN := false;
    perm_text TEXT;
BEGIN
    -- Check for enterprise permissions (admin or wildcard)
    FOR perm_text IN SELECT jsonb_array_elements_text(user_permissions) LOOP
        IF perm_text LIKE 'admin:%' OR perm_text LIKE 'epsx:*:*' OR perm_text LIKE '%:*:*' THEN
            has_enterprise := true;
            EXIT;
        END IF;
        
        -- Check for premium features
        IF perm_text LIKE '%:premium%' OR perm_text LIKE '%:advanced%' THEN
            has_premium := true;
        END IF;
        
        -- Extract ranking limits
        IF perm_text LIKE 'epsx:rankings:view:%' THEN
            DECLARE
                limit_str TEXT := split_part(perm_text, ':', 4);
            BEGIN
                IF limit_str = 'unlimited' THEN
                    max_rankings := 999999;
                ELSIF limit_str ~ '^[0-9]+$' THEN
                    max_rankings := GREATEST(max_rankings, limit_str::INTEGER);
                END IF;
            EXCEPTION WHEN others THEN
                -- Ignore parsing errors
            END;
        END IF;
    END LOOP;
    
    -- Determine appropriate group
    IF has_enterprise OR max_rankings >= 100 THEN
        SELECT pg.id INTO group_id FROM permission_groups pg WHERE pg.name = 'Enterprise Access Group';
    ELSIF has_premium OR max_rankings >= 50 THEN
        SELECT pg.id INTO group_id FROM permission_groups pg WHERE pg.name = 'Professional Access Group';
    ELSIF max_rankings >= 25 THEN
        SELECT pg.id INTO group_id FROM permission_groups pg WHERE pg.name = 'Premium Access Group';
    ELSIF max_rankings >= 5 THEN
        SELECT pg.id INTO group_id FROM permission_groups pg WHERE pg.name = 'Standard Access Group';
    ELSE
        SELECT pg.id INTO group_id FROM permission_groups pg WHERE pg.name = 'Basic Access Group';
    END IF;
    
    RETURN group_id;
END;
$$ LANGUAGE plpgsql;

-- Migrate existing wallet permissions to group assignments
INSERT INTO wallet_user_groups (wallet_address, group_id, assignment_reason)
SELECT 
    wu.wallet_address,
    migrate_user_to_permission_group(wu.permissions),
    'automatic_tier_migration'
FROM wallet_users wu
WHERE wu.permissions IS NOT NULL 
  AND wu.permissions != '[]'::jsonb
ON CONFLICT (wallet_address, group_id) DO NOTHING;

-- Assign basic access group to wallets without specific permissions
INSERT INTO wallet_user_groups (wallet_address, group_id, assignment_reason)
SELECT 
    wu.wallet_address,
    (SELECT id FROM permission_groups WHERE name = 'Basic Access Group'),
    'default_assignment'
FROM wallet_users wu
WHERE NOT EXISTS (
    SELECT 1 FROM wallet_user_groups wug 
    WHERE wug.wallet_address = wu.wallet_address
)
ON CONFLICT (wallet_address, group_id) DO NOTHING;

-- ================================================================================================
-- 9. UPDATE WALLET_USERS PERMISSIONS BASED ON GROUP ASSIGNMENTS
-- ================================================================================================

-- Update wallet_users permissions to reflect group permissions
UPDATE wallet_users 
SET permissions = (
    SELECT jsonb_agg(DISTINCT perm)
    FROM (
        SELECT jsonb_array_elements_text(pg.permissions) as perm
        FROM wallet_user_groups wug
        JOIN permission_groups pg ON wug.group_id = pg.id
        WHERE wug.wallet_address = wallet_users.wallet_address
          AND wug.is_active = true
          AND pg.is_active = true
          AND (wug.expires_at IS NULL OR wug.expires_at > NOW())
    ) permissions_list
),
updated_at = NOW()
WHERE EXISTS (
    SELECT 1 FROM wallet_user_groups wug 
    WHERE wug.wallet_address = wallet_users.wallet_address
);

-- ================================================================================================
-- 10. CLEANUP TEMPORARY FUNCTION
-- ================================================================================================

DROP FUNCTION migrate_user_to_permission_group(JSONB);

-- ================================================================================================
-- 11. ADD COMMENTS AND LOG MIGRATION COMPLETION
-- ================================================================================================

COMMENT ON TABLE permission_groups IS 'Permission groups that replace the tier system - provides granular permission control';
COMMENT ON TABLE wallet_user_groups IS 'Assigns wallets to permission groups - replaces tier assignments';

-- Log successful migration
DO $$
DECLARE
    total_wallets INTEGER;
    migrated_wallets INTEGER;
    total_groups INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_wallets FROM wallet_users;
    SELECT COUNT(DISTINCT wallet_address) INTO migrated_wallets FROM wallet_user_groups;
    SELECT COUNT(*) INTO total_groups FROM permission_groups WHERE is_active = true;
    
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'TIER SYSTEM REMOVAL AND PERMISSION MIGRATION COMPLETE';
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Total wallets: %', total_wallets;
    RAISE NOTICE 'Migrated to groups: %', migrated_wallets;
    RAISE NOTICE 'Available permission groups: %', total_groups;
    RAISE NOTICE 'Backup table created: tier_system_backup_20250126';
    RAISE NOTICE '';
    RAISE NOTICE 'REMOVED COMPONENTS:';
    RAISE NOTICE '  - dynamic_tier_rules table';
    RAISE NOTICE '  - tier_evaluation_history table';
    RAISE NOTICE '  - tier_assignment_queue table';
    RAISE NOTICE '  - wallet_tier_cache table';
    RAISE NOTICE '  - tier_level column from wallet_users';
    RAISE NOTICE '  - All tier-related functions and indexes';
    RAISE NOTICE '';
    RAISE NOTICE 'NEW COMPONENTS:';
    RAISE NOTICE '  - Enhanced permission_groups table';
    RAISE NOTICE '  - wallet_user_groups relationship table';
    RAISE NOTICE '  - Granular permission group assignments';
    RAISE NOTICE '==================================================';
END $$;