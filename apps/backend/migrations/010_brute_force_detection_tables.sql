-- Create comprehensive brute force detection and IP blocking tables
-- Migration: 009_brute_force_detection_tables.sql
-- Purpose: Advanced threat detection and blocking capabilities

-- Table for tracking individual attack attempts and patterns
CREATE TABLE IF NOT EXISTS attack_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL,
    user_id VARCHAR(255), -- nullable for failed login attempts
    attempt_type VARCHAR(50) NOT NULL, -- 'brute_force', 'password_spray', 'credential_stuffing', 'slow_brute'
    target_endpoint VARCHAR(255) NOT NULL,
    http_method VARCHAR(10) NOT NULL,
    user_agent TEXT,
    session_id VARCHAR(255),
    device_fingerprint TEXT,
    geolocation JSONB, -- {country_code, region, city, lat, lng}
    success BOOLEAN NOT NULL DEFAULT false,
    response_code INTEGER,
    response_time_ms INTEGER,
    request_payload_hash VARCHAR(64), -- hash of request data for pattern detection
    correlation_id VARCHAR(255), -- for tracking distributed attacks
    risk_score DECIMAL(4,2) DEFAULT 0.0, -- 0-10.0 scale
    metadata JSONB, -- additional context data
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table for IP blocking management
CREATE TABLE IF NOT EXISTS blocked_ips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL UNIQUE,
    ip_range CIDR, -- for subnet blocking
    block_type VARCHAR(50) NOT NULL, -- 'temporary', 'permanent', 'whitelist', 'blacklist'
    blocking_reason VARCHAR(255) NOT NULL,
    threat_level VARCHAR(20) NOT NULL, -- 'low', 'medium', 'high', 'critical'
    auto_blocked BOOLEAN NOT NULL DEFAULT true,
    blocked_by VARCHAR(255), -- user_id or 'system'
    blocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- null for permanent blocks
    unblocked_at TIMESTAMPTZ,
    unblocked_by VARCHAR(255),
    unblock_reason VARCHAR(255),
    metadata JSONB, -- additional blocking context
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table for tracking attack patterns and campaigns
CREATE TABLE IF NOT EXISTS attack_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_id VARCHAR(255) NOT NULL UNIQUE,
    pattern_type VARCHAR(50) NOT NULL, -- 'distributed_brute', 'password_spray', 'geographic_anomaly'
    source_ips INET[], -- array of involved IPs
    target_accounts TEXT[], -- array of targeted accounts
    attack_vector VARCHAR(100) NOT NULL,
    detection_algorithm VARCHAR(50) NOT NULL,
    confidence_score DECIMAL(4,2) NOT NULL, -- 0-10.0 confidence in detection
    severity VARCHAR(20) NOT NULL, -- 'low', 'medium', 'high', 'critical'
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- 'active', 'mitigated', 'resolved'
    first_detected TIMESTAMPTZ NOT NULL,
    last_detected TIMESTAMPTZ NOT NULL,
    total_attempts INTEGER NOT NULL DEFAULT 1,
    successful_attempts INTEGER NOT NULL DEFAULT 0,
    affected_users INTEGER NOT NULL DEFAULT 0,
    geographic_spread JSONB, -- countries/regions involved
    time_pattern JSONB, -- timing analysis data
    response_actions JSONB, -- automated responses taken
    analyst_notes TEXT,
    resolved_at TIMESTAMPTZ,
    resolved_by VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table for IP reputation and threat intelligence
CREATE TABLE IF NOT EXISTS ip_reputation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL UNIQUE,
    reputation_score DECIMAL(4,2) NOT NULL DEFAULT 5.0, -- 0-10.0 scale (5.0 = neutral)
    threat_level VARCHAR(20) NOT NULL DEFAULT 'unknown',
    is_malicious BOOLEAN NOT NULL DEFAULT false,
    is_vpn BOOLEAN NOT NULL DEFAULT false,
    is_proxy BOOLEAN NOT NULL DEFAULT false,
    is_tor BOOLEAN NOT NULL DEFAULT false,
    is_hosting_provider BOOLEAN NOT NULL DEFAULT false,
    country_code CHAR(2),
    region VARCHAR(100),
    city VARCHAR(100),
    organization VARCHAR(255),
    isp VARCHAR(255),
    asn INTEGER,
    threat_categories TEXT[], -- array of threat types
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_requests INTEGER NOT NULL DEFAULT 0,
    malicious_requests INTEGER NOT NULL DEFAULT 0,
    blocked_requests INTEGER NOT NULL DEFAULT 0,
    threat_intel_sources JSONB, -- external threat intel data
    whitelisted BOOLEAN NOT NULL DEFAULT false,
    whitelist_reason VARCHAR(255),
    blacklisted BOOLEAN NOT NULL DEFAULT false,
    blacklist_reason VARCHAR(255),
    metadata JSONB,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table for tracking rate limiting and thresholds
