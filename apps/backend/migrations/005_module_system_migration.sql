-- Module System Migration - Complete IAM Restructure
-- This migration introduces a module-based IAM system replacing permission profiles

-- ========================================
-- PHASE 1: CREATE NEW MODULE SYSTEM TABLES
-- ========================================

-- Sub-Module Definitions
-- Core table defining available modules in the system
CREATE TABLE IF NOT EXISTS sub_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL, -- 'analytics', 'trading', 'data', 'reporting'
    icon VARCHAR(100), -- UI icon reference
    
    -- API and UI Configuration
    api_endpoints JSONB NOT NULL DEFAULT '{}', -- Allowed endpoints and patterns
    ui_components JSONB NOT NULL DEFAULT '{}', -- Frontend components to show/hide
    feature_flags JSONB NOT NULL DEFAULT '{}', -- Feature toggles within module
    
    -- Access Level Configurations
    access_levels JSONB NOT NULL DEFAULT '{}', -- bronze/silver/gold/platinum configs per module
    default_quotas JSONB NOT NULL DEFAULT '{}', -- Default quotas for each access level
    pricing_tiers JSONB NOT NULL DEFAULT '{}', -- Pricing information per tier
    
    -- Dependencies and Relationships
    dependencies JSONB DEFAULT '[]', -- Required modules for this module to work
    conflicts JSONB DEFAULT '[]', -- Modules that conflict with this one
    
    -- Module Status and Lifecycle
    status VARCHAR(50) DEFAULT 'active', -- active, deprecated, maintenance
    version VARCHAR(20) DEFAULT '1.0',
    min_package_tier VARCHAR(50), -- Minimum user package tier required
    
    -- Audit Fields
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- User Sub-Module Assignments
-- Replaces admin_permission_profile_assignments with module-specific assignments
CREATE TABLE IF NOT EXISTS user_sub_module_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Core Assignment Data
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sub_module_id UUID NOT NULL REFERENCES sub_modules(id) ON DELETE CASCADE,
    
    -- Access Configuration
    access_level VARCHAR(50) NOT NULL, -- bronze, silver, gold, platinum, enterprise
    custom_quotas JSONB NOT NULL DEFAULT '{}', -- Override default quotas if needed
    restrictions JSONB DEFAULT '{}', -- IP restrictions, time limits, etc.
    
    -- Assignment Metadata
    assigned_by UUID NOT NULL REFERENCES users(id),
    assignment_reason TEXT NOT NULL,
    assignment_type VARCHAR(50) NOT NULL DEFAULT 'manual', -- manual, auto, subscription
    
    -- Lifecycle Management
    starts_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- NULL = never expires
    status VARCHAR(50) DEFAULT 'active', -- active, suspended, expired, revoked
    
    -- Usage Tracking
    first_used_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    usage_count INTEGER DEFAULT 0,
    
    -- Audit Fields
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(user_id, sub_module_id) -- One assignment per user per module
);

