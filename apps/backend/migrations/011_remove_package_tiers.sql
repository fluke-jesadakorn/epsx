-- Migration: Remove package tier system and unify under permissions
-- This migration eliminates all tier-based logic and converts existing data to permission arrays

-- ============================================================================
-- STEP 1: Create permission templates for easy assignment
-- ============================================================================

-- Permission Templates table - replaces tier-based plans
CREATE TABLE permission_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE, -- e.g., "Bronze Template", "Enterprise Template"
    description TEXT,
    permissions JSONB NOT NULL, -- Array of permission strings
    display_tier VARCHAR(50), -- For UI compatibility: "BRONZE", "SILVER", etc.
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Add updated_at trigger for permission_templates
CREATE TRIGGER update_permission_templates_updated_at 
    BEFORE UPDATE ON permission_templates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Index for performance
CREATE INDEX idx_permission_templates_name ON permission_templates(name);
CREATE INDEX idx_permission_templates_display_tier ON permission_templates(display_tier);
CREATE INDEX idx_permission_templates_active ON permission_templates(is_active);

-- ============================================================================
-- STEP 2: Insert standard permission templates (replaces tier packages)
-- ============================================================================

INSERT INTO permission_templates (name, description, permissions, display_tier) VALUES
('Free Template', 'Free tier with basic access', 
 '["epsx:rankings:view:3", "epsx:trading:basic", "epsx:portfolio:view", "epsx:notifications:basic"]', 'FREE'),

('Bronze Template', 'Bronze tier with enhanced access', 
 '["epsx:rankings:view:5", "epsx:trading:basic", "epsx:portfolio:view", "epsx:portfolio:history", "epsx:notifications:enhanced"]', 'BRONZE'),

('Silver Template', 'Silver tier with professional access', 
 '["epsx:rankings:view:25", "epsx:trading:basic", "epsx:trading:advanced", "epsx:portfolio:view", "epsx:portfolio:history", "epsx:notifications:enhanced", "epsx:analytics:basic", "epsx:alerts:email"]', 'SILVER'),

('Gold Template', 'Gold tier with VIP access', 
 '["epsx:rankings:view:50", "epsx:trading:basic", "epsx:trading:advanced", "epsx:trading:premium", "epsx:portfolio:view", "epsx:portfolio:history", "epsx:portfolio:tools", "epsx:notifications:enhanced", "epsx:analytics:basic", "epsx:analytics:advanced", "epsx:analytics:premium", "epsx:alerts:email", "epsx:support:priority"]', 'GOLD'),

('Platinum Template', 'Platinum tier with elite access', 
 '["epsx:rankings:view:100", "epsx:trading:basic", "epsx:trading:advanced", "epsx:trading:premium", "epsx:portfolio:view", "epsx:portfolio:history", "epsx:portfolio:tools", "epsx:notifications:enhanced", "epsx:analytics:basic", "epsx:analytics:advanced", "epsx:analytics:premium", "epsx:alerts:email", "epsx:support:priority", "epsx:research:reports", "epsx:dashboards:custom"]', 'PLATINUM'),

('Enterprise Template', 'Enterprise tier with unlimited access', 
 '["epsx:rankings:view:unlimited", "epsx:*:*", "epsx-pay:*:*", "epsx-token:*:*"]', 'ENTERPRISE');

-- ============================================================================
-- STEP 3: Migrate existing user tiers to permissions
-- ============================================================================

-- Function to convert tier to permissions
CREATE OR REPLACE FUNCTION migrate_user_tier_to_permissions()
RETURNS void AS $$
DECLARE
    user_record RECORD;
    permission_array JSONB;
    template_permissions JSONB;
