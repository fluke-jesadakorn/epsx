-- Grant full admin permissions to wallet address
-- Wallet: 0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7

DO $$
DECLARE
    target_wallet TEXT := '0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7';
    admin_user_id UUID;
    admin_permissions TEXT[] := ARRAY[
        'admin:*:*',
        'admin:users:*',
        'admin:permissions:*',
        'admin:system:*',
        'admin:analytics:*',
        'admin:notifications:*',
        'admin:api:*',
        'admin:enterprise:*',
        'epsx:*:*',
        'epsx:trading:*',
        'epsx:analytics:*',
        'epsx:portfolio:*',
        'epsx:alerts:*',
        'epsx:premium:*',
        'enterprise:*:*',
        'enterprise:teams:*',
        'enterprise:api:*',
        'enterprise:billing:*',
        'web3:*:*',
        'web3:wallets:*',
        'web3:permissions:*',
        'web3:nft:*',
        'web3:token:*',
        'web3:dao:*',
        'api:*:*',
        'api:keys:*',
        'api:enterprise:*',
        'api:unlimited:*'
    ];
    perm TEXT;
    existing_count INTEGER;
BEGIN
    -- First, delete any existing permissions for this wallet to avoid duplicates
    DELETE FROM wallet_permissions WHERE wallet_address = target_wallet;
    
    -- Get admin user for granting permissions
    SELECT id INTO admin_user_id
    FROM users 
    WHERE email = 'info@epsx.io' 
    LIMIT 1;
    
    IF admin_user_id IS NULL THEN
        -- Create a system admin user if none exists
        INSERT INTO users (
            firebase_uid,
            email,
            display_name,
            role,
            email_verified,
            is_active
        ) VALUES (
            'system-admin-' || gen_random_uuid()::text,
            'system@epsx.io',
            'System Admin',
            'admin',
            true,
            true
        ) RETURNING id INTO admin_user_id;
    END IF;
    
    -- Grant wallet permissions for each permission type
    FOREACH perm IN ARRAY admin_permissions
    LOOP
        INSERT INTO wallet_permissions (
            wallet_address,
            permission,
            permission_type,
            granted_by,
            granted_at,
            is_active,
            verification_data
        ) VALUES (
            target_wallet,
            perm,
            'manual',
            admin_user_id,
            NOW(),
            true,
            jsonb_build_object(
                'reason', 'Admin wallet grant',
                'granted_via', 'database_script',
                'timestamp', NOW()
            )
        );
    END LOOP;
    
    -- Also create a user record linked to the wallet
    INSERT INTO users (
        firebase_uid,
        email,
        display_name,
        role,
        email_verified,
        is_active
    ) VALUES (
        'wallet-' || target_wallet,
        'admin@' || target_wallet || '.wallet',
        'Admin Wallet User',
        'admin',
        true,
        true
    ) ON CONFLICT (firebase_uid) DO UPDATE SET
        role = 'admin',
        is_active = true,
        updated_at = NOW();
    
    GET DIAGNOSTICS existing_count = ROW_COUNT;
    
    RAISE NOTICE 'Successfully granted % permissions to wallet: %', array_length(admin_permissions, 1), target_wallet;
    RAISE NOTICE 'Wallet can now access all admin, EPSX, enterprise, Web3, and API features';
    
END $$;

-- Verify the permissions were granted
SELECT 
    wallet_address,
    permission,
    permission_type,
    granted_at,
    is_active
FROM wallet_permissions 
WHERE wallet_address = '0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7'
AND is_active = true
ORDER BY permission;