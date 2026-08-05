-- ================================================================================================
-- EPSX CONSOLIDATED SCHEMA v5
-- ================================================================================================
-- Version: 5.0.0 (Consolidated February 2026)
-- Description: Complete production-ready schema for EPSX Web3-first platform.
--              Consolidates all migrations into a single clean baseline.
--
-- Core Components:
--   - Web3-first wallet authentication (SIWE compliant)
--   - Plans & permissions system (plans, plan_permissions)
--   - Event sourcing infrastructure (CQRS)
--   - Developer portal with API key management
--   - System settings
--
-- Changes from v4:
--   - Removed sessions table (bearer-only auth)
--   - Renamed groups → plans, group_* → plan_*
--   - Added tier_level, is_public to plans
--   - Added api_key_permissions table
--   - Granular admin permissions (no admin:*:* wildcard)
--   - No Free Plan in seed data
--
-- Database Requirements: PostgreSQL 14+
-- Extensions: uuid-ossp, pg_trgm, btree_gist
-- ================================================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- ================================================================================================
-- EXTENSIONS
-- ================================================================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;
CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;
CREATE EXTENSION IF NOT EXISTS btree_gist WITH SCHEMA public;

-- ================================================================================================
-- CORE TABLES
-- ================================================================================================

-- WALLET USERS - Primary user accounts
CREATE TABLE wallet_users (
    wallet_address VARCHAR(42) PRIMARY KEY,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    tier_level VARCHAR(20) DEFAULT 'Bronze' NOT NULL,
    wallet_metadata JSONB DEFAULT '{}' NOT NULL,
    permission_plans JSONB DEFAULT '[]',
    disable_info JSONB DEFAULT NULL,
    plan_expires_at TIMESTAMPTZ DEFAULT NULL,
    current_plan_id INTEGER DEFAULT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    last_auth_at TIMESTAMPTZ,
    
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    ),
    CONSTRAINT valid_tier_level CHECK (
        tier_level IN ('Bronze', 'Silver', 'Gold', 'Platinum', 'Diamond')
    )
);

CREATE INDEX idx_wallet_users_active ON wallet_users(is_active, wallet_address);
CREATE INDEX idx_wallet_users_tier ON wallet_users(tier_level, is_active);
CREATE INDEX idx_wallet_users_created ON wallet_users(created_at DESC);
CREATE INDEX idx_wallet_users_updated ON wallet_users(updated_at DESC);
CREATE INDEX idx_wallet_users_last_auth ON wallet_users(last_auth_at DESC) WHERE last_auth_at IS NOT NULL;
CREATE INDEX idx_wallet_users_metadata_gin ON wallet_users USING gin(wallet_metadata);
CREATE INDEX idx_wallet_users_plan_expires_at ON wallet_users(plan_expires_at) WHERE plan_expires_at IS NOT NULL;

-- PERMISSIONS - Permission definitions (no wallet assignments here)
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    permission_string VARCHAR(255) UNIQUE NOT NULL,
    platform VARCHAR(100) NOT NULL,
    resource VARCHAR(100) NOT NULL,
    action VARCHAR(100) NOT NULL,
    description TEXT,
    permission_type VARCHAR(50) DEFAULT 'manual' NOT NULL,
    name VARCHAR(100),
    category VARCHAR(50),
    is_system BOOLEAN DEFAULT FALSE NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_by VARCHAR(42),
    
    CONSTRAINT permissions_type_check CHECK (
        permission_type IN ('manual', 'nft_gated', 'token_gated', 'dao_governance', 'system')
    ),
    CONSTRAINT permissions_platform_not_empty CHECK (LENGTH(TRIM(platform)) > 0),
    CONSTRAINT permissions_resource_not_empty CHECK (LENGTH(TRIM(resource)) > 0),
    CONSTRAINT permissions_action_not_empty CHECK (LENGTH(TRIM(action)) > 0)
);