BEGIN
    -- Iterate through all users with package_tier
    FOR user_record IN 
        SELECT id, package_tier, created_at 
        FROM users 
        WHERE package_tier IS NOT NULL
    LOOP
        -- Get permission template based on tier
        SELECT permissions INTO template_permissions
        FROM permission_templates
        WHERE UPPER(display_tier) = UPPER(COALESCE(user_record.package_tier, 'FREE'));
        
        -- If no template found, use Free Template as fallback
        IF template_permissions IS NULL THEN
            SELECT permissions INTO template_permissions
            FROM permission_templates
            WHERE display_tier = 'FREE';
        END IF;
        
        -- Convert JSONB array to individual permission inserts
        FOR permission_array IN SELECT jsonb_array_elements_text(template_permissions)
        LOOP
            -- Insert permission if it doesn't already exist
            INSERT INTO user_permissions (user_id, permission, granted_at, is_active)
            VALUES (
                user_record.id, 
                permission_array::text, 
                user_record.created_at, 
                true
            )
            ON CONFLICT DO NOTHING; -- Avoid duplicates
        END LOOP;
        
        RAISE NOTICE 'Migrated user % from tier % to permissions', user_record.id, user_record.package_tier;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Execute the migration function
SELECT migrate_user_tier_to_permissions();

-- Drop the migration function as it's no longer needed
DROP FUNCTION migrate_user_tier_to_permissions();

-- ============================================================================
-- STEP 4: Update pricing_plans to use permission templates
-- ============================================================================

-- Add permission_template_id to pricing_plans (replaces tier logic)
ALTER TABLE pricing_plans ADD COLUMN permission_template_id UUID REFERENCES permission_templates(id);

-- Migrate existing plans to use permission templates based on plan names
UPDATE pricing_plans 
SET permission_template_id = (SELECT id FROM permission_templates WHERE display_tier = 'BRONZE')
WHERE LOWER(name) LIKE '%bronze%' OR LOWER(plan_type) LIKE '%bronze%';

UPDATE pricing_plans 
SET permission_template_id = (SELECT id FROM permission_templates WHERE display_tier = 'SILVER')
WHERE LOWER(name) LIKE '%silver%' OR LOWER(plan_type) LIKE '%silver%';

UPDATE pricing_plans 
SET permission_template_id = (SELECT id FROM permission_templates WHERE display_tier = 'GOLD')
WHERE LOWER(name) LIKE '%gold%' OR LOWER(plan_type) LIKE '%gold%';

UPDATE pricing_plans 
SET permission_template_id = (SELECT id FROM permission_templates WHERE display_tier = 'PLATINUM')
WHERE LOWER(name) LIKE '%platinum%' OR LOWER(plan_type) LIKE '%platinum%';

UPDATE pricing_plans 
SET permission_template_id = (SELECT id FROM permission_templates WHERE display_tier = 'ENTERPRISE')
WHERE LOWER(name) LIKE '%enterprise%' OR LOWER(plan_type) LIKE '%enterprise%' OR LOWER(name) LIKE '%vip%';

-- Set Free Template for any remaining plans without template assignment
UPDATE pricing_plans 
SET permission_template_id = (SELECT id FROM permission_templates WHERE display_tier = 'FREE')
WHERE permission_template_id IS NULL;

-- ============================================================================
-- STEP 5: Remove package_tier column from users table
-- ============================================================================

-- Verify all users have permissions before dropping column
DO $$
DECLARE
    users_without_permissions INTEGER;
BEGIN
    SELECT COUNT(*) INTO users_without_permissions
    FROM users u
    LEFT JOIN user_permissions up ON u.id = up.user_id AND up.is_active = true
    WHERE up.user_id IS NULL;
    
    IF users_without_permissions > 0 THEN
        RAISE EXCEPTION 'Cannot proceed: % users found without permissions. Migration incomplete.', users_without_permissions;
    END IF;
    
    RAISE NOTICE 'Verification passed: All users have permissions assigned';
END $$;

-- Now safe to drop the package_tier column
ALTER TABLE users DROP COLUMN package_tier;

RAISE NOTICE 'Successfully dropped package_tier column from users table';

