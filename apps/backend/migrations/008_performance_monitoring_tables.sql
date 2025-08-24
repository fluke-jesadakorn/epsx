-- Performance Monitoring Tables
-- Enterprise-grade performance monitoring with real-time analytics and alerting

-- Performance metrics collection table
CREATE TABLE IF NOT EXISTS performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    duration_ms BIGINT NOT NULL,
    status_code INTEGER NOT NULL,
    cache_hit BOOLEAN DEFAULT FALSE,
    session_validation_ms BIGINT,
    db_query_ms BIGINT,
    middleware_stack_ms BIGINT,
    request_size_bytes BIGINT,
    response_size_bytes BIGINT,
    user_id UUID,
    client_ip INET,
    user_agent TEXT,
    error_message TEXT,
    trace_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_performance_metrics_timestamp 
    ON performance_metrics(timestamp DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_performance_metrics_endpoint 
    ON performance_metrics(endpoint);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_performance_metrics_method 
    ON performance_metrics(method);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_performance_metrics_status 
    ON performance_metrics(status_code);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_performance_metrics_duration 
    ON performance_metrics(duration_ms DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_performance_metrics_user_id 
    ON performance_metrics(user_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_performance_metrics_trace_id 
    ON performance_metrics(trace_id);

-- Composite indexes for analytics queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_performance_endpoint_timestamp 
    ON performance_metrics(endpoint, timestamp DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_performance_status_timestamp 
    ON performance_metrics(status_code, timestamp DESC);

-- Cache performance metrics table
CREATE TABLE IF NOT EXISTS cache_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cache_type VARCHAR(50) NOT NULL, -- redis, memory, session
    operation VARCHAR(20) NOT NULL, -- get, set, delete, exists
    key_pattern VARCHAR(255),
    hit BOOLEAN NOT NULL,
    duration_ms BIGINT NOT NULL,
    key_size_bytes BIGINT,
    value_size_bytes BIGINT,
    ttl_seconds INTEGER,
    evicted BOOLEAN DEFAULT FALSE,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cache performance indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_cache_metrics_timestamp 
    ON cache_performance_metrics(timestamp DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_cache_metrics_type 
    ON cache_performance_metrics(cache_type);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_cache_metrics_operation 
    ON cache_performance_metrics(operation);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_cache_metrics_hit 
    ON cache_performance_metrics(hit);

-- Performance alerts configuration table
CREATE TABLE IF NOT EXISTS performance_alert_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    metric_type VARCHAR(50) NOT NULL, -- latency, error_rate, cache_hit_rate, throughput
    threshold_value DECIMAL(10,2) NOT NULL,
    threshold_operator VARCHAR(10) NOT NULL, -- gt, lt, gte, lte, eq
    time_window_minutes INTEGER NOT NULL DEFAULT 5,
    endpoint_pattern VARCHAR(255), -- null for all endpoints
    severity VARCHAR(20) NOT NULL DEFAULT 'medium', -- low, medium, high, critical
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    cooldown_minutes INTEGER NOT NULL DEFAULT 15,
    notification_channels JSONB, -- email, slack, webhook
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Performance alerts history table
CREATE TABLE IF NOT EXISTS performance_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_config_id UUID NOT NULL REFERENCES performance_alert_config(id),
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    metric_value DECIMAL(10,2) NOT NULL,
    threshold_value DECIMAL(10,2) NOT NULL,
    endpoint VARCHAR(255),
    time_window_start TIMESTAMPTZ NOT NULL,
    time_window_end TIMESTAMPTZ NOT NULL,
    alert_message TEXT NOT NULL,
    severity VARCHAR(20) NOT NULL,
    acknowledged BOOLEAN DEFAULT FALSE,
    acknowledged_by UUID,
    acknowledged_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Performance alerts indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_alerts_config_id 
    ON performance_alerts(alert_config_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_alerts_triggered_at 
    ON performance_alerts(triggered_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_alerts_severity 
    ON performance_alerts(severity);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_alerts_acknowledged 
    ON performance_alerts(acknowledged);

-- Performance aggregates table for fast dashboard queries
CREATE TABLE IF NOT EXISTS performance_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL,
    time_bucket VARCHAR(20) NOT NULL, -- minute, hour, day
    endpoint VARCHAR(255),
    method VARCHAR(10),
    total_requests BIGINT NOT NULL DEFAULT 0,
    error_requests BIGINT NOT NULL DEFAULT 0,
    avg_duration_ms DECIMAL(10,2) NOT NULL DEFAULT 0,
    p50_duration_ms BIGINT NOT NULL DEFAULT 0,
    p95_duration_ms BIGINT NOT NULL DEFAULT 0,
    p99_duration_ms BIGINT NOT NULL DEFAULT 0,
    min_duration_ms BIGINT NOT NULL DEFAULT 0,
    max_duration_ms BIGINT NOT NULL DEFAULT 0,
    cache_hits BIGINT NOT NULL DEFAULT 0,
    cache_misses BIGINT NOT NULL DEFAULT 0,
    avg_request_size_bytes BIGINT NOT NULL DEFAULT 0,
    avg_response_size_bytes BIGINT NOT NULL DEFAULT 0,
    throughput_rps DECIMAL(10,2) NOT NULL DEFAULT 0,
    error_rate_percent DECIMAL(5,2) NOT NULL DEFAULT 0,
    cache_hit_rate_percent DECIMAL(5,2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(timestamp, time_bucket, endpoint, method)
);

-- Performance aggregates indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_aggregates_timestamp_bucket 
    ON performance_aggregates(timestamp DESC, time_bucket);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_aggregates_endpoint 
    ON performance_aggregates(endpoint);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_aggregates_error_rate 
    ON performance_aggregates(error_rate_percent DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_aggregates_p99_duration 
    ON performance_aggregates(p99_duration_ms DESC);

-- System resource metrics table
CREATE TABLE IF NOT EXISTS system_resource_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cpu_usage_percent DECIMAL(5,2),
    memory_usage_percent DECIMAL(5,2),
    memory_usage_bytes BIGINT,
    disk_usage_percent DECIMAL(5,2),
    disk_io_read_bytes BIGINT,
    disk_io_write_bytes BIGINT,
    network_rx_bytes BIGINT,
    network_tx_bytes BIGINT,
    active_connections INTEGER,
    db_connection_pool_active INTEGER,
    db_connection_pool_idle INTEGER,
    redis_connections INTEGER,
    goroutines_count INTEGER, -- For monitoring runtime
    gc_pause_ms DECIMAL(10,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- System resource indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_system_metrics_timestamp 
    ON system_resource_metrics(timestamp DESC);

-- Performance recommendations table
CREATE TABLE IF NOT EXISTS performance_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    recommendation_type VARCHAR(50) NOT NULL, -- cache_optimization, query_optimization, scaling, index_creation
    priority VARCHAR(20) NOT NULL DEFAULT 'medium', -- low, medium, high, critical
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    impact_score INTEGER NOT NULL DEFAULT 0, -- 1-100
    estimated_improvement TEXT,
    affected_endpoints TEXT[],
    implementation_effort VARCHAR(20), -- low, medium, high
    auto_implementable BOOLEAN DEFAULT FALSE,
    metadata JSONB,
    status VARCHAR(20) DEFAULT 'pending', -- pending, approved, implemented, rejected
    implemented_at TIMESTAMPTZ,
    notes TEXT
);

-- Recommendations indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recommendations_type 
    ON performance_recommendations(recommendation_type);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recommendations_priority 
    ON performance_recommendations(priority);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recommendations_status 
    ON performance_recommendations(status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recommendations_impact 
    ON performance_recommendations(impact_score DESC);

-- Insert default alert configurations
INSERT INTO performance_alert_config (name, description, metric_type, threshold_value, threshold_operator, time_window_minutes, severity, notification_channels) VALUES
('High Latency Alert', 'Alert when P95 latency exceeds 500ms', 'latency', 500.00, 'gt', 5, 'high', '["email", "slack"]'),
('Low Cache Hit Rate', 'Alert when cache hit rate drops below 80%', 'cache_hit_rate', 80.00, 'lt', 10, 'medium', '["email"]'),
('High Error Rate', 'Alert when error rate exceeds 5%', 'error_rate', 5.00, 'gt', 5, 'critical', '["email", "slack", "webhook"]'),
('Resource Exhaustion', 'Alert when CPU or memory usage exceeds 85%', 'resource_usage', 85.00, 'gt', 3, 'high', '["email", "slack"]'),
('Low Throughput', 'Alert when throughput drops below expected levels', 'throughput', 10.00, 'lt', 15, 'medium', '["email"]')
ON CONFLICT (name) DO NOTHING;

-- Create function to automatically aggregate performance data
CREATE OR REPLACE FUNCTION aggregate_performance_data()
RETURNS void AS $$
BEGIN
    -- Aggregate minute-level data
    INSERT INTO performance_aggregates (
        timestamp, time_bucket, endpoint, method,
        total_requests, error_requests, avg_duration_ms,
        p50_duration_ms, p95_duration_ms, p99_duration_ms,
        min_duration_ms, max_duration_ms,
        cache_hits, cache_misses,
        avg_request_size_bytes, avg_response_size_bytes,
        throughput_rps, error_rate_percent, cache_hit_rate_percent
    )
    SELECT 
        DATE_TRUNC('minute', timestamp) as timestamp,
        'minute' as time_bucket,
        endpoint,
        method,
        COUNT(*) as total_requests,
        COUNT(*) FILTER (WHERE status_code >= 400) as error_requests,
        AVG(duration_ms) as avg_duration_ms,
        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY duration_ms) as p50_duration_ms,
        PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms) as p95_duration_ms,
        PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY duration_ms) as p99_duration_ms,
        MIN(duration_ms) as min_duration_ms,
        MAX(duration_ms) as max_duration_ms,
        COUNT(*) FILTER (WHERE cache_hit = true) as cache_hits,
        COUNT(*) FILTER (WHERE cache_hit = false) as cache_misses,
        AVG(request_size_bytes) as avg_request_size_bytes,
        AVG(response_size_bytes) as avg_response_size_bytes,
        COUNT(*)::decimal / 60 as throughput_rps, -- requests per second
        (COUNT(*) FILTER (WHERE status_code >= 400)::decimal / COUNT(*) * 100) as error_rate_percent,
        (COUNT(*) FILTER (WHERE cache_hit = true)::decimal / NULLIF(COUNT(*), 0) * 100) as cache_hit_rate_percent
    FROM performance_metrics 
    WHERE timestamp >= DATE_TRUNC('minute', NOW() - INTERVAL '1 minute')
      AND timestamp < DATE_TRUNC('minute', NOW())
    GROUP BY DATE_TRUNC('minute', timestamp), endpoint, method
    ON CONFLICT (timestamp, time_bucket, endpoint, method) DO UPDATE SET
        total_requests = EXCLUDED.total_requests,
        error_requests = EXCLUDED.error_requests,
        avg_duration_ms = EXCLUDED.avg_duration_ms,
        p50_duration_ms = EXCLUDED.p50_duration_ms,
        p95_duration_ms = EXCLUDED.p95_duration_ms,
        p99_duration_ms = EXCLUDED.p99_duration_ms,
        min_duration_ms = EXCLUDED.min_duration_ms,
        max_duration_ms = EXCLUDED.max_duration_ms,
        cache_hits = EXCLUDED.cache_hits,
        cache_misses = EXCLUDED.cache_misses,
        avg_request_size_bytes = EXCLUDED.avg_request_size_bytes,
        avg_response_size_bytes = EXCLUDED.avg_response_size_bytes,
        throughput_rps = EXCLUDED.throughput_rps,
        error_rate_percent = EXCLUDED.error_rate_percent,
        cache_hit_rate_percent = EXCLUDED.cache_hit_rate_percent;
END;
$$ LANGUAGE plpgsql;

-- Create function to clean old performance data
CREATE OR REPLACE FUNCTION cleanup_old_performance_data()
RETURNS void AS $$
BEGIN
    -- Keep detailed metrics for 7 days
    DELETE FROM performance_metrics 
    WHERE created_at < NOW() - INTERVAL '7 days';
    
    -- Keep cache metrics for 3 days
    DELETE FROM cache_performance_metrics 
    WHERE created_at < NOW() - INTERVAL '3 days';
    
    -- Keep minute aggregates for 30 days
    DELETE FROM performance_aggregates 
    WHERE time_bucket = 'minute' AND created_at < NOW() - INTERVAL '30 days';
    
    -- Keep hour aggregates for 90 days
    DELETE FROM performance_aggregates 
    WHERE time_bucket = 'hour' AND created_at < NOW() - INTERVAL '90 days';
    
    -- Keep system metrics for 30 days
    DELETE FROM system_resource_metrics 
    WHERE created_at < NOW() - INTERVAL '30 days';
    
    -- Keep resolved alerts for 90 days
    DELETE FROM performance_alerts 
    WHERE resolved_at IS NOT NULL AND resolved_at < NOW() - INTERVAL '90 days';
    
    -- Keep recommendations for 180 days if implemented
    DELETE FROM performance_recommendations 
    WHERE status = 'implemented' AND implemented_at < NOW() - INTERVAL '180 days';
END;
$$ LANGUAGE plpgsql;

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_alert_config_updated_at
    BEFORE UPDATE ON performance_alert_config
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create views for common performance queries
CREATE OR REPLACE VIEW performance_dashboard_summary AS
SELECT 
    DATE_TRUNC('hour', timestamp) as hour,
    COUNT(*) as total_requests,
    AVG(duration_ms) as avg_response_time,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms) as p95_response_time,
    COUNT(*) FILTER (WHERE status_code >= 400)::decimal / COUNT(*) * 100 as error_rate,
    COUNT(*) FILTER (WHERE cache_hit = true)::decimal / NULLIF(COUNT(*), 0) * 100 as cache_hit_rate,
    COUNT(DISTINCT endpoint) as unique_endpoints,
    COUNT(DISTINCT user_id) as unique_users
FROM performance_metrics 
WHERE timestamp >= NOW() - INTERVAL '24 hours'
GROUP BY DATE_TRUNC('hour', timestamp)
ORDER BY hour DESC;

CREATE OR REPLACE VIEW slowest_endpoints AS
SELECT 
    endpoint,
    method,
    COUNT(*) as request_count,
    AVG(duration_ms) as avg_duration,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms) as p95_duration,
    MAX(duration_ms) as max_duration,
    COUNT(*) FILTER (WHERE status_code >= 400) as error_count
FROM performance_metrics 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY endpoint, method
HAVING COUNT(*) >= 5
ORDER BY PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms) DESC
LIMIT 20;

CREATE OR REPLACE VIEW active_alerts AS
SELECT 
    pa.id,
    pa.triggered_at,
    pac.name as alert_name,
    pac.description,
    pa.severity,
    pa.metric_value,
    pa.threshold_value,
    pa.endpoint,
    pa.acknowledged,
    pa.acknowledged_by,
    pa.alert_message
FROM performance_alerts pa
JOIN performance_alert_config pac ON pa.alert_config_id = pac.id
WHERE pa.resolved_at IS NULL
ORDER BY pa.triggered_at DESC;

-- Add comments for documentation
COMMENT ON TABLE performance_metrics IS 'Detailed performance metrics for every request';
COMMENT ON TABLE cache_performance_metrics IS 'Cache operation performance and hit rate tracking';
COMMENT ON TABLE performance_alert_config IS 'Configuration for automated performance alerts';
COMMENT ON TABLE performance_alerts IS 'History of triggered performance alerts';
COMMENT ON TABLE performance_aggregates IS 'Pre-computed performance aggregates for fast dashboard queries';
COMMENT ON TABLE system_resource_metrics IS 'System-level resource utilization metrics';
COMMENT ON TABLE performance_recommendations IS 'AI-generated performance optimization recommendations';

COMMENT ON FUNCTION aggregate_performance_data() IS 'Aggregates raw performance data into summary statistics';
COMMENT ON FUNCTION cleanup_old_performance_data() IS 'Removes old performance data according to retention policies';

COMMENT ON VIEW performance_dashboard_summary IS 'Hourly performance summary for the last 24 hours';
COMMENT ON VIEW slowest_endpoints IS 'Identifies the slowest endpoints in the last hour';
COMMENT ON VIEW active_alerts IS 'Currently active performance alerts requiring attention';