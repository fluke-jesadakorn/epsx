-- Migration: API Key and Enterprise Tables
-- Created: 2024-12-19
-- Description: Add tables for API key management and enterprise features

-- API Keys table for enhanced Bearer token functionality
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_value VARCHAR(512) NOT NULL UNIQUE, -- Hashed API key value
    permissions TEXT[] NOT NULL DEFAULT '{}', -- Array of permissions
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    usage_count BIGINT NOT NULL DEFAULT 0,
    rate_limit JSONB NOT NULL DEFAULT '{}', -- Rate limit configuration
    is_active BOOLEAN NOT NULL DEFAULT true,
    CONSTRAINT api_keys_user_name_unique UNIQUE (user_id, name)
);

-- Index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_value ON api_keys(key_value);
CREATE INDEX IF NOT EXISTS idx_api_keys_expires_at ON api_keys(expires_at);

-- Enterprise Teams table for team-based permissions
CREATE TABLE IF NOT EXISTS enterprise_teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    plan_tier VARCHAR(50) NOT NULL DEFAULT 'basic',
    monthly_quota BIGINT NOT NULL DEFAULT 10000,
    current_usage BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- Enterprise Team Members table
CREATE TABLE IF NOT EXISTS enterprise_team_members (
    team_id UUID NOT NULL REFERENCES enterprise_teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member', -- 'admin', 'member', 'viewer'
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, user_id)
);

-- API Requests table for rate limiting and analytics
CREATE TABLE IF NOT EXISTS api_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER,
    request_size BIGINT,
    response_size BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT
);

-- Indexes for efficient API request queries
CREATE INDEX IF NOT EXISTS idx_api_requests_user_id ON api_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_api_requests_api_key_id ON api_requests(api_key_id);
CREATE INDEX IF NOT EXISTS idx_api_requests_created_at ON api_requests(created_at);
CREATE INDEX IF NOT EXISTS idx_api_requests_endpoint ON api_requests(endpoint);

-- Indexes for enterprise teams
CREATE INDEX IF NOT EXISTS idx_enterprise_team_members_user_id ON enterprise_team_members(user_id);
CREATE INDEX IF NOT EXISTS idx_enterprise_team_members_team_id ON enterprise_team_members(team_id);

-- Comments for documentation
COMMENT ON TABLE api_keys IS 'API keys for enhanced Bearer token authentication';
COMMENT ON TABLE enterprise_teams IS 'Enterprise teams for organizational API management';
COMMENT ON TABLE enterprise_team_members IS 'Members of enterprise teams with roles';
COMMENT ON TABLE api_requests IS 'API request logs for rate limiting and analytics';

COMMENT ON COLUMN api_keys.key_value IS 'Hashed API key value for security';
COMMENT ON COLUMN api_keys.permissions IS 'Array of structured permissions for this API key';
COMMENT ON COLUMN api_keys.rate_limit IS 'JSONB configuration for rate limiting rules';
COMMENT ON COLUMN enterprise_teams.plan_tier IS 'Subscription tier: basic, pro, enterprise';
COMMENT ON COLUMN enterprise_teams.monthly_quota IS 'Monthly API call quota for the team';
COMMENT ON COLUMN enterprise_team_members.role IS 'Team role: admin, member, viewer';