CREATE INDEX idx_permissions_lookup ON permissions(permission_string) WHERE is_active = TRUE;
CREATE INDEX idx_permissions_platform ON permissions(platform, resource, action);
CREATE INDEX idx_permissions_type ON permissions(permission_type, is_active);
CREATE INDEX idx_permissions_audit ON permissions(created_at, updated_at);
CREATE INDEX idx_permissions_active ON permissions(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_permissions_string_pattern ON permissions(permission_string varchar_pattern_ops);

-- PLANS - Subscription plans (formerly groups)
CREATE TABLE plans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    description TEXT DEFAULT '' NOT NULL,
    plan_type VARCHAR(20) DEFAULT 'manual' NOT NULL,
    plan_metadata JSONB DEFAULT '{}' NOT NULL,
    price NUMERIC(10,2) DEFAULT 0.00,
    currency VARCHAR(3) DEFAULT 'USD',
    billing_cycle VARCHAR(20) DEFAULT 'pay_per_use',
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    is_promoted BOOLEAN DEFAULT FALSE NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT TRUE,
    display_order INTEGER DEFAULT 0,
    max_members INTEGER,
    auto_assign_enabled BOOLEAN DEFAULT FALSE,
    assignment_rules JSONB DEFAULT '{}',
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    rate_limit_per_hour INTEGER NOT NULL DEFAULT 1000,
    rate_limit_per_day INTEGER NOT NULL DEFAULT 10000,
    burst_capacity INTEGER NOT NULL DEFAULT 10,
    tier_level INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_by VARCHAR(42),
    last_modified_by VARCHAR(42),
    
    CONSTRAINT valid_plan_type CHECK (
        plan_type IN ('manual', 'subscription', 'web3_asset', 'dao_membership', 'admin')
    ),
    CONSTRAINT valid_currency CHECK (
        currency IN ('USD', 'EUR', 'BTC', 'ETH', 'BNB')
    ),
    CONSTRAINT valid_billing_cycle CHECK (
        billing_cycle IN ('monthly', 'yearly', 'one_time', 'lifetime', 'pay_per_use')
    )
);

CREATE INDEX idx_plans_active ON plans(is_active, display_order);
CREATE INDEX idx_plans_type ON plans(plan_type, is_active);
CREATE INDEX idx_plans_slug ON plans(slug);
CREATE INDEX idx_plans_price ON plans(price, currency) WHERE plan_type = 'subscription';
CREATE INDEX idx_plans_created ON plans(created_at DESC);
CREATE INDEX idx_plans_metadata_gin ON plans USING gin(plan_metadata);

-- PLAN_PERMISSIONS - Plan to Permission mapping (formerly group_permissions)
CREATE TABLE plan_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plan_id UUID NOT NULL,
    permission_id UUID NOT NULL,
    granted_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    granted_by VARCHAR(42),
    grant_reason TEXT,
    UNIQUE(plan_id, permission_id)
);

CREATE INDEX idx_plan_permissions_plan ON plan_permissions(plan_id);
CREATE INDEX idx_plan_permissions_permission ON plan_permissions(permission_id);
CREATE INDEX idx_plan_permissions_audit ON plan_permissions(granted_at);

-- WALLET_PLAN_ASSIGNMENTS - Wallet to Plan mapping (formerly wallet_group_assignments)
CREATE TABLE wallet_plan_assignments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    plan_id UUID NOT NULL,
    assigned_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ,
    assigned_by VARCHAR(42),
    assignment_reason TEXT,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    assignment_source VARCHAR(50) DEFAULT 'manual' NOT NULL,
    payment_reference VARCHAR(255),
    subscription_id VARCHAR(255),
    auto_renew BOOLEAN DEFAULT FALSE NOT NULL,
    next_billing_date TIMESTAMPTZ,
    assignment_metadata JSONB DEFAULT '{}' NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE(wallet_address, plan_id)
);

