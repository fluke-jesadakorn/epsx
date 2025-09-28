-- ================================================================================================
-- UNIFIED PERMISSION GROUPS MIGRATION
-- ================================================================================================
-- This migration creates a unified permission_groups table that replaces the separate
-- tier groups and Web3 permission groups with a single, consistent system.
-- ================================================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ================================================================================================
-- 1. UPGRADE EXISTING PERMISSION_GROUPS TABLE TO UNIFIED STRUCTURE
-- ================================================================================================

-- Drop the existing permission_groups table and recreate with unified structure
-- This replaces the fragmented system from migration 004 with our unified approach
DROP TABLE IF EXISTS permission_groups CASCADE;

-- Create the unified permission_groups table
CREATE TABLE permission_groups (
    -- Primary identity
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Group identification
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    
    -- Group type for different assignment methods
    group_type VARCHAR(20) NOT NULL DEFAULT 'manual',
    
    -- Permission definition (JSON array of permission strings)
    permissions JSONB DEFAULT '[]' NOT NULL,
    
    -- Group metadata (JSON object for type-specific configuration)
    group_metadata JSONB DEFAULT '{}' NOT NULL,
    
    -- Pricing information (for subscription-type groups)
    price DECIMAL(10,2) DEFAULT 0.00,
    currency VARCHAR(3) DEFAULT 'USD',
    billing_cycle VARCHAR(20) DEFAULT 'monthly',
    
    -- Group status and configuration
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    is_promoted BOOLEAN DEFAULT FALSE NOT NULL,
    display_order INTEGER DEFAULT 0,
    max_members INTEGER,
    
    -- Auto-assignment configuration
    auto_assign_enabled BOOLEAN DEFAULT FALSE,
    assignment_rules JSONB DEFAULT '{}',
    
    -- Audit fields
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_by VARCHAR(42), -- Wallet address of creator
    last_modified_by VARCHAR(42), -- Wallet address of last modifier
    
    -- Constraints
    CONSTRAINT valid_group_type CHECK (group_type IN ('manual', 'subscription', 'web3_asset', 'dao_membership', 'admin')),
    CONSTRAINT valid_currency CHECK (currency IN ('USD', 'EUR', 'BTC', 'ETH', 'BNB')),
    CONSTRAINT valid_billing_cycle CHECK (billing_cycle IN ('monthly', 'yearly', 'one_time', 'lifetime')),
    CONSTRAINT valid_wallet_addresses CHECK (
        (created_by IS NULL OR (created_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(created_by) = 42)) AND
        (last_modified_by IS NULL OR (last_modified_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(last_modified_by) = 42))
    )
);

-- ================================================================================================
-- 2. CREATE WALLET GROUP MEMBERSHIPS TABLE
-- ================================================================================================

-- Create the wallet group memberships table for wallet-first assignments
CREATE TABLE IF NOT EXISTS wallet_group_memberships (
    -- Primary identity
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Wallet and group relationship
    wallet_address VARCHAR(42) NOT NULL,
    group_id UUID NOT NULL REFERENCES permission_groups(id) ON DELETE CASCADE,
    
    -- Assignment information
    assigned_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    
    -- Assignment source tracking
    assignment_source VARCHAR(50) NOT NULL DEFAULT 'manual',
    assignment_reason TEXT,
    assigned_by VARCHAR(42), -- Wallet address who made the assignment
    
    -- Payment tracking (for subscription groups)
    payment_reference VARCHAR(255),
    subscription_id VARCHAR(255),
    auto_renew BOOLEAN DEFAULT FALSE,
    next_billing_date TIMESTAMPTZ,
    
    -- Assignment metadata (JSON object)
    assignment_metadata JSONB DEFAULT '{}',
    
    -- Audit fields
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    
    -- Constraints
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    ),
    CONSTRAINT valid_assigned_by_format CHECK (
        assigned_by IS NULL OR (assigned_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(assigned_by) = 42)
    ),
    CONSTRAINT valid_assignment_source CHECK (
        assignment_source IN ('manual', 'payment', 'web3_asset', 'dao_governance', 'admin', 'migration', 'auto_assignment')
    ),
    
    -- Unique constraint: one active membership per wallet per group
    CONSTRAINT unique_active_wallet_group UNIQUE (wallet_address, group_id)
);

