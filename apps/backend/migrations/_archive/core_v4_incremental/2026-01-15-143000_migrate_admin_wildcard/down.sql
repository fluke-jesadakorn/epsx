-- Revert from granular admin permissions back to admin:*:* wildcard
-- WARNING: This restores the "Super Admin" wildcard if it was previously active.

DO $$
DECLARE
    admin_wildcard_id UUID;
    wallet_addr RECORD;
    granular_ids UUID[];
BEGIN
    -- 1. Get the wildcard ID (it might be inactive)
    SELECT id INTO admin_wildcard_id FROM permissions WHERE permission_string = 'admin:*:*';
    
    -- 2. Reactivate the wildcard if it exists
    IF admin_wildcard_id IS NOT NULL THEN
        UPDATE permissions SET is_active = true WHERE id = admin_wildcard_id;
        
        -- 3. Get IDs of granular permissions to clean up later
        SELECT ARRAY_AGG(id) INTO granular_ids FROM permissions WHERE permission_string IN (
            'admin:users:view', 'admin:users:manage', 
            'admin:permissions:view', 'admin:permissions:manage',
            'admin:payments:view', 'admin:payments:manage',
            'admin:system:view', 'admin:system:manage'
        );

        -- 4. Restore wildcard for all wallets that have any of the granular permissions
        FOR wallet_addr IN (
            SELECT DISTINCT wallet_address 
            FROM wallet_direct_permissions 
            WHERE permission_id = ANY(granular_ids) AND is_active = true
        ) LOOP
            INSERT INTO wallet_direct_permissions (wallet_address, permission_id, granted_by, grant_reason, is_active)
            VALUES (wallet_addr.wallet_address, admin_wildcard_id, 'SystemRollback', 'Reverting to wildcard from granular', true)
            ON CONFLICT (wallet_address, permission_id) DO UPDATE SET is_active = true;
            
            -- Keep granular ones as fallback or remove them? Let's remove them for a clean revert
            DELETE FROM wallet_direct_permissions 
            WHERE wallet_address = wallet_addr.wallet_address AND permission_id = ANY(granular_ids);
        END LOOP;
        
        -- Same for groups
        FOR wallet_addr IN (
            SELECT DISTINCT group_id 
            FROM group_permissions 
            WHERE permission_id = ANY(granular_ids)
        ) LOOP
            INSERT INTO group_permissions (group_id, permission_id, granted_by, grant_reason)
            VALUES (wallet_addr.group_id, admin_wildcard_id, 'SystemRollback', 'Reverting to wildcard from granular')
            ON CONFLICT (group_id, permission_id) DO NOTHING;
            
            DELETE FROM group_permissions 
            WHERE group_id = wallet_addr.group_id AND permission_id = ANY(granular_ids);
        END LOOP;

        RAISE NOTICE 'Rollback complete: admin:*:* restored and granular permissions removed.';
    ELSE
        RAISE NOTICE 'admin:*:* wildcard not found. Rollback cannot restore it.';
    END IF;
END $$;