CREATE INDEX idx_wpa_wallet ON wallet_plan_assignments(wallet_address, is_active);
CREATE INDEX idx_wpa_plan ON wallet_plan_assignments(plan_id, is_active);
CREATE INDEX idx_wpa_expires ON wallet_plan_assignments(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_wpa_audit ON wallet_plan_assignments(assigned_at);
CREATE INDEX idx_wpa_active_lookup ON wallet_plan_assignments(wallet_address, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_wpa_expires_lookup ON wallet_plan_assignments(wallet_address, expires_at) WHERE is_active = TRUE AND expires_at IS NOT NULL;

-- WALLET_DIRECT_PERMISSIONS - Direct permission grants
CREATE TABLE wallet_direct_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    permission_id UUID NOT NULL,
    granted_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ,
    granted_by VARCHAR(42),
    grant_reason TEXT,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    UNIQUE(wallet_address, permission_id)
);

CREATE INDEX idx_direct_perms_wallet ON wallet_direct_permissions(wallet_address, is_active);
CREATE INDEX idx_direct_perms_permission ON wallet_direct_permissions(permission_id);
CREATE INDEX idx_direct_perms_expires ON wallet_direct_permissions(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_direct_perms_audit ON wallet_direct_permissions(granted_at);
CREATE INDEX idx_wdp_active_lookup ON wallet_direct_permissions(wallet_address, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_wdp_expires_lookup ON wallet_direct_permissions(wallet_address, expires_at) WHERE is_active = TRUE AND expires_at IS NOT NULL;

-- ================================================================================================
-- AUTHENTICATION TABLES
-- ================================================================================================

-- WEB3_AUTH_NONCES - SIWE authentication challenges
CREATE TABLE web3_auth_nonces (
    wallet_address VARCHAR(42) PRIMARY KEY,
    nonce VARCHAR(64) NOT NULL,
    message TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    
    CONSTRAINT valid_nonce_wallet_address CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    )
);

CREATE INDEX idx_web3_auth_nonces_expires_at ON web3_auth_nonces(expires_at);

-- OPENID_REFRESH_TOKENS - Token management
CREATE TABLE openid_refresh_tokens (
    token_id VARCHAR(36) PRIMARY KEY,
    wallet_address VARCHAR(42) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    is_revoked BOOLEAN DEFAULT FALSE NOT NULL,
    
    CONSTRAINT valid_token_wallet_address CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    )
);

CREATE INDEX idx_openid_refresh_tokens_wallet_address ON openid_refresh_tokens(wallet_address);
CREATE INDEX idx_openid_refresh_tokens_expires_at ON openid_refresh_tokens(expires_at);
CREATE INDEX idx_openid_refresh_tokens_active ON openid_refresh_tokens(wallet_address, is_revoked, expires_at) WHERE is_revoked = FALSE;

-- ================================================================================================
-- ROUTE PERMISSIONS TABLE
-- ================================================================================================

CREATE TABLE route_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    route_pattern VARCHAR(500) NOT NULL,
    http_method VARCHAR(10) DEFAULT '*' NOT NULL,
    required_permission VARCHAR(255) NOT NULL,
    priority INTEGER DEFAULT 100 NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    is_public BOOLEAN DEFAULT FALSE NOT NULL,
    description TEXT,
    route_category VARCHAR(100) DEFAULT 'api',
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_by VARCHAR(255),
    last_modified_by VARCHAR(255),
    
    CONSTRAINT route_permissions_method_check CHECK (
        http_method IN ('*', 'GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS', 'HEAD')
    ),
    CONSTRAINT route_permissions_pattern_not_empty CHECK (LENGTH(TRIM(route_pattern)) > 0),
    CONSTRAINT route_permissions_permission_not_empty CHECK (LENGTH(TRIM(required_permission)) > 0),
    CONSTRAINT route_permissions_priority_check CHECK (priority >= 0 AND priority <= 9999)
);