-- Third-Party API Keys
-- New system for external API access with module-specific scoping
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Key Identity
    key_hash VARCHAR(255) UNIQUE NOT NULL, -- Hashed API key
    key_prefix VARCHAR(20) NOT NULL, -- First few chars for identification (e.g., "ak_1234...")
    
    -- Client Information
    client_name VARCHAR(255) NOT NULL,
    client_description TEXT,
    client_contact_email VARCHAR(255),
    client_website VARCHAR(255),
    
    -- Access Configuration
    allowed_modules JSONB NOT NULL, -- Array of {module_id, access_level, custom_quotas}
    rate_limits JSONB NOT NULL DEFAULT '{}', -- Per-module rate limits
    permissions JSONB NOT NULL DEFAULT '{}', -- Specific permissions per module
    
    -- Security Configuration
    ip_restrictions JSONB DEFAULT '[]', -- Allowed IP addresses/ranges
    allowed_domains JSONB DEFAULT '[]', -- CORS domains
    allowed_user_agents JSONB DEFAULT '[]', -- Restrict by user agent if needed
    require_https BOOLEAN DEFAULT true,
    
    -- Lifecycle Management
    starts_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- NULL = never expires
    status VARCHAR(50) DEFAULT 'active', -- active, suspended, revoked, expired
    
    -- Usage Tracking
    first_used_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    total_requests INTEGER DEFAULT 0,
    usage_stats JSONB DEFAULT '{}', -- Detailed usage statistics
    
    -- Management
    created_by UUID NOT NULL REFERENCES users(id),
    managed_by UUID REFERENCES users(id), -- Who can manage this key
    notes TEXT,
    
    -- Audit Fields
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Module Usage Logs
-- Comprehensive logging for billing, analytics, and monitoring
CREATE TABLE IF NOT EXISTS module_usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Request Identity
    user_id UUID REFERENCES users(id), -- NULL for API key requests
    api_key_id UUID REFERENCES api_keys(id), -- NULL for user requests
    sub_module_id UUID NOT NULL REFERENCES sub_modules(id),
    
    -- Request Details
    endpoint VARCHAR(500) NOT NULL,
    request_method VARCHAR(10) NOT NULL,
    response_status INTEGER,
    response_time_ms INTEGER,
    
    -- Usage Tracking
    quota_consumed INTEGER DEFAULT 1,
    quota_type VARCHAR(100), -- 'api_calls', 'data_points', 'analysis_runs', etc.
    
    -- Request Metadata
    client_ip INET,
    user_agent TEXT,
    request_id VARCHAR(100), -- For request tracing
    session_id UUID,
    
    -- Data Context
    request_size_bytes INTEGER,
    response_size_bytes INTEGER,
    cache_hit BOOLEAN DEFAULT false,
    
    -- Billing and Analytics
    billable BOOLEAN DEFAULT true,
    cost_units DECIMAL(10,4) DEFAULT 0, -- Cost calculation for billing
    
    -- Timestamp
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    
    -- Additional context for debugging/analytics
    request_metadata JSONB DEFAULT '{}'
);

-- Module Assignment Audit Log
-- Track all changes to module assignments for compliance
CREATE TABLE IF NOT EXISTS module_assignment_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- What Changed
    assignment_id UUID REFERENCES user_sub_module_assignments(id) ON DELETE SET NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    sub_module_id UUID NOT NULL REFERENCES sub_modules(id),
    
    -- Change Details
    action VARCHAR(50) NOT NULL, -- created, updated, suspended, revoked, expired
    old_values JSONB, -- Previous state
    new_values JSONB, -- New state
    changes JSONB, -- Specific fields that changed
    
    -- Who and Why
    performed_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    
    -- Context
    session_id UUID,
    client_ip INET,
    user_agent TEXT,
    
    -- Timestamp
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- ========================================
-- INDEXES FOR PERFORMANCE
-- ========================================

-- Sub-modules indexes
CREATE INDEX IF NOT EXISTS idx_sub_modules_category ON sub_modules(category);
CREATE INDEX IF NOT EXISTS idx_sub_modules_status ON sub_modules(status);
CREATE INDEX IF NOT EXISTS idx_sub_modules_created_at ON sub_modules(created_at);

