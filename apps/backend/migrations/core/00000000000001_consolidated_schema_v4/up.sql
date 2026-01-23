-- ================================================================================================
-- EPSX CONSOLIDATED SCHEMA v4
-- ================================================================================================
-- Version: 4.0.0 (Consolidated January 2026)
-- Description: Complete production-ready schema for EPSX Web3-first platform.
--              Consolidates all migrations into a single clean baseline.
--
-- Core Components:
--   - Web3-first wallet authentication (SIWE compliant)
--   - Permission groups system (groups, group_permissions)
--   - Event sourcing infrastructure (CQRS)
--   - Notification system with SSE support
--   - Developer portal with API key management
--   - System settings
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
    permission_groups JSONB DEFAULT '[]',
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
        permission_type IN ('manual', 'nft_gated', 'token_gated', 'dao_governance')
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

-- GROUPS - Permission groups (subscription plans, etc.)
CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    description TEXT DEFAULT '' NOT NULL,
    group_type VARCHAR(20) DEFAULT 'manual' NOT NULL,
    group_metadata JSONB DEFAULT '{}' NOT NULL,
    price NUMERIC(10,2) DEFAULT 0.00,
    currency VARCHAR(3) DEFAULT 'USD',
    billing_cycle VARCHAR(20) DEFAULT 'pay_per_use',
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    is_promoted BOOLEAN DEFAULT FALSE NOT NULL,
    display_order INTEGER DEFAULT 0,
    max_members INTEGER,
    auto_assign_enabled BOOLEAN DEFAULT FALSE,
    assignment_rules JSONB DEFAULT '{}',
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    rate_limit_per_hour INTEGER NOT NULL DEFAULT 1000,
    rate_limit_per_day INTEGER NOT NULL DEFAULT 10000,
    burst_capacity INTEGER NOT NULL DEFAULT 10,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_by VARCHAR(42),
    last_modified_by VARCHAR(42),
    
    CONSTRAINT valid_group_type CHECK (
        group_type IN ('manual', 'subscription', 'web3_asset', 'dao_membership', 'admin')
    ),
    CONSTRAINT valid_currency CHECK (
        currency IN ('USD', 'EUR', 'BTC', 'ETH', 'BNB')
    ),
    CONSTRAINT valid_billing_cycle CHECK (
        billing_cycle IN ('monthly', 'yearly', 'one_time', 'lifetime', 'pay_per_use')
    )
);

CREATE INDEX idx_groups_active ON groups(is_active, display_order);
CREATE INDEX idx_groups_type ON groups(group_type, is_active);
CREATE INDEX idx_groups_slug ON groups(slug);
CREATE INDEX idx_groups_price ON groups(price, currency) WHERE group_type = 'subscription';
CREATE INDEX idx_groups_created ON groups(created_at DESC);
CREATE INDEX idx_groups_metadata_gin ON groups USING gin(group_metadata);

-- GROUP_PERMISSIONS - Group to Permission mapping
CREATE TABLE group_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id UUID NOT NULL,
    permission_id UUID NOT NULL,
    granted_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    granted_by VARCHAR(42),
    grant_reason TEXT,
    UNIQUE(group_id, permission_id)
);

CREATE INDEX idx_group_permissions_group ON group_permissions(group_id);
CREATE INDEX idx_group_permissions_permission ON group_permissions(permission_id);
CREATE INDEX idx_group_permissions_audit ON group_permissions(granted_at);

-- WALLET_GROUP_ASSIGNMENTS - Wallet to Group mapping
CREATE TABLE wallet_group_assignments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    group_id UUID NOT NULL,
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
    UNIQUE(wallet_address, group_id)
);

CREATE INDEX idx_wg_assignments_wallet ON wallet_group_assignments(wallet_address, is_active);
CREATE INDEX idx_wg_assignments_group ON wallet_group_assignments(group_id, is_active);
CREATE INDEX idx_wg_assignments_expires ON wallet_group_assignments(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_wg_assignments_audit ON wallet_group_assignments(assigned_at);
CREATE INDEX idx_wga_active_lookup ON wallet_group_assignments(wallet_address, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_wga_expires_lookup ON wallet_group_assignments(wallet_address, expires_at) WHERE is_active = TRUE AND expires_at IS NOT NULL;

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

-- SESSIONS - Active user sessions
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    wallet_address VARCHAR(42) NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    last_accessed_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    is_revoked BOOLEAN DEFAULT FALSE NOT NULL,
    version BIGINT DEFAULT 1 NOT NULL,
    
    CONSTRAINT sessions_valid_wallet CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    ),
    CONSTRAINT access_token_not_empty CHECK (LENGTH(TRIM(access_token)) > 0),
    CONSTRAINT expires_at_future CHECK (expires_at > created_at),
    CONSTRAINT version_positive CHECK (version > 0)
);

CREATE INDEX idx_sessions_wallet_address ON sessions(wallet_address, is_revoked, expires_at) WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_access_token ON sessions(access_token) WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_refresh_token ON sessions(refresh_token) WHERE refresh_token IS NOT NULL AND is_revoked = FALSE;
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at) WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_active ON sessions(wallet_address, is_revoked, expires_at, last_accessed_at) WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX idx_sessions_last_accessed ON sessions(last_accessed_at DESC) WHERE is_revoked = FALSE;
CREATE INDEX idx_sessions_ip_address ON sessions(ip_address, wallet_address) WHERE ip_address IS NOT NULL AND is_revoked = FALSE;

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
-- AUDIT TABLES
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