CREATE INDEX idx_route_permissions_lookup ON route_permissions(is_active, priority DESC, route_pattern, http_method) WHERE is_active = TRUE;
CREATE INDEX idx_route_permissions_method ON route_permissions(http_method, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_route_permissions_permission ON route_permissions(required_permission, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_route_permissions_category ON route_permissions(route_category, is_active, priority DESC);
CREATE INDEX idx_route_permissions_patterns ON route_permissions(route_pattern) WHERE is_active = TRUE;
CREATE INDEX idx_route_permissions_audit ON route_permissions(created_at, updated_at);
CREATE UNIQUE INDEX idx_route_permissions_unique_route ON route_permissions(route_pattern, http_method) WHERE is_active = TRUE;

-- ================================================================================================
-- AUDIT & READ MODEL TABLES
-- ================================================================================================

-- Read model schema for projections
CREATE SCHEMA IF NOT EXISTS read_model;

CREATE TABLE read_model.projection_checkpoints (
    projection_name VARCHAR(255) PRIMARY KEY,
    last_processed_event_id UUID,
    last_processed_sequence BIGINT NOT NULL DEFAULT 0,
    events_processed_count BIGINT NOT NULL DEFAULT 0,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_healthy BOOLEAN NOT NULL DEFAULT true,
    
    CONSTRAINT projection_checkpoints_sequence_positive CHECK (last_processed_sequence >= 0),
    CONSTRAINT projection_checkpoints_count_positive CHECK (events_processed_count >= 0)
);

-- ================================================================================================
-- SYSTEM SETTINGS
-- ================================================================================================

CREATE TABLE system_settings (
    id SERIAL PRIMARY KEY,
    category VARCHAR(50) NOT NULL,
    key VARCHAR(100) NOT NULL,
    value JSONB NOT NULL DEFAULT '{}',
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(category, key)
);

CREATE INDEX idx_system_settings_category ON system_settings(category);

-- ================================================================================================
-- DEVELOPER PORTAL (API KEYS)
-- ================================================================================================

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash VARCHAR(256) NOT NULL UNIQUE,
    key_prefix VARCHAR(16) NOT NULL,
    full_key VARCHAR(128),
    client_name VARCHAR(255) NOT NULL,
    client_description TEXT,
    client_contact_email VARCHAR(255),
    wallet_address VARCHAR(42) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    total_requests BIGINT NOT NULL DEFAULT 0,
    ip_restrictions TEXT[],
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    rate_limit_per_day INTEGER NOT NULL DEFAULT 10000,
    selected_permissions TEXT[] NOT NULL DEFAULT '{}',
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    revoked_by VARCHAR(42),
    revocation_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(42) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_wallet ON api_keys(wallet_address);
CREATE INDEX idx_api_keys_status ON api_keys(status);
CREATE INDEX idx_api_keys_key_prefix ON api_keys(key_prefix);
CREATE INDEX idx_api_keys_selected_permissions ON api_keys USING GIN (selected_permissions);

CREATE TABLE api_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    base_path VARCHAR(255) NOT NULL,
    default_rate_limit INTEGER NOT NULL DEFAULT 60,
    access_levels JSONB NOT NULL DEFAULT '{}',
    endpoints JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_modules_status ON api_modules(status);
CREATE INDEX idx_api_modules_category ON api_modules(category);

CREATE TABLE api_key_module_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL,
    module_id UUID NOT NULL,
    access_level VARCHAR(20) NOT NULL DEFAULT 'bronze',
    custom_rate_limit INTEGER,
    custom_quotas JSONB NOT NULL DEFAULT '{}',
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by VARCHAR(42),
    UNIQUE(api_key_id, module_id)
);

CREATE INDEX idx_api_key_module_access_key ON api_key_module_access(api_key_id);
CREATE INDEX idx_api_key_module_access_module ON api_key_module_access(module_id);

-- API_KEY_PERMISSIONS - Links API keys to plans for plan-based permissions
CREATE TABLE api_key_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL,
    plan_id UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by VARCHAR(42),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    metadata JSONB DEFAULT '{}' NOT NULL,
    UNIQUE(api_key_id, plan_id)
);

CREATE INDEX idx_api_key_permissions_api_key ON api_key_permissions(api_key_id) WHERE is_active = TRUE;
CREATE INDEX idx_api_key_permissions_plan ON api_key_permissions(plan_id) WHERE is_active = TRUE;
CREATE INDEX idx_api_key_permissions_expires ON api_key_permissions(expires_at) WHERE expires_at IS NOT NULL AND is_active = TRUE;
CREATE INDEX idx_api_key_permissions_combined ON api_key_permissions(api_key_id, plan_id, is_active) WHERE is_active = TRUE;

-- ================================================================================================
-- FOREIGN KEY CONSTRAINTS
-- ================================================================================================

ALTER TABLE plan_permissions ADD CONSTRAINT plan_permissions_plan_id_fkey
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE;

ALTER TABLE plan_permissions ADD CONSTRAINT plan_permissions_permission_id_fkey
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE;

ALTER TABLE wallet_plan_assignments ADD CONSTRAINT wpa_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

ALTER TABLE wallet_plan_assignments ADD CONSTRAINT wpa_plan_id_fkey
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE;

ALTER TABLE wallet_direct_permissions ADD CONSTRAINT wdp_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

ALTER TABLE wallet_direct_permissions ADD CONSTRAINT wdp_permission_id_fkey
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE;

ALTER TABLE openid_refresh_tokens ADD CONSTRAINT fk_openid_refresh_tokens_wallet_address
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

ALTER TABLE api_keys ADD CONSTRAINT api_keys_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address);

