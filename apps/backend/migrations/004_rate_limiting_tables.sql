-- Rate Limiting Database Tables
-- Support for distributed rate limiting with Redis fallback

-- Rate limit usage tracking table
CREATE TABLE IF NOT EXISTS rate_limit_usage (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL, 
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ip_address INET,
    user_agent TEXT
);

-- Indexes for rate limit usage
CREATE INDEX IF NOT EXISTS idx_rate_limit_usage_user_resource 
ON rate_limit_usage (user_id, resource, created_at);

CREATE INDEX IF NOT EXISTS idx_rate_limit_usage_cleanup 
ON rate_limit_usage (created_at) 
WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '7 days';

-- Rate limit violations tracking
CREATE TABLE IF NOT EXISTS rate_limit_violations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    violation_count INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure one record per user-resource combination
    UNIQUE(user_id, resource)
);

-- Indexes for rate limit violations
CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_user 
ON rate_limit_violations (user_id, created_at);

CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_resource 
ON rate_limit_violations (resource, created_at);

-- Trigger for updated_at
CREATE TRIGGER update_rate_limit_violations_updated_at 
    BEFORE UPDATE ON rate_limit_violations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();