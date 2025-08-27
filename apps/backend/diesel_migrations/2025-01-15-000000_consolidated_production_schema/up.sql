-- ============================================================================
-- EPSX PRODUCTION DATABASE SCHEMA - Consolidated & Optimized
-- ============================================================================
-- Modern, streamlined production database schema for the EPSX platform
-- Removes legacy permission complexity, optimizes for JWT-based authentication
-- Version: Production Consolidated 2025
-- Created: 2025-01-15

-- ============================================================================
-- EXTENSIONS
-- ============================================================================

-- UUID generation support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- ENUMS AND TYPES
-- ============================================================================

-- Core package tier system (simplified)
CREATE TYPE package_tier AS ENUM ('free', 'bronze', 'silver', 'gold', 'platinum', 'admin', 'disabled');

-- Admin module types (streamlined to 8 core modules)
CREATE TYPE admin_module AS ENUM (
    'user-management',     -- User CRUD, profile management
    'analytics-access',    -- Analytics dashboards and reports  
    'billing-admin',       -- Payment and subscription management
    'system-admin',        -- System configuration and monitoring
    'content-management',  -- Content and resource management
    'support-access',      -- User support and troubleshooting
    'security-management', -- Security monitoring and compliance
    'api-management'       -- API keys and developer tools
);

-- Notification system types
CREATE TYPE notification_type AS ENUM (
    'system', 'security', 'payment', 'user_update', 
    'feature_expiration', 'module_access', 'quota_warning', 'marketing'
);

CREATE TYPE notification_priority AS ENUM ('low', 'medium', 'high', 'critical');

-- User role system (simplified to 3 core roles)
CREATE TYPE user_role AS ENUM ('admin', 'user', 'guest');

-- ============================================================================
-- CORE USER MANAGEMENT TABLES
-- ============================================================================

