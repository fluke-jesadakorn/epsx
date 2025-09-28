-- ================================================================================================
-- WEB3 WALLET USERS TABLE CREATION (Direct Approach)
-- ================================================================================================
-- This migration creates the wallet_users table for the Web3-first WalletUser aggregate
-- without dependencies on the existing group-based permission system
-- ================================================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ================================================================================================
-- 1. CREATE CONSOLIDATED WALLET_USERS TABLE
-- ================================================================================================

-- Create the unified wallet_users table matching WalletUser aggregate
CREATE TABLE IF NOT EXISTS wallet_users (
    -- Primary identity - wallet address is the primary key
    wallet_address VARCHAR(42) PRIMARY KEY,
    
    -- User status
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    
    -- Direct permissions storage (JSON array of Permission objects)
    permissions JSONB DEFAULT '[]' NOT NULL,
    
    -- Tier system for subscription/access levels
    tier_level VARCHAR(20) DEFAULT 'Bronze' NOT NULL,
    
    -- Web3-specific metadata (JSON object)
    wallet_metadata JSONB DEFAULT '{}' NOT NULL,
    
    -- Audit fields
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    last_auth_at TIMESTAMPTZ,
    
    -- Additional constraints
    CONSTRAINT valid_tier_level CHECK (tier_level IN ('Bronze', 'Silver', 'Gold', 'Platinum', 'Diamond')),
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    )
);

-- ================================================================================================
-- 2. CREATE OPTIMIZED INDEXES FOR WALLET_USERS
-- ================================================================================================

-- Primary indexes for performance
CREATE INDEX IF NOT EXISTS idx_wallet_users_active ON wallet_users(is_active, wallet_address);
CREATE INDEX IF NOT EXISTS idx_wallet_users_tier ON wallet_users(tier_level, is_active);
CREATE INDEX IF NOT EXISTS idx_wallet_users_last_auth ON wallet_users(last_auth_at DESC) WHERE last_auth_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_wallet_users_created ON wallet_users(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_wallet_users_updated ON wallet_users(updated_at DESC);

-- GIN indexes for JSON operations
CREATE INDEX IF NOT EXISTS idx_wallet_users_permissions_gin ON wallet_users USING GIN (permissions);
CREATE INDEX IF NOT EXISTS idx_wallet_users_metadata_gin ON wallet_users USING GIN (wallet_metadata);

-- ================================================================================================
-- 3. CREATE BASIC WALLET USER MANAGEMENT FUNCTIONS
-- ================================================================================================

-- Function to add permission to wallet user
CREATE OR REPLACE FUNCTION add_wallet_user_permission(
    target_wallet VARCHAR(42),
    permission_name TEXT,
    permission_type TEXT DEFAULT 'Manual',
    expires_at TIMESTAMPTZ DEFAULT NULL,
    metadata JSONB DEFAULT '{}'
) RETURNS BOOLEAN AS $$
DECLARE
    current_permissions JSONB;
    new_permission JSONB;
    permission_exists BOOLEAN := FALSE;
BEGIN
    -- Check if wallet exists
    IF NOT EXISTS (SELECT 1 FROM wallet_users WHERE wallet_address = target_wallet) THEN
        RAISE EXCEPTION 'Wallet user not found: %', target_wallet;
    END IF;
    
    -- Get current permissions
    SELECT permissions INTO current_permissions 
    FROM wallet_users 
    WHERE wallet_address = target_wallet;
    
    -- Check if permission already exists
    SELECT EXISTS(
        SELECT 1 FROM jsonb_array_elements(current_permissions) AS perm
        WHERE perm->>'name' = permission_name
    ) INTO permission_exists;
    
    IF permission_exists THEN
        RETURN FALSE; -- Permission already exists
    END IF;
    
    -- Create new permission object
    new_permission := jsonb_build_object(
        'name', permission_name,
        'permission_type', permission_type,
        'granted_at', NOW(),
        'expires_at', expires_at,
        'is_active', true,
        'metadata', metadata
    );
    
    -- Add permission to array
    UPDATE wallet_users 
    SET 
        permissions = permissions || new_permission,
        updated_at = NOW()
    WHERE wallet_address = target_wallet;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Function to check if wallet has permission
CREATE OR REPLACE FUNCTION wallet_user_has_permission(
    target_wallet VARCHAR(42),
    permission_name TEXT
) RETURNS BOOLEAN AS $$
DECLARE
    has_permission BOOLEAN := FALSE;
BEGIN
    -- Check for exact match or wildcard match
    SELECT EXISTS(
        SELECT 1 FROM wallet_users wu,
        jsonb_array_elements(wu.permissions) AS perm
        WHERE wu.wallet_address = target_wallet
        AND wu.is_active = TRUE
        AND perm->>'is_active' = 'true'
        AND (perm->>'expires_at' IS NULL OR (perm->>'expires_at')::timestamptz > NOW())
        AND (
            perm->>'name' = permission_name OR
            perm->>'name' LIKE (SPLIT_PART(permission_name, ':', 1) || ':*:*') OR
            perm->>'name' = 'admin:*:*'
        )
    ) INTO has_permission;
    
    RETURN has_permission;
END;
$$ LANGUAGE plpgsql;

-- ================================================================================================
-- 4. CREATE TRIGGERS FOR WALLET USERS
-- ================================================================================================

-- Trigger to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION trigger_update_wallet_users_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_wallet_users_updated_at
    BEFORE UPDATE ON wallet_users
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_wallet_users_timestamp();

-- ================================================================================================
-- 5. INSERT SAMPLE DATA FOR TESTING
-- ================================================================================================

-- Insert a few sample wallet users for testing
INSERT INTO wallet_users (
    wallet_address,
    is_active,
    permissions,
    tier_level,
    wallet_metadata
) VALUES 
(
    '0x742d35cc6634c0532925a3b8d369d7763f3c45c6',
    TRUE,
    '[
        {"name": "epsx:analytics:view", "permission_type": "Manual", "granted_at": "2024-01-01T00:00:00Z", "expires_at": null, "is_active": true, "metadata": {}},
        {"name": "epsx:trading:basic", "permission_type": "Manual", "granted_at": "2024-01-01T00:00:00Z", "expires_at": null, "is_active": true, "metadata": {}}
    ]'::jsonb,
    'Silver',
    '{"display_name": "Test Wallet", "is_verified": true, "tier": "silver"}'::jsonb
),
(
    '0x1234567890123456789012345678901234567890',
    TRUE,
    '[
        {"name": "admin:*:*", "permission_type": "Manual", "granted_at": "2024-01-01T00:00:00Z", "expires_at": null, "is_active": true, "metadata": {}}
    ]'::jsonb,
    'Diamond',
    '{"display_name": "Admin Wallet", "is_verified": true, "tier": "admin"}'::jsonb
)
ON CONFLICT (wallet_address) DO NOTHING;

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'WALLET USERS TABLE CREATION COMPLETED SUCCESSFULLY! 🎉';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Created Components:';
    RAISE NOTICE '• ✅ wallet_users table with Web3-first design';
    RAISE NOTICE '• ✅ Optimized indexes for JSON permission queries';
    RAISE NOTICE '• ✅ Permission management functions';
    RAISE NOTICE '• ✅ Automatic timestamp triggers';
    RAISE NOTICE '• ✅ Sample data for testing';
    RAISE NOTICE '';
    RAISE NOTICE 'Ready for WalletUser aggregate and WalletUserRepository implementation!';
    RAISE NOTICE '=================================================================================';
END $$;