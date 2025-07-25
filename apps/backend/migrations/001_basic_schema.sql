-- Basic schema for testing auto-migration functionality
-- Users table (core user data - Firebase UID based)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) UNIQUE NOT NULL,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Dynamic permission profiles (core feature system)
CREATE TABLE IF NOT EXISTS permission_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    version VARCHAR(20) NOT NULL DEFAULT '1.0',
    status VARCHAR(50) DEFAULT 'active',
    profile_data JSONB NOT NULL,
    pricing_tier JSONB DEFAULT '{}',
    auto_assignment_rules JSONB DEFAULT '{}',
    api_endpoints JSONB DEFAULT '{}',
    frontend_routes JSONB DEFAULT '{}',
    compliance_level VARCHAR(50) DEFAULT 'educational',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- Admin permission profile assignments (direct admin assignments)
CREATE TABLE IF NOT EXISTS admin_permission_profile_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    permission_profile_id UUID REFERENCES permission_profiles(id) ON DELETE CASCADE,
    assigned_by UUID REFERENCES users(id),
    assignment_type VARCHAR(50) NOT NULL,
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
    performed_by UUID REFERENCES users(id),
    details JSONB NOT NULL,
    timestamp TIMESTAMPTZ DEFAULT NOW()
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

-- Audit logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES users(id),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    result VARCHAR(50) DEFAULT 'success',
    event_category VARCHAR(50) DEFAULT 'system_security',
    severity VARCHAR(20) DEFAULT 'medium',
    success BOOLEAN DEFAULT true,
    details JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    client_ip INET,
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id UUID REFERENCES sessions(id)
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_admin_assignments_user_id ON admin_permission_profile_assignments(user_id);
CREATE INDEX IF NOT EXISTS idx_admin_assignments_profile_id ON admin_permission_profile_assignments(permission_profile_id);
CREATE INDEX IF NOT EXISTS idx_assignment_audit_assignment_id ON assignment_audit_log(assignment_id);
CREATE INDEX IF NOT EXISTS idx_assignment_audit_timestamp ON assignment_audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);

-- Default permission profiles
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
('Admin Dashboard', 'Administrative interface access', 'admin',
 '{"features": ["user_management", "permission_profiles", "system_monitoring", "audit_logs", "bulk_operations"], "limits": {}}',
 '{"tier": "Admin", "price_usd": 0, "crypto_price": 0}',
 '{"admin_only": true, "auto_assign": false}',
 '{"allowed": ["/api/admin/*", "/api/v1/*"], "rate_limits": {"unlimited": true}}',
 '{"allowed": ["/*"], "blocked": []}'
)
ON CONFLICT (name) DO NOTHING;