ALTER TABLE api_key_module_access ADD CONSTRAINT api_key_module_access_key_fkey
    FOREIGN KEY (api_key_id) REFERENCES api_keys(id) ON DELETE CASCADE;

ALTER TABLE api_key_module_access ADD CONSTRAINT api_key_module_access_module_fkey
    FOREIGN KEY (module_id) REFERENCES api_modules(id) ON DELETE CASCADE;

ALTER TABLE api_key_permissions ADD CONSTRAINT api_key_permissions_api_key_fkey
    FOREIGN KEY (api_key_id) REFERENCES api_keys(id) ON DELETE CASCADE;

ALTER TABLE api_key_permissions ADD CONSTRAINT api_key_permissions_plan_fkey
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE;

-- ================================================================================================
-- STORED FUNCTIONS
-- ================================================================================================

-- Permission lookup function
CREATE OR REPLACE FUNCTION public.get_wallet_permissions_detailed_working(p_wallet_address character varying)
 RETURNS TABLE(permission_string character varying, permission_id uuid, source_type character varying, source_id uuid, source_name character varying, expires_at timestamp with time zone, granted_at timestamp with time zone, is_permanent boolean)
 LANGUAGE plpgsql
 SECURITY DEFINER
AS $function$
BEGIN
    RETURN QUERY
    -- Direct permissions
    SELECT
        p.permission_string,
        p.id,
        'direct'::VARCHAR(20),
        p.id as source_id,
        'Direct Permission' as source_name,
        wdp.expires_at,
        wdp.granted_at,
        (wdp.expires_at IS NULL) as is_permanent
    FROM wallet_direct_permissions wdp
    INNER JOIN permissions p ON wdp.permission_id = p.id
    WHERE LOWER(wdp.wallet_address) = LOWER(p_wallet_address)
      AND wdp.is_active = TRUE
      AND p.is_active = TRUE
      AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())

    UNION ALL

    -- Plan permissions (formerly group permissions)
    SELECT
        p.permission_string,
        p.id,
        'plan'::VARCHAR(20),
        pl.id as source_id,
        pl.name as source_name,
        wpa.expires_at,
        wpa.assigned_at as granted_at,
        (wpa.expires_at IS NULL) as is_permanent
    FROM wallet_plan_assignments wpa
    INNER JOIN plans pl ON wpa.plan_id = pl.id
    INNER JOIN plan_permissions pp ON pl.id = pp.plan_id
    INNER JOIN permissions p ON pp.permission_id = p.id
    WHERE LOWER(wpa.wallet_address) = LOWER(p_wallet_address)
      AND wpa.is_active = TRUE
      AND pl.is_active = TRUE
      AND p.is_active = TRUE
      AND (wpa.expires_at IS NULL OR wpa.expires_at > NOW());
END;
$function$;

-- Permission check function
CREATE OR REPLACE FUNCTION check_wallet_permissions(
    p_wallet_address VARCHAR(42),
    p_required_permission VARCHAR(255)
)
RETURNS BOOLEAN
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    has_permission BOOLEAN := FALSE;
BEGIN
    SELECT EXISTS (
        SELECT 1 FROM get_wallet_permissions_detailed_working(LOWER(p_wallet_address))
        WHERE permission_string = p_required_permission
    ) INTO has_permission;
    
    RETURN has_permission;
END;
$$;

-- ================================================================================================
-- SEED DATA
-- ================================================================================================

-- Default system settings
INSERT INTO system_settings (category, key, value, description) VALUES
    ('general', 'systemName', '"EPSX Admin Console"', 'Display name for the admin console'),
    ('general', 'adminEmail', '"admin@epsx.com"', 'Primary admin contact email'),
    ('general', 'maintenanceMode', 'false', 'Enable/disable maintenance mode'),
    ('notifications', 'emailNotifications', 'true', 'Enable email notifications'),
    ('notifications', 'pushNotifications', 'false', 'Enable push notifications'),
    ('notifications', 'smsNotifications', 'true', 'Enable SMS notifications'),
    ('notifications', 'securityAlerts', 'true', 'Enable security alert notifications'),
    ('security', 'sessionTimeout', '30', 'Session timeout in minutes'),
    ('appearance', 'theme', '"light"', 'Theme mode: light, dark, or auto'),
    ('appearance', 'primaryColor', '"#FF8C00"', 'Primary accent color')
ON CONFLICT (category, key) DO NOTHING;

