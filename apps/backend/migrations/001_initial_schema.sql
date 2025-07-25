-- EPSX Trading Platform Database Schema
-- Consolidated seed file - creates complete database from scratch
-- This file is idempotent and can be run multiple times safely

-- Users table (core user data - Firebase UID based)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) UNIQUE NOT NULL,
    email VARCHAR(255) NOT NULL, -- Placeholder email (format: firebase_uid@firebase.user)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Dynamic permission profiles (core feature system)
CREATE TABLE IF NOT EXISTS permission_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    category VARCHAR(50) NOT NULL, -- 'analytics', 'premium', 'admin'
    version VARCHAR(20) NOT NULL DEFAULT '1.0',
    status VARCHAR(50) DEFAULT 'active',
    profile_data JSONB NOT NULL, -- Features, modules, limits
    pricing_tier JSONB DEFAULT '{}',
    auto_assignment_rules JSONB DEFAULT '{}',
    api_endpoints JSONB DEFAULT '{}', -- API access control: {"allowed": [...], "rate_limits": {...}}
    frontend_routes JSONB DEFAULT '{}', -- Frontend route access: {"allowed": [...], "blocked": [...]}
    compliance_level VARCHAR(50) DEFAULT 'educational',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- Permission profile variables
CREATE TABLE IF NOT EXISTS profile_variables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID REFERENCES permission_profiles(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    type VARCHAR(50) NOT NULL, -- 'string', 'number', 'boolean', 'json'
    required BOOLEAN DEFAULT FALSE,
    default_value JSONB,
    user_configurable BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Auto assignment rules (detailed rule tracking for profile assignment)
CREATE TABLE IF NOT EXISTS auto_assignment_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_name VARCHAR(255) NOT NULL,
    permission_profile_id UUID REFERENCES permission_profiles(id) NOT NULL ON DELETE CASCADE,
    trigger_type VARCHAR(50) NOT NULL, -- 'registration', 'package_tier', 'email_domain', 'utm_campaign', 'referral'
    trigger_conditions JSONB NOT NULL, -- Conditions for rule activation
    assignment_variables JSONB DEFAULT '{}', -- Variables to pass to profile
    priority INTEGER DEFAULT 0, -- Higher numbers = higher priority
    status VARCHAR(50) DEFAULT 'active',
    usage_count INTEGER DEFAULT 0, -- Track how many times rule was applied
    success_count INTEGER DEFAULT 0, -- Track successful applications
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- Permission profile templates (marketplace/template system)
CREATE TABLE IF NOT EXISTS permission_profile_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL, -- 'community', 'official', 'enterprise', 'custom'
    template_data JSONB NOT NULL, -- Complete profile structure as template
    tags JSONB DEFAULT '[]', -- Search tags for marketplace
    author_id UUID REFERENCES users(id) NOT NULL,
    organization VARCHAR(255), -- Company/org if applicable
    version VARCHAR(20) DEFAULT '1.0',
    is_public BOOLEAN DEFAULT FALSE,
    usage_count INTEGER DEFAULT 0, -- How many times template was used
    rating_average DECIMAL(3,2) DEFAULT 0.0,
    rating_count INTEGER DEFAULT 0,
    price_usd DECIMAL(10,2) DEFAULT 0, -- 0 for free templates
    status VARCHAR(50) DEFAULT 'draft', -- 'draft', 'published', 'deprecated'
    approval_status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'approved', 'rejected'
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Admin permission profile assignments (direct admin assignments)
CREATE TABLE IF NOT EXISTS admin_permission_profile_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL ON DELETE CASCADE,
    permission_profile_id UUID REFERENCES permission_profiles(id) NOT NULL ON DELETE CASCADE,
    assigned_by UUID REFERENCES users(id) NOT NULL,
    assignment_type VARCHAR(50) NOT NULL, -- 'promotional', 'trial', 'permanent'
    assignment_reason TEXT NOT NULL,
    expires_at TIMESTAMPTZ,
    variables JSONB DEFAULT '{}',
    override_pricing BOOLEAN DEFAULT FALSE,
    notification_settings JSONB DEFAULT '{}',
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Assignment audit trail
CREATE TABLE IF NOT EXISTS assignment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID REFERENCES admin_permission_profile_assignments(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    performed_by UUID REFERENCES users(id) NOT NULL,
    details JSONB NOT NULL,
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- User permission profile assignments (comprehensive assignment tracking)
CREATE TABLE IF NOT EXISTS user_permission_profile_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL ON DELETE CASCADE,
    permission_profile_id UUID REFERENCES permission_profiles(id) NOT NULL ON DELETE CASCADE,
    assignment_type VARCHAR(50) NOT NULL, -- 'auto', 'admin', 'payment', 'trial'
    assignment_source VARCHAR(50) NOT NULL, -- 'registration', 'payment_webhook', 'admin_dashboard', 'promotion'
    assigned_by UUID REFERENCES users(id), -- NULL for auto assignments
    expires_at TIMESTAMPTZ,
    variables JSONB DEFAULT '{}',
    activation_metadata JSONB DEFAULT '{}', -- Payment info, campaign details, etc.
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deactivated_at TIMESTAMPTZ,
    deactivation_reason VARCHAR(255)
);

-- User features (tracks active features per user)
CREATE TABLE IF NOT EXISTS user_features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL ON DELETE CASCADE,
    permission_profile_id UUID REFERENCES permission_profiles(id) NOT NULL ON DELETE CASCADE,
    assignment_id UUID REFERENCES user_permission_profile_assignments(id) ON DELETE CASCADE,
    feature_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    configuration JSONB DEFAULT '{}',
    activated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    activated_via VARCHAR(50) DEFAULT 'auto'
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT true
);