-- User assignments indexes
CREATE INDEX IF NOT EXISTS idx_user_assignments_user_id ON user_sub_module_assignments(user_id);
CREATE INDEX IF NOT EXISTS idx_user_assignments_module_id ON user_sub_module_assignments(sub_module_id);
CREATE INDEX IF NOT EXISTS idx_user_assignments_status ON user_sub_module_assignments(status);
CREATE INDEX IF NOT EXISTS idx_user_assignments_expires_at ON user_sub_module_assignments(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_assignments_assigned_by ON user_sub_module_assignments(assigned_by);

-- API keys indexes
CREATE INDEX IF NOT EXISTS idx_api_keys_status ON api_keys(status);
CREATE INDEX IF NOT EXISTS idx_api_keys_created_by ON api_keys(created_by);
CREATE INDEX IF NOT EXISTS idx_api_keys_expires_at ON api_keys(expires_at);
CREATE INDEX IF NOT EXISTS idx_api_keys_last_used ON api_keys(last_used_at);

-- Usage logs indexes (critical for analytics and billing)
CREATE INDEX IF NOT EXISTS idx_usage_logs_user_id_timestamp ON module_usage_logs(user_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_usage_logs_api_key_timestamp ON module_usage_logs(api_key_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_usage_logs_module_timestamp ON module_usage_logs(sub_module_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_usage_logs_timestamp ON module_usage_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_usage_logs_billable ON module_usage_logs(billable, timestamp);

-- Audit logs indexes
CREATE INDEX IF NOT EXISTS idx_assignment_audit_user_id ON module_assignment_audit(user_id);
CREATE INDEX IF NOT EXISTS idx_assignment_audit_timestamp ON module_assignment_audit(timestamp);
CREATE INDEX IF NOT EXISTS idx_assignment_audit_performed_by ON module_assignment_audit(performed_by);

-- ========================================
-- SEED DATA: INITIAL MODULE DEFINITIONS
-- ========================================

INSERT INTO sub_modules (
    name, display_name, description, category, icon,
    api_endpoints, ui_components, feature_flags, 
    access_levels, default_quotas, pricing_tiers,
    dependencies, status
) VALUES 
-- Stock Ranking Module
(
    'stock-ranking',
    'Stock Ranking & Analysis',
    'Advanced stock ranking algorithms with EPS analysis, market cap filtering, and AI-powered insights',
    'analytics',
    'trending-up',
    '{
        "patterns": ["/api/v1/stock-ranking/*", "/api/v1/rankings/*"],
        "endpoints": {
            "/api/v1/rankings/top-performers": {"methods": ["GET"], "quota_cost": 1},
            "/api/v1/rankings/by-eps": {"methods": ["GET"], "quota_cost": 2},
            "/api/v1/rankings/custom": {"methods": ["POST"], "quota_cost": 3},
            "/api/v1/rankings/export": {"methods": ["POST"], "quota_cost": 5}
        }
    }',
    '{
        "components": ["StockRankingDashboard", "RankingTable", "RankingFilters", "ExportButton"],
        "routes": ["/dashboard/rankings", "/analytics/stocks"]
    }',
    '{
        "realTimeUpdates": {"bronze": false, "silver": true, "gold": true, "platinum": true},
        "customAlgorithms": {"bronze": false, "silver": false, "gold": true, "platinum": true},
        "aiInsights": {"bronze": false, "silver": true, "gold": true, "platinum": true},
        "historicalData": {"bronze": false, "silver": true, "gold": true, "platinum": true}
    }',
    '{
        "bronze": {"name": "Basic Rankings", "description": "Top 10 rankings, basic filters"},
        "silver": {"name": "Enhanced Rankings", "description": "Top 50 rankings, AI insights, alerts"},
        "gold": {"name": "Professional Rankings", "description": "Top 100 rankings, custom algorithms, full history"},
        "platinum": {"name": "Enterprise Rankings", "description": "Unlimited rankings, custom models, real-time feeds"}
    }',
    '{
        "bronze": {"api_calls": 50, "rankings_per_request": 10, "exports_per_day": 2, "rate_limit_per_minute": 10},
        "silver": {"api_calls": 200, "rankings_per_request": 50, "exports_per_day": 10, "rate_limit_per_minute": 50},
        "gold": {"api_calls": 1000, "rankings_per_request": 100, "exports_per_day": 50, "rate_limit_per_minute": 200},
        "platinum": {"api_calls": -1, "rankings_per_request": -1, "exports_per_day": -1, "rate_limit_per_minute": 500}
    }',
    '{
        "bronze": {"monthly_usd": 0, "per_request": 0},
        "silver": {"monthly_usd": 29.99, "per_request": 0.01},
        "gold": {"monthly_usd": 99.99, "per_request": 0.005},
        "platinum": {"monthly_usd": 299.99, "per_request": 0.001}
    }',
    '[]',
    'active'
),