-- Subscription plans
INSERT INTO plans (name, slug, description, plan_type, plan_metadata, price, currency, billing_cycle, is_active, is_promoted, is_public, display_order, rate_limit_per_minute, rate_limit_per_hour, rate_limit_per_day, burst_capacity, tier_level, created_by) VALUES

-- 1. ONE DAY PLAN (trial)
('One Day Plan', 'one-day', '24-hour trial access to explore the platform', 'subscription',
 '{"features":["Basic analytics view","Rankings from position 6+","Basic trading features","24-hour trial access","Explore the platform"],"ranking_offset":5,"rankings_limit":5,"promotion":{"enabled":true,"type":"percentage","value":80.0,"price":1.0,"start_date":"","end_date":"2026-03-25T14:00:00Z"}}'::jsonb,
 5.00, 'USD', 'one_time', true, false, true, 1, 60, 1000, 10000, 10, 0, '0x0000000000000000000000000000000000000000'),

-- 2. STARTER PLAN (30-day)
('Starter Plan', 'starter', 'Advanced analytics for individual investors and traders', 'subscription',
 '{"features":["Advanced analytics view","25 stock rankings","Basic Analytic features","Price alerts","Email support","30-day access"],"ranking_offset":1,"rankings_limit":25,"promotion":{"enabled":true,"type":"percentage","value":90.0,"price":9.9,"start_date":"","end_date":"2026-03-25T14:00:00Z"}}'::jsonb,
 99.00, 'USD', 'one_time', true, false, true, 2, 120, 3000, 50000, 20, 1, '0x0000000000000000000000000000000000000000'),

-- 3. LIFE TIME (lifetime)
('Life Time', 'lifetime', 'Full platform access with lifetime membership', 'subscription',
 '{"features":["Advanced analytics suite","Full rankings access (Rank 1+)","API read access","Basic & Pro trading","Priority support","Lifetime access"],"ranking_offset":0,"rankings_limit":-1,"promotion":{"enabled":true,"type":"percentage","value":50.0,"price":4999.0,"start_date":"","end_date":"2026-03-25T14:00:00Z"}}'::jsonb,
 9999.00, 'USD', 'lifetime', true, true, true, 3, 300, 10000, 200000, 50, 3, '0x0000000000000000000000000000000000000000'),

-- 4. COMPANY PLAN (365-day)
('Company Plan', 'company', 'Complete solutions for professional teams and institutions', 'subscription',
 '{"features":["Advanced analytics suite","Full trading suite (Basic, Pro & Advanced)","API read & write access","Data export","Notifications management","365-day corporate access","Dedicated support"],"ranking_offset":0,"rankings_limit":-1,"promotion":{"enabled":true,"type":"percentage","value":57.0,"price":2999.0,"start_date":"","end_date":"2026-04-04T05:00:00Z"}}'::jsonb,
 6999.00, 'USD', 'one_time', true, false, true, 4, 1000, 50000, 1000000, 200, 4, '0x0000000000000000000000000000000000000000'),

-- 5. API PERSONAL (30-day)
('API Personal', 'api-personal', 'Integrate our powerful API into your systems', 'subscription',
 '{"features":["Analytics view access","API read access","Data export capability","Full developer documentation","30-day access"],"ranking_offset":1,"rankings_limit":-1,"promotion":{"enabled":true,"type":"percentage","value":75.0,"price":999.0,"start_date":"","end_date":"2026-03-25T14:00:00Z"}}'::jsonb,
 3999.00, 'USD', 'one_time', true, false, true, 5, 300, 10000, 100000, 50, 2, '0x0000000000000000000000000000000000000000'),

-- 6. CUSTOM (revenue share)
('Custom', 'custom', 'Tailored solutions for partners, corporate, and enterprise needs', 'manual',
 '{"features":["Custom feature set & permissions","Dedicated support & SLA","Volume-based pricing","Custom API rate limits","White-label options","Priority onboarding"],"contact_sales":true}'::jsonb,
 0.00, 'USD', 'pay_per_use', true, false, true, 6, 1000, 50000, 1000000, 200, 5, '0x0000000000000000000000000000000000000000')

ON CONFLICT (slug) DO NOTHING;