-- Stock ranking packages (feature referenced in untracked files)
CREATE TABLE IF NOT EXISTS stock_ranking_packages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    ranking_criteria JSONB NOT NULL, -- Algorithm parameters, weights, etc.
    target_sectors JSONB DEFAULT '[]', -- Specific sectors or 'all'
    update_frequency VARCHAR(50) DEFAULT 'daily', -- 'real_time', 'hourly', 'daily'
    permission_profile_id UUID REFERENCES permission_profiles(id) NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- Stock ranking package assignments
CREATE TABLE IF NOT EXISTS user_stock_ranking_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL ON DELETE CASCADE,
    package_id UUID REFERENCES stock_ranking_packages(id) NOT NULL ON DELETE CASCADE,
    assigned_by UUID REFERENCES users(id),
    assignment_type VARCHAR(50) DEFAULT 'auto', -- 'auto', 'admin', 'payment'
    expires_at TIMESTAMPTZ,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Feature payment integration for activation
CREATE TABLE IF NOT EXISTS feature_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL ON DELETE CASCADE,
    permission_profile_id UUID REFERENCES permission_profiles(id) NOT NULL ON DELETE CASCADE,
    external_payment_id VARCHAR(255) NOT NULL, -- External payment system reference
    payment_provider VARCHAR(50) NOT NULL, -- 'stripe', 'crypto', 'manual'
    amount DECIMAL(10,2),
    currency VARCHAR(10) DEFAULT 'USD',
    features_unlocked JSONB NOT NULL,
    activation_status VARCHAR(50) DEFAULT 'pending',
    activated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Stocks table
CREATE TABLE IF NOT EXISTS stocks (
    symbol VARCHAR(10) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    market VARCHAR(10) NOT NULL DEFAULT 'NASDAQ',
    current_price DECIMAL(10,4),
    previous_close DECIMAL(10,4),
    volume BIGINT,
    market_cap BIGINT,
    pe_ratio DECIMAL(8,2),
    eps DECIMAL(8,4),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Stock price history table
CREATE TABLE IF NOT EXISTS stock_price_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(10) NOT NULL REFERENCES stocks(symbol) ON DELETE CASCADE,
    price DECIMAL(10,4) NOT NULL,
    volume BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Market data table
CREATE TABLE IF NOT EXISTS market_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) NOT NULL,
    data_type VARCHAR(50) NOT NULL,
    data_value JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    source VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Analytics results
