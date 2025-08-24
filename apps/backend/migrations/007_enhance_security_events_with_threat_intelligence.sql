-- Migration: Enhance security_events table with threat intelligence and analytics capabilities
-- This migration adds fields for risk scoring, geographic analysis, device fingerprinting, and correlation

-- Add new columns to security_events table for enhanced threat detection
ALTER TABLE security_events 
ADD COLUMN IF NOT EXISTS risk_score DECIMAL(4,2) CHECK (risk_score >= 0.0 AND risk_score <= 10.0),
ADD COLUMN IF NOT EXISTS country_code VARCHAR(2), -- ISO 3166-1 alpha-2 country code
ADD COLUMN IF NOT EXISTS city VARCHAR(100),
ADD COLUMN IF NOT EXISTS device_fingerprint VARCHAR(255), -- Hashed device fingerprint
ADD COLUMN IF NOT EXISTS correlation_id UUID, -- Links related events
ADD COLUMN IF NOT EXISTS alert_triggered BOOLEAN DEFAULT FALSE;

-- Create indexes for new fields to support analytics queries
CREATE INDEX IF NOT EXISTS idx_security_events_risk_score ON security_events(risk_score DESC) WHERE risk_score IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_security_events_country_code ON security_events(country_code) WHERE country_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_security_events_device_fingerprint ON security_events(device_fingerprint) WHERE device_fingerprint IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_security_events_correlation_id ON security_events(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_security_events_alert_triggered ON security_events(alert_triggered, timestamp DESC) WHERE alert_triggered = TRUE;

-- Composite indexes for threat intelligence queries
CREATE INDEX IF NOT EXISTS idx_security_events_risk_severity_timestamp ON security_events(risk_score DESC, severity, timestamp DESC) WHERE risk_score IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_security_events_country_timestamp ON security_events(country_code, timestamp DESC) WHERE country_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_security_events_ip_country_timestamp ON security_events(ip_address, country_code, timestamp DESC);

-- Create table for IP reputation and threat intelligence
CREATE TABLE IF NOT EXISTS ip_threat_intelligence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL UNIQUE,
    is_malicious BOOLEAN NOT NULL DEFAULT FALSE,
    is_vpn BOOLEAN NOT NULL DEFAULT FALSE,
    is_proxy BOOLEAN NOT NULL DEFAULT FALSE,
    is_tor BOOLEAN NOT NULL DEFAULT FALSE,
    reputation_score DECIMAL(4,3) NOT NULL DEFAULT 0.500 CHECK (reputation_score >= 0.0 AND reputation_score <= 1.0),
    country_code VARCHAR(2),
    city VARCHAR(100),
    isp VARCHAR(200),
    threat_types TEXT[], -- Array of threat types (malware, phishing, spam, etc.)
    confidence_score DECIMAL(4,3) NOT NULL DEFAULT 0.500,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    blocked BOOLEAN NOT NULL DEFAULT FALSE,
    blocked_reason TEXT,
    blocked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for IP threat intelligence
CREATE INDEX IF NOT EXISTS idx_ip_threat_intelligence_ip ON ip_threat_intelligence(ip_address);
CREATE INDEX IF NOT EXISTS idx_ip_threat_intelligence_malicious ON ip_threat_intelligence(is_malicious, reputation_score DESC);
CREATE INDEX IF NOT EXISTS idx_ip_threat_intelligence_blocked ON ip_threat_intelligence(blocked, blocked_at DESC) WHERE blocked = TRUE;
CREATE INDEX IF NOT EXISTS idx_ip_threat_intelligence_country ON ip_threat_intelligence(country_code) WHERE country_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ip_threat_intelligence_last_seen ON ip_threat_intelligence(last_seen DESC);

-- Create table for threat patterns and anomaly detection
CREATE TABLE IF NOT EXISTS threat_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_id VARCHAR(255) NOT NULL UNIQUE,
    pattern_type VARCHAR(100) NOT NULL, -- BRUTE_FORCE, GEOGRAPHIC_ANOMALY, etc.
    description TEXT NOT NULL,
    source_ips INET[] NOT NULL, -- Array of IP addresses involved
    affected_users TEXT[], -- Array of user IDs affected
    detection_count INTEGER NOT NULL DEFAULT 1,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    confidence_score DECIMAL(4,3) NOT NULL DEFAULT 0.500,
    first_detected TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_detected TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    auto_blocked BOOLEAN NOT NULL DEFAULT FALSE,
    mitigated BOOLEAN NOT NULL DEFAULT FALSE,
    mitigation_notes TEXT,
    pattern_data JSONB NOT NULL DEFAULT '{}', -- Pattern-specific data
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for threat patterns
CREATE INDEX IF NOT EXISTS idx_threat_patterns_pattern_type ON threat_patterns(pattern_type);
CREATE INDEX IF NOT EXISTS idx_threat_patterns_severity ON threat_patterns(severity);
CREATE INDEX IF NOT EXISTS idx_threat_patterns_last_detected ON threat_patterns(last_detected DESC);
CREATE INDEX IF NOT EXISTS idx_threat_patterns_source_ips ON threat_patterns USING GIN (source_ips);
CREATE INDEX IF NOT EXISTS idx_threat_patterns_affected_users ON threat_patterns USING GIN (affected_users);
CREATE INDEX IF NOT EXISTS idx_threat_patterns_auto_blocked ON threat_patterns(auto_blocked, last_detected DESC) WHERE auto_blocked = TRUE;

-- Create table for user behavior analytics
CREATE TABLE IF NOT EXISTS user_behavior_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    behavior_type VARCHAR(100) NOT NULL, -- LOGIN_PATTERN, GEOGRAPHIC_PATTERN, etc.
    baseline_data JSONB NOT NULL DEFAULT '{}', -- Normal behavior baseline
    current_data JSONB NOT NULL DEFAULT '{}', -- Current behavior data
    anomaly_score DECIMAL(4,3) NOT NULL DEFAULT 0.000 CHECK (anomaly_score >= 0.0 AND anomaly_score <= 1.0),
    risk_level VARCHAR(20) NOT NULL CHECK (risk_level IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    detection_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for user behavior analytics
CREATE INDEX IF NOT EXISTS idx_user_behavior_analytics_user_id ON user_behavior_analytics(user_id);
CREATE INDEX IF NOT EXISTS idx_user_behavior_analytics_behavior_type ON user_behavior_analytics(behavior_type);
CREATE INDEX IF NOT EXISTS idx_user_behavior_analytics_anomaly_score ON user_behavior_analytics(anomaly_score DESC);
CREATE INDEX IF NOT EXISTS idx_user_behavior_analytics_risk_level ON user_behavior_analytics(risk_level, detection_date DESC);
CREATE INDEX IF NOT EXISTS idx_user_behavior_analytics_unresolved ON user_behavior_analytics(resolved, detection_date DESC) WHERE resolved = FALSE;

-- Create table for security event correlations
CREATE TABLE IF NOT EXISTS security_event_correlations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    correlation_id UUID NOT NULL,
    primary_event_id UUID NOT NULL REFERENCES security_events(id),
    related_event_ids UUID[] NOT NULL, -- Array of related event IDs
    correlation_type VARCHAR(100) NOT NULL, -- IP_BASED, USER_BASED, PATTERN_BASED, etc.
    correlation_strength DECIMAL(4,3) NOT NULL DEFAULT 0.500,
    analysis_summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for event correlations
CREATE INDEX IF NOT EXISTS idx_security_event_correlations_correlation_id ON security_event_correlations(correlation_id);
CREATE INDEX IF NOT EXISTS idx_security_event_correlations_primary_event ON security_event_correlations(primary_event_id);
CREATE INDEX IF NOT EXISTS idx_security_event_correlations_type ON security_event_correlations(correlation_type);
CREATE INDEX IF NOT EXISTS idx_security_event_correlations_strength ON security_event_correlations(correlation_strength DESC);

-- Create triggers for automatic timestamp updates
CREATE TRIGGER trigger_ip_threat_intelligence_updated_at
    BEFORE UPDATE ON ip_threat_intelligence
    FOR EACH ROW
    EXECUTE FUNCTION update_security_events_updated_at();

CREATE TRIGGER trigger_threat_patterns_updated_at
    BEFORE UPDATE ON threat_patterns
    FOR EACH ROW
    EXECUTE FUNCTION update_security_events_updated_at();

-- Create enhanced materialized view for security analytics dashboard
DROP MATERIALIZED VIEW IF EXISTS security_analytics_dashboard;
CREATE MATERIALIZED VIEW security_analytics_dashboard AS
WITH threat_summary AS (
    SELECT 
        date_trunc('hour', timestamp) as hour_bucket,
        COUNT(*) as total_events,
        COUNT(*) FILTER (WHERE severity IN ('HIGH', 'CRITICAL')) as high_severity_events,
        COUNT(*) FILTER (WHERE alert_triggered = TRUE) as alert_events,
        COUNT(*) FILTER (WHERE risk_score >= 7.0) as high_risk_events,
        COUNT(DISTINCT ip_address) as unique_ips,
        COUNT(DISTINCT user_id) as unique_users,
        COUNT(DISTINCT country_code) as unique_countries,
        AVG(risk_score) as avg_risk_score
    FROM security_events 
    WHERE timestamp >= NOW() - INTERVAL '7 days'
    GROUP BY date_trunc('hour', timestamp)
),
ip_summary AS (
    SELECT 
        ip_address,
        COUNT(*) as event_count,
        MAX(risk_score) as max_risk_score,
        bool_or(alert_triggered) as has_alerts,
        array_agg(DISTINCT event_type) as event_types,
        MAX(timestamp) as last_seen
    FROM security_events 
    WHERE timestamp >= NOW() - INTERVAL '24 hours'
    GROUP BY ip_address
    HAVING COUNT(*) >= 5 OR MAX(risk_score) >= 6.0
),
country_summary AS (
    SELECT 
        country_code,
        COUNT(*) as event_count,
        COUNT(DISTINCT ip_address) as unique_ips,
        AVG(risk_score) as avg_risk_score
    FROM security_events 
    WHERE timestamp >= NOW() - INTERVAL '24 hours' 
      AND country_code IS NOT NULL
    GROUP BY country_code
)
SELECT 
    ts.hour_bucket,
    ts.total_events,
    ts.high_severity_events,
    ts.alert_events,
    ts.high_risk_events,
    ts.unique_ips,
    ts.unique_users,
    ts.unique_countries,
    ts.avg_risk_score,
    (SELECT json_agg(row_to_json(ip_summary)) FROM ip_summary) as suspicious_ips,
    (SELECT json_agg(row_to_json(country_summary)) FROM country_summary) as country_analysis,
    NOW() as generated_at
FROM threat_summary ts
ORDER BY ts.hour_bucket DESC;

-- Create unique index for the materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_security_analytics_dashboard_hour 
ON security_analytics_dashboard(hour_bucket);

-- Create function to refresh security analytics
CREATE OR REPLACE FUNCTION refresh_security_analytics()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY security_analytics_dashboard;
    -- Also refresh the existing hourly stats view
    REFRESH MATERIALIZED VIEW CONCURRENTLY security_event_stats_hourly;
END;
$$ LANGUAGE plpgsql;

-- Create function for automatic threat pattern detection
CREATE OR REPLACE FUNCTION detect_threat_patterns()
RETURNS void AS $$
DECLARE
    pattern_record RECORD;
BEGIN
    -- Detect brute force patterns (5+ failed attempts from same IP in 10 minutes)
    FOR pattern_record IN
        SELECT 
            ip_address,
            COUNT(*) as attempt_count,
            MIN(timestamp) as first_attempt,
            MAX(timestamp) as last_attempt
        FROM security_events 
        WHERE event_type IN ('BRUTE_FORCE_ATTEMPT', 'MULTIPLE_FAILED_LOGINS', 'BRUTE_FORCE_DETECTED')
          AND timestamp >= NOW() - INTERVAL '10 minutes'
        GROUP BY ip_address
        HAVING COUNT(*) >= 5
    LOOP
        -- Insert or update threat pattern
        INSERT INTO threat_patterns (
            pattern_id, 
            pattern_type, 
            description, 
            source_ips, 
            detection_count, 
            severity, 
            first_detected, 
            last_detected,
            auto_blocked,
            pattern_data
        ) VALUES (
            'brute_force_' || pattern_record.ip_address || '_' || extract(epoch from pattern_record.first_attempt),
            'BRUTE_FORCE',
            'Multiple failed login attempts detected from ' || pattern_record.ip_address,
            ARRAY[pattern_record.ip_address::inet],
            pattern_record.attempt_count,
            CASE 
                WHEN pattern_record.attempt_count >= 20 THEN 'CRITICAL'
                WHEN pattern_record.attempt_count >= 10 THEN 'HIGH'
                ELSE 'MEDIUM'
            END,
            pattern_record.first_attempt,
            pattern_record.last_attempt,
            pattern_record.attempt_count >= 10,
            json_build_object(
                'attempt_count', pattern_record.attempt_count,
                'time_window_minutes', 10,
                'detection_method', 'automatic'
            )::jsonb
        ) ON CONFLICT (pattern_id) DO UPDATE SET
            detection_count = pattern_record.attempt_count,
            last_detected = pattern_record.last_attempt,
            updated_at = NOW();
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Add comments for documentation
COMMENT ON COLUMN security_events.risk_score IS 'Calculated risk score from 0.0 (low) to 10.0 (critical)';
COMMENT ON COLUMN security_events.country_code IS 'ISO 3166-1 alpha-2 country code derived from IP address';
COMMENT ON COLUMN security_events.device_fingerprint IS 'Hashed device fingerprint for device tracking';
COMMENT ON COLUMN security_events.correlation_id IS 'UUID linking related security events';
COMMENT ON COLUMN security_events.alert_triggered IS 'Whether this event triggered an automated alert';

COMMENT ON TABLE ip_threat_intelligence IS 'Threat intelligence data for IP addresses including reputation and blocking status';
COMMENT ON TABLE threat_patterns IS 'Detected threat patterns and attack signatures';
COMMENT ON TABLE user_behavior_analytics IS 'User behavior baselines and anomaly detection results';
COMMENT ON TABLE security_event_correlations IS 'Relationships and correlations between security events';
COMMENT ON MATERIALIZED VIEW security_analytics_dashboard IS 'Pre-computed security analytics for dashboard performance';

-- Create initial security configuration
INSERT INTO security_alerts (alert_type, condition, threshold, is_active) VALUES
('high_risk_score', 'risk_score >= threshold.min_score', 
 '{"min_score": 8.0, "immediate": true}', true),
('geographic_anomaly', 'event_type = ''GEOGRAPHIC_ANOMALY_DETECTED''', 
 '{"immediate": true}', true),
('device_fingerprint_mismatch', 'event_type = ''DEVICE_FINGERPRINT_MISMATCH''', 
 '{"immediate": true}', true),
('vpn_proxy_detection', 'event_type = ''VPN_OR_PROXY_DETECTED''', 
 '{"threshold_count": 3, "time_window_minutes": 60}', true),
('concurrent_session_anomaly', 'event_type = ''CONCURRENT_SESSION_ANOMALY''', 
 '{"immediate": true}', true)
ON CONFLICT (alert_type) DO NOTHING;