CREATE TABLE IF NOT EXISTS rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identifier VARCHAR(255) NOT NULL, -- IP or user_id
    identifier_type VARCHAR(20) NOT NULL, -- 'ip', 'user', 'session'
    endpoint_pattern VARCHAR(255) NOT NULL,
    time_window_seconds INTEGER NOT NULL,
    max_requests INTEGER NOT NULL,
    current_requests INTEGER NOT NULL DEFAULT 0,
    window_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    exceeded_at TIMESTAMPTZ,
    reset_at TIMESTAMPTZ,
    penalty_multiplier DECIMAL(4,2) DEFAULT 1.0,
    adaptive_threshold BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table for user account security states
CREATE TABLE IF NOT EXISTS account_security (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL,
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    successful_logins INTEGER NOT NULL DEFAULT 0,
    last_failed_login TIMESTAMPTZ,
    last_successful_login TIMESTAMPTZ,
    account_locked BOOLEAN NOT NULL DEFAULT false,
    lock_reason VARCHAR(255),
    locked_at TIMESTAMPTZ,
    locked_until TIMESTAMPTZ,
    unlock_attempts INTEGER NOT NULL DEFAULT 0,
    password_spray_detected BOOLEAN NOT NULL DEFAULT false,
    geographic_anomaly_score DECIMAL(4,2) DEFAULT 0.0,
    device_anomaly_score DECIMAL(4,2) DEFAULT 0.0,
    behavioral_anomaly_score DECIMAL(4,2) DEFAULT 0.0,
    risk_profile VARCHAR(20) NOT NULL DEFAULT 'normal', -- 'low', 'normal', 'elevated', 'high'
    mfa_required BOOLEAN NOT NULL DEFAULT false,
    captcha_required BOOLEAN NOT NULL DEFAULT false,
    known_ips INET[],
    known_countries CHAR(2)[],
    known_devices JSONB,
    security_events_count INTEGER NOT NULL DEFAULT 0,
    last_security_event TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for high-performance queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_attempts_ip_created ON attack_attempts(ip_address, created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_attempts_user_created ON attack_attempts(user_id, created_at DESC) WHERE user_id IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_attempts_type_created ON attack_attempts(attempt_type, created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_attempts_endpoint ON attack_attempts(target_endpoint);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_attempts_risk ON attack_attempts(risk_score DESC) WHERE risk_score > 5.0;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_attempts_geo ON attack_attempts USING GIN(geolocation);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_blocked_ips_address ON blocked_ips(ip_address);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_blocked_ips_range ON blocked_ips USING GIST(ip_range) WHERE ip_range IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_blocked_ips_type_expires ON blocked_ips(block_type, expires_at);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_blocked_ips_active ON blocked_ips(blocked_at, expires_at) WHERE unblocked_at IS NULL;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_patterns_type_status ON attack_patterns(pattern_type, status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_patterns_detected ON attack_patterns(last_detected DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_patterns_severity ON attack_patterns(severity, confidence_score DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ip_reputation_address ON ip_reputation(ip_address);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ip_reputation_score ON ip_reputation(reputation_score);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ip_reputation_malicious ON ip_reputation(is_malicious, threat_level) WHERE is_malicious = true;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ip_reputation_vpn_proxy ON ip_reputation(is_vpn, is_proxy) WHERE is_vpn = true OR is_proxy = true;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_rate_limits_identifier ON rate_limits(identifier, endpoint_pattern);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_rate_limits_window ON rate_limits(window_start, time_window_seconds);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_rate_limits_exceeded ON rate_limits(exceeded_at) WHERE exceeded_at IS NOT NULL;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_account_security_user ON account_security(user_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_account_security_locked ON account_security(account_locked, locked_until) WHERE account_locked = true;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_account_security_risk ON account_security(risk_profile) WHERE risk_profile != 'normal';

-- Create composite indexes for complex queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_attempts_ip_time_type ON attack_attempts(ip_address, created_at, attempt_type);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_attack_patterns_source_ips ON attack_patterns USING GIN(source_ips);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ip_reputation_country ON ip_reputation(country_code) WHERE country_code IS NOT NULL;

-- Add row-level security policies (optional, for multi-tenant scenarios)
-- ALTER TABLE attack_attempts ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE blocked_ips ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE attack_patterns ENABLE ROW LEVEL SECURITY;

-- Create function to automatically update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers for automatic timestamp updates
CREATE TRIGGER update_attack_attempts_updated_at BEFORE UPDATE ON attack_attempts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_blocked_ips_updated_at BEFORE UPDATE ON blocked_ips FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_attack_patterns_updated_at BEFORE UPDATE ON attack_patterns FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ip_reputation_updated_at BEFORE UPDATE ON ip_reputation FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_rate_limits_updated_at BEFORE UPDATE ON rate_limits FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_account_security_updated_at BEFORE UPDATE ON account_security FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create materialized views for analytics and reporting
CREATE MATERIALIZED VIEW IF NOT EXISTS attack_summary_hourly AS
SELECT 
    DATE_TRUNC('hour', created_at) as time_window,
    attempt_type,
    COUNT(*) as attempt_count,
    COUNT(DISTINCT ip_address) as unique_ips,
    COUNT(DISTINCT user_id) as affected_users,
    AVG(risk_score) as avg_risk_score,
    COUNT(*) FILTER (WHERE success = true) as successful_attempts,
    COUNT(*) FILTER (WHERE risk_score > 7.0) as high_risk_attempts
FROM attack_attempts 
GROUP BY DATE_TRUNC('hour', created_at), attempt_type;

CREATE UNIQUE INDEX ON attack_summary_hourly (time_window, attempt_type);

CREATE MATERIALIZED VIEW IF NOT EXISTS ip_threat_summary AS
SELECT 
    ir.ip_address,
    ir.reputation_score,
    ir.country_code,
    ir.is_malicious,
    ir.is_vpn,
    COALESCE(aa.recent_attempts, 0) as recent_attempts,
    COALESCE(bi.is_blocked, false) as currently_blocked,
    ir.last_seen
FROM ip_reputation ir
LEFT JOIN (
    SELECT ip_address, COUNT(*) as recent_attempts
    FROM attack_attempts 
    WHERE created_at > NOW() - INTERVAL '24 hours'
    GROUP BY ip_address
) aa ON ir.ip_address = aa.ip_address
LEFT JOIN (
    SELECT ip_address, true as is_blocked
    FROM blocked_ips 
    WHERE unblocked_at IS NULL 
    AND (expires_at IS NULL OR expires_at > NOW())
) bi ON ir.ip_address = bi.ip_address;

-- Create function for automatic cleanup of old data
CREATE OR REPLACE FUNCTION cleanup_old_security_data()
RETURNS void AS $$
BEGIN
    -- Clean up old attack attempts (keep 90 days)
    DELETE FROM attack_attempts WHERE created_at < NOW() - INTERVAL '90 days';
    
    -- Clean up expired blocked IPs
    UPDATE blocked_ips 
    SET unblocked_at = NOW(), unblocked_by = 'system', unblock_reason = 'expired'
    WHERE expires_at < NOW() AND unblocked_at IS NULL;
    
    -- Clean up old rate limit records (keep 7 days)
    DELETE FROM rate_limits WHERE created_at < NOW() - INTERVAL '7 days';
    
    -- Refresh materialized views
    REFRESH MATERIALIZED VIEW CONCURRENTLY attack_summary_hourly;
    REFRESH MATERIALIZED VIEW CONCURRENTLY ip_threat_summary;
END;
$$ LANGUAGE plpgsql;

-- Schedule cleanup (would need pg_cron extension or external scheduler)
-- SELECT cron.schedule('cleanup-security-data', '0 2 * * *', 'SELECT cleanup_old_security_data();');

COMMENT ON TABLE attack_attempts IS 'Detailed logging of all security-related attack attempts and suspicious activities';
COMMENT ON TABLE blocked_ips IS 'IP blocking management with support for temporary, permanent, and automatic blocking';
COMMENT ON TABLE attack_patterns IS 'Pattern detection and campaign tracking for advanced threat analysis';
COMMENT ON TABLE ip_reputation IS 'IP reputation scoring and threat intelligence data';
COMMENT ON TABLE rate_limits IS 'Rate limiting tracking and adaptive threshold management';
COMMENT ON TABLE account_security IS 'User account security state and risk profiling';