CREATE TABLE IF NOT EXISTS analytics_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL ON DELETE CASCADE,
    symbol VARCHAR(20) NOT NULL,
    analysis_type VARCHAR(100) NOT NULL,
    result JSONB NOT NULL,
    confidence_score DECIMAL(3,2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Notification templates (for system notifications and alerts)
CREATE TABLE IF NOT EXISTS notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    title_template VARCHAR(255) NOT NULL,
    body_template TEXT NOT NULL,
    notification_type VARCHAR(50) NOT NULL, -- 'expiration_warning', 'feature_activation', 'payment_success', 'system_alert'
    channels JSONB DEFAULT '["in_app"]', -- Available channels: in_app, email, push
    variables JSONB DEFAULT '[]', -- Template variables like {user_name}, {expires_at}
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- User notifications (notification queue and history)
CREATE TABLE IF NOT EXISTS user_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL ON DELETE CASCADE,
    template_id UUID REFERENCES notification_templates(id),
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    notification_type VARCHAR(50) NOT NULL,
    channel VARCHAR(50) NOT NULL, -- 'in_app', 'email', 'push'
    priority VARCHAR(20) DEFAULT 'medium', -- 'low', 'medium', 'high', 'urgent'
    status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'sent', 'delivered', 'read', 'failed'
    metadata JSONB DEFAULT '{}', -- Channel-specific metadata
    scheduled_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Background jobs (for scheduled tasks and job monitoring)
CREATE TABLE IF NOT EXISTS background_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(100) NOT NULL, -- 'expiration_check', 'notification_send', 'analytics_update'
    job_name VARCHAR(255) NOT NULL,
    payload JSONB DEFAULT '{}',
    status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'running', 'completed', 'failed', 'cancelled'
    priority INTEGER DEFAULT 0, -- Higher numbers = higher priority
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    scheduled_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    error_message TEXT,
    result JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit logs table (consolidated with all features)
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    event_category VARCHAR(50) DEFAULT 'system_security',
    severity VARCHAR(20) DEFAULT 'medium',
    success BOOLEAN DEFAULT true,
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id UUID REFERENCES sessions(id)
);

-- Performance indexes (idempotent - will not fail if already exist)
CREATE INDEX IF NOT EXISTS idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_feature_payments_user_id ON feature_payments(user_id);
CREATE INDEX IF NOT EXISTS idx_feature_payments_profile_id ON feature_payments(permission_profile_id);
CREATE INDEX IF NOT EXISTS idx_feature_payments_external_id ON feature_payments(external_payment_id);
CREATE INDEX IF NOT EXISTS idx_feature_payments_provider ON feature_payments(payment_provider);
CREATE INDEX IF NOT EXISTS idx_stock_price_history_symbol ON stock_price_history(symbol);
CREATE INDEX IF NOT EXISTS idx_stock_price_history_timestamp ON stock_price_history(timestamp);
CREATE INDEX IF NOT EXISTS idx_user_features_user_id ON user_features(user_id);
CREATE INDEX IF NOT EXISTS idx_user_features_permission_profile_id ON user_features(permission_profile_id);
CREATE INDEX IF NOT EXISTS idx_permission_profiles_category ON permission_profiles(category);
CREATE INDEX IF NOT EXISTS idx_permission_profiles_status ON permission_profiles(status);
CREATE INDEX IF NOT EXISTS idx_profile_variables_profile_id ON profile_variables(profile_id);
CREATE INDEX IF NOT EXISTS idx_admin_assignments_user_id ON admin_permission_profile_assignments(user_id);
CREATE INDEX IF NOT EXISTS idx_admin_assignments_profile_id ON admin_permission_profile_assignments(permission_profile_id);
CREATE INDEX IF NOT EXISTS idx_admin_assignments_assigned_by ON admin_permission_profile_assignments(assigned_by);
CREATE INDEX IF NOT EXISTS idx_assignment_audit_assignment_id ON assignment_audit_log(assignment_id);
CREATE INDEX IF NOT EXISTS idx_assignment_audit_timestamp ON assignment_audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_market_data_symbol_timestamp ON market_data(symbol, timestamp);
CREATE INDEX IF NOT EXISTS idx_analytics_results_user_id ON analytics_results(user_id);
CREATE INDEX IF NOT EXISTS idx_analytics_results_symbol ON analytics_results(symbol);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_category ON audit_logs(event_category);
CREATE INDEX IF NOT EXISTS idx_audit_logs_severity ON audit_logs(severity);
CREATE INDEX IF NOT EXISTS idx_audit_logs_security_filter ON audit_logs(event_category, severity, success, timestamp DESC);

-- Composite index for payment-based auto-assignment optimization
CREATE INDEX IF NOT EXISTS idx_feature_payments_status_expires 
ON feature_payments(activation_status, expires_at) 
WHERE activation_status = 'active';