-- Portfolio Analysis Module
(
    'portfolio-analysis',
    'Portfolio Analysis Tools',
    'Comprehensive portfolio analysis with risk metrics, performance tracking, and benchmarking',
    'analytics',
    'pie-chart',
    '{
        "patterns": ["/api/v1/portfolio/*"],
        "endpoints": {
            "/api/v1/portfolio/analyze": {"methods": ["POST"], "quota_cost": 3},
            "/api/v1/portfolio/risk-metrics": {"methods": ["GET"], "quota_cost": 2},
            "/api/v1/portfolio/performance": {"methods": ["GET"], "quota_cost": 1},
            "/api/v1/portfolio/benchmark": {"methods": ["POST"], "quota_cost": 4}
        }
    }',
    '{
        "components": ["PortfolioDashboard", "RiskAnalysis", "PerformanceChart", "BenchmarkTable"],
        "routes": ["/portfolio", "/analytics/portfolio"]
    }',
    '{
        "riskAnalysis": {"bronze": false, "silver": true, "gold": true, "platinum": true},
        "performanceTracking": {"bronze": true, "silver": true, "gold": true, "platinum": true},
        "benchmarking": {"bronze": false, "silver": false, "gold": true, "platinum": true},
        "alertSystem": {"bronze": false, "silver": true, "gold": true, "platinum": true}
    }',
    '{
        "bronze": {"name": "Basic Portfolio", "description": "Simple performance tracking"},
        "silver": {"name": "Risk-Aware Portfolio", "description": "Risk analysis and alerts"},
        "gold": {"name": "Professional Portfolio", "description": "Full analysis with benchmarking"},
        "platinum": {"name": "Institutional Portfolio", "description": "Advanced models and unlimited analysis"}
    }',
    '{
        "bronze": {"api_calls": 20, "analyses_per_day": 5, "portfolios": 1, "rate_limit_per_minute": 5},
        "silver": {"api_calls": 100, "analyses_per_day": 25, "portfolios": 3, "rate_limit_per_minute": 20},
        "gold": {"api_calls": 500, "analyses_per_day": 100, "portfolios": 10, "rate_limit_per_minute": 100},
        "platinum": {"api_calls": -1, "analyses_per_day": -1, "portfolios": -1, "rate_limit_per_minute": 300}
    }',
    '{
        "bronze": {"monthly_usd": 0, "per_analysis": 0},
        "silver": {"monthly_usd": 19.99, "per_analysis": 0.5},
        "gold": {"monthly_usd": 79.99, "per_analysis": 0.2},
        "platinum": {"monthly_usd": 199.99, "per_analysis": 0.05}
    }',
    '[]',
    'active'
),

-- Market Data Module
(
    'market-data',
    'Real-Time Market Data',
    'Live market data feeds with historical data, technical indicators, and market alerts',
    'data',
    'activity',
    '{
        "patterns": ["/api/v1/market-data/*"],
        "endpoints": {
            "/api/v1/market-data/quotes": {"methods": ["GET"], "quota_cost": 1},
            "/api/v1/market-data/historical": {"methods": ["GET"], "quota_cost": 2},
            "/api/v1/market-data/indicators": {"methods": ["GET"], "quota_cost": 3},
            "/api/v1/market-data/alerts": {"methods": ["POST", "GET"], "quota_cost": 1}
        }
    }',
    '{
        "components": ["MarketDataFeed", "PriceChart", "TechnicalIndicators", "AlertsPanel"],
        "routes": ["/market-data", "/data/live"]
    }',
    '{
        "realTimeData": {"bronze": false, "silver": true, "gold": true, "platinum": true},
        "historicalData": {"bronze": "1month", "silver": "6months", "gold": "2years", "platinum": "unlimited"},
        "technicalIndicators": {"bronze": "basic", "silver": "standard", "gold": "advanced", "platinum": "all"},
        "marketAlerts": {"bronze": false, "silver": true, "gold": true, "platinum": true}
    }',
    '{
        "bronze": {"name": "Basic Data", "description": "Delayed quotes, limited history"},
        "silver": {"name": "Live Data", "description": "Real-time quotes, 6-month history"},
        "gold": {"name": "Professional Data", "description": "Full real-time feed, 2-year history"},
        "platinum": {"name": "Institutional Data", "description": "Premium feeds, unlimited history"}
    }',
    '{
        "bronze": {"api_calls": 100, "symbols": 10, "alerts": 0, "rate_limit_per_minute": 20},
        "silver": {"api_calls": 500, "symbols": 50, "alerts": 5, "rate_limit_per_minute": 100},
        "gold": {"api_calls": 2000, "symbols": 200, "alerts": 25, "rate_limit_per_minute": 400},
        "platinum": {"api_calls": -1, "symbols": -1, "alerts": -1, "rate_limit_per_minute": 1000}
    }',
    '{
        "bronze": {"monthly_usd": 0, "per_request": 0},
        "silver": {"monthly_usd": 39.99, "per_request": 0.001},
        "gold": {"monthly_usd": 149.99, "per_request": 0.0005},
        "platinum": {"monthly_usd": 499.99, "per_request": 0.0001}
    }',
    '[]',
    'active'
),

