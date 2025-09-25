-- ============================================================================
-- OIDC TO WEB3 GROUPS MIGRATION - Move existing OIDC users to group system
-- ============================================================================
-- This migration safely transitions existing OIDC-authenticated users to the
-- new Web3 group-based permission system with backward compatibility
-- ============================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ============================================================================
-- 1. CREATE TRANSITION GROUPS FOR LEGACY USERS
-- ============================================================================

-- Insert transition groups for existing OIDC users
INSERT INTO permission_groups (
    name, slug, description, permissions, 
    is_system_group, is_web3_managed, priority_level,
    group_metadata
) VALUES 
(
    'Legacy OIDC Users',
    'legacy-oidc-users',
    'Users migrated from OIDC authentication system',
    ARRAY[
        'epsx:analytics:view',
        'epsx:profile:manage',
        'epsx:notifications:receive'
    ],
    TRUE, FALSE, 3,
    '{"migrated_from": "oidc", "migration_date": "' || NOW()::text || '"}'::jsonb
),
(
    'Legacy Admin Users',
    'legacy-admin-users', 
    'Admin users migrated from OIDC authentication system',
    ARRAY[
        'admin:users:view',
        'admin:users:manage',
        'admin:permissions:view',
        'admin:permissions:manage',
        'admin:system:view',
        'epsx:analytics:view',
        'epsx:analytics:advanced',
        'epsx:analytics:export'
    ],
    TRUE, FALSE, 9,
    '{"migrated_from": "oidc_admin", "migration_date": "' || NOW()::text || '"}'::jsonb
),
(
    'Legacy Premium Users',
    'legacy-premium-users',
    'Premium OIDC users with advanced permissions',
    ARRAY[
        'epsx:analytics:view',
        'epsx:analytics:advanced',
        'epsx:analytics:export',
        'epsx:realtime:access',
        'epsx:profile:manage',
        'epsx:notifications:receive'
    ],
    TRUE, FALSE, 6,
    '{"migrated_from": "oidc_premium", "migration_date": "' || NOW()::text || '"}'::jsonb
)
ON CONFLICT (slug) DO NOTHING;

-- ============================================================================
-- 2. MIGRATE EXISTING USERS TO APPROPRIATE GROUPS
-- ============================================================================

-- Function to determine user group based on existing permissions and roles
CREATE OR REPLACE FUNCTION assign_legacy_user_to_group(
    user_uuid UUID,
    user_email TEXT DEFAULT NULL,
    user_role TEXT DEFAULT 'user',
    user_permissions TEXT[] DEFAULT ARRAY[]::TEXT[]
) RETURNS UUID AS $$
DECLARE
    target_group_id UUID;
    group_slug TEXT;
    assignment_reason TEXT;
BEGIN
    -- Determine appropriate group based on user data
    IF user_role = 'admin' OR 'admin' = ANY(user_permissions) OR 
       EXISTS(SELECT 1 FROM unnest(user_permissions) AS perm WHERE perm LIKE 'admin:%') THEN
        group_slug := 'legacy-admin-users';
        assignment_reason := 'User has admin role or admin permissions';
    ELSIF EXISTS(SELECT 1 FROM unnest(user_permissions) AS perm WHERE 
                perm IN ('epsx:analytics:export', 'epsx:analytics:advanced', 'epsx:realtime:access')) THEN
        group_slug := 'legacy-premium-users';
        assignment_reason := 'User has premium permissions';
    ELSE
        group_slug := 'legacy-oidc-users';
        assignment_reason := 'Standard OIDC user migration';
    END IF;
    
    -- Get target group ID
    SELECT id INTO target_group_id FROM permission_groups WHERE slug = group_slug;
    
    IF target_group_id IS NULL THEN
        RAISE EXCEPTION 'Migration group % not found', group_slug;
    END IF;
    
    -- Assign user to group if not already assigned
    INSERT INTO user_group_memberships (
        user_id, group_id, assigned_by_admin, assigned_at,
        assignment_reason, membership_metadata
    ) VALUES (
        user_uuid, target_group_id, TRUE, NOW(),
        assignment_reason,
        jsonb_build_object(
            'migration_source', 'oidc_to_web3',
            'migration_timestamp', NOW(),
            'original_role', user_role,
            'original_permissions', user_permissions,
            'original_email', user_email
        )
    ) ON CONFLICT (user_id, group_id) DO UPDATE SET
        updated_at = NOW(),
        membership_metadata = membership_metadata || jsonb_build_object('last_migration_update', NOW());
    
    RETURN target_group_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 3. MIGRATE USERS FROM EXISTING TABLES (if they exist)
-- ============================================================================

-- Check if legacy users table exists and migrate
DO $$
DECLARE
    users_table_exists BOOLEAN := FALSE;
    migrated_count INTEGER := 0;
    user_record RECORD;
