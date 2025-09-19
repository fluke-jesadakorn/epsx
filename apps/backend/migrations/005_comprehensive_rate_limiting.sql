-- Comprehensive Rate Limiting System Migration
-- Creates tables for sliding window rate limiting with progressive penalties

-- Rate limiting entries table (persistent storage)
CREATE TABLE IF NOT EXISTS rate_limit_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id VARCHAR(255) NOT NULL,
    client_type VARCHAR(50) NOT NULL, -- 'user', 'ip', 'api', 'service', 'anonymous'
    endpoint VARCHAR(255) NOT NULL,
    window_start BIGINT NOT NULL,
    window_size_seconds INTEGER NOT NULL DEFAULT 60,
    request_timestamps BIGINT[] NOT NULL DEFAULT '{}',
    burst_tokens INTEGER NOT NULL DEFAULT 0,
    penalty_factor DECIMAL(3,2) NOT NULL DEFAULT 1.0,
    violation_count INTEGER NOT NULL DEFAULT 0,
    last_violation BIGINT,
    recovery_time BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(client_id, client_type, endpoint)
);

-- Rate limiting violations table (audit trail)
CREATE TABLE IF NOT EXISTS rate_limit_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id VARCHAR(255) NOT NULL,
    client_type VARCHAR(50) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    violation_type VARCHAR(100) NOT NULL,
    timestamp_occurred BIGINT NOT NULL,
    severity SMALLINT NOT NULL CHECK (severity >= 1 AND severity <= 10),
    penalty_applied BOOLEAN NOT NULL DEFAULT false,
    recovery_time BIGINT,
    user_agent TEXT,
    ip_address INET,
    additional_context JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Rate limiting tiers configuration table
CREATE TABLE IF NOT EXISTS rate_limit_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    requests_per_minute INTEGER NOT NULL,
    requests_per_hour INTEGER NOT NULL,
    requests_per_day INTEGER NOT NULL,
    burst_allowance INTEGER NOT NULL DEFAULT 0,
    penalty_multiplier DECIMAL(3,2) NOT NULL DEFAULT 0.5,
    recovery_time_hours INTEGER NOT NULL DEFAULT 24,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Client tier mappings table
CREATE TABLE IF NOT EXISTS client_tier_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id VARCHAR(255) NOT NULL,
    client_type VARCHAR(50) NOT NULL,
    tier_name VARCHAR(50) NOT NULL REFERENCES rate_limit_tiers(name),
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    assigned_by UUID, -- User ID who assigned this tier
    expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(client_id, client_type, tier_name)
);

-- Rate limiting statistics table (for monitoring)
CREATE TABLE IF NOT EXISTS rate_limit_statistics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date_hour TIMESTAMP WITH TIME ZONE NOT NULL, -- Hourly aggregation
    tier_name VARCHAR(50) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    total_violations BIGINT NOT NULL DEFAULT 0,
    unique_clients INTEGER NOT NULL DEFAULT 0,
    average_penalty_factor DECIMAL(3,2) NOT NULL DEFAULT 1.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(date_hour, tier_name, endpoint)
);

-- Indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_rate_limit_entries_client ON rate_limit_entries(client_id, client_type);
CREATE INDEX IF NOT EXISTS idx_rate_limit_entries_endpoint ON rate_limit_entries(endpoint);
CREATE INDEX IF NOT EXISTS idx_rate_limit_entries_updated ON rate_limit_entries(updated_at);
CREATE INDEX IF NOT EXISTS idx_rate_limit_entries_recovery ON rate_limit_entries(recovery_time) WHERE recovery_time IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_client ON rate_limit_violations(client_id, client_type);
CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_timestamp ON rate_limit_violations(timestamp_occurred);
CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_severity ON rate_limit_violations(severity);
CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_created ON rate_limit_violations(created_at);

CREATE INDEX IF NOT EXISTS idx_client_tier_mappings_client ON client_tier_mappings(client_id, client_type);
CREATE INDEX IF NOT EXISTS idx_client_tier_mappings_tier ON client_tier_mappings(tier_name);
CREATE INDEX IF NOT EXISTS idx_client_tier_mappings_active ON client_tier_mappings(is_active) WHERE is_active = true;

CREATE INDEX IF NOT EXISTS idx_rate_limit_statistics_date ON rate_limit_statistics(date_hour);
CREATE INDEX IF NOT EXISTS idx_rate_limit_statistics_tier ON rate_limit_statistics(tier_name);