-- API modules seed data
INSERT INTO api_modules (name, display_name, description, category, base_path, default_rate_limit, access_levels, endpoints) VALUES
    ('stock-ranking', 'Stock Ranking', 'Access stock ranking and performance data', 'analytics', '/api/modules/stock-ranking', 60, 
     '{"bronze": {"requests_per_minute": 10}, "silver": {"requests_per_minute": 30}, "gold": {"requests_per_minute": 100}}'::jsonb,
     '[{"path": "/rankings", "method": "GET", "description": "Get stock rankings"}]'::jsonb),
    ('market-data', 'Market Data', 'Real-time and historical market data', 'data', '/api/modules/market-data', 30,
     '{"bronze": {"requests_per_minute": 5}, "silver": {"requests_per_minute": 20}, "gold": {"requests_per_minute": 60}}'::jsonb,
     '[{"path": "/overview", "method": "GET", "description": "Market overview"}]'::jsonb),
    ('analytics', 'Analytics API', 'Advanced analytics and insights', 'analytics', '/api/modules/analytics', 60,
     '{"bronze": {"requests_per_minute": 10}, "silver": {"requests_per_minute": 30}, "gold": {"requests_per_minute": 100}}'::jsonb,
     '[{"path": "/performance", "method": "GET", "description": "Performance analytics"}]'::jsonb)
ON CONFLICT (name) DO NOTHING;

-- Default route permissions
INSERT INTO route_permissions (route_pattern, http_method, required_permission, priority, is_public) VALUES
    ('/api/auth/login', 'POST', 'auth:login', 10, false),
    ('/api/auth/logout', 'POST', 'auth:logout', 10, false),
    ('/api/auth/refresh', 'POST', 'auth:refresh', 10, false),
    ('/api/users/profile', 'GET', 'users:profile', 10, false),
    ('/api/users/profile', 'PUT', 'users:profile', 10, false),
    ('/api/admin/auth/login', 'POST', 'admin:auth:login', 20, false),
    ('/api/admin/users/list', 'GET', 'admin:users:list', 30, false),
    ('/api/admin/permissions/validate', 'POST', 'admin:permissions:validate', 30, false),
    ('/api/admin/permissions/grant', 'POST', 'admin:permissions:grant', 30, false),
    ('/api/admin/permissions/revoke', 'DELETE', 'admin:permissions:revoke', 30, false)
ON CONFLICT DO NOTHING;

-- Core permissions (including granular admin permissions)
INSERT INTO permissions (permission_string, platform, resource, action, description, permission_type, is_system, is_active) VALUES
    ('epsx:analytics:view', 'epsx', 'analytics', 'view', 'View analytics data', 'manual', true, true),
    ('epsx:analytics:advanced', 'epsx', 'analytics', 'advanced', 'Access advanced analytics', 'manual', true, true),
    ('epsx:trading:basic', 'epsx', 'trading', 'basic', 'Basic trading features', 'manual', true, true),
    ('epsx:trading:pro', 'epsx', 'trading', 'pro', 'Pro trading features', 'manual', true, true),
    ('epsx:trading:advanced', 'epsx', 'trading', 'advanced', 'Advanced trading features', 'manual', true, true),
    ('epsx:api:read', 'epsx', 'api', 'read', 'API read access', 'manual', true, true),
    ('epsx:api:write', 'epsx', 'api', 'write', 'API write access', 'manual', true, true),
    ('epsx:data:export', 'epsx', 'data', 'export', 'Export data', 'manual', true, true),
    ('epsx:notifications:manage', 'epsx', 'notifications', 'manage', 'Manage notifications', 'manual', true, true),
    ('epsx:alerts:create', 'epsx', 'alerts', 'create', 'Create price alerts', 'manual', true, true),
    ('admin:users:view', 'admin', 'users', 'view', 'View user lists and profiles', 'manual', true, true),
    ('admin:users:manage', 'admin', 'users', 'manage', 'Full user management (edit, ban)', 'manual', true, true),
    ('admin:permissions:view', 'admin', 'permissions', 'view', 'View granted permissions and groups', 'manual', true, true),
    ('admin:permissions:manage', 'admin', 'permissions', 'manage', 'Manage permissions and group assignments', 'manual', true, true),
    ('admin:payments:view', 'admin', 'payments', 'view', 'View billing and revenue analytics', 'manual', true, true),
    ('admin:payments:manage', 'admin', 'payments', 'manage', 'Manage refunds and payment adjustments', 'manual', true, true),
    ('admin:system:view', 'admin', 'system', 'view', 'View system health and logs', 'manual', true, true),
    ('admin:system:manage', 'admin', 'system', 'manage', 'Manage system configuration and maintenance', 'manual', true, true),
    ('admin:system:configure', 'admin', 'system', 'configure', 'Configure system settings', 'manual', true, true)
