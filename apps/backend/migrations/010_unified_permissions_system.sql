-- Unified Permission System Migration
-- This migration creates comprehensive permission tables for the EPSX trading platform
-- Supporting both admin modules and package tiers with inheritance, caching, and audit trails

-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "btree_gist";

-- Permission types and enums
CREATE TYPE permission_scope AS ENUM ('global', 'module', 'resource', 'user');
CREATE TYPE permission_level AS ENUM ('read', 'write', 'admin', 'owner');
CREATE TYPE admin_module AS ENUM (
    'user-management',
    'analytics-access', 
    'system-configuration',
    'audit-logs',
    'financial-oversight',
    'content-management',
    'support-access',
    'security-management'
);
CREATE TYPE admin_module_permission AS ENUM ('view', 'create', 'update', 'delete', 'admin', 'owner');
CREATE TYPE tier_feature AS ENUM (
    'basic-trading',
    'advanced-analytics',
    'api-access',
    'priority-support',
    'advanced-orders',
    'portfolio-tools',
    'research-reports',
    'institutional-features'
);
CREATE TYPE tier_limit_type AS ENUM (
    'requests-per-minute',
    'requests-per-hour', 
    'requests-per-day',
    'concurrent-connections',
    'data-export-mb',
    'storage-mb',
    'api-calls',
    'advanced-features',
    'support-tickets'
);
CREATE TYPE tier_reset_period AS ENUM ('minute', 'hour', 'day', 'week', 'month', 'year', 'never');
CREATE TYPE package_tier AS ENUM ('free', 'bronze', 'silver', 'gold', 'platinum', 'admin', 'super_admin');
CREATE TYPE denial_reason AS ENUM (
    'insufficient-permissions',
    'expired-permission',
    'resource-not-found',
    'policy-violation',
    'rate-limited',
    'security-threat',
    'maintenance-mode',
    'insufficient-tier',
    'module-disabled',
    'temporary-block'
);

-- Core permissions table
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    permission_id VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL DEFAULT '*',
    scope permission_scope NOT NULL DEFAULT 'global',
    level permission_level NOT NULL DEFAULT 'read',
    conditions JSONB,
    expires_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes for performance
    UNIQUE(permission_id, resource, scope)
);

-- User permission profiles (combining all permission sources)
CREATE TABLE user_permission_profiles (
    user_id VARCHAR(255) PRIMARY KEY,
    package_tier package_tier NOT NULL DEFAULT 'free',
    enabled_admin_modules admin_module[] DEFAULT '{}',
    enabled_tier_features tier_feature[] DEFAULT '{}',
    direct_permissions UUID[] DEFAULT '{}', -- References to permissions table
    inherited_permissions UUID[] DEFAULT '{}',
    temporary_permissions UUID[] DEFAULT '{}',
    denied_permissions UUID[] DEFAULT '{}',
    last_updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cache_version BIGINT NOT NULL DEFAULT 1,
    expires_at TIMESTAMPTZ,
    trial_mode BOOLEAN DEFAULT FALSE,
    auto_renew BOOLEAN DEFAULT FALSE,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Admin module access configurations
CREATE TABLE admin_module_access (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    module admin_module NOT NULL,
    permissions admin_module_permission[] NOT NULL DEFAULT '{}',
    granted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    granted_by VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ,
    conditions JSONB,
    reason TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (granted_by) REFERENCES users(id) ON DELETE RESTRICT,
    UNIQUE(user_id, module) -- One access record per user per module
);

-- Package tier access configurations
CREATE TABLE package_tier_access (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    package_tier package_tier NOT NULL,
    enabled_features tier_feature[] DEFAULT '{}',
    upgraded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ,
    auto_renew BOOLEAN DEFAULT FALSE,
    trial_mode BOOLEAN DEFAULT FALSE,
    billing_cycle_days INTEGER DEFAULT 30,
    last_billing_date TIMESTAMPTZ,
    next_billing_date TIMESTAMPTZ,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Tier limits for rate limiting and quotas
CREATE TABLE tier_limits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    feature tier_feature NOT NULL,
    limit_type tier_limit_type NOT NULL,
    max_value BIGINT NOT NULL, -- -1 for unlimited
    current_value BIGINT NOT NULL DEFAULT 0,
    reset_period tier_reset_period,
    burst_allowance BIGINT,
    last_reset TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, feature, limit_type)
);