-- Partial indexes for specific queries
CREATE INDEX IF NOT EXISTS idx_rate_limit_entries_active_penalties 
ON rate_limit_entries(client_id, penalty_factor) 
WHERE penalty_factor < 1.0;

CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_recent 
ON rate_limit_violations(client_id, client_type, timestamp_occurred) 
WHERE timestamp_occurred > extract(epoch from now() - interval '24 hours');

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for automatic timestamp updates
CREATE TRIGGER update_rate_limit_entries_updated_at
    BEFORE UPDATE ON rate_limit_entries
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rate_limit_tiers_updated_at
    BEFORE UPDATE ON rate_limit_tiers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert default rate limiting tiers
INSERT INTO rate_limit_tiers (name, description, requests_per_minute, requests_per_hour, requests_per_day, burst_allowance, penalty_multiplier, recovery_time_hours)
VALUES 
    ('FREE', 'Free tier with basic rate limits', 10, 100, 1000, 5, 0.5, 24),
    ('PREMIUM', 'Premium tier with higher limits', 60, 1000, 10000, 20, 0.7, 12),
    ('ADMIN', 'Administrative tier with elevated limits', 300, 5000, 50000, 100, 1.0, 6),
    ('INTERNAL', 'Internal service tier with highest limits', 1000, 20000, 200000, 500, 1.0, 1)
ON CONFLICT (name) DO UPDATE SET
    description = EXCLUDED.description,
    requests_per_minute = EXCLUDED.requests_per_minute,
    requests_per_hour = EXCLUDED.requests_per_hour,
    requests_per_day = EXCLUDED.requests_per_day,
    burst_allowance = EXCLUDED.burst_allowance,
    penalty_multiplier = EXCLUDED.penalty_multiplier,
    recovery_time_hours = EXCLUDED.recovery_time_hours,
    updated_at = NOW();

-- Function to clean up old rate limiting data
CREATE OR REPLACE FUNCTION cleanup_old_rate_limit_data()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
BEGIN
    -- Delete old rate limit entries (older than 7 days)
    DELETE FROM rate_limit_entries 
    WHERE updated_at < NOW() - INTERVAL '7 days'
    AND recovery_time IS NULL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Delete old violations (older than 30 days)
    DELETE FROM rate_limit_violations 
    WHERE created_at < NOW() - INTERVAL '30 days';
    
    -- Delete old statistics (older than 90 days)
    DELETE FROM rate_limit_statistics 
    WHERE date_hour < NOW() - INTERVAL '90 days';
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to get rate limit status for a client
CREATE OR REPLACE FUNCTION get_rate_limit_status(
    p_client_id VARCHAR(255),
    p_client_type VARCHAR(50),
    p_endpoint VARCHAR(255)
)
RETURNS TABLE (
    current_requests INTEGER,
    limit_per_minute INTEGER,
    penalty_factor DECIMAL(3,2),
    violation_count INTEGER,
    recovery_time BIGINT,
    tier_name VARCHAR(50)
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        array_length(rle.request_timestamps, 1) as current_requests,
        rlt.requests_per_minute,
        rle.penalty_factor,
        rle.violation_count,
        rle.recovery_time,
        rlt.name as tier_name
    FROM rate_limit_entries rle
    JOIN client_tier_mappings ctm ON (ctm.client_id = rle.client_id AND ctm.client_type = rle.client_type)
    JOIN rate_limit_tiers rlt ON rlt.name = ctm.tier_name
    WHERE rle.client_id = p_client_id 
    AND rle.client_type = p_client_type 
    AND rle.endpoint = p_endpoint
    AND ctm.is_active = true
    AND rlt.is_active = true;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE rate_limit_entries IS 'Sliding window rate limiting entries with progressive penalties';
COMMENT ON TABLE rate_limit_violations IS 'Audit trail of rate limiting violations for security monitoring';
COMMENT ON TABLE rate_limit_tiers IS 'Configurable rate limiting tiers with different limits and penalties';
COMMENT ON TABLE client_tier_mappings IS 'Maps clients to their assigned rate limiting tiers';
COMMENT ON TABLE rate_limit_statistics IS 'Aggregated statistics for rate limiting monitoring and analytics';

COMMENT ON FUNCTION cleanup_old_rate_limit_data() IS 'Cleans up old rate limiting data to maintain performance';
COMMENT ON FUNCTION get_rate_limit_status(VARCHAR, VARCHAR, VARCHAR) IS 'Returns current rate limit status for a specific client and endpoint';