ON CONFLICT (permission_string) DO NOTHING;

-- Seed plan_permissions (link permissions to plans)
DO $$
DECLARE
    p_one_day UUID;
    p_starter UUID;
    p_lifetime UUID;
    p_company UUID;
    p_api_pers UUID;
BEGIN
    SELECT id INTO p_one_day FROM plans WHERE slug = 'one-day';
    SELECT id INTO p_starter FROM plans WHERE slug = 'starter';
    SELECT id INTO p_lifetime FROM plans WHERE slug = 'lifetime';
    SELECT id INTO p_company FROM plans WHERE slug = 'company';
    SELECT id INTO p_api_pers FROM plans WHERE slug = 'api-personal';

    -- One Day: analytics:view, trading:basic
    IF p_one_day IS NOT NULL THEN
        INSERT INTO plan_permissions (plan_id, permission_id)
        SELECT p_one_day, id FROM permissions
        WHERE permission_string IN ('epsx:analytics:view', 'epsx:trading:basic')
        ON CONFLICT (plan_id, permission_id) DO NOTHING;
    END IF;

    -- Starter: analytics:view/advanced, trading:basic, alerts:create
    IF p_starter IS NOT NULL THEN
        INSERT INTO plan_permissions (plan_id, permission_id)
        SELECT p_starter, id FROM permissions
        WHERE permission_string IN ('epsx:analytics:view', 'epsx:analytics:advanced', 'epsx:trading:basic', 'epsx:alerts:create')
        ON CONFLICT (plan_id, permission_id) DO NOTHING;
    END IF;

    -- Lifetime: analytics:view/advanced, trading:basic/pro, api:read
    IF p_lifetime IS NOT NULL THEN
        INSERT INTO plan_permissions (plan_id, permission_id)
        SELECT p_lifetime, id FROM permissions
        WHERE permission_string IN ('epsx:analytics:view', 'epsx:analytics:advanced', 'epsx:trading:basic', 'epsx:trading:pro', 'epsx:api:read')
        ON CONFLICT (plan_id, permission_id) DO NOTHING;
    END IF;

    -- Company: all trading, api read+write, data export, notifications
    IF p_company IS NOT NULL THEN
        INSERT INTO plan_permissions (plan_id, permission_id)
        SELECT p_company, id FROM permissions
        WHERE permission_string IN ('epsx:analytics:view', 'epsx:analytics:advanced', 'epsx:trading:basic', 'epsx:trading:pro', 'epsx:trading:advanced', 'epsx:api:read', 'epsx:api:write', 'epsx:data:export', 'epsx:notifications:manage')
        ON CONFLICT (plan_id, permission_id) DO NOTHING;
    END IF;

    -- API Personal: analytics:view, api:read, data:export
    IF p_api_pers IS NOT NULL THEN
        INSERT INTO plan_permissions (plan_id, permission_id)
        SELECT p_api_pers, id FROM permissions
        WHERE permission_string IN ('epsx:analytics:view', 'epsx:api:read', 'epsx:data:export')
        ON CONFLICT (plan_id, permission_id) DO NOTHING;
    END IF;
END $$;

-- ================================================================================================
-- TABLE COMMENTS
-- ================================================================================================

COMMENT ON TABLE wallet_users IS 'Wallet user accounts - core identity table';
COMMENT ON TABLE permissions IS 'Permission definitions catalog';
COMMENT ON TABLE plans IS 'Unified subscription plans with permissions, pricing, and rate limits';
COMMENT ON TABLE plan_permissions IS 'Maps permissions to plans';
COMMENT ON TABLE wallet_plan_assignments IS 'Assigns wallets to subscription plans';
COMMENT ON TABLE wallet_direct_permissions IS 'Direct permission grants to wallets';
COMMENT ON TABLE system_settings IS 'Global admin console settings';
COMMENT ON TABLE api_keys IS 'Developer API keys';
COMMENT ON TABLE api_key_permissions IS 'Links API keys to plans for plan-based permissions';

SELECT 'EPSX CONSOLIDATED SCHEMA v5 CREATED SUCCESSFULLY! 🎉' AS success_message;
