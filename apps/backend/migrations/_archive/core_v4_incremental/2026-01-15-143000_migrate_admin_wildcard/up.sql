-- Migrate from admin:*:* wildcard to granular admin permissions
-- SECURITY: Strictly separate Platform and Admin privileges

DO $$
DECLARE
    admin_wildcard_id UUID;
    wallet_addr RECORD;
    
    -- New Permission IDs
    p_users_view UUID;
    p_users_manage UUID;
    p_perms_view UUID;
    p_perms_manage UUID;
    p_payments_view UUID;
    p_payments_manage UUID;
    p_system_view UUID;
    p_system_manage UUID;
BEGIN
    -- 1. Ensure granular permissions exist in the permissions table
    INSERT INTO permissions (permission_string, platform, resource, action, description, permission_type, is_active, is_system)
    VALUES 
        ('admin:users:view', 'admin', 'users', 'view', 'View user lists and profiles', 'manual', true, true),
        ('admin:users:manage', 'admin', 'users', 'manage', 'Full user management (edit, ban)', 'manual', true, true),
        ('admin:permissions:view', 'admin', 'permissions', 'view', 'View granted permissions and groups', 'manual', true, true),
        ('admin:permissions:manage', 'admin', 'permissions', 'manage', 'Manage permissions and group assignments', 'manual', true, true),
        ('admin:payments:view', 'admin', 'payments', 'view', 'View billing and revenue analytics', 'manual', true, true),
        ('admin:payments:manage', 'admin', 'payments', 'manage', 'Manage refunds and payment adjustments', 'manual', true, true),
        ('admin:system:view', 'admin', 'system', 'view', 'View system health and logs', 'manual', true, true),
        ('admin:system:manage', 'admin', 'system', 'manage', 'Manage system configuration and maintenance', 'manual', true, true)
    ON CONFLICT (permission_string) DO UPDATE SET is_active = true, is_system = true;

    -- 2. Fetch the IDs of the new permissions
    SELECT id INTO p_users_view FROM permissions WHERE permission_string = 'admin:users:view';
    SELECT id INTO p_users_manage FROM permissions WHERE permission_string = 'admin:users:manage';
    SELECT id INTO p_perms_view FROM permissions WHERE permission_string = 'admin:permissions:view';
    SELECT id INTO p_perms_manage FROM permissions WHERE permission_string = 'admin:permissions:manage';
    SELECT id INTO p_payments_view FROM permissions WHERE permission_string = 'admin:payments:view';
    SELECT id INTO p_payments_manage FROM permissions WHERE permission_string = 'admin:payments:manage';
    SELECT id INTO p_system_view FROM permissions WHERE permission_string = 'admin:system:view';
    SELECT id INTO p_system_manage FROM permissions WHERE permission_string = 'admin:system:manage';

    -- 3. Find the ID for the old wildcard permission
    SELECT id INTO admin_wildcard_id FROM permissions WHERE permission_string = 'admin:*:*';
    
    -- 4. If wildcard exists, migrate all wallets that use it
    IF admin_wildcard_id IS NOT NULL THEN
        RAISE NOTICE 'Migrating wallets with admin:*:* (ID: %)', admin_wildcard_id;
        
        -- Iterate through all active wallets with the wildcard
        FOR wallet_addr IN (SELECT DISTINCT wallet_address FROM wallet_direct_permissions WHERE permission_id = admin_wildcard_id AND is_active = true) LOOP
            RAISE NOTICE 'Migrating wallet: %', wallet_addr.wallet_address;
            
            -- Grant all granular admin permissions
            INSERT INTO wallet_direct_permissions (wallet_address, permission_id, granted_by, grant_reason, is_active)
            VALUES 
                (wallet_addr.wallet_address, p_users_view, 'SystemMigration', 'Transition from wildcard to granular', true),
                (wallet_addr.wallet_address, p_users_manage, 'SystemMigration', 'Transition from wildcard to granular', true),
                (wallet_addr.wallet_address, p_perms_view, 'SystemMigration', 'Transition from wildcard to granular', true),
                (wallet_addr.wallet_address, p_perms_manage, 'SystemMigration', 'Transition from wildcard to granular', true),
                (wallet_addr.wallet_address, p_payments_view, 'SystemMigration', 'Transition from wildcard to granular', true),
                (wallet_addr.wallet_address, p_payments_manage, 'SystemMigration', 'Transition from wildcard to granular', true),
                (wallet_addr.wallet_address, p_system_view, 'SystemMigration', 'Transition from wildcard to granular', true),
                (wallet_addr.wallet_address, p_system_manage, 'SystemMigration', 'Transition from wildcard to granular', true)
            ON CONFLICT (wallet_address, permission_id) DO UPDATE SET is_active = true;
            
            -- Deactivate the old wildcard for this wallet
            UPDATE wallet_direct_permissions 
            SET is_active = false, grant_reason = grant_reason || ' (Migrated to granular)'
            WHERE wallet_address = wallet_addr.wallet_address AND permission_id = admin_wildcard_id;
        END LOOP;
        
        -- Also check group permissions just in case any group used the wildcard
        FOR wallet_addr IN (SELECT DISTINCT group_id FROM group_permissions WHERE permission_id = admin_wildcard_id) LOOP
             INSERT INTO group_permissions (group_id, permission_id, granted_by, grant_reason)
             VALUES 
                (wallet_addr.group_id, p_users_view, 'SystemMigration', 'Transition from wildcard to granular'),
                (wallet_addr.group_id, p_users_manage, 'SystemMigration', 'Transition from wildcard to granular'),
                (wallet_addr.group_id, p_perms_view, 'SystemMigration', 'Transition from wildcard to granular'),
                (wallet_addr.group_id, p_perms_manage, 'SystemMigration', 'Transition from wildcard to granular'),
                (wallet_addr.group_id, p_payments_view, 'SystemMigration', 'Transition from wildcard to granular'),
                (wallet_addr.group_id, p_payments_manage, 'SystemMigration', 'Transition from wildcard to granular'),
                (wallet_addr.group_id, p_system_view, 'SystemMigration', 'Transition from wildcard to granular'),
                (wallet_addr.group_id, p_system_manage, 'SystemMigration', 'Transition from wildcard to granular')
             ON CONFLICT (group_id, permission_id) DO NOTHING;
             
             DELETE FROM group_permissions WHERE group_id = wallet_addr.group_id AND permission_id = admin_wildcard_id;
        END LOOP;

        -- Finally, deactivate the wildcard definition itself
        UPDATE permissions SET is_active = false WHERE id = admin_wildcard_id;
        RAISE NOTICE 'DONE: admin:*:* has been migrated and deactivated.';
    ELSE
        RAISE NOTICE 'No admin:*:* permission found in definition table. Skipping wallet migration.';
    END IF;

END $$;
