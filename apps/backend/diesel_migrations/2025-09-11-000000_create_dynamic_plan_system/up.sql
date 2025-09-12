-- Migration: Dynamic Plan System with Resource Tracking
-- Creates tables for flexible plan management and contextual access control

-- Access contexts for different types of system usage
CREATE TABLE access_contexts (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE, -- 'web_app', 'api_access', 'admin_interface'
    description TEXT,
    is_billable BOOLEAN DEFAULT false,
    requires_audit BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default access contexts
INSERT INTO access_contexts (name, description, is_billable, requires_audit) VALUES
('web_app', 'Internal web application access for end users', false, false),
('api_access', 'External API access for developers and integrations', true, false), 
('admin_interface', 'Administrative interface access', false, true);

-- Enhanced plan features with context-specific configurations
CREATE TABLE plan_features (
    id SERIAL PRIMARY KEY,
    plan_id INT NOT NULL REFERENCES pricing_plans(id) ON DELETE CASCADE,
    context_id INT NOT NULL REFERENCES access_contexts(id) ON DELETE CASCADE,
    feature_key VARCHAR(100) NOT NULL, -- 'ranking_limit', 'api_calls_per_day', 'webhook_enabled'
    feature_config JSONB NOT NULL,     -- {"limit": 1000, "burst": 100, "reset_period": "daily"}
    resource_cost DECIMAL(10,6) DEFAULT 0.000001, -- Cost per unit for billing calculation
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(plan_id, context_id, feature_key)
);

-- Create index for efficient feature lookups
CREATE INDEX idx_plan_features_plan_context ON plan_features(plan_id, context_id);
CREATE INDEX idx_plan_features_key ON plan_features(feature_key);

-- User plan subscriptions with context-specific access
CREATE TABLE user_plan_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id INT NOT NULL REFERENCES pricing_plans(id) ON DELETE RESTRICT,
    access_context VARCHAR(20) NOT NULL, -- 'internal', 'external', 'both'
    api_key VARCHAR(100) UNIQUE,         -- For external access (nullable for internal-only)
    api_key_name VARCHAR(100),           -- User-friendly name for API key
    status VARCHAR(20) DEFAULT 'active', -- 'active', 'suspended', 'expired', 'canceled'
    
    -- Usage tracking and limits
    usage_tracking JSONB DEFAULT '{}',                -- Real-time usage counters
    resource_consumption JSONB DEFAULT '{}',          -- Billable resource tracking
    quota_limits JSONB DEFAULT '{}',                  -- Plan-specific quota limits
    current_usage JSONB DEFAULT '{}',                 -- Current period usage stats
    
    -- Subscription lifecycle
    started_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    auto_renew BOOLEAN DEFAULT false,
    last_billed_at TIMESTAMPTZ,
    next_billing_date TIMESTAMPTZ,
    
    -- Audit and metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    subscription_metadata JSONB DEFAULT '{}'
);

-- Create indexes for efficient subscription lookups
CREATE INDEX idx_user_subscriptions_user_id ON user_plan_subscriptions(user_id);
CREATE INDEX idx_user_subscriptions_plan_id ON user_plan_subscriptions(plan_id);
CREATE INDEX idx_user_subscriptions_api_key ON user_plan_subscriptions(api_key) WHERE api_key IS NOT NULL;
CREATE INDEX idx_user_subscriptions_status ON user_plan_subscriptions(status);
CREATE INDEX idx_user_subscriptions_expires ON user_plan_subscriptions(expires_at) WHERE expires_at IS NOT NULL;

-- Resource usage tracking for billing and analytics
CREATE TABLE resource_usage_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Event identification
    subscription_id UUID REFERENCES user_plan_subscriptions(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    session_id VARCHAR(100),
    api_key VARCHAR(100),
    
    -- Resource details
    resource_type VARCHAR(50) NOT NULL,    -- 'api_call', 'data_transfer', 'web_action'
    resource_category VARCHAR(20) NOT NULL, -- 'web_app', 'external_api', 'admin', 'infrastructure'
    endpoint VARCHAR(200),                   -- API endpoint or web page
    method VARCHAR(10),                      -- HTTP method
    
    -- Usage metrics
    quantity BIGINT DEFAULT 1,               -- Number of units consumed
    data_size_bytes BIGINT,                  -- Data transfer size
    processing_time_ms INT,                  -- Processing time
    response_code INT,                       -- HTTP response code
    
    -- Cost calculation
    unit_cost DECIMAL(10,6),                -- Cost per unit
    total_cost DECIMAL(12,6),               -- Total cost for this event
    is_billable BOOLEAN DEFAULT false,      -- Whether this usage is billable
    
    -- Context and metadata  
    access_context VARCHAR(20) NOT NULL,    -- 'internal', 'external', 'admin'
    client_ip INET,
    user_agent TEXT,
    request_id VARCHAR(100),
    
    -- Timing
    occurred_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ DEFAULT NOW(),
    billing_period DATE DEFAULT CURRENT_DATE
);

