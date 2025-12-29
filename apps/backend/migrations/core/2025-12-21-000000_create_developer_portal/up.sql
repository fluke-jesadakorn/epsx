-- Create Developer Portal tables for API key management

-- API Keys table: stores developer API keys
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash VARCHAR(256) NOT NULL UNIQUE,
    key_prefix VARCHAR(16) NOT NULL,
    client_name VARCHAR(255) NOT NULL,
    client_description TEXT,
    client_contact_email VARCHAR(255),
    wallet_address VARCHAR(42) NOT NULL REFERENCES wallet_users(wallet_address),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    total_requests BIGINT NOT NULL DEFAULT 0,
    ip_restrictions TEXT[],
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    rate_limit_per_day INTEGER NOT NULL DEFAULT 10000,
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    revoked_by VARCHAR(42),
    revocation_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(42) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- API Modules table: defines accessible backend modules
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

-- Junction table: API key module access permissions
CREATE TABLE api_key_module_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    module_id UUID NOT NULL REFERENCES api_modules(id) ON DELETE CASCADE,
    access_level VARCHAR(20) NOT NULL DEFAULT 'bronze',
    custom_rate_limit INTEGER,
    custom_quotas JSONB NOT NULL DEFAULT '{}',
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by VARCHAR(42),
    UNIQUE(api_key_id, module_id)
);

-- Usage logs for analytics and rate limiting
CREATE TABLE api_key_usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    module_id UUID REFERENCES api_modules(id),
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    response_status INTEGER,
    response_time_ms INTEGER,
    ip_address INET,
    user_agent TEXT,
    request_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_api_keys_wallet ON api_keys(wallet_address);
CREATE INDEX idx_api_keys_status ON api_keys(status);
CREATE INDEX idx_api_keys_key_prefix ON api_keys(key_prefix);
CREATE INDEX idx_api_modules_status ON api_modules(status);
CREATE INDEX idx_api_modules_category ON api_modules(category);
CREATE INDEX idx_api_key_module_access_key ON api_key_module_access(api_key_id);
CREATE INDEX idx_api_key_module_access_module ON api_key_module_access(module_id);
CREATE INDEX idx_api_key_usage_key ON api_key_usage_logs(api_key_id);
CREATE INDEX idx_api_key_usage_time ON api_key_usage_logs(request_at);
CREATE INDEX idx_api_key_usage_key_time ON api_key_usage_logs(api_key_id, request_at);

-- Seed default modules
INSERT INTO api_modules (name, display_name, description, category, base_path, default_rate_limit, access_levels, endpoints) VALUES
('stock-ranking', 'Stock Ranking', 'Access stock ranking and performance data', 'analytics', '/api/v1/modules/stock-ranking', 60, 
 '{"bronze": {"requests_per_minute": 10, "requests_per_day": 100}, "silver": {"requests_per_minute": 30, "requests_per_day": 500}, "gold": {"requests_per_minute": 100, "requests_per_day": 5000}, "platinum": {"requests_per_minute": 500, "requests_per_day": 50000}, "enterprise": {"requests_per_minute": -1, "requests_per_day": -1}}',
 '[{"path": "/rankings", "method": "GET", "description": "Get stock rankings", "access_level_required": "bronze"}, {"path": "/rankings/:id", "method": "GET", "description": "Get single stock details", "access_level_required": "bronze"}]'),
('market-data', 'Market Data', 'Real-time and historical market data', 'data', '/api/v1/modules/market-data', 30,
 '{"bronze": {"requests_per_minute": 5, "requests_per_day": 50}, "silver": {"requests_per_minute": 20, "requests_per_day": 300}, "gold": {"requests_per_minute": 60, "requests_per_day": 3000}, "platinum": {"requests_per_minute": 300, "requests_per_day": 30000}, "enterprise": {"requests_per_minute": -1, "requests_per_day": -1}}',
 '[{"path": "/overview", "method": "GET", "description": "Market overview", "access_level_required": "bronze"}, {"path": "/trends", "method": "GET", "description": "Market trends", "access_level_required": "silver"}]'),
('analytics', 'Analytics API', 'Advanced analytics and insights', 'analytics', '/api/v1/modules/analytics', 60,
 '{"bronze": {"requests_per_minute": 10, "requests_per_day": 100}, "silver": {"requests_per_minute": 30, "requests_per_day": 500}, "gold": {"requests_per_minute": 100, "requests_per_day": 5000}, "platinum": {"requests_per_minute": 500, "requests_per_day": 50000}, "enterprise": {"requests_per_minute": -1, "requests_per_day": -1}}',
 '[{"path": "/performance", "method": "GET", "description": "Performance analytics", "access_level_required": "bronze"}, {"path": "/portfolio", "method": "GET", "description": "Portfolio analysis", "access_level_required": "gold"}]');