-- Trading Signals Module
(
    'trading-signals',
    'AI Trading Signals',
    'Machine learning-powered trading signals with backtesting and strategy optimization',
    'trading',
    'zap',
    '{
        "patterns": ["/api/v1/trading-signals/*", "/api/v1/strategies/*"],
        "endpoints": {
            "/api/v1/trading-signals/generate": {"methods": ["POST"], "quota_cost": 5},
            "/api/v1/trading-signals/backtest": {"methods": ["POST"], "quota_cost": 10},
            "/api/v1/strategies/optimize": {"methods": ["POST"], "quota_cost": 15},
            "/api/v1/trading-signals/live": {"methods": ["GET"], "quota_cost": 3}
        }
    }',
    '{
        "components": ["TradingSignals", "BacktestResults", "StrategyBuilder", "LiveSignals"],
        "routes": ["/trading", "/signals", "/strategies"]
    }',
    '{
        "aiSignals": {"bronze": false, "silver": true, "gold": true, "platinum": true},
        "backtesting": {"bronze": false, "silver": false, "gold": true, "platinum": true},
        "strategyOptimization": {"bronze": false, "silver": false, "gold": false, "platinum": true},
        "liveSignals": {"bronze": false, "silver": true, "gold": true, "platinum": true}
    }',
    '{
        "bronze": {"name": "Basic Signals", "description": "Simple technical signals"},
        "silver": {"name": "AI Signals", "description": "Machine learning signals, basic backtesting"},
        "gold": {"name": "Professional Signals", "description": "Advanced AI, full backtesting suite"},
        "platinum": {"name": "Institutional Signals", "description": "Custom models, strategy optimization"}
    }',
    '{
        "bronze": {"api_calls": 10, "signals_per_day": 5, "backtests": 0, "rate_limit_per_minute": 2},
        "silver": {"api_calls": 50, "signals_per_day": 25, "backtests": 3, "rate_limit_per_minute": 10},
        "gold": {"api_calls": 200, "signals_per_day": 100, "backtests": 20, "rate_limit_per_minute": 50},
        "platinum": {"api_calls": -1, "signals_per_day": -1, "backtests": -1, "rate_limit_per_minute": 200}
    }',
    '{
        "bronze": {"monthly_usd": 0, "per_signal": 0},
        "silver": {"monthly_usd": 79.99, "per_signal": 1.0},
        "gold": {"monthly_usd": 199.99, "per_signal": 0.5},
        "platinum": {"monthly_usd": 599.99, "per_signal": 0.1}
    }',
    '["market-data"]',
    'active'
)
ON CONFLICT (name) DO NOTHING;

-- ========================================
-- MIGRATION VALIDATION VIEWS
-- ========================================

-- View to see module assignments with details
CREATE OR REPLACE VIEW user_module_access AS
SELECT 
    u.id as user_id,
    u.email,
    sm.name as module_name,
    sm.display_name,
    uma.access_level,
    uma.status,
    uma.expires_at,
    uma.assigned_by,
    uma.assignment_reason,
    uma.created_at as assigned_at
FROM users u
JOIN user_sub_module_assignments uma ON u.id = uma.user_id
JOIN sub_modules sm ON uma.sub_module_id = sm.id
WHERE uma.status = 'active';

-- View for API key access summary
CREATE OR REPLACE VIEW api_key_access AS
SELECT 
    ak.id as api_key_id,
    ak.key_prefix,
    ak.client_name,
    ak.status,
    ak.allowed_modules,
    ak.total_requests,
    ak.last_used_at,
    ak.created_at
FROM api_keys ak
WHERE ak.status = 'active';

-- Usage statistics view
CREATE OR REPLACE VIEW module_usage_summary AS
SELECT 
    sm.name as module_name,
    sm.display_name,
    COUNT(DISTINCT mul.user_id) as active_users,
    COUNT(DISTINCT mul.api_key_id) as active_api_keys,
    COUNT(*) as total_requests,
    SUM(mul.quota_consumed) as total_quota_consumed,
    AVG(mul.response_time_ms) as avg_response_time,
    DATE_TRUNC('day', mul.timestamp) as date
FROM sub_modules sm
LEFT JOIN module_usage_logs mul ON sm.id = mul.sub_module_id
WHERE mul.timestamp >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY sm.id, sm.name, sm.display_name, DATE_TRUNC('day', mul.timestamp)
ORDER BY date DESC, total_requests DESC;