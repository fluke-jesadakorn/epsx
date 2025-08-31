-- ============================================================================
-- EPSX SIMPLE SEED DATA - Essential Data Only
-- ============================================================================
-- This script only inserts essential seed data without modifying schema
-- Safe to run on existing database with proper permissions
-- ============================================================================

-- ============================================================================
-- ADMIN MODULES SEED DATA
-- ============================================================================

-- Insert core admin modules (8 essential modules)
INSERT INTO admin_modules (module_code, module_name, description, icon, color) VALUES
('user-management', 'User Management', 'User CRUD operations, profile management, and account administration', 'users', 'blue'),
('analytics-access', 'Analytics Access', 'Dashboard access, reporting, and data analysis capabilities', 'chart-bar', 'green'),
('billing-admin', 'Billing Administration', 'Payment management, subscriptions, and package assignments', 'credit-card', 'emerald'),
('system-admin', 'System Administration', 'Database management, system configuration, and infrastructure monitoring', 'server', 'red'),
('content-management', 'Content Management', 'Content creation, editing, and resource management', 'document-text', 'purple'),
('support-access', 'Support Access', 'User support tools, ticketing, and troubleshooting capabilities', 'support', 'yellow'),
('security-management', 'Security Management', 'Security monitoring, compliance, and audit management', 'shield-check', 'orange'),
('api-management', 'API Management', 'API key management, developer tools, and integration oversight', 'code', 'indigo')
ON CONFLICT (module_code) DO UPDATE SET
    module_name = EXCLUDED.module_name,
    description = EXCLUDED.description,
    icon = EXCLUDED.icon,
    color = EXCLUDED.color,
    updated_at = NOW();

-- ============================================================================
-- EPS ANALYTICS SEED DATA
-- ============================================================================

-- Sample EPS analytics data for development and testing
INSERT INTO eps_growth_analytics (symbol, name, country, sector, exchange, current_eps, qoq_growth_rate, price_current, market_cap, volume, ranking_score) VALUES
-- US Tech Leaders
('AAPL', 'Apple Inc.', 'US', 'Technology', 'NASDAQ', 6.15, 8.50, 175.25, 2800000000000, 45000000, 95.8),
('MSFT', 'Microsoft Corporation', 'US', 'Technology', 'NASDAQ', 9.65, 12.30, 335.50, 2500000000000, 28000000, 94.2),
('GOOGL', 'Alphabet Inc.', 'US', 'Technology', 'NASDAQ', 5.80, 15.75, 138.75, 1750000000000, 22000000, 92.5),
('NVDA', 'NVIDIA Corporation', 'US', 'Technology', 'NASDAQ', 12.35, 22.40, 485.75, 1200000000000, 35000000, 97.2),
-- International Growth Leaders  
('ASML', 'ASML Holding N.V.', 'NL', 'Technology', 'AMS', 15.25, 18.90, 675.30, 280000000000, 1200000, 91.4),
('TSM', 'Taiwan Semiconductor', 'TW', 'Technology', 'NYSE', 8.90, 14.20, 98.45, 520000000000, 18000000, 89.7),
('SHOP', 'Shopify Inc.', 'CA', 'Technology', 'NYSE', 2.45, 35.60, 58.90, 75000000000, 8500000, 88.3),
-- Additional Growth Stocks for Testing
('META', 'Meta Platforms Inc.', 'US', 'Technology', 'NASDAQ', 14.87, 16.25, 298.75, 760000000000, 32000000, 93.1),
('AMZN', 'Amazon.com Inc.', 'US', 'Consumer Discretionary', 'NASDAQ', 3.65, 11.80, 142.50, 1500000000000, 38000000, 89.9),
('NFLX', 'Netflix Inc.', 'US', 'Communication Services', 'NASDAQ', 15.45, 9.75, 445.20, 195000000000, 8200000, 87.6),
-- Emerging Market Leaders
('BABA', 'Alibaba Group', 'CN', 'Consumer Discretionary', 'NYSE', 8.92, 7.30, 95.20, 245000000000, 15000000, 86.4),
('TSLA', 'Tesla Inc.', 'US', 'Consumer Discretionary', 'NASDAQ', 4.12, 13.80, 248.50, 795000000000, 42000000, 90.3),
-- Financial Sector Growth
('V', 'Visa Inc.', 'US', 'Financial Services', 'NYSE', 8.45, 10.50, 275.80, 540000000000, 12000000, 88.9),
('MA', 'Mastercard Inc.', 'US', 'Financial Services', 'NYSE', 11.25, 12.75, 415.30, 385000000000, 8500000, 89.1),
-- Healthcare Innovation
('UNH', 'UnitedHealth Group', 'US', 'Healthcare', 'NYSE', 24.75, 6.80, 485.90, 450000000000, 6200000, 85.7)
ON CONFLICT (symbol) DO UPDATE SET
    name = EXCLUDED.name,
    country = EXCLUDED.country,
    sector = EXCLUDED.sector,
    exchange = EXCLUDED.exchange,
    current_eps = EXCLUDED.current_eps,
    qoq_growth_rate = EXCLUDED.qoq_growth_rate,
    price_current = EXCLUDED.price_current,
    market_cap = EXCLUDED.market_cap,
    volume = EXCLUDED.volume,
    ranking_score = EXCLUDED.ranking_score,
    updated_at = NOW();