-- Permission inheritance mappings
CREATE TABLE permission_inheritance (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    parent_permission_id UUID NOT NULL,
    child_permission_id UUID NOT NULL,
    inheritance_type VARCHAR(50) NOT NULL DEFAULT 'explicit', -- explicit, implicit, hierarchical
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (parent_permission_id) REFERENCES permissions(id) ON DELETE CASCADE,
    FOREIGN KEY (child_permission_id) REFERENCES permissions(id) ON DELETE CASCADE,
    UNIQUE(parent_permission_id, child_permission_id)
);

-- Temporary permission grants
CREATE TABLE temporary_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    permission VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL DEFAULT '*',
    action VARCHAR(100) NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    granted_by VARCHAR(255) NOT NULL,
    reason TEXT,
    conditions JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, expired, revoked
    revoked_at TIMESTAMPTZ,
    revoked_by VARCHAR(255),
    revocation_reason TEXT,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (granted_by) REFERENCES users(id) ON DELETE RESTRICT,
    FOREIGN KEY (revoked_by) REFERENCES users(id) ON DELETE RESTRICT
);

-- Permission audit logs (comprehensive logging for all permission operations)
CREATE TABLE permission_audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(50) NOT NULL, -- validation, grant, revoke, escalation, violation
    user_id VARCHAR(255) NOT NULL,
    permission VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    action VARCHAR(100) NOT NULL,
    result VARCHAR(20) NOT NULL, -- granted, denied, error
    denial_reason denial_reason,
    request_id UUID,
    session_id VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    context_data JSONB DEFAULT '{}',
    security_score INTEGER DEFAULT 0, -- 0-100 risk score
    performance_ms INTEGER, -- Response time in milliseconds
    cache_hit BOOLEAN DEFAULT FALSE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Additional security fields
    geolocation_country VARCHAR(2),
    geolocation_city VARCHAR(100),
    device_fingerprint VARCHAR(255),
    threat_indicators TEXT[], -- Array of detected threats
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Permission validation cache (Redis-backed cache metadata)
CREATE TABLE permission_cache_metadata (
    cache_key VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    permission VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    cached_result JSONB NOT NULL,
    cache_version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    last_accessed TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    access_count INTEGER NOT NULL DEFAULT 1,
    hit_rate DECIMAL(5,4) DEFAULT 1.0000,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Permission templates for common role setups
CREATE TABLE permission_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    category VARCHAR(100) NOT NULL, -- user, moderator, admin, custom
    target_tier package_tier,
    admin_modules admin_module[] DEFAULT '{}',
    tier_features tier_feature[] DEFAULT '{}',
    permissions_config JSONB NOT NULL DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(255) NOT NULL,
    version VARCHAR(20) NOT NULL DEFAULT '1.0',
    
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE RESTRICT
);

-- Permission policy violations (for security monitoring)
CREATE TABLE permission_policy_violations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    violation_type VARCHAR(100) NOT NULL, -- elevation_attempt, pattern_abuse, brute_force
    permission VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium', -- low, medium, high, critical
    detected_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ip_address INET,
    user_agent TEXT,
    request_pattern JSONB,
    mitigation_actions TEXT[],
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_at TIMESTAMPTZ,
    resolved_by VARCHAR(255),
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (resolved_by) REFERENCES users(id) ON DELETE RESTRICT
);

-- Performance monitoring for permission system
CREATE TABLE permission_performance_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_type VARCHAR(50) NOT NULL, -- validation_time, cache_hit_rate, error_rate
    metric_value DECIMAL(10,4) NOT NULL,
    measurement_window INTERVAL NOT NULL DEFAULT '1 hour',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    tags JSONB DEFAULT '{}',
    
    -- Partitioning hint
    created_date DATE NOT NULL DEFAULT CURRENT_DATE
);

-- Indexes for optimal performance
CREATE INDEX idx_permissions_permission_id ON permissions(permission_id);
CREATE INDEX idx_permissions_scope_level ON permissions(scope, level);
CREATE INDEX idx_permissions_expires_at ON permissions(expires_at) WHERE expires_at IS NOT NULL;

CREATE INDEX idx_user_permission_profiles_tier ON user_permission_profiles(package_tier);
CREATE INDEX idx_user_permission_profiles_updated ON user_permission_profiles(last_updated);
CREATE INDEX idx_user_permission_profiles_expires ON user_permission_profiles(expires_at) WHERE expires_at IS NOT NULL;