-- ================================================================================================
-- 3. CREATE OPTIMIZED INDEXES
-- ================================================================================================

-- Permission groups indexes
CREATE INDEX IF NOT EXISTS idx_permission_groups_type ON permission_groups(group_type, is_active);
CREATE INDEX IF NOT EXISTS idx_permission_groups_active ON permission_groups(is_active, display_order);
CREATE INDEX IF NOT EXISTS idx_permission_groups_slug ON permission_groups(slug);
CREATE INDEX IF NOT EXISTS idx_permission_groups_created ON permission_groups(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_permission_groups_price ON permission_groups(price, currency) WHERE group_type = 'subscription';

-- GIN indexes for JSON operations
CREATE INDEX IF NOT EXISTS idx_permission_groups_permissions_gin ON permission_groups USING GIN (permissions);
CREATE INDEX IF NOT EXISTS idx_permission_groups_metadata_gin ON permission_groups USING GIN (group_metadata);
CREATE INDEX IF NOT EXISTS idx_permission_groups_assignment_rules_gin ON permission_groups USING GIN (assignment_rules);

-- Wallet group memberships indexes
CREATE INDEX IF NOT EXISTS idx_wallet_group_memberships_wallet ON wallet_group_memberships(wallet_address, is_active);
CREATE INDEX IF NOT EXISTS idx_wallet_group_memberships_group ON wallet_group_memberships(group_id, is_active);
CREATE INDEX IF NOT EXISTS idx_wallet_group_memberships_expires ON wallet_group_memberships(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_wallet_group_memberships_billing ON wallet_group_memberships(next_billing_date) WHERE next_billing_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_wallet_group_memberships_source ON wallet_group_memberships(assignment_source, assigned_at DESC);

-- GIN index for assignment metadata
CREATE INDEX IF NOT EXISTS idx_wallet_group_memberships_metadata_gin ON wallet_group_memberships USING GIN (assignment_metadata);

-- ================================================================================================
-- 4. CREATE PERMISSION GROUP MANAGEMENT FUNCTIONS
-- ================================================================================================

-- Function to get effective permissions for a wallet
CREATE OR REPLACE FUNCTION get_wallet_effective_permissions(
    target_wallet VARCHAR(42)
) RETURNS JSONB AS $$
DECLARE
    effective_permissions JSONB := '[]';
    group_permissions JSONB;
    membership_record RECORD;
BEGIN
    -- Get all active group memberships for the wallet
    FOR membership_record IN
        SELECT wgm.*, pg.permissions, pg.name as group_name, pg.group_type
        FROM wallet_group_memberships wgm
        JOIN permission_groups pg ON wgm.group_id = pg.id
        WHERE wgm.wallet_address = target_wallet 
        AND wgm.is_active = TRUE
        AND pg.is_active = TRUE
        AND (wgm.expires_at IS NULL OR wgm.expires_at > NOW())
    LOOP
        -- Merge permissions from each active group
        SELECT jsonb_agg(DISTINCT elem) INTO group_permissions
        FROM (
            SELECT jsonb_array_elements_text(effective_permissions) AS elem
            UNION
            SELECT jsonb_array_elements_text(membership_record.permissions) AS elem
        ) AS combined;
        
        effective_permissions := group_permissions;
    END LOOP;
    
    -- Return empty array if no permissions found
    RETURN COALESCE(effective_permissions, '[]'::jsonb);
END;
$$ LANGUAGE plpgsql;

-- Function to check if wallet has specific permission
CREATE OR REPLACE FUNCTION wallet_has_permission(
    target_wallet VARCHAR(42),
    permission_name TEXT
) RETURNS BOOLEAN AS $$
DECLARE
    has_permission BOOLEAN := FALSE;
    effective_permissions JSONB;
    permission_pattern TEXT;
BEGIN
    -- Get effective permissions for the wallet
    effective_permissions := get_wallet_effective_permissions(target_wallet);
    
    -- Check for exact permission match
    IF effective_permissions ? permission_name THEN
        RETURN TRUE;
    END IF;
    
    -- Check for wildcard permission patterns
    FOR permission_pattern IN
        SELECT jsonb_array_elements_text(effective_permissions)
    LOOP
        -- Check for admin wildcard (admin:*:*)
        IF permission_pattern = 'admin:*:*' THEN
            RETURN TRUE;
        END IF;
        
        -- Check for platform wildcard (epsx:*:*)
        IF permission_pattern LIKE 'epsx:*:*' AND permission_name LIKE 'epsx:%' THEN
            RETURN TRUE;
        END IF;
        
        -- Check for resource wildcard (platform:resource:*)
        IF permission_pattern LIKE '%:*' AND 
           permission_name LIKE REPLACE(permission_pattern, ':*', ':%') THEN
            RETURN TRUE;
        END IF;
    END LOOP;
    
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Function to assign wallet to permission group
CREATE OR REPLACE FUNCTION assign_wallet_to_group(
    target_wallet VARCHAR(42),
    target_group_id UUID,
    assigned_by_wallet VARCHAR(42) DEFAULT NULL,
    assignment_source_param VARCHAR(50) DEFAULT 'manual',
    assignment_reason_param TEXT DEFAULT NULL,
    expires_at_param TIMESTAMPTZ DEFAULT NULL,
    payment_ref VARCHAR(255) DEFAULT NULL,
    subscription_id_param VARCHAR(255) DEFAULT NULL
) RETURNS UUID AS $$
DECLARE
    membership_id UUID;
    group_exists BOOLEAN;
BEGIN
    -- Check if group exists and is active
    SELECT EXISTS(
        SELECT 1 FROM permission_groups 
        WHERE id = target_group_id AND is_active = TRUE
    ) INTO group_exists;
    
    IF NOT group_exists THEN
        RAISE EXCEPTION 'Permission group not found or inactive: %', target_group_id;
    END IF;
    
    -- Insert or update membership
    INSERT INTO wallet_group_memberships (
        wallet_address,
        group_id,
        assignment_source,
        assignment_reason,
        assigned_by,
        expires_at,
        payment_reference,
        subscription_id,
        is_active
    ) VALUES (
        target_wallet,
        target_group_id,
        assignment_source_param,
        assignment_reason_param,
        assigned_by_wallet,
        expires_at_param,
        payment_ref,
        subscription_id_param,
        TRUE
    ) 
    ON CONFLICT (wallet_address, group_id) 
    DO UPDATE SET
        is_active = TRUE,
        assignment_source = assignment_source_param,
        assignment_reason = assignment_reason_param,
        assigned_by = assigned_by_wallet,
        expires_at = expires_at_param,
        payment_reference = payment_ref,
        subscription_id = subscription_id_param,
        updated_at = NOW()
    RETURNING id INTO membership_id;
    
    RETURN membership_id;
END;
$$ LANGUAGE plpgsql;

-- ================================================================================================
-- 5. CREATE TRIGGERS
-- ================================================================================================

-- Trigger to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION trigger_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_permission_groups_updated_at
    BEFORE UPDATE ON permission_groups
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_timestamp();

CREATE TRIGGER trigger_wallet_group_memberships_updated_at
    BEFORE UPDATE ON wallet_group_memberships
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_timestamp();

-- ================================================================================================
-- 6. INSERT DEFAULT PERMISSION GROUPS
-- ================================================================================================

-- Insert default subscription-based permission groups (replacing old tier system)
INSERT INTO permission_groups (
    name,
    slug,
    description,
    group_type,
    permissions,
    price,
    currency,
    billing_cycle,
    is_active,
    is_promoted,
    display_order,
    group_metadata
) VALUES 
-- Free tier
(
    'Free Plan',
    'free-plan',
    'Basic analytics access for getting started with EPSX',
    'subscription',
    '[
        "epsx:analytics:view:3",
        "epsx:portfolio:view",
        "epsx:profile:manage"
    ]'::jsonb,
    0.00,
    'USD',
    'monthly',
    TRUE,
    FALSE,
    0,
    '{
        "tier_level": "Bronze",
        "features": ["3 stock rankings", "Basic analytics", "Community support"],
        "limits": {"rankings_per_month": 3, "exports_per_month": 0}
    }'::jsonb
),
-- Professional tier
(
    'Professional Plan',
    'professional-plan', 
    'Advanced analytics and trading features for serious investors',
    'subscription',
    '[
        "epsx:analytics:view:50",
        "epsx:analytics:export",
        "epsx:trading:advanced",
        "epsx:portfolio:manage",
        "epsx:portfolio:advanced",
        "epsx:alerts:create",
        "epsx:alerts:manage"
    ]'::jsonb,
    29.99,
    'USD', 
    'monthly',
    TRUE,
    TRUE,
    1,
    '{
        "tier_level": "Gold",
        "features": ["50 stock rankings", "Advanced analytics", "Export capabilities", "Priority support"],
        "limits": {"rankings_per_month": 50, "exports_per_month": 10}
    }'::jsonb
),
-- Enterprise tier
(
    'Enterprise Plan',
    'enterprise-plan',
    'Full platform access with enterprise features and support',
    'subscription', 
    '[
        "epsx:analytics:*",
        "epsx:trading:*", 
        "epsx:portfolio:*",
        "epsx:alerts:*",
        "epsx:api:*",
        "epsx:enterprise:*"
    ]'::jsonb,
    99.99,
    'USD',
    'monthly',
    TRUE,
    FALSE,
    2,
    '{
        "tier_level": "Diamond", 
        "features": ["Unlimited rankings", "All analytics features", "API access", "Enterprise support"],
        "limits": {"rankings_per_month": -1, "exports_per_month": -1}
    }'::jsonb
),
-- Admin group
(
    'Platform Administrators',
    'platform-admins',
    'Full administrative access to the EPSX platform',
    'admin',
    '[
        "admin:*:*",
        "epsx:*:*"
    ]'::jsonb,
    0.00,
    'USD',
    'lifetime',
    TRUE,
    FALSE,
    999,
    '{
        "tier_level": "Admin",
        "features": ["Full platform access", "User management", "System administration"],
        "limits": {}
    }'::jsonb
)
ON CONFLICT (slug) DO NOTHING;

-- ================================================================================================
-- 7. MIGRATE EXISTING DATA (if applicable)
-- ================================================================================================

-- Note: This migration assumes we're starting fresh with the unified system
-- If there's existing tier or permission data, it would need to be migrated here

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'UNIFIED PERMISSION GROUPS MIGRATION COMPLETED SUCCESSFULLY! 🎉';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Created Components:';
    RAISE NOTICE '• ✅ permission_groups table (unified tier and permission groups)';
    RAISE NOTICE '• ✅ wallet_group_memberships table (wallet-first assignments)';
    RAISE NOTICE '• ✅ Optimized indexes for performance';
    RAISE NOTICE '• ✅ Permission validation functions';
    RAISE NOTICE '• ✅ Automatic timestamp triggers';
    RAISE NOTICE '• ✅ Default permission groups (Free, Pro, Enterprise, Admin)';
    RAISE NOTICE '';
    RAISE NOTICE 'Ready for unified permission group system implementation!';
    RAISE NOTICE '=================================================================================';
END $$;