-- ============================================================================
-- SAMPLE ADMIN USER CREATION
-- ============================================================================

-- Create a sample admin user for development (should be replaced with real Firebase UID)
-- This will only insert if the firebase_uid doesn't exist
DO $$
DECLARE
    sample_user_id UUID;
BEGIN
    -- Insert sample admin user if not exists
    INSERT INTO users (firebase_uid, email, display_name, name, package_tier, is_active) 
    VALUES ('admin-sample-uid-12345', 'admin@epsx.io', 'Admin User', 'EPSX Admin', 'admin', true)
    ON CONFLICT (firebase_uid) DO NOTHING
    RETURNING id INTO sample_user_id;
    
    -- If we have the user_admin_roles table, grant all admin modules
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'user_admin_roles') THEN
        INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, is_active)
        SELECT 'admin-sample-uid-12345', module_code, 'system', true
        FROM admin_modules
        ON CONFLICT (firebase_uid, module_code) DO NOTHING;
    END IF;
    
    -- If we have the user_permissions table (new structured permissions), grant admin permissions
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'user_permissions') THEN
        INSERT INTO user_permissions (user_id, permission, granted_by, is_active)
        SELECT 
            (SELECT id FROM users WHERE firebase_uid = 'admin-sample-uid-12345'),
            'admin:*:*',
            (SELECT id FROM users WHERE firebase_uid = 'admin-sample-uid-12345'),
            true
        WHERE EXISTS (SELECT id FROM users WHERE firebase_uid = 'admin-sample-uid-12345')
        ON CONFLICT (user_id, permission) DO NOTHING;
    END IF;
    
    RAISE NOTICE 'Sample admin user created/updated: admin@epsx.io';
END $$;

-- ============================================================================
-- NOTIFICATION PREFERENCES FOR EXISTING USERS
-- ============================================================================

-- Ensure all active users have default notification preferences
-- This is safe to run multiple times
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'notification_preferences') THEN
        INSERT INTO notification_preferences (user_id, email_enabled, push_enabled, feature_expiration, security_alerts, account_updates, marketing) 
        SELECT id, true, true, true, true, true, false 
        FROM users 
        WHERE is_active = true
        ON CONFLICT (user_id) DO NOTHING;
        
        RAISE NOTICE 'Default notification preferences set for all active users';
    END IF;
END $$;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
DECLARE
    admin_modules_count INTEGER;
    eps_count INTEGER;
    users_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO admin_modules_count FROM admin_modules;
    SELECT COUNT(*) INTO eps_count FROM eps_growth_analytics;
    SELECT COUNT(*) INTO users_count FROM users;
    
    RAISE NOTICE '============================================================================';
    RAISE NOTICE 'EPSX SEED DATA INITIALIZATION COMPLETED SUCCESSFULLY';
    RAISE NOTICE '============================================================================';
    RAISE NOTICE 'Database seeded with:';
    RAISE NOTICE '✓ % Admin modules', admin_modules_count;
    RAISE NOTICE '✓ % EPS analytics records', eps_count;
    RAISE NOTICE '✓ % Total users', users_count;
    RAISE NOTICE '✓ Sample admin user: admin@epsx.io (Firebase UID: admin-sample-uid-12345)';
    RAISE NOTICE '✓ Default notification preferences for all users';
    RAISE NOTICE '============================================================================';
    RAISE NOTICE 'Database is ready for development and testing!';
    RAISE NOTICE 'NOTE: Replace sample admin Firebase UID with real Firebase UID in production';
    RAISE NOTICE '============================================================================';
END $$;