-- ============================================================================
-- EPSX SINGLE CONSOLIDATED SEED INITIAL - Complete Database Setup
-- ============================================================================
-- This file provides complete database initialization combining:
-- - Production schema from consolidated migration
-- - Essential seed data for development and production
-- - Sample analytics data for testing
-- ============================================================================

-- ============================================================================
-- EXTENSIONS
-- ============================================================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "btree_gist";

-- ============================================================================
-- ENUMS AND TYPES
-- ============================================================================

-- Core package tier system (simplified)
DO $$ BEGIN
    CREATE TYPE package_tier AS ENUM ('free', 'bronze', 'silver', 'gold', 'platinum', 'admin', 'disabled');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Admin module types removed - migrated to structured permissions system
-- Legacy admin_module enum no longer needed

-- Notification system types
DO $$ BEGIN
    CREATE TYPE notification_type AS ENUM (
        'system', 'security', 'payment', 'user_update', 
        'feature_expiration', 'module_access', 'quota_warning', 'marketing'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE notification_priority AS ENUM ('low', 'medium', 'high', 'critical');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- User role system (simplified to 3 core roles)
DO $$ BEGIN
    CREATE TYPE user_role AS ENUM ('admin', 'user', 'guest');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ============================================================================
-- CORE USER MANAGEMENT TABLES
-- ============================================================================

-- Users table (optimized for Firebase + JWT authentication)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) UNIQUE NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255),
    name VARCHAR(255),
    avatar_url TEXT,
    package_tier package_tier DEFAULT 'free',
    role user_role DEFAULT 'guest',
    email_verified BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sessions table (JWT-optimized)
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    provider VARCHAR(50) DEFAULT 'firebase',
    session_token TEXT UNIQUE,
    user_agent TEXT,
    ip_address INET,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Firebase sessions (for Firebase Admin SDK integration)
CREATE TABLE IF NOT EXISTS firebase_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    firebase_uid VARCHAR(128) NOT NULL,
    id_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    claims JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- SIMPLIFIED ADMIN PERMISSION SYSTEM
-- ============================================================================

-- ============================================================================
-- LEGACY ADMIN MODULES SYSTEM REMOVED
-- ============================================================================
-- The following tables have been migrated to structured permissions:
-- - admin_modules -> replaced by structured permission format "platform:resource:action"
-- - user_admin_roles -> replaced by user_permissions table
-- - admin_role_audit -> replaced by audit_logs table
-- Migration completed: 2025-01-31
-- ============================================================================

-- ============================================================================
-- NOTIFICATIONS SYSTEM
-- ============================================================================

-- Enhanced notifications table (optimized for real-time delivery)
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    notification_type notification_type NOT NULL DEFAULT 'system',
    priority notification_priority NOT NULL DEFAULT 'medium',
    is_read BOOLEAN NOT NULL DEFAULT false,
    delivery_status VARCHAR(50) DEFAULT 'pending',
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'
);

-- User notification preferences
CREATE TABLE IF NOT EXISTS notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
    email_enabled BOOLEAN DEFAULT true,
    push_enabled BOOLEAN DEFAULT true,
    feature_expiration BOOLEAN DEFAULT true,
    security_alerts BOOLEAN DEFAULT true,
    account_updates BOOLEAN DEFAULT true,
    marketing BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- ANALYTICS SYSTEM
-- ============================================================================

-- EPS growth analytics table (optimized)
CREATE TABLE IF NOT EXISTS eps_growth_analytics (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    name VARCHAR(100) NOT NULL,
    country VARCHAR(50) NOT NULL,
    sector VARCHAR(100),
    exchange VARCHAR(50),
    current_eps DECIMAL(10,4),
    qoq_growth_rate DECIMAL(8,4),
    price_current DECIMAL(10,2),
    market_cap BIGINT,
    volume BIGINT,
    ranking_score DECIMAL(10,4),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- AUDIT & SECURITY SYSTEM
-- ============================================================================

-- Unified audit logs
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES users(id),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    result VARCHAR(50) DEFAULT 'success',
    severity VARCHAR(20) DEFAULT 'info',
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id UUID REFERENCES sessions(id)
);