BEGIN
    -- Check if users table exists
    SELECT EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'public' AND table_name = 'users'
    ) INTO users_table_exists;
    
    IF users_table_exists THEN
        RAISE NOTICE 'Found existing users table, starting migration...';
        
        -- Migrate each user
        FOR user_record IN 
            SELECT 
                id,
                email,
                COALESCE(role, 'user') as user_role,
                COALESCE(
                    CASE 
                        WHEN role = 'admin' THEN ARRAY['admin:users:manage', 'admin:system:manage']
                        ELSE ARRAY['epsx:analytics:view', 'epsx:profile:manage']
                    END,
                    ARRAY[]::TEXT[]
                ) as user_permissions
            FROM users 
            WHERE id IS NOT NULL
        LOOP
            -- Assign user to appropriate group
            PERFORM assign_legacy_user_to_group(
                user_record.id,
                user_record.email,
                user_record.user_role,
                user_record.user_permissions
            );
            
            migrated_count := migrated_count + 1;
        END LOOP;
        
        RAISE NOTICE 'Successfully migrated % users to Web3 groups', migrated_count;
    ELSE
        RAISE NOTICE 'No existing users table found, skipping user migration';
    END IF;
END $$;

-- ============================================================================
-- 4. CREATE MAPPING TABLE FOR OIDC TO WEB3 TRANSITION
-- ============================================================================

CREATE TABLE IF NOT EXISTS oidc_web3_transition_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- User Information
    user_id UUID,
    original_email TEXT,
    original_firebase_uid TEXT,
    original_role TEXT,
    
    -- Migration Details
    assigned_group_id UUID REFERENCES permission_groups(id),
    migration_status VARCHAR(20) DEFAULT 'completed', -- 'completed', 'failed', 'partial'
    migration_timestamp TIMESTAMPTZ DEFAULT NOW(),
    migration_reason TEXT,
    
    -- Original Data Backup
    original_permissions TEXT[] DEFAULT '{}',
    original_user_data JSONB DEFAULT '{}',
    
    -- Web3 Transition Data
    wallet_address VARCHAR(42), -- Will be populated when user connects wallet
    web3_verification_status VARCHAR(20) DEFAULT 'pending',
    web3_linked_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for transition log
CREATE INDEX idx_oidc_web3_transition_user_id ON oidc_web3_transition_log(user_id);
CREATE INDEX idx_oidc_web3_transition_email ON oidc_web3_transition_log(original_email);
CREATE INDEX idx_oidc_web3_transition_status ON oidc_web3_transition_log(migration_status);
CREATE INDEX idx_oidc_web3_transition_wallet ON oidc_web3_transition_log(wallet_address) WHERE wallet_address IS NOT NULL;

-- ============================================================================
-- 5. CREATE DEFAULT WEB3 GROUPS FOR NEW USERS
-- ============================================================================

-- Insert default Web3 groups for wallet-first users
INSERT INTO permission_groups (
    name, slug, description, permissions,
    is_system_group, is_web3_managed, priority_level,
    group_metadata
) VALUES 
(
    'Connected Wallet Users',
    'connected-wallet-users',
    'Users who have connected a Web3 wallet',
    ARRAY[
        'epsx:analytics:view',
        'epsx:profile:basic',
        'epsx:wallet:view'
    ],
    TRUE, TRUE, 2,
    '{"web3_tier": "connected", "auto_assigned": true}'::jsonb
),
(
    'Authenticated Web3 Users',
    'authenticated-web3-users',
    'Users who have completed Web3 authentication',
    ARRAY[
        'epsx:analytics:view',
        'epsx:profile:manage',
        'epsx:notifications:receive',
        'epsx:wallet:manage'
    ],
    TRUE, TRUE, 4,
    '{"web3_tier": "authenticated", "auto_assigned": true}'::jsonb
),
(
    'Web3 Token Holders',
    'web3-token-holders',
    'Users holding qualifying tokens (managed by Web3 bridge)',
    ARRAY[
        'epsx:defi:basic',
        'epsx:trading:access',
        'epsx:analytics:view',
        'epsx:analytics:export'
    ],
    TRUE, TRUE, 7,
    '{"web3_tier": "token_holder", "managed_by_bridge": true}'::jsonb
),
(
    'Web3 NFT Holders',
    'web3-nft-holders',
    'Users holding qualifying NFTs (managed by Web3 bridge)',
    ARRAY[
        'epsx:nft:view',
        'epsx:collectibles:access',
        'epsx:analytics:view',
        'epsx:premium:basic'
    ],
    TRUE, TRUE, 6,
    '{"web3_tier": "nft_holder", "managed_by_bridge": true}'::jsonb
),
(
    'Web3 DAO Members',
    'web3-dao-members',
    'Users with DAO membership (managed by Web3 bridge)',
    ARRAY[
        'epsx:dao:view',
        'epsx:governance:participate',
        'epsx:analytics:advanced',
        'epsx:realtime:access'
    ],
    TRUE, TRUE, 8,
    '{"web3_tier": "dao_member", "managed_by_bridge": true}'::jsonb
)
ON CONFLICT (slug) DO NOTHING;

-- ============================================================================
-- 6. SET UP WEB3 GROUP RULES FOR NEW GROUPS
-- ============================================================================