CREATE INDEX idx_admin_module_access_user_module ON admin_module_access(user_id, module);
CREATE INDEX idx_admin_module_access_expires ON admin_module_access(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_admin_module_access_active ON admin_module_access(is_active) WHERE is_active = TRUE;

CREATE INDEX idx_package_tier_access_user_tier ON package_tier_access(user_id, package_tier);
CREATE INDEX idx_package_tier_access_expires ON package_tier_access(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_package_tier_access_trial ON package_tier_access(trial_mode) WHERE trial_mode = TRUE;

CREATE INDEX idx_tier_limits_user_feature ON tier_limits(user_id, feature, limit_type);
CREATE INDEX idx_tier_limits_reset ON tier_limits(last_reset);

CREATE INDEX idx_temporary_permissions_user_expires ON temporary_permissions(user_id, expires_at);
CREATE INDEX idx_temporary_permissions_status ON temporary_permissions(status);

CREATE INDEX idx_permission_audit_logs_user_timestamp ON permission_audit_logs(user_id, timestamp DESC);
CREATE INDEX idx_permission_audit_logs_permission ON permission_audit_logs(permission);
CREATE INDEX idx_permission_audit_logs_result ON permission_audit_logs(result);
CREATE INDEX idx_permission_audit_logs_security_score ON permission_audit_logs(security_score) WHERE security_score > 50;
CREATE INDEX idx_permission_audit_logs_ip ON permission_audit_logs(ip_address);

CREATE INDEX idx_permission_cache_user ON permission_cache_metadata(user_id);
CREATE INDEX idx_permission_cache_expires ON permission_cache_metadata(expires_at);
CREATE INDEX idx_permission_cache_accessed ON permission_cache_metadata(last_accessed);

CREATE INDEX idx_permission_templates_category ON permission_templates(category);
CREATE INDEX idx_permission_templates_tier ON permission_templates(target_tier);
CREATE INDEX idx_permission_templates_active ON permission_templates(is_active) WHERE is_active = TRUE;

CREATE INDEX idx_permission_violations_user_severity ON permission_policy_violations(user_id, severity);
CREATE INDEX idx_permission_violations_detected ON permission_policy_violations(detected_at);
CREATE INDEX idx_permission_violations_resolved ON permission_policy_violations(resolved) WHERE resolved = FALSE;

-- Partial index for performance metrics by type and recent data
CREATE INDEX idx_permission_performance_type_recent ON permission_performance_metrics(metric_type, recorded_at DESC) 
    WHERE recorded_at > CURRENT_TIMESTAMP - INTERVAL '7 days';

-- GIN indexes for JSONB columns
CREATE INDEX idx_permissions_conditions ON permissions USING GIN(conditions);
CREATE INDEX idx_permissions_metadata ON permissions USING GIN(metadata);
CREATE INDEX idx_permission_audit_logs_context ON permission_audit_logs USING GIN(context_data);
CREATE INDEX idx_permission_templates_config ON permission_templates USING GIN(permissions_config);

-- Array indexes for better performance on array operations
CREATE INDEX idx_user_permission_profiles_admin_modules ON user_permission_profiles USING GIN(enabled_admin_modules);
CREATE INDEX idx_user_permission_profiles_tier_features ON user_permission_profiles USING GIN(enabled_tier_features);
CREATE INDEX idx_admin_module_access_permissions ON admin_module_access USING GIN(permissions);
CREATE INDEX idx_package_tier_access_features ON package_tier_access USING GIN(enabled_features);

-- Triggers for automatic timestamp updates
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER permissions_updated_at BEFORE UPDATE ON permissions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER tier_limits_updated_at BEFORE UPDATE ON tier_limits
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER permission_templates_updated_at BEFORE UPDATE ON permission_templates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Trigger for updating user permission profile cache version
CREATE OR REPLACE FUNCTION increment_cache_version()
RETURNS TRIGGER AS $$
BEGIN
    NEW.cache_version = OLD.cache_version + 1;
    NEW.last_updated = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_permission_profiles_cache_version 
    BEFORE UPDATE ON user_permission_profiles
    FOR EACH ROW EXECUTE FUNCTION increment_cache_version();

-- Function to automatically expire permissions
CREATE OR REPLACE FUNCTION cleanup_expired_permissions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    -- Clean up expired temporary permissions
    UPDATE temporary_permissions 
    SET status = 'expired'
    WHERE expires_at < CURRENT_TIMESTAMP AND status = 'active';
    
    -- Clean up expired cache entries
    DELETE FROM permission_cache_metadata 
    WHERE expires_at < CURRENT_TIMESTAMP;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Log cleanup operation
    INSERT INTO permission_audit_logs (
        event_type, user_id, permission, resource, action, result, 
        context_data, timestamp
    ) VALUES (
        'cleanup', 'system', 'expired-permissions', '*', 'cleanup', 'granted',
        json_build_object('cleaned_count', deleted_count)::jsonb,
        CURRENT_TIMESTAMP
    );
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to reset tier limits based on reset period
CREATE OR REPLACE FUNCTION reset_tier_limits()
RETURNS INTEGER AS $$
DECLARE
    reset_count INTEGER;
    limit_record RECORD;
BEGIN
    reset_count := 0;
    
    FOR limit_record IN 
        SELECT * FROM tier_limits 
        WHERE reset_period IS NOT NULL 
        AND reset_period != 'never'
    LOOP
        -- Check if limit should be reset based on reset period
        IF (
            (limit_record.reset_period = 'minute' AND EXTRACT(MINUTE FROM CURRENT_TIMESTAMP) != EXTRACT(MINUTE FROM limit_record.last_reset)) OR
            (limit_record.reset_period = 'hour' AND EXTRACT(HOUR FROM CURRENT_TIMESTAMP) != EXTRACT(HOUR FROM limit_record.last_reset)) OR
            (limit_record.reset_period = 'day' AND DATE(CURRENT_TIMESTAMP) != DATE(limit_record.last_reset)) OR
            (limit_record.reset_period = 'week' AND EXTRACT(WEEK FROM CURRENT_TIMESTAMP) != EXTRACT(WEEK FROM limit_record.last_reset)) OR
            (limit_record.reset_period = 'month' AND EXTRACT(MONTH FROM CURRENT_TIMESTAMP) != EXTRACT(MONTH FROM limit_record.last_reset)) OR
            (limit_record.reset_period = 'year' AND EXTRACT(YEAR FROM CURRENT_TIMESTAMP) != EXTRACT(YEAR FROM limit_record.last_reset))
        ) THEN
            UPDATE tier_limits 
            SET current_value = 0, last_reset = CURRENT_TIMESTAMP
            WHERE id = limit_record.id;
            
            reset_count := reset_count + 1;
        END IF;
    END LOOP;
    
    RETURN reset_count;
END;
$$ LANGUAGE plpgsql;

-- View for effective user permissions (combines all sources)
CREATE OR REPLACE VIEW user_effective_permissions AS
SELECT 
    upp.user_id,
    upp.package_tier,
    upp.enabled_admin_modules,
    upp.enabled_tier_features,
    
    -- Direct permissions
    COALESCE(
        (SELECT array_agg(permission_id) 
         FROM permissions p 
         WHERE p.id = ANY(upp.direct_permissions)
         AND (p.expires_at IS NULL OR p.expires_at > CURRENT_TIMESTAMP)),
        '{}'::varchar[]
    ) as direct_permissions,
    
    -- Active admin module access
    COALESCE(
        (SELECT array_agg(DISTINCT module::text) 
         FROM admin_module_access ama 
         WHERE ama.user_id = upp.user_id 
         AND ama.is_active = TRUE
         AND (ama.expires_at IS NULL OR ama.expires_at > CURRENT_TIMESTAMP)),
        '{}'::text[]
    ) as active_admin_modules,
    
    -- Current tier limits
    COALESCE(
        (SELECT json_agg(
            json_build_object(
                'feature', feature,
                'limit_type', limit_type,
                'max_value', max_value,
                'current_value', current_value,
                'remaining', GREATEST(0, max_value - current_value)
            )
        ) 
         FROM tier_limits tl 
         WHERE tl.user_id = upp.user_id),
        '[]'::json
    ) as tier_limits,
    
    upp.last_updated,
    upp.cache_version,
    
    -- Computed fields
    CASE 
        WHEN upp.expires_at IS NULL THEN FALSE
        WHEN upp.expires_at <= CURRENT_TIMESTAMP THEN TRUE
        ELSE FALSE
    END as is_expired,
    
    CASE 
        WHEN upp.trial_mode = TRUE THEN 'trial'
        WHEN upp.expires_at IS NOT NULL AND upp.expires_at <= CURRENT_TIMESTAMP THEN 'expired'
        ELSE 'active'
    END as status

FROM user_permission_profiles upp;

-- Insert default permission templates
INSERT INTO permission_templates (name, description, category, target_tier, admin_modules, tier_features, permissions_config, created_by) VALUES
-- User templates
('Free User', 'Basic free tier user with limited features', 'user', 'free', '{}', '{"basic-trading"}', 
 '{"permissions": ["dashboard:read", "profile:read", "profile:write"]}', 'system'),

('Bronze User', 'Bronze tier user with API access', 'user', 'bronze', '{}', '{"basic-trading", "api-access"}',
 '{"permissions": ["dashboard:read", "profile:read", "profile:write", "api:basic"]}', 'system'),

('Silver User', 'Silver tier user with analytics access', 'user', 'silver', '{}', '{"basic-trading", "advanced-analytics", "api-access", "portfolio-tools"}',
 '{"permissions": ["dashboard:read", "profile:read", "profile:write", "api:standard", "analytics:basic"]}', 'system'),

('Gold User', 'Gold tier user with advanced features', 'user', 'gold', '{}', '{"basic-trading", "advanced-analytics", "api-access", "priority-support", "advanced-orders", "portfolio-tools", "research-reports"}',
 '{"permissions": ["dashboard:read", "profile:read", "profile:write", "api:premium", "analytics:advanced", "trading:advanced"]}', 'system'),

-- Admin templates
('Content Moderator', 'Moderator with content management permissions', 'moderator', 'silver', '{"content-management"}', '{"basic-trading", "advanced-analytics"}',
 '{"permissions": ["content:read", "content:moderate", "users:read-basic"]}', 'system'),

('User Manager', 'Admin with user management capabilities', 'admin', 'admin', '{"user-management"}', '{"basic-trading", "advanced-analytics", "api-access", "priority-support"}',
 '{"permissions": ["users:read", "users:write", "users:delete", "roles:manage"]}', 'system'),

('Security Manager', 'Admin with security management permissions', 'admin', 'admin', '{"security-management", "audit-logs"}', '{"basic-trading", "advanced-analytics", "api-access", "priority-support"}',
 '{"permissions": ["security:read", "security:write", "audit:read", "incidents:manage"]}', 'system'),

('System Administrator', 'Full system admin with all permissions', 'admin', 'super_admin', '{"user-management", "analytics-access", "system-configuration", "audit-logs", "financial-oversight", "content-management", "support-access", "security-management"}', '{"basic-trading", "advanced-analytics", "api-access", "priority-support", "advanced-orders", "portfolio-tools", "research-reports", "institutional-features"}',
 '{"permissions": ["*:*"]}', 'system');

-- Insert performance monitoring baseline
INSERT INTO permission_performance_metrics (metric_type, metric_value, measurement_window, tags) VALUES
('baseline_validation_time_ms', 5.0, '1 hour', '{"component": "permission_system", "version": "1.0"}'),
('baseline_cache_hit_rate', 0.85, '1 hour', '{"component": "permission_cache", "version": "1.0"}'),
('baseline_error_rate', 0.001, '1 hour', '{"component": "permission_validation", "version": "1.0"}');

-- Comments for documentation
COMMENT ON TABLE permissions IS 'Core permissions table storing all individual permissions';
COMMENT ON TABLE user_permission_profiles IS 'User permission profiles combining all permission sources';
COMMENT ON TABLE admin_module_access IS 'Admin module access configurations for granular admin permissions';
COMMENT ON TABLE package_tier_access IS 'Package tier access configurations for feature-based permissions';
COMMENT ON TABLE tier_limits IS 'Rate limits and quotas per tier and feature';
COMMENT ON TABLE temporary_permissions IS 'Temporary permission grants with expiration';
COMMENT ON TABLE permission_audit_logs IS 'Comprehensive audit log for all permission operations';
COMMENT ON TABLE permission_cache_metadata IS 'Metadata for Redis-backed permission cache';
COMMENT ON TABLE permission_templates IS 'Templates for common permission setups';
COMMENT ON TABLE permission_policy_violations IS 'Security violations and policy breaches';
COMMENT ON TABLE permission_performance_metrics IS 'Performance monitoring for the permission system';

COMMENT ON VIEW user_effective_permissions IS 'Materialized view of effective permissions per user combining all sources';

-- Final verification
DO $$
BEGIN
    RAISE NOTICE 'Unified Permission System migration completed successfully';
    RAISE NOTICE 'Created % tables and % indexes for enterprise-grade permission management', 
        (SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE '%permission%'),
        (SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public' AND tablename LIKE '%permission%');
END $$;