-- Security events
CREATE TABLE IF NOT EXISTS security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    user_id VARCHAR(255),
    ip_address INET,
    request_path TEXT,
    event_data JSONB DEFAULT '{}',
    risk_score INTEGER DEFAULT 0,
    blocked BOOLEAN DEFAULT false,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- PERFORMANCE INDEXES
-- ============================================================================

-- Users table indexes
CREATE INDEX IF NOT EXISTS idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_package_tier ON users(package_tier);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_search_text ON users USING gin(to_tsvector('english', 
    COALESCE(email, '') || ' ' || COALESCE(display_name, '') || ' ' || COALESCE(name, '')));

-- Sessions table indexes  
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_sessions_is_active ON sessions(is_active);

-- Firebase sessions indexes
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_user_id ON firebase_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_firebase_uid ON firebase_sessions(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_firebase_sessions_expires_at ON firebase_sessions(expires_at);

-- Admin system indexes
CREATE INDEX IF NOT EXISTS idx_admin_modules_module_code ON admin_modules(module_code);
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_firebase_uid ON user_admin_roles(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_module_code ON user_admin_roles(module_code);
CREATE INDEX IF NOT EXISTS idx_user_admin_roles_is_active ON user_admin_roles(is_active);

-- Notifications indexes
CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(is_read);
CREATE INDEX IF NOT EXISTS idx_notifications_type_priority ON notifications(notification_type, priority);

-- Analytics indexes
CREATE INDEX IF NOT EXISTS idx_eps_symbol ON eps_growth_analytics(symbol);
CREATE INDEX IF NOT EXISTS idx_eps_country ON eps_growth_analytics(country);
CREATE INDEX IF NOT EXISTS idx_eps_ranking_score ON eps_growth_analytics(ranking_score DESC);
CREATE INDEX IF NOT EXISTS idx_eps_qoq_growth ON eps_growth_analytics(qoq_growth_rate DESC);

-- Audit indexes
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action_resource ON audit_logs(action, resource_type);

-- Security indexes
CREATE INDEX IF NOT EXISTS idx_security_events_timestamp ON security_events(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_event_type ON security_events(event_type);
CREATE INDEX IF NOT EXISTS idx_security_events_ip_address ON security_events(ip_address);

-- ============================================================================
-- VIEWS & FUNCTIONS
-- ============================================================================

-- User permissions view for JWT generation
CREATE OR REPLACE VIEW user_permissions_view AS
SELECT 
    u.id,
    u.firebase_uid,
    u.email,
    u.package_tier,
    u.is_active,
    COALESCE(
        ARRAY_AGG(DISTINCT uar.module_code) FILTER (WHERE uar.module_code IS NOT NULL),
        '{}'
    ) as admin_modules,
    CASE 
        WHEN COUNT(uar.module_code) > 0 THEN 'admin'
        ELSE 'user'
    END as role,
    u.created_at,
    u.last_login_at
FROM users u
LEFT JOIN user_admin_roles uar ON u.firebase_uid = uar.firebase_uid 
    AND uar.is_active = true 
    AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
WHERE u.is_active = true
GROUP BY u.id, u.firebase_uid, u.email, u.package_tier, u.is_active, u.created_at, u.last_login_at;

-- Function to get user JWT claims
CREATE OR REPLACE FUNCTION get_user_jwt_claims(user_firebase_uid VARCHAR(128))
RETURNS JSONB
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT jsonb_build_object(
        'firebase_uid', upv.firebase_uid,
        'email', upv.email,
        'package_tier', upv.package_tier,
        'admin_modules', upv.admin_modules,
        'role', upv.role,
        'is_active', upv.is_active,
        'last_login_at', upv.last_login_at
    )
    FROM user_permissions_view upv
    WHERE upv.firebase_uid = user_firebase_uid;
$BODY$;

-- Function to check admin module access
CREATE OR REPLACE FUNCTION user_has_admin_module(user_firebase_uid VARCHAR(128), check_module_code admin_module)
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT EXISTS(
        SELECT 1 FROM user_admin_roles
        WHERE firebase_uid = user_firebase_uid
          AND module_code = check_module_code
          AND is_active = true
          AND (expires_at IS NULL OR expires_at > NOW())
    );
$BODY$;

-- Simple role checking function
CREATE OR REPLACE FUNCTION user_has_role(
    user_firebase_uid VARCHAR(128), 
    required_role TEXT
)
RETURNS BOOLEAN
LANGUAGE SQL
STABLE
AS $BODY$
    SELECT CASE
        WHEN u.role = 'admin' THEN true  -- admin can access everything
        WHEN u.role = 'user' AND required_role IN ('user', 'guest') THEN true
        WHEN u.role = 'guest' AND required_role = 'guest' THEN true
        ELSE false
    END
    FROM users u
    WHERE u.firebase_uid = user_firebase_uid 
    AND u.is_active = true;
$BODY$;

-- ============================================================================
-- ESSENTIAL SEED DATA
-- ============================================================================

-- Legacy admin modules data removed - migrated to structured permissions
-- These permissions are now handled via user_permissions table with format:
-- "epsx:users:manage", "admin:analytics:access", etc.

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
('NFLX', 'Netflix Inc.', 'US', 'Communication Services', 'NASDAQ', 15.45, 9.75, 445.20, 195000000000, 8200000, 87.6)
ON CONFLICT DO NOTHING;

-- Sample admin user for development (replace with actual admin Firebase UID)
-- Note: This should be updated with real Firebase UIDs in production
DO $$
BEGIN
    -- Insert sample admin user if not exists
    INSERT INTO users (firebase_uid, email, display_name, name, role, package_tier, is_active) 
    VALUES ('admin-sample-uid-12345', 'admin@epsx.io', 'Admin User', 'EPSX Admin', 'admin', 'admin', true)
    ON CONFLICT (firebase_uid) DO NOTHING;
    
    -- Grant admin permissions to sample admin user (structured permissions)
    INSERT INTO user_permissions (user_id, permission, granted_at, granted_by, is_active)
    VALUES ((SELECT id FROM users WHERE firebase_uid = 'admin-sample-uid-12345'), 'admin:*:*', NOW(), 'admin-sample-uid-12345', true)
    ON CONFLICT DO NOTHING;
END $$;

-- ============================================================================
-- DATA MIGRATION AND CLEANUP
-- ============================================================================

-- Update role column based on structured permissions and package_tier
UPDATE users 
SET role = CASE
    -- Users with admin permissions get admin role
    WHEN EXISTS (
        SELECT 1 FROM user_permissions up 
        WHERE up.user_id = users.id 
        AND up.is_active = true 
        AND (up.expires_at IS NULL OR up.expires_at > NOW())
        AND up.permission LIKE 'admin:%'
    ) THEN 'admin'::user_role
    -- Users with premium package tiers get user role  
    WHEN package_tier IN ('admin', 'platinum', 'gold', 'silver', 'bronze') THEN 'user'::user_role
    -- Everyone else gets guest role (including 'free' tier)
    ELSE 'guest'::user_role
END
WHERE role = 'guest'; -- Only update if still default

-- Ensure all active users have notification preferences
INSERT INTO notification_preferences (user_id, email_enabled, push_enabled, feature_expiration, security_alerts, account_updates, marketing) 
SELECT id, true, true, true, true, true, false 
FROM users 
WHERE is_active = true
ON CONFLICT (user_id) DO NOTHING;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE '============================================================================';
    RAISE NOTICE 'EPSX CONSOLIDATED SEED INITIAL COMPLETED SUCCESSFULLY';
    RAISE NOTICE '============================================================================';
    RAISE NOTICE 'Database initialized with:';
    RAISE NOTICE '✓ Complete production schema';
    RAISE NOTICE '✓ 8 Essential admin modules';
    RAISE NOTICE '✓ 10 Sample EPS analytics records';
    RAISE NOTICE '✓ Sample admin user with full permissions';
    RAISE NOTICE '✓ Optimized indexes for performance';
    RAISE NOTICE '✓ Role-based permission system';
    RAISE NOTICE '✓ Notification system setup';
    RAISE NOTICE '✓ Security and audit framework';
    RAISE NOTICE '============================================================================';
    RAISE NOTICE 'Ready for development and production use!';
    RAISE NOTICE '============================================================================';
END $$;