-- Partitioning by month for performance
CREATE INDEX idx_resource_usage_events_billing_period ON resource_usage_events(billing_period);
CREATE INDEX idx_resource_usage_events_subscription ON resource_usage_events(subscription_id, occurred_at);
CREATE INDEX idx_resource_usage_events_user_time ON resource_usage_events(user_id, occurred_at);
CREATE INDEX idx_resource_usage_events_api_key ON resource_usage_events(api_key, occurred_at) WHERE api_key IS NOT NULL;
CREATE INDEX idx_resource_usage_events_billable ON resource_usage_events(is_billable, billing_period) WHERE is_billable = true;
CREATE INDEX idx_resource_usage_events_context ON resource_usage_events(access_context, occurred_at);

-- Usage analytics aggregation table (for performance)
CREATE TABLE usage_analytics_summary (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Aggregation dimensions
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    subscription_id UUID REFERENCES user_plan_subscriptions(id) ON DELETE CASCADE,
    plan_id INT REFERENCES pricing_plans(id) ON DELETE RESTRICT,
    access_context VARCHAR(20) NOT NULL,
    resource_category VARCHAR(20) NOT NULL,
    
    -- Time dimension
    time_period VARCHAR(20) NOT NULL,       -- 'hour', 'day', 'month'
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    
    -- Aggregated metrics
    total_requests BIGINT DEFAULT 0,
    total_cost DECIMAL(12,6) DEFAULT 0,
    total_data_bytes BIGINT DEFAULT 0,
    average_response_time_ms FLOAT DEFAULT 0,
    error_count INT DEFAULT 0,
    unique_endpoints INT DEFAULT 0,
    
    -- Efficiency metrics
    cost_per_request DECIMAL(10,6) DEFAULT 0,
    requests_per_hour FLOAT DEFAULT 0,
    efficiency_score FLOAT DEFAULT 1.0,
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(user_id, subscription_id, access_context, resource_category, time_period, period_start)
);

-- Indexes for analytics queries
CREATE INDEX idx_analytics_summary_user_period ON usage_analytics_summary(user_id, time_period, period_start);
CREATE INDEX idx_analytics_summary_subscription_period ON usage_analytics_summary(subscription_id, time_period, period_start);
CREATE INDEX idx_analytics_summary_plan_period ON usage_analytics_summary(plan_id, time_period, period_start);
CREATE INDEX idx_analytics_summary_context_period ON usage_analytics_summary(access_context, time_period, period_start);

