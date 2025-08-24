-- Migration: Create comprehensive brute force detection and IP blocking system tables
-- This migration creates all necessary tables for the sophisticated brute force detection system

-- Attack attempts tracking table
CREATE TABLE IF NOT EXISTS attack_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL,
    attack_type VARCHAR(50) NOT NULL,
    severity_level VARCHAR(20) NOT NULL DEFAULT 'LOW',
    attempt_count INTEGER NOT NULL DEFAULT 1,
    first_attempt TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_attempt TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    blocked BOOLEAN NOT NULL DEFAULT FALSE,
    blocked_until TIMESTAMPTZ,
    user_agents JSONB,
    target_usernames JSONB,
    geographic_data JSONB,
    pattern_signatures JSONB,
    ml_confidence_score DECIMAL(5,4),
    response_actions JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- IP reputation and intelligence table
CREATE TABLE IF NOT EXISTS ip_reputation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL UNIQUE,
    reputation_score DECIMAL(3,2) NOT NULL DEFAULT 0.5 CHECK (reputation_score >= 0.0 AND reputation_score <= 1.0),
    is_malicious BOOLEAN NOT NULL DEFAULT FALSE,
    is_vpn BOOLEAN NOT NULL DEFAULT FALSE,
    is_proxy BOOLEAN NOT NULL DEFAULT FALSE,
    is_tor BOOLEAN NOT NULL DEFAULT FALSE,
    country_code VARCHAR(2),
    asn INTEGER,
    organization VARCHAR(255),
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    attack_history_count INTEGER NOT NULL DEFAULT 0,
    successful_attacks INTEGER NOT NULL DEFAULT 0,
    blocked_count INTEGER NOT NULL DEFAULT 0,
    threat_categories JSONB DEFAULT '[]'::jsonb,
    confidence_level DECIMAL(3,2) NOT NULL DEFAULT 0.5 CHECK (confidence_level >= 0.0 AND confidence_level <= 1.0),
    data_sources JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Attack pattern analysis table for ML insights
CREATE TABLE IF NOT EXISTS attack_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_type VARCHAR(50) NOT NULL,
    signature_hash VARCHAR(32) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    detection_count INTEGER NOT NULL DEFAULT 1,
    success_rate DECIMAL(5,4) NOT NULL DEFAULT 0.0 CHECK (success_rate >= 0.0 AND success_rate <= 1.0),
    avg_requests_per_minute DECIMAL(8,2) NOT NULL DEFAULT 0.0,
    common_user_agents JSONB DEFAULT '[]'::jsonb,
    target_endpoints JSONB DEFAULT '[]'::jsonb,
    timing_characteristics JSONB DEFAULT '{}'::jsonb,
    geographic_distribution JSONB DEFAULT '{}'::jsonb,
    severity_score DECIMAL(4,2) NOT NULL DEFAULT 0.0 CHECK (severity_score >= 0.0 AND severity_score <= 10.0),
    false_positive_rate DECIMAL(5,4) NOT NULL DEFAULT 0.0 CHECK (false_positive_rate >= 0.0 AND false_positive_rate <= 1.0),
    first_detected TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_detected TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'deprecated', 'false_positive')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Response actions logging table
CREATE TABLE IF NOT EXISTS response_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attack_attempt_id UUID REFERENCES attack_attempts(id) ON DELETE SET NULL,
    action_type VARCHAR(50) NOT NULL,
    action_details JSONB NOT NULL DEFAULT '{}'::jsonb,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL DEFAULT TRUE,
    error_message TEXT,
    duration_minutes INTEGER,
    automated BOOLEAN NOT NULL DEFAULT TRUE,
    operator_id UUID, -- Reference to user who executed manual action
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Machine learning training data table
CREATE TABLE IF NOT EXISTS ml_training_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feature_vector JSONB NOT NULL,
    label VARCHAR(20) NOT NULL CHECK (label IN ('attack', 'legitimate', 'suspicious')),
    confidence DECIMAL(5,4) NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    ip_address INET NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    attack_type VARCHAR(50),
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    feedback_score DECIMAL(3,2) CHECK (feedback_score >= 0.0 AND feedback_score <= 1.0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Blocked entities table (IPs, user agents, etc.)
CREATE TABLE IF NOT EXISTS blocked_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(20) NOT NULL CHECK (entity_type IN ('ip', 'user_agent', 'country', 'asn')),
    entity_value TEXT NOT NULL,
    entity_hash VARCHAR(64) NOT NULL, -- Hash for privacy/performance
    blocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    blocked_until TIMESTAMPTZ, -- NULL for permanent blocks
    block_reason TEXT NOT NULL,
    automated BOOLEAN NOT NULL DEFAULT TRUE,
    blocked_by UUID, -- Reference to user who created block
    unblocked_at TIMESTAMPTZ,
    unblocked_by UUID, -- Reference to user who removed block
    unblock_reason TEXT,
    block_count INTEGER NOT NULL DEFAULT 1,
    last_seen TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(entity_type, entity_hash)
);