-- ================================================================================================
-- FOREIGN KEY CONSTRAINTS
-- ================================================================================================

ALTER TABLE sessions ADD CONSTRAINT sessions_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

ALTER TABLE group_permissions ADD CONSTRAINT group_permissions_group_id_fkey
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE;

ALTER TABLE group_permissions ADD CONSTRAINT group_permissions_permission_id_fkey
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE;

ALTER TABLE wallet_group_assignments ADD CONSTRAINT wga_wallet_address_fkey
    FOREIGN KEY (wallet_address) REFERENCES wallet_users(wallet_address) ON DELETE CASCADE;

ALTER TABLE wallet_group_assignments ADD CONSTRAINT wga_group_id_fkey
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE;

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

    -- Group permissions
    SELECT
        p.permission_string,
        p.id,
        'group'::VARCHAR(20),
        g.id as source_id,
        g.name as source_name,
        wga.expires_at,
        wga.assigned_at as granted_at,
        (wga.expires_at IS NULL) as is_permanent
    FROM wallet_group_assignments wga
    INNER JOIN groups g ON wga.group_id = g.id
    INNER JOIN group_permissions gp ON g.id = gp.group_id
    INNER JOIN permissions p ON gp.permission_id = p.id
    WHERE LOWER(wga.wallet_address) = LOWER(p_wallet_address)
      AND wga.is_active = TRUE
      AND g.is_active = TRUE
      AND p.is_active = TRUE
      AND (wga.expires_at IS NULL OR wga.expires_at > NOW());
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

-- Subscription plans (groups)
INSERT INTO groups (id, name, slug, description, group_type, group_metadata, price, currency, billing_cycle, is_active, is_promoted, display_order, rate_limit_per_minute, rate_limit_per_hour, rate_limit_per_day, burst_capacity, created_by) VALUES
    (uuid_generate_v4(), 'Free Plan', 'free', 'Perfect for getting started with basic analytics', 'subscription', 
     '{"permissions": ["epsx:analytics:view:5"], "features": ["Basic analytics", "5 stock rankings", "Community support"]}'::jsonb,
     0.00, 'USD', 'monthly', true, false, 1, 60, 1000, 10000, 10, '0x0000000000000000000000000000000000000000'),
    (uuid_generate_v4(), 'Starter Plan', 'starter', 'Ideal for individual investors', 'subscription',
     '{"permissions": ["epsx:analytics:view:25"], "features": ["Advanced analytics", "25 stock rankings", "Email support"]}'::jsonb,
     14.99, 'USD', 'monthly', true, false, 2, 120, 3000, 50000, 20, '0x0000000000000000000000000000000000000000'),
    (uuid_generate_v4(), 'Pro Plan', 'pro', 'For serious traders who need advanced tools', 'subscription',
     '{"permissions": ["epsx:analytics:view:100"], "features": ["Premium analytics", "100 stock rankings", "Priority support"], "highlighted": true}'::jsonb,
     29.99, 'USD', 'monthly', true, true, 3, 300, 10000, 200000, 50, '0x0000000000000000000000000000000000000000'),
    (uuid_generate_v4(), 'Enterprise Plan', 'enterprise', 'Complete solution for teams', 'subscription',
     '{"permissions": ["epsx:*:*"], "features": ["Unlimited access", "API access", "Dedicated support"], "contact_sales": true}'::jsonb,
     99.99, 'USD', 'monthly', true, false, 4, 1000, 50000, 1000000, 200, '0x0000000000000000000000000000000000000000'),
    (uuid_generate_v4(), 'API Developer', 'api-developer', 'For developers building on EPSX', 'subscription',
     '{"permissions": ["epsx:api:access"], "features": ["Full API access", "Developer docs", "100k calls/month"], "plan_type": "api"}'::jsonb,
     49.99, 'USD', 'monthly', true, false, 5, 300, 10000, 100000, 50, '0x0000000000000000000000000000000000000000')
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

-- Core permissions
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
    ('admin:users:manage', 'admin', 'users', 'manage', 'Manage users', 'manual', true, true),
    ('admin:permissions:manage', 'admin', 'permissions', 'manage', 'Manage permissions', 'manual', true, true),
    ('admin:system:configure', 'admin', 'system', 'configure', 'Configure system settings', 'manual', true, true)
ON CONFLICT (permission_string) DO NOTHING;

-- ================================================================================================
-- TABLE COMMENTS
-- ================================================================================================

COMMENT ON TABLE wallet_users IS 'Wallet user accounts - core identity table';
COMMENT ON TABLE permissions IS 'Permission definitions catalog';
COMMENT ON TABLE groups IS 'Permission groups for subscription plans';
COMMENT ON TABLE group_permissions IS 'Maps permissions to groups';
COMMENT ON TABLE wallet_group_assignments IS 'Assigns wallets to permission groups';
COMMENT ON TABLE wallet_direct_permissions IS 'Direct permission grants to wallets';
COMMENT ON TABLE sessions IS 'Active user sessions';

COMMENT ON TABLE system_settings IS 'Global admin console settings';
COMMENT ON TABLE api_keys IS 'Developer API keys';

SELECT 'EPSX CONSOLIDATED SCHEMA v4 CREATED SUCCESSFULLY! 🎉' AS success_message;