-- Users table (optimized for Firebase + JWT authentication)
CREATE TABLE users (
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
CREATE TABLE sessions (
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
CREATE TABLE firebase_sessions (
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

-- Admin modules definition
CREATE TABLE admin_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_code admin_module UNIQUE NOT NULL,
    module_name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    icon VARCHAR(50),
    color VARCHAR(20),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- User admin role assignments (simplified)
CREATE TABLE user_admin_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL,
    module_code admin_module NOT NULL,
    granted_by VARCHAR(128),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_firebase_uid_module UNIQUE (firebase_uid, module_code)
);

-- Admin role assignment audit
CREATE TABLE admin_role_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) NOT NULL,
    module_code admin_module NOT NULL,
    action VARCHAR(50) NOT NULL, -- 'granted', 'revoked', 'expired'
    performed_by VARCHAR(128),
    reason TEXT,
    metadata JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- NOTIFICATIONS SYSTEM
-- ============================================================================

-- Enhanced notifications table (optimized for real-time delivery)
CREATE TABLE notifications (
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
CREATE TABLE notification_preferences (
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
CREATE TABLE eps_growth_analytics (
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
-- AUDIT & SECURITY SYSTEM (Simplified)
-- ============================================================================

-- Unified audit logs (replaces multiple audit tables)
CREATE TABLE audit_logs (
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

-- Security events (simplified, essential only)
CREATE TABLE security_events (
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
-- PERFORMANCE INDEXES (Optimized - Reduced from 50+ to ~25)
-- ============================================================================

-- Users table indexes
CREATE INDEX idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_package_tier ON users(package_tier);
CREATE INDEX idx_users_is_active ON users(is_active);
CREATE INDEX idx_users_search_text ON users USING gin(to_tsvector('english', 
    COALESCE(email, '') || ' ' || COALESCE(display_name, '') || ' ' || COALESCE(name, '')));

-- Sessions table indexes  
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_is_active ON sessions(is_active);

-- Firebase sessions indexes
CREATE INDEX idx_firebase_sessions_user_id ON firebase_sessions(user_id);
CREATE INDEX idx_firebase_sessions_firebase_uid ON firebase_sessions(firebase_uid);
CREATE INDEX idx_firebase_sessions_expires_at ON firebase_sessions(expires_at);

-- Admin system indexes
CREATE INDEX idx_admin_modules_module_code ON admin_modules(module_code);
CREATE INDEX idx_user_admin_roles_firebase_uid ON user_admin_roles(firebase_uid);
CREATE INDEX idx_user_admin_roles_module_code ON user_admin_roles(module_code);
CREATE INDEX idx_user_admin_roles_is_active ON user_admin_roles(is_active);

-- Notifications indexes
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_is_read ON notifications(is_read);
CREATE INDEX idx_notifications_type_priority ON notifications(notification_type, priority);

-- Analytics indexes
CREATE INDEX idx_eps_symbol ON eps_growth_analytics(symbol);
CREATE INDEX idx_eps_country ON eps_growth_analytics(country);
CREATE INDEX idx_eps_ranking_score ON eps_growth_analytics(ranking_score DESC);
CREATE INDEX idx_eps_qoq_growth ON eps_growth_analytics(qoq_growth_rate DESC);

-- Audit indexes
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_action_resource ON audit_logs(action, resource_type);

-- Security indexes
CREATE INDEX idx_security_events_timestamp ON security_events(timestamp DESC);
CREATE INDEX idx_security_events_event_type ON security_events(event_type);
CREATE INDEX idx_security_events_ip_address ON security_events(ip_address);

-- ============================================================================
-- OPTIMIZED VIEWS & FUNCTIONS
-- ============================================================================

-- User permissions view for JWT generation (simplified)
CREATE VIEW user_permissions_view AS
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

-- Function to get user JWT claims (simplified)
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

-- ============================================================================
-- PRODUCTION SEED DATA
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
ON CONFLICT (module_code) DO NOTHING;

-- Insert sample EPS analytics data for development
INSERT INTO eps_growth_analytics (symbol, name, country, sector, exchange, current_eps, qoq_growth_rate, price_current, market_cap, volume, ranking_score) VALUES
-- US Tech Leaders
('AAPL', 'Apple Inc.', 'US', 'Technology', 'NASDAQ', 6.15, 8.50, 175.25, 2800000000000, 45000000, 95.8),
('MSFT', 'Microsoft Corporation', 'US', 'Technology', 'NASDAQ', 9.65, 12.30, 335.50, 2500000000000, 28000000, 94.2),
('GOOGL', 'Alphabet Inc.', 'US', 'Technology', 'NASDAQ', 5.80, 15.75, 138.75, 1750000000000, 22000000, 92.5),
('NVDA', 'NVIDIA Corporation', 'US', 'Technology', 'NASDAQ', 12.35, 22.40, 485.75, 1200000000000, 35000000, 97.2),
-- International Growth Leaders  
('ASML', 'ASML Holding N.V.', 'NL', 'Technology', 'AMS', 15.25, 18.90, 675.30, 280000000000, 1200000, 91.4),
('TSM', 'Taiwan Semiconductor', 'TW', 'Technology', 'NYSE', 8.90, 14.20, 98.45, 520000000000, 18000000, 89.7),
('SHOP', 'Shopify Inc.', 'CA', 'Technology', 'NYSE', 2.45, 35.60, 58.90, 75000000000, 8500000, 88.3)
ON CONFLICT DO NOTHING;

-- Insert default notification preferences template
-- This will be used as default when users sign up
INSERT INTO notification_preferences (user_id, email_enabled, push_enabled, feature_expiration, security_alerts, account_updates, marketing) 
SELECT id, true, true, true, true, true, false 
FROM users 
ON CONFLICT (user_id) DO NOTHING;

-- ============================================================================
-- DATA MIGRATION: Populate role column from existing data
-- ============================================================================

-- Update role column based on existing admin_modules and package_tier
UPDATE users 
SET role = CASE
    -- Users with admin modules get admin role
    WHEN EXISTS (
        SELECT 1 FROM user_admin_roles uar 
        WHERE uar.firebase_uid = users.firebase_uid 
        AND uar.is_active = true 
        AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
    ) THEN 'admin'::user_role
    -- Users with admin or premium package tiers get user role  
    WHEN package_tier IN ('admin', 'platinum', 'gold', 'silver', 'bronze') THEN 'user'::user_role
    -- Everyone else gets guest role (including 'free' tier)
    ELSE 'guest'::user_role
END
WHERE role = 'guest'; -- Only update if still default