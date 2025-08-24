-- Migration: Create security_events table for middleware security logging
-- This table stores all security events from middleware operations across frontend and admin applications

-- Create security_events table
CREATE TABLE IF NOT EXISTS security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id VARCHAR(255), -- Can be null for anonymous events
    session_id VARCHAR(255), -- Session identifier
    ip_address INET NOT NULL,
    user_agent TEXT,
    path VARCHAR(1000), -- Request path that triggered the event
    method VARCHAR(10), -- HTTP method (GET, POST, etc.)
    details JSONB NOT NULL DEFAULT '{}', -- Event-specific details
    source VARCHAR(50) NOT NULL CHECK (source IN ('admin-frontend', 'frontend', 'backend')),
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_security_events_timestamp ON security_events(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_event_type ON security_events(event_type);
CREATE INDEX IF NOT EXISTS idx_security_events_severity ON security_events(severity);
CREATE INDEX IF NOT EXISTS idx_security_events_user_id ON security_events(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_security_events_ip_address ON security_events(ip_address);
CREATE INDEX IF NOT EXISTS idx_security_events_source ON security_events(source);
CREATE INDEX IF NOT EXISTS idx_security_events_resolved ON security_events(resolved);

-- Composite indexes for common filter combinations
CREATE INDEX IF NOT EXISTS idx_security_events_severity_timestamp ON security_events(severity, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_source_timestamp ON security_events(source, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_user_timestamp ON security_events(user_id, timestamp DESC) WHERE user_id IS NOT NULL;

-- Index for suspicious IP detection (events from same IP in short time)
CREATE INDEX IF NOT EXISTS idx_security_events_ip_timestamp ON security_events(ip_address, timestamp DESC);

-- JSONB index for efficient querying of event details
CREATE INDEX IF NOT EXISTS idx_security_events_details_gin ON security_events USING GIN (details);

-- Create function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_security_events_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER trigger_security_events_updated_at
    BEFORE UPDATE ON security_events
    FOR EACH ROW
    EXECUTE FUNCTION update_security_events_updated_at();

-- Create security_alerts table for alert configuration
CREATE TABLE IF NOT EXISTS security_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type VARCHAR(100) NOT NULL,
    condition TEXT NOT NULL, -- SQL-like condition for triggering alert
    threshold JSONB NOT NULL DEFAULT '{}', -- Threshold values (count, time window, etc.)
    webhook_url TEXT, -- Optional webhook URL for notifications
    email_recipients TEXT[], -- Array of email addresses for notifications
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    triggered_count INTEGER NOT NULL DEFAULT 0,
    last_triggered TIMESTAMPTZ
);

-- Index for active alerts
CREATE INDEX IF NOT EXISTS idx_security_alerts_active ON security_alerts(is_active, alert_type);

-- Create trigger for security_alerts updated_at
CREATE TRIGGER trigger_security_alerts_updated_at
    BEFORE UPDATE ON security_alerts
    FOR EACH ROW
    EXECUTE FUNCTION update_security_events_updated_at();

-- Insert default security alerts
INSERT INTO security_alerts (alert_type, condition, threshold, is_active) VALUES
('brute_force_detection', 'COUNT(*) >= threshold.max_attempts IN time_window', 
 '{"max_attempts": 5, "time_window_minutes": 5}', true),
('high_severity_events', 'severity IN (''HIGH'', ''CRITICAL'')', 
 '{"immediate": true}', true),
('multiple_failures_single_ip', 'COUNT(*) >= threshold.max_failures FROM single IP', 
 '{"max_failures": 10, "time_window_minutes": 15}', true),
('admin_access_anomaly', 'event_type = ''UNAUTHORIZED_ADMIN_ACCESS''', 
 '{"immediate": true}', true),
('permission_escalation', 'event_type = ''ROLE_ESCALATION_ATTEMPT''', 
 '{"immediate": true}', true);

-- Create performance_metrics table for storing middleware performance data
CREATE TABLE IF NOT EXISTS performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    middleware_execution_time DECIMAL(10,3) NOT NULL, -- milliseconds
    cache_hit_rate DECIMAL(5,4) NOT NULL, -- 0.0 to 1.0
    session_validation_time DECIMAL(10,3) NOT NULL, -- milliseconds
    permission_check_time DECIMAL(10,3) NOT NULL, -- milliseconds
    total_request_time DECIMAL(10,3) NOT NULL, -- milliseconds
    requests_per_minute INTEGER NOT NULL DEFAULT 0,
    error_rate DECIMAL(5,4) NOT NULL DEFAULT 0.0, -- 0.0 to 1.0
    source VARCHAR(50) NOT NULL CHECK (source IN ('admin-frontend', 'frontend', 'backend')),
    additional_metrics JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance metrics
CREATE INDEX IF NOT EXISTS idx_performance_metrics_timestamp ON performance_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_source ON performance_metrics(source);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_source_timestamp ON performance_metrics(source, timestamp DESC);

-- Create materialized view for security event statistics (refreshed periodically)
CREATE MATERIALIZED VIEW IF NOT EXISTS security_event_stats_hourly AS
SELECT 
    date_trunc('hour', timestamp) as hour_bucket,
    event_type,
    severity,
    source,
    COUNT(*) as event_count,
    COUNT(DISTINCT user_id) as unique_users,
    COUNT(DISTINCT ip_address) as unique_ips
FROM security_events 
WHERE timestamp >= NOW() - INTERVAL '30 days'
GROUP BY date_trunc('hour', timestamp), event_type, severity, source
ORDER BY hour_bucket DESC;

-- Index for the materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_security_event_stats_hourly 
ON security_event_stats_hourly(hour_bucket, event_type, severity, source);

-- Create function to refresh the materialized view
CREATE OR REPLACE FUNCTION refresh_security_stats()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY security_event_stats_hourly;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE security_events IS 'Stores security events from middleware operations across all applications';
COMMENT ON COLUMN security_events.event_type IS 'Type of security event (UNAUTHORIZED_ACCESS, PERMISSION_DENIED, etc.)';
COMMENT ON COLUMN security_events.severity IS 'Severity level: LOW, MEDIUM, HIGH, CRITICAL';
COMMENT ON COLUMN security_events.details IS 'Event-specific details in JSON format';
COMMENT ON COLUMN security_events.source IS 'Application that generated the event';

COMMENT ON TABLE security_alerts IS 'Configuration for automated security alerts and notifications';
COMMENT ON TABLE performance_metrics IS 'Middleware performance metrics for monitoring and optimization';
COMMENT ON MATERIALIZED VIEW security_event_stats_hourly IS 'Pre-aggregated security event statistics for dashboard performance';