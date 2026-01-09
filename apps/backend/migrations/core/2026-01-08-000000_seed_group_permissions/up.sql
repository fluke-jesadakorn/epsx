-- Seed group_permissions table with permission mappings for subscription plans
-- This migration populates the empty group_permissions table so that the
-- get_wallet_permissions_detailed_working() stored procedure can resolve
-- permissions for users assigned to subscription plan groups.

-- Get UUIDs for groups and permissions first
DO $$
DECLARE
    -- Group IDs
    free_plan_id UUID;
    starter_plan_id UUID;
    pro_plan_id UUID;
    enterprise_plan_id UUID;
    api_developer_id UUID;
    
    -- Permission IDs
    perm_analytics_view UUID;
    perm_analytics_advanced UUID;
    perm_trading_basic UUID;
    perm_trading_pro UUID;
    perm_trading_advanced UUID;
    perm_api_read UUID;
    perm_api_write UUID;
    perm_data_export UUID;
    perm_notifications_manage UUID;
BEGIN
    -- Fetch group IDs by slug
    SELECT id INTO free_plan_id FROM groups WHERE slug = 'free';
    SELECT id INTO starter_plan_id FROM groups WHERE slug = 'starter';
    SELECT id INTO pro_plan_id FROM groups WHERE slug = 'pro';
    SELECT id INTO enterprise_plan_id FROM groups WHERE slug = 'enterprise';
    SELECT id INTO api_developer_id FROM groups WHERE slug = 'api-developer';
    
    -- Fetch permission IDs by permission_string
    SELECT id INTO perm_analytics_view FROM permissions WHERE permission_string = 'epsx:analytics:view';
    SELECT id INTO perm_analytics_advanced FROM permissions WHERE permission_string = 'epsx:analytics:advanced';
    SELECT id INTO perm_trading_basic FROM permissions WHERE permission_string = 'epsx:trading:basic';
    SELECT id INTO perm_trading_pro FROM permissions WHERE permission_string = 'epsx:trading:pro';
    SELECT id INTO perm_trading_advanced FROM permissions WHERE permission_string = 'epsx:trading:advanced';
    SELECT id INTO perm_api_read FROM permissions WHERE permission_string = 'epsx:api:read';
    SELECT id INTO perm_api_write FROM permissions WHERE permission_string = 'epsx:api:write';
    SELECT id INTO perm_data_export FROM permissions WHERE permission_string = 'epsx:data:export';
    SELECT id INTO perm_notifications_manage FROM permissions WHERE permission_string = 'epsx:notifications:manage';
    
    -- Log what we found
    RAISE NOTICE 'Group IDs: free=%, starter=%, pro=%, enterprise=%, api=%', 
        free_plan_id, starter_plan_id, pro_plan_id, enterprise_plan_id, api_developer_id;
    RAISE NOTICE 'Permission IDs: analytics_view=%, trading_basic=%, api_read=%',
        perm_analytics_view, perm_trading_basic, perm_api_read;
    
    -- =========================================
    -- FREE PLAN: Basic analytics view only
    -- =========================================
    IF free_plan_id IS NOT NULL AND perm_analytics_view IS NOT NULL THEN
        INSERT INTO group_permissions (group_id, permission_id, granted_at)
        VALUES (free_plan_id, perm_analytics_view, NOW())
        ON CONFLICT (group_id, permission_id) DO NOTHING;
        RAISE NOTICE '✅ Free Plan: Added analytics:view';
    END IF;
    
    -- =========================================
    -- STARTER PLAN: Basic analytics + Basic trading
    -- =========================================
    IF starter_plan_id IS NOT NULL THEN
        -- Analytics view
        IF perm_analytics_view IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (starter_plan_id, perm_analytics_view, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        -- Trading basic
        IF perm_trading_basic IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (starter_plan_id, perm_trading_basic, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        RAISE NOTICE '✅ Starter Plan: Added analytics:view, trading:basic';
    END IF;
    
    -- =========================================
    -- PRO PLAN: Starter + Advanced analytics, Pro trading, API read
    -- =========================================
    IF pro_plan_id IS NOT NULL THEN
        -- Analytics view
        IF perm_analytics_view IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (pro_plan_id, perm_analytics_view, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        -- Analytics advanced
        IF perm_analytics_advanced IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (pro_plan_id, perm_analytics_advanced, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        -- Trading basic
        IF perm_trading_basic IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (pro_plan_id, perm_trading_basic, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        -- Trading pro
        IF perm_trading_pro IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (pro_plan_id, perm_trading_pro, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        -- API read
        IF perm_api_read IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (pro_plan_id, perm_api_read, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        RAISE NOTICE '✅ Pro Plan: Added analytics:view/advanced, trading:basic/pro, api:read';
    END IF;
    
    -- =========================================
    -- ENTERPRISE PLAN: All permissions (excluding admin)
    -- =========================================
    IF enterprise_plan_id IS NOT NULL THEN
        -- All EPSX permissions
        IF perm_analytics_view IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (enterprise_plan_id, perm_analytics_view, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        IF perm_analytics_advanced IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (enterprise_plan_id, perm_analytics_advanced, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        IF perm_trading_basic IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (enterprise_plan_id, perm_trading_basic, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        IF perm_trading_pro IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (enterprise_plan_id, perm_trading_pro, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        IF perm_trading_advanced IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (enterprise_plan_id, perm_trading_advanced, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        IF perm_api_read IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (enterprise_plan_id, perm_api_read, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        IF perm_api_write IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (enterprise_plan_id, perm_api_write, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        IF perm_data_export IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (enterprise_plan_id, perm_data_export, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        IF perm_notifications_manage IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (enterprise_plan_id, perm_notifications_manage, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        RAISE NOTICE '✅ Enterprise Plan: Added all EPSX permissions';
    END IF;
    
    -- =========================================
    -- API DEVELOPER: API-focused permissions
    -- =========================================
    IF api_developer_id IS NOT NULL THEN
        -- API read
        IF perm_api_read IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (api_developer_id, perm_api_read, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        -- API write
        IF perm_api_write IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (api_developer_id, perm_api_write, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        -- Data export
        IF perm_data_export IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (api_developer_id, perm_data_export, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        -- Analytics view (for API access to analytics endpoints)
        IF perm_analytics_view IS NOT NULL THEN
            INSERT INTO group_permissions (group_id, permission_id, granted_at)
            VALUES (api_developer_id, perm_analytics_view, NOW())
            ON CONFLICT (group_id, permission_id) DO NOTHING;
        END IF;
        
        RAISE NOTICE '✅ API Developer: Added api:read/write, data:export, analytics:view';
    END IF;
    
    RAISE NOTICE '🎉 group_permissions seeding complete!';
END $$;

-- Log the final count
DO $$
DECLARE
    total_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_count FROM group_permissions;
    RAISE NOTICE '📊 Total group_permissions entries: %', total_count;
END $$;