-- ============================================================================
-- STEP 6: Add indexes for performance with new permission template system
-- ============================================================================

CREATE INDEX idx_pricing_plans_permission_template_id ON pricing_plans(permission_template_id);

-- ============================================================================
-- STEP 7: Create helper functions for permission-based operations
-- ============================================================================

-- Function to assign permission template to user
CREATE OR REPLACE FUNCTION assign_permission_template_to_user(
    p_user_id UUID,
    p_template_name VARCHAR,
    p_expires_at TIMESTAMPTZ DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
    template_permissions JSONB;
    permission_text TEXT;
BEGIN
    -- Get template permissions
    SELECT permissions INTO template_permissions
    FROM permission_templates
    WHERE name = p_template_name AND is_active = true;
    
    IF template_permissions IS NULL THEN
        RAISE EXCEPTION 'Permission template "%" not found', p_template_name;
    END IF;
    
    -- Revoke existing permissions
    UPDATE user_permissions 
    SET is_active = false 
    WHERE user_id = p_user_id AND is_active = true;
    
    -- Insert new permissions from template
    FOR permission_text IN SELECT jsonb_array_elements_text(template_permissions)
    LOOP
        INSERT INTO user_permissions (user_id, permission, expires_at, is_active)
        VALUES (p_user_id, permission_text, p_expires_at, true);
    END LOOP;
    
    RETURN true;
END;
$$ LANGUAGE plpgsql;

-- Function to get user's effective tier name for UI compatibility
CREATE OR REPLACE FUNCTION get_user_display_tier(p_user_id UUID)
RETURNS VARCHAR AS $$
DECLARE
    ranking_limit TEXT;
    tier_name VARCHAR;
BEGIN
    -- Get ranking limit from user's permissions
    SELECT regexp_replace(permission, '^epsx:rankings:view:', '') INTO ranking_limit
    FROM user_permissions
    WHERE user_id = p_user_id 
      AND permission LIKE 'epsx:rankings:view:%'
      AND is_active = true
      AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
    ORDER BY 
        CASE 
            WHEN permission LIKE '%unlimited%' THEN 999999
            ELSE CAST(regexp_replace(permission, '^epsx:rankings:view:', '') AS INTEGER)
        END DESC
    LIMIT 1;
    
    -- Convert ranking limit to tier name
    tier_name := CASE
        WHEN ranking_limit = 'unlimited' THEN 'ENTERPRISE'
        WHEN ranking_limit::INTEGER >= 100 THEN 'PLATINUM'
        WHEN ranking_limit::INTEGER >= 50 THEN 'GOLD'
        WHEN ranking_limit::INTEGER >= 25 THEN 'SILVER'
        WHEN ranking_limit::INTEGER >= 5 THEN 'BRONZE'
        ELSE 'FREE'
    END;
    
    RETURN tier_name;
EXCEPTION
    WHEN OTHERS THEN
        RETURN 'FREE'; -- Fallback to FREE tier
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- MIGRATION SUMMARY
-- ============================================================================

-- Log successful migration
INSERT INTO user_permissions (user_id, permission, granted_at, is_active)
SELECT 
    id as user_id,
    'migration:tier_to_permissions:completed' as permission,
    CURRENT_TIMESTAMP as granted_at,
    true as is_active
FROM users
WHERE id = (SELECT id FROM users ORDER BY created_at ASC LIMIT 1); -- Add to oldest user as migration marker

RAISE NOTICE '============================================================================';
RAISE NOTICE 'MIGRATION COMPLETED SUCCESSFULLY';
RAISE NOTICE '- Removed package_tier column from users table';
RAISE NOTICE '- Created permission_templates table with standard templates';
RAISE NOTICE '- Migrated all existing user tiers to permission arrays';
RAISE NOTICE '- Updated pricing_plans to use permission templates';
RAISE NOTICE '- Added helper functions for permission template management';
RAISE NOTICE '- All users now use unified permission-based access control';
RAISE NOTICE '============================================================================';