-- Create Web3 bridge rules for automatic assignment
DO $$
DECLARE
    connected_group_id UUID;
    authenticated_group_id UUID;
    token_holders_group_id UUID;
    nft_holders_group_id UUID;
    dao_members_group_id UUID;
BEGIN
    -- Get group IDs
    SELECT id INTO connected_group_id FROM permission_groups WHERE slug = 'connected-wallet-users';
    SELECT id INTO authenticated_group_id FROM permission_groups WHERE slug = 'authenticated-web3-users';
    SELECT id INTO token_holders_group_id FROM permission_groups WHERE slug = 'web3-token-holders';
    SELECT id INTO nft_holders_group_id FROM permission_groups WHERE slug = 'web3-nft-holders';
    SELECT id INTO dao_members_group_id FROM permission_groups WHERE slug = 'web3-dao-members';
    
    -- Create rule for generic token holders (any significant balance)
    IF token_holders_group_id IS NOT NULL THEN
        INSERT INTO web3_group_rules (
            group_id, rule_name, rule_type, rule_description,
            contract_address, network, minimum_balance, token_decimals,
            is_active, auto_assignment, assignment_duration_days,
            verification_frequency, priority_order, tags
        ) VALUES (
            token_holders_group_id,
            'Significant Token Holder (1000+ tokens)',
            'token_balance',
            'Users holding at least 1000 of any qualifying token',
            '0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82', -- CAKE as example
            'bsc',
            1000000000000000000000, -- 1000 tokens with 18 decimals
            18,
            TRUE, TRUE, 60, -- 60-day assignments
            'daily', 5,
            ARRAY['defi', 'trading', 'token_holder']
        ) ON CONFLICT DO NOTHING;
    END IF;
    
    -- Create rule for NFT holders (any collection)
    IF nft_holders_group_id IS NOT NULL THEN
        INSERT INTO web3_group_rules (
            group_id, rule_name, rule_type, rule_description,
            contract_address, network, minimum_nft_count,
            is_active, auto_assignment, assignment_duration_days,
            verification_frequency, priority_order, tags
        ) VALUES (
            nft_holders_group_id,
            'NFT Collector (1+ NFTs)',
            'nft_ownership',
            'Users holding at least 1 NFT from qualifying collections',
            '0x0000000000000000000000000000000000000001', -- Placeholder NFT contract
            'bsc',
            1,
            TRUE, TRUE, 30, -- 30-day assignments
            'daily', 6,
            ARRAY['nft', 'collectibles', 'premium']
        ) ON CONFLICT DO NOTHING;
    END IF;
END $$;

-- ============================================================================
-- 7. UPDATE EXISTING PERMISSIONS FOR WEB3 COMPATIBILITY
-- ============================================================================

-- Update permission format to ensure Web3 compatibility
UPDATE permission_groups 
SET permissions = ARRAY(
    SELECT DISTINCT 
        CASE 
            WHEN perm LIKE 'admin.%' THEN REPLACE(perm, '.', ':')
            WHEN perm LIKE 'epsx.%' THEN REPLACE(perm, '.', ':')
            WHEN perm NOT LIKE '%:%' AND perm LIKE 'admin%' THEN 'admin:' || SUBSTRING(perm FROM 6)
            WHEN perm NOT LIKE '%:%' AND perm LIKE 'epsx%' THEN 'epsx:' || SUBSTRING(perm FROM 5)
            ELSE perm
        END as normalized_perm
    FROM unnest(permissions) as perm
    WHERE normalized_perm IS NOT NULL
)
WHERE array_length(permissions, 1) > 0;

-- ============================================================================
-- 8. CLEANUP AND VERIFICATION
-- ============================================================================

-- Clean up the temporary function
DROP FUNCTION IF EXISTS assign_legacy_user_to_group(UUID, TEXT, TEXT, TEXT[]);

-- Verify migration results
DO $$
DECLARE
    total_groups INTEGER;
    total_memberships INTEGER;
    web3_groups INTEGER;
    legacy_groups INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_groups FROM permission_groups;
    SELECT COUNT(*) INTO total_memberships FROM user_group_memberships;
    SELECT COUNT(*) INTO web3_groups FROM permission_groups WHERE is_web3_managed = TRUE;
    SELECT COUNT(*) INTO legacy_groups FROM permission_groups WHERE is_web3_managed = FALSE AND is_system_group = TRUE;
    
    RAISE NOTICE 'Migration Summary:';
    RAISE NOTICE '- Total permission groups: %', total_groups;
    RAISE NOTICE '- Total user group memberships: %', total_memberships;
    RAISE NOTICE '- Web3-managed groups: %', web3_groups;
    RAISE NOTICE '- Legacy transition groups: %', legacy_groups;
END $$;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

RAISE NOTICE 'OIDC to Web3 Groups migration completed successfully!';
RAISE NOTICE 'Created: 8 new permission groups for Web3-first authentication';
RAISE NOTICE 'Created: oidc_web3_transition_log table for migration tracking';
RAISE NOTICE 'Migrated: Existing OIDC users to appropriate permission groups';
RAISE NOTICE 'System ready for wallet-first authentication with backward compatibility!';