-- New table indexes for performance optimization
-- User permission profile assignments indexes
CREATE INDEX IF NOT EXISTS idx_user_assignments_user_id ON user_permission_profile_assignments(user_id);
CREATE INDEX IF NOT EXISTS idx_user_assignments_profile_id ON user_permission_profile_assignments(permission_profile_id);
CREATE INDEX IF NOT EXISTS idx_user_assignments_type_source ON user_permission_profile_assignments(assignment_type, assignment_source);
CREATE INDEX IF NOT EXISTS idx_user_assignments_expires_at ON user_permission_profile_assignments(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_assignments_status ON user_permission_profile_assignments(status);

-- Stock ranking package indexes
CREATE INDEX IF NOT EXISTS idx_stock_ranking_packages_profile_id ON stock_ranking_packages(permission_profile_id);
CREATE INDEX IF NOT EXISTS idx_stock_ranking_packages_status ON stock_ranking_packages(status);
CREATE INDEX IF NOT EXISTS idx_user_stock_assignments_user_id ON user_stock_ranking_assignments(user_id);
CREATE INDEX IF NOT EXISTS idx_user_stock_assignments_package_id ON user_stock_ranking_assignments(package_id);
CREATE INDEX IF NOT EXISTS idx_user_stock_assignments_expires_at ON user_stock_ranking_assignments(expires_at) WHERE expires_at IS NOT NULL;

-- Notification system indexes
CREATE INDEX IF NOT EXISTS idx_notification_templates_type ON notification_templates(notification_type);
CREATE INDEX IF NOT EXISTS idx_notification_templates_status ON notification_templates(status);
CREATE INDEX IF NOT EXISTS idx_user_notifications_user_id ON user_notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_user_notifications_status ON user_notifications(status);
CREATE INDEX IF NOT EXISTS idx_user_notifications_type ON user_notifications(notification_type);
CREATE INDEX IF NOT EXISTS idx_user_notifications_channel ON user_notifications(channel);
CREATE INDEX IF NOT EXISTS idx_user_notifications_scheduled_at ON user_notifications(scheduled_at) WHERE scheduled_at IS NOT NULL;

-- Background jobs indexes
CREATE INDEX IF NOT EXISTS idx_background_jobs_status ON background_jobs(status);
CREATE INDEX IF NOT EXISTS idx_background_jobs_type ON background_jobs(job_type);
CREATE INDEX IF NOT EXISTS idx_background_jobs_scheduled_at ON background_jobs(scheduled_at);
CREATE INDEX IF NOT EXISTS idx_background_jobs_priority_scheduled ON background_jobs(priority DESC, scheduled_at ASC) WHERE status = 'pending';

-- Auto assignment rules indexes
CREATE INDEX IF NOT EXISTS idx_auto_assignment_rules_profile_id ON auto_assignment_rules(permission_profile_id);
CREATE INDEX IF NOT EXISTS idx_auto_assignment_rules_trigger_type ON auto_assignment_rules(trigger_type);
CREATE INDEX IF NOT EXISTS idx_auto_assignment_rules_status ON auto_assignment_rules(status);
CREATE INDEX IF NOT EXISTS idx_auto_assignment_rules_priority ON auto_assignment_rules(priority DESC) WHERE status = 'active';

-- Permission profile templates indexes
CREATE INDEX IF NOT EXISTS idx_profile_templates_category ON permission_profile_templates(category);
CREATE INDEX IF NOT EXISTS idx_profile_templates_author_id ON permission_profile_templates(author_id);
CREATE INDEX IF NOT EXISTS idx_profile_templates_status ON permission_profile_templates(status);
CREATE INDEX IF NOT EXISTS idx_profile_templates_approval_status ON permission_profile_templates(approval_status);
CREATE INDEX IF NOT EXISTS idx_profile_templates_public ON permission_profile_templates(is_public) WHERE is_public = true;
CREATE INDEX IF NOT EXISTS idx_profile_templates_rating ON permission_profile_templates(rating_average DESC, rating_count DESC) WHERE is_public = true;

-- Constraints (with conflict handling)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_event_category') THEN
        ALTER TABLE audit_logs ADD CONSTRAINT chk_event_category 
        CHECK (event_category IN (
            'authentication', 
            'authorization', 
            'session_management', 
            'data_access', 
            'system_security', 
            'user_management',
            'permission_assignment',
            'payment_processing',
            'feature_activation',
            'notification_delivery',
            'background_job',
            'analytics_access',
            'template_usage',
            'stock_ranking',
            'api_access',
            'route_access'
        ));
    END IF;
END $$;

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_severity') THEN
        ALTER TABLE audit_logs ADD CONSTRAINT chk_severity 
        CHECK (severity IN ('low', 'medium', 'high', 'critical'));
    END IF;
END $$;


-- Default permission profiles (dynamic feature system with conflict handling)
INSERT INTO permission_profiles (name, description, category, profile_data, pricing_tier, auto_assignment_rules, api_endpoints, frontend_routes) VALUES
('Bronze Analytics', 'Basic EPS analysis and market data', 'analytics', 
 '{"features": ["basic_charts", "eps_analysis", "market_data_view"], "limits": {"api_calls": 100, "analysis_per_day": 10}}',
 '{"tier": "Bronze", "price_usd": 0, "crypto_price": 0}',
 '{"package_tier": ["Bronze"], "auto_assign": true}',
 '{"allowed": ["/api/v1/market-data/basic", "/api/v1/analytics/eps"], "rate_limits": {"per_minute": 10, "per_hour": 100}}',
 '{"allowed": ["/dashboard", "/analytics/basic"], "blocked": ["/admin", "/analytics/premium"]}'
),
('Silver Analytics', 'Enhanced analytics with pattern recognition', 'analytics',
 '{"features": ["basic_charts", "eps_analysis", "pattern_recognition", "alerts"], "limits": {"api_calls": 500, "analysis_per_day": 50}}',
 '{"tier": "Silver", "price_usd": 29.99, "crypto_price": 30}',
 '{"package_tier": ["Silver"], "auto_assign": true}',
 '{"allowed": ["/api/v1/market-data/*", "/api/v1/analytics/*", "/api/v1/alerts/basic"], "rate_limits": {"per_minute": 50, "per_hour": 500}}',
 '{"allowed": ["/dashboard", "/analytics/*", "/alerts"], "blocked": ["/admin", "/analytics/ai-insights"]}'
),
('Gold Premium', 'Full analytics suite with AI insights', 'premium',
 '{"features": ["advanced_charts", "ai_insights", "real_time_data", "custom_alerts", "export_data"], "limits": {"api_calls": 2000, "unlimited_analysis": true}}',
 '{"tier": "Gold", "price_usd": 99.99, "crypto_price": 100}',
 '{"package_tier": ["Gold"], "auto_assign": true}',
 '{"allowed": ["/api/v1/*"], "rate_limits": {"per_minute": 200, "per_hour": 2000}}',
 '{"allowed": ["/*"], "blocked": ["/admin"]}'
),
('Admin Dashboard', 'Administrative interface access', 'admin',
 '{"features": ["user_management", "permission_profiles", "system_monitoring", "audit_logs", "bulk_operations"], "limits": {}}',
 '{"tier": "Admin", "price_usd": 0, "crypto_price": 0}',
 '{"admin_only": true, "auto_assign": false}',
 '{"allowed": ["/api/admin/*", "/api/v1/*"], "rate_limits": {"unlimited": true}}',
 '{"allowed": ["/*"], "blocked": []}'
)
ON CONFLICT (name) DO NOTHING;

-- Update timestamp function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at (with conflict handling)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_users_updated_at') THEN
        CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_permission_profiles_updated_at') THEN
        CREATE TRIGGER update_permission_profiles_updated_at BEFORE UPDATE ON permission_profiles
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_stock_ranking_packages_updated_at') THEN
        CREATE TRIGGER update_stock_ranking_packages_updated_at BEFORE UPDATE ON stock_ranking_packages
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_notification_templates_updated_at') THEN
        CREATE TRIGGER update_notification_templates_updated_at BEFORE UPDATE ON notification_templates
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_background_jobs_updated_at') THEN
        CREATE TRIGGER update_background_jobs_updated_at BEFORE UPDATE ON background_jobs
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_auto_assignment_rules_updated_at') THEN
        CREATE TRIGGER update_auto_assignment_rules_updated_at BEFORE UPDATE ON auto_assignment_rules
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_profile_templates_updated_at') THEN
        CREATE TRIGGER update_profile_templates_updated_at BEFORE UPDATE ON permission_profile_templates
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
    
END $$;