-- Brute force detection configuration table
CREATE TABLE IF NOT EXISTS bf_detection_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    config_name VARCHAR(100) NOT NULL UNIQUE,
    config_data JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Attack statistics aggregation table (for performance)
CREATE TABLE IF NOT EXISTS attack_statistics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date_bucket DATE NOT NULL,
    hour_bucket INTEGER NOT NULL CHECK (hour_bucket >= 0 AND hour_bucket <= 23),
    attack_type VARCHAR(50) NOT NULL,
    country_code VARCHAR(2),
    attack_count INTEGER NOT NULL DEFAULT 0,
    unique_ips INTEGER NOT NULL DEFAULT 0,
    blocked_count INTEGER NOT NULL DEFAULT 0,
    success_rate DECIMAL(5,4) NOT NULL DEFAULT 0.0,
    avg_requests_per_minute DECIMAL(8,2) NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(date_bucket, hour_bucket, attack_type, country_code)
);

-- Create indexes for optimal performance
CREATE INDEX IF NOT EXISTS idx_attack_attempts_ip_time ON attack_attempts(ip_address, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_attack_attempts_type_severity ON attack_attempts(attack_type, severity_level);
CREATE INDEX IF NOT EXISTS idx_attack_attempts_blocked ON attack_attempts(blocked, blocked_until) WHERE blocked = TRUE;
CREATE INDEX IF NOT EXISTS idx_attack_attempts_ml_confidence ON attack_attempts(ml_confidence_score DESC) WHERE ml_confidence_score IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_ip_reputation_ip ON ip_reputation(ip_address);
CREATE INDEX IF NOT EXISTS idx_ip_reputation_malicious ON ip_reputation(is_malicious, reputation_score) WHERE is_malicious = TRUE;
CREATE INDEX IF NOT EXISTS idx_ip_reputation_score ON ip_reputation(reputation_score DESC);
CREATE INDEX IF NOT EXISTS idx_ip_reputation_country ON ip_reputation(country_code) WHERE country_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ip_reputation_updated ON ip_reputation(updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_attack_patterns_type ON attack_patterns(pattern_type);
CREATE INDEX IF NOT EXISTS idx_attack_patterns_signature ON attack_patterns(signature_hash);
CREATE INDEX IF NOT EXISTS idx_attack_patterns_status_severity ON attack_patterns(status, severity_score DESC) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_attack_patterns_detected ON attack_patterns(last_detected DESC);

CREATE INDEX IF NOT EXISTS idx_response_actions_attempt ON response_actions(attack_attempt_id);
CREATE INDEX IF NOT EXISTS idx_response_actions_type_time ON response_actions(action_type, executed_at DESC);
CREATE INDEX IF NOT EXISTS idx_response_actions_success ON response_actions(success, executed_at DESC);

CREATE INDEX IF NOT EXISTS idx_ml_training_label_confidence ON ml_training_data(label, confidence DESC);
CREATE INDEX IF NOT EXISTS idx_ml_training_ip_time ON ml_training_data(ip_address, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_ml_training_verified ON ml_training_data(verified) WHERE verified = TRUE;
CREATE INDEX IF NOT EXISTS idx_ml_training_attack_type ON ml_training_data(attack_type) WHERE attack_type IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_blocked_entities_type_hash ON blocked_entities(entity_type, entity_hash);
CREATE INDEX IF NOT EXISTS idx_blocked_entities_active ON blocked_entities(entity_type, blocked_until) WHERE blocked_until IS NULL OR blocked_until > NOW();
CREATE INDEX IF NOT EXISTS idx_blocked_entities_expires ON blocked_entities(blocked_until) WHERE blocked_until IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_attack_statistics_bucket ON attack_statistics(date_bucket, hour_bucket);
CREATE INDEX IF NOT EXISTS idx_attack_statistics_type ON attack_statistics(attack_type, date_bucket DESC);
CREATE INDEX IF NOT EXISTS idx_attack_statistics_country ON attack_statistics(country_code, date_bucket DESC) WHERE country_code IS NOT NULL;

-- Create partial indexes for better performance on common queries
CREATE INDEX IF NOT EXISTS idx_attack_attempts_recent_high_severity 
    ON attack_attempts(created_at DESC, severity_level) 
    WHERE created_at > NOW() - INTERVAL '24 hours' AND severity_level IN ('HIGH', 'CRITICAL');

CREATE INDEX IF NOT EXISTS idx_ip_reputation_high_risk 
    ON ip_reputation(reputation_score DESC, last_seen DESC) 
    WHERE is_malicious = TRUE OR reputation_score < 0.3;

-- Create GIN indexes for JSONB fields
CREATE INDEX IF NOT EXISTS idx_attack_attempts_user_agents_gin ON attack_attempts USING GIN(user_agents);
CREATE INDEX IF NOT EXISTS idx_attack_attempts_target_usernames_gin ON attack_attempts USING GIN(target_usernames);
CREATE INDEX IF NOT EXISTS idx_attack_attempts_geographic_data_gin ON attack_attempts USING GIN(geographic_data);
CREATE INDEX IF NOT EXISTS idx_attack_attempts_pattern_signatures_gin ON attack_attempts USING GIN(pattern_signatures);

CREATE INDEX IF NOT EXISTS idx_ip_reputation_threat_categories_gin ON ip_reputation USING GIN(threat_categories);
CREATE INDEX IF NOT EXISTS idx_ip_reputation_data_sources_gin ON ip_reputation USING GIN(data_sources);

CREATE INDEX IF NOT EXISTS idx_attack_patterns_timing_characteristics_gin ON attack_patterns USING GIN(timing_characteristics);
CREATE INDEX IF NOT EXISTS idx_attack_patterns_geographic_distribution_gin ON attack_patterns USING GIN(geographic_distribution);

CREATE INDEX IF NOT EXISTS idx_ml_training_feature_vector_gin ON ml_training_data USING GIN(feature_vector);

-- Add update triggers for updated_at columns
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_attack_attempts_updated_at 
    BEFORE UPDATE ON attack_attempts 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_ip_reputation_updated_at 
    BEFORE UPDATE ON ip_reputation 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_attack_patterns_updated_at 
    BEFORE UPDATE ON attack_patterns 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_ml_training_data_updated_at 
    BEFORE UPDATE ON ml_training_data 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_blocked_entities_updated_at 
    BEFORE UPDATE ON blocked_entities 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_bf_detection_config_updated_at 
    BEFORE UPDATE ON bf_detection_config 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_attack_statistics_updated_at 
    BEFORE UPDATE ON attack_statistics 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default configuration
INSERT INTO bf_detection_config (config_name, config_data) VALUES 
('default', '{
    "max_attempts_per_minute": 10,
    "max_attempts_per_hour": 50,
    "max_attempts_per_day": 200,
    "enable_pattern_recognition": true,
    "pattern_analysis_window_hours": 24,
    "enable_ml_scoring": true,
    "ml_confidence_threshold": 0.7,
    "enable_auto_blocking": true,
    "default_block_duration_minutes": 30,
    "escalation_block_multiplier": 2.0,
    "enable_distributed_attack_detection": true,
    "enable_slow_brute_force_detection": true,
    "enable_credential_stuffing_detection": true,
    "enable_password_spraying_detection": true,
    "whitelisted_ips": [],
    "blacklisted_ips": [],
    "trusted_user_agents": [],
    "enable_geographic_anomaly_detection": true,
    "suspicious_countries": ["CN", "RU", "KP", "IR"]
}'::jsonb)
ON CONFLICT (config_name) DO NOTHING;

-- Create materialized view for attack statistics dashboard
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_attack_dashboard AS
SELECT 
    date_trunc('hour', created_at) as hour_bucket,
    attack_type,
    COUNT(*) as total_attacks,
    COUNT(DISTINCT ip_address) as unique_ips,
    COUNT(*) FILTER (WHERE blocked = true) as blocked_attacks,
    AVG(ml_confidence_score) FILTER (WHERE ml_confidence_score IS NOT NULL) as avg_ml_confidence,
    COUNT(*) FILTER (WHERE severity_level = 'CRITICAL') as critical_attacks,
    COUNT(*) FILTER (WHERE severity_level = 'HIGH') as high_attacks
FROM attack_attempts 
WHERE created_at >= NOW() - INTERVAL '7 days'
GROUP BY date_trunc('hour', created_at), attack_type
ORDER BY hour_bucket DESC, total_attacks DESC;

-- Create unique index on materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_attack_dashboard_unique 
    ON mv_attack_dashboard(hour_bucket, attack_type);

-- Create refresh function for materialized view
CREATE OR REPLACE FUNCTION refresh_attack_dashboard() RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_attack_dashboard;
END;
$$ LANGUAGE plpgsql;

-- Add comments for documentation
COMMENT ON TABLE attack_attempts IS 'Primary table for tracking brute force attack attempts with ML scoring';
COMMENT ON TABLE ip_reputation IS 'IP reputation database with threat intelligence and historical data';
COMMENT ON TABLE attack_patterns IS 'Machine learning derived attack patterns for automated detection';
COMMENT ON TABLE response_actions IS 'Audit log of all automated and manual security responses';
COMMENT ON TABLE ml_training_data IS 'Training data for machine learning model improvement and validation';
COMMENT ON TABLE blocked_entities IS 'Centralized blocking system for IPs, user agents, and other entities';
COMMENT ON TABLE attack_statistics IS 'Aggregated statistics for performance optimization and reporting';
COMMENT ON MATERIALIZED VIEW mv_attack_dashboard IS 'Real-time dashboard view of attack patterns and trends';

-- Grant appropriate permissions (adjust based on your user roles)
-- GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO brute_force_service;
-- GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO brute_force_service;