-- Grant full admin permissions to wallet address
-- Wallet: 0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7
-- This script creates a user with comprehensive admin permissions

DO $$
DECLARE
    target_wallet TEXT := '0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7';
    user_uuid UUID;
    admin_permissions TEXT[] := ARRAY[
        -- Core admin permissions
        'admin:*:*',
        'admin:users:*',
        'admin:permissions:*',
        'admin:system:*',
        'admin:analytics:*',
        'admin:notifications:*',
        'admin:api:*',
        'admin:enterprise:*',
        
        -- EPSX platform permissions
        'epsx:*:*',
        'epsx:trading:*',
        'epsx:analytics:*',
        'epsx:portfolio:*',
        'epsx:alerts:*',
        'epsx:premium:*',
        
        -- Enterprise permissions
        'enterprise:*:*',
        'enterprise:teams:*',
        'enterprise:api:*',
        'enterprise:billing:*',
        
        -- Web3 permissions
        'web3:*:*',
        'web3:wallets:*',
        'web3:permissions:*',
        'web3:nft:*',
        'web3:token:*',
        'web3:dao:*',
        
        -- API permissions
        'api:*:*',
        'api:keys:*',
        'api:enterprise:*',
        'api:unlimited:*'
    ];
    perm TEXT;
BEGIN
    -- Check if user already exists with this wallet
    SELECT id INTO user_uuid
    FROM users 
    WHERE wallet_address = target_wallet;
    
    IF user_uuid IS NULL THEN
        -- Create new user with wallet address
        INSERT INTO users (
            id,
            email,
            wallet_address,
            role,
            permissions,
            created_at,
            updated_at,
            email_verified,
            is_active
        ) VALUES (
            gen_random_uuid(),
            'admin@' || target_wallet || '.wallet',
            target_wallet,
            'admin',
            admin_permissions,
            NOW(),
            NOW(),
            true,
            true
        ) RETURNING id INTO user_uuid;
        
        RAISE NOTICE 'Created new admin user with ID: % for wallet: %', user_uuid, target_wallet;
    ELSE
        -- Update existing user with admin permissions
        UPDATE users 
        SET 
            role = 'admin',
            permissions = admin_permissions,
            updated_at = NOW(),
            is_active = true
        WHERE id = user_uuid;
        
        RAISE NOTICE 'Updated existing user with ID: % for wallet: %', user_uuid, target_wallet;
    END IF;
    
    -- Grant manual permissions for each permission type
    FOREACH perm IN ARRAY admin_permissions
    LOOP
        INSERT INTO user_permissions (
            id,
            user_id,
            permission,
            source_type,
            source_metadata,
            granted_by,
            granted_at,
            expires_at,
            is_active
        ) VALUES (
            gen_random_uuid(),
            user_uuid,
            perm,
            'manual',
            jsonb_build_object(
                'reason', 'Admin wallet grant',
                'wallet_address', target_wallet,
                'granted_via', 'database_script'
            ),
            user_uuid, -- Self-granted for initial admin
            NOW(),
            NULL, -- No expiration
            true
        ) ON CONFLICT (user_id, permission) DO UPDATE SET
            is_active = true,
            granted_at = NOW(),
            expires_at = NULL,
            source_metadata = jsonb_build_object(
                'reason', 'Admin wallet grant updated',
                'wallet_address', target_wallet,
                'granted_via', 'database_script'
            );
    END LOOP;
    
    -- Create enterprise team for this admin
    INSERT INTO enterprise_teams (
        id,
        name,
        description,
        created_at,
        plan_tier,
        monthly_quota,
        current_usage,
        is_active
    ) VALUES (
        gen_random_uuid(),
        'Admin Team - ' || target_wallet,
        'Enterprise admin team for wallet ' || target_wallet,
        NOW(),
        'enterprise',
        1000000, -- 1M requests per month
        0,
        true
    ) ON CONFLICT (name) DO NOTHING;
    
    RAISE NOTICE 'Successfully granted all permissions to wallet: %', target_wallet;
    RAISE NOTICE 'User can now access all admin, EPSX, enterprise, Web3, and API features';
    
END $$;