-- Real-time usage tracking cache (for rate limiting)
CREATE TABLE real_time_usage_cache (
    id VARCHAR(100) PRIMARY KEY,           -- user_id, api_key, or session_id
    identifier_type VARCHAR(20) NOT NULL, -- 'user', 'api_key', 'session'
    
    -- Current usage windows
    current_minute_requests INT DEFAULT 0,
    current_hour_requests INT DEFAULT 0,
    current_day_requests INT DEFAULT 0,
    current_month_requests INT DEFAULT 0,
    
    -- Current costs (for billable contexts)
    current_minute_cost DECIMAL(10,6) DEFAULT 0,
    current_hour_cost DECIMAL(10,6) DEFAULT 0,
    current_day_cost DECIMAL(10,6) DEFAULT 0,
    current_month_cost DECIMAL(10,6) DEFAULT 0,
    
    -- Window boundaries
    minute_window_start TIMESTAMPTZ,
    hour_window_start TIMESTAMPTZ,
    day_window_start TIMESTAMPTZ,
    month_window_start TIMESTAMPTZ,
    
    -- Metadata
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    subscription_id UUID REFERENCES user_plan_subscriptions(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ DEFAULT NOW() + INTERVAL '1 hour'
);

-- Index for cleanup of expired cache entries
CREATE INDEX idx_real_time_cache_expires ON real_time_usage_cache(expires_at);
CREATE INDEX idx_real_time_cache_subscription ON real_time_usage_cache(subscription_id) WHERE subscription_id IS NOT NULL;

-- Plan migration tracking (for upgrading/downgrading subscriptions)
CREATE TABLE subscription_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES user_plan_subscriptions(id) ON DELETE CASCADE,
    
    -- Change details
    change_type VARCHAR(20) NOT NULL,       -- 'upgrade', 'downgrade', 'cancel', 'reactivate', 'api_key_reset'
    old_plan_id INT REFERENCES pricing_plans(id),
    new_plan_id INT REFERENCES pricing_plans(id),
    old_status VARCHAR(20),
    new_status VARCHAR(20),
    
    -- Billing impact
    prorated_amount DECIMAL(12,2),
    effective_date TIMESTAMPTZ DEFAULT NOW(),
    billing_adjustment JSONB,
    
    -- Audit trail
    changed_by UUID REFERENCES users(id),
    change_reason TEXT,
    admin_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_subscription_changes_subscription ON subscription_changes(subscription_id, created_at);
CREATE INDEX idx_subscription_changes_type ON subscription_changes(change_type, created_at);

-- Update existing pricing_plans table with enhanced features
ALTER TABLE pricing_plans 
ADD COLUMN IF NOT EXISTS plan_category VARCHAR(20) DEFAULT 'standard', -- 'standard', 'api', 'enterprise', 'custom'
ADD COLUMN IF NOT EXISTS target_audience VARCHAR(20) DEFAULT 'general', -- 'web_users', 'api_developers', 'enterprises'
ADD COLUMN IF NOT EXISTS billing_model VARCHAR(20) DEFAULT 'subscription', -- 'subscription', 'pay_per_use', 'hybrid'
ADD COLUMN IF NOT EXISTS plan_metadata JSONB DEFAULT '{}';

-- Add some sample plan features
INSERT INTO plan_features (plan_id, context_id, feature_key, feature_config, resource_cost) VALUES
-- Web app features for Bronze plan (assuming plan_id 1 exists)
(1, 1, 'ranking_limit', '{"limit": 5, "type": "daily"}', 0.0),
(1, 1, 'analytics_access', '{"enabled": true, "basic_only": true}', 0.0),

-- API features for API starter plan (assuming plan exists)
(2, 2, 'api_calls_per_day', '{"limit": 1000, "burst": 10}', 0.001),
(2, 2, 'webhook_support', '{"enabled": false}', 0.0),
(2, 2, 'data_transfer_gb', '{"limit": 1, "overage_cost": 0.10}', 0.10),

-- Admin features (assuming admin plan exists)
(3, 3, 'user_management', '{"enabled": true, "bulk_operations": false}', 0.0),
(3, 3, 'system_monitoring', '{"enabled": true, "real_time": true}', 0.0);

-- Function to automatically update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add update triggers for tables with updated_at columns
CREATE TRIGGER update_access_contexts_updated_at BEFORE UPDATE ON access_contexts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_plan_features_updated_at BEFORE UPDATE ON plan_features FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_plan_subscriptions_updated_at BEFORE UPDATE ON user_plan_subscriptions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_usage_analytics_summary_updated_at BEFORE UPDATE ON usage_analytics_summary FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create a view for easy subscription querying with plan details
CREATE VIEW subscription_details AS
SELECT 
    s.id as subscription_id,
    s.user_id,
    s.access_context,
    s.api_key,
    s.api_key_name,
    s.status,
    s.started_at,
    s.expires_at,
    s.usage_tracking,
    s.current_usage,
    p.id as plan_id,
    p.name as plan_name,
    p.plan_type,
    p.current_price,
    p.currency,
    p.features as plan_features,
    ac.name as context_name,
    ac.is_billable,
    ac.requires_audit
FROM user_plan_subscriptions s
JOIN pricing_plans p ON s.plan_id = p.id
JOIN access_contexts ac ON s.access_context = ac.name
WHERE s.status = 'active';

-- Grant appropriate permissions (adjust as needed for your user roles)
-- GRANT SELECT ON subscription_details TO app_readonly_user;
-- GRANT SELECT, INSERT, UPDATE ON user_plan_subscriptions TO app_readwrite_user;
-- GRANT SELECT, INSERT ON resource_usage_events TO app_readwrite_user;

COMMENT ON TABLE access_contexts IS 'Defines different access contexts (web app, API, admin) with their billing and audit requirements';
COMMENT ON TABLE plan_features IS 'Context-specific feature configurations for plans, enabling dynamic plan management';
COMMENT ON TABLE user_plan_subscriptions IS 'User subscriptions with context-specific access and real-time usage tracking';
COMMENT ON TABLE resource_usage_events IS 'Detailed tracking of all resource usage events for billing and analytics';
COMMENT ON TABLE usage_analytics_summary IS 'Pre-aggregated usage analytics for performance optimization';
COMMENT ON TABLE real_time_usage_cache IS 'Real-time usage tracking for rate limiting and quota enforcement';
COMMENT ON VIEW subscription_details IS 'Convenient view joining subscriptions with plan and context details';