-- Enhanced Access Control Migration
-- Adds permission hierarchy and dynamic policy support to EPSX

-- Permission Hierarchy Support
CREATE TABLE permission_hierarchy (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    parent_permission VARCHAR(255) NOT NULL,
    child_permission VARCHAR(255) NOT NULL,
    inheritance_type VARCHAR(50) NOT NULL DEFAULT 'automatic', -- 'automatic', 'conditional'
    inheritance_conditions JSONB,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id),
    
    -- Ensure no circular inheritance
    CONSTRAINT chk_no_self_inheritance CHECK (parent_permission != child_permission),
    
    -- Unique parent-child relationship
    UNIQUE(parent_permission, child_permission)
);

-- Indexes for permission hierarchy
CREATE INDEX idx_permission_hierarchy_parent ON permission_hierarchy(parent_permission);
CREATE INDEX idx_permission_hierarchy_child ON permission_hierarchy(child_permission);
CREATE INDEX idx_permission_hierarchy_active ON permission_hierarchy(is_active) WHERE is_active = true;

-- Dynamic Policy Engine Tables
CREATE TABLE dynamic_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    policy_type VARCHAR(100) NOT NULL, -- 'time_based', 'location_based', 'risk_based', 'device_based', 'behavioral'
    target_actions TEXT[] NOT NULL, -- Array of permission patterns this policy applies to
    conditions JSONB NOT NULL, -- Policy conditions in structured format
    actions JSONB NOT NULL, -- Actions to take when conditions are met
    priority INTEGER DEFAULT 100, -- Higher number = higher priority
    is_active BOOLEAN DEFAULT true,
    effective_from TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    effective_until TIMESTAMPTZ, -- NULL = no expiry
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id),
    last_modified_by UUID REFERENCES users(id)
);

-- Indexes for dynamic policies
CREATE INDEX idx_dynamic_policies_type ON dynamic_policies(policy_type);
CREATE INDEX idx_dynamic_policies_active ON dynamic_policies(is_active) WHERE is_active = true;
CREATE INDEX idx_dynamic_policies_priority ON dynamic_policies(priority DESC);
CREATE INDEX idx_dynamic_policies_effective ON dynamic_policies(effective_from, effective_until);
CREATE INDEX idx_dynamic_policies_target_actions ON dynamic_policies USING GIN(target_actions);

-- Policy Evaluation Log
CREATE TABLE policy_evaluations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    policy_id UUID REFERENCES dynamic_policies(id),
    user_id VARCHAR(128) NOT NULL, -- Firebase UID
    user_email VARCHAR(255),
    action_attempted VARCHAR(255) NOT NULL,
    resource_context JSONB, -- Context about the resource being accessed
    user_context JSONB, -- User attributes at time of evaluation
    environment_context JSONB, -- Environment attributes (time, location, device, etc.)
    decision VARCHAR(50) NOT NULL, -- 'allow', 'deny', 'require_mfa', 'require_approval', 'restricted_access'
    decision_reason TEXT,
    evaluation_time_ms INTEGER,
    policy_conditions_met JSONB, -- Which conditions were met/not met
    evaluated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes for performance
    INDEX idx_policy_evaluations_user_id (user_id),
    INDEX idx_policy_evaluations_policy_id (policy_id),
    INDEX idx_policy_evaluations_decision (decision),
    INDEX idx_policy_evaluations_evaluated_at (evaluated_at),
    INDEX idx_policy_evaluations_action (action_attempted)
);

-- Policy Templates for common use cases
CREATE TABLE policy_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL, -- 'financial', 'analytics', 'security', 'compliance'
    template_data JSONB NOT NULL, -- Template structure that can be instantiated
    usage_count INTEGER DEFAULT 0,
    is_public BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID REFERENCES users(id)
);

-- Indexes for policy templates
CREATE INDEX idx_policy_templates_category ON policy_templates(category);
CREATE INDEX idx_policy_templates_public ON policy_templates(is_public) WHERE is_public = true;

-- Policy Approval Queue (for policies that require manual approval)
CREATE TABLE policy_approval_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    policy_evaluation_id UUID REFERENCES policy_evaluations(id),
    user_id VARCHAR(128) NOT NULL,
    action_attempted VARCHAR(255) NOT NULL,
    approval_reason TEXT NOT NULL,
    current_status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'expired'
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    rejection_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes
    INDEX idx_policy_approval_queue_status (current_status),
    INDEX idx_policy_approval_queue_user (user_id),
    INDEX idx_policy_approval_queue_expires (expires_at)
);

-- Permission Inheritance Cache (for performance optimization)
CREATE TABLE permission_inheritance_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    direct_permissions TEXT[] NOT NULL,
    inherited_permissions TEXT[] NOT NULL,
    all_effective_permissions TEXT[] NOT NULL,
    cache_version INTEGER NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Ensure one cache entry per user
    UNIQUE(user_id)
);

-- Index for permission cache
CREATE INDEX idx_permission_cache_expires ON permission_inheritance_cache(expires_at);
CREATE INDEX idx_permission_cache_user ON permission_inheritance_cache(user_id);

-- Policy Performance Metrics
CREATE TABLE policy_performance_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    policy_id UUID REFERENCES dynamic_policies(id),
    metric_date DATE NOT NULL,
    evaluations_count INTEGER DEFAULT 0,
    avg_evaluation_time_ms NUMERIC(8,2),
    max_evaluation_time_ms INTEGER,
    allow_count INTEGER DEFAULT 0,
    deny_count INTEGER DEFAULT 0,
    require_mfa_count INTEGER DEFAULT 0,
    require_approval_count INTEGER DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    
    -- Unique constraint for one metric per policy per day
    UNIQUE(policy_id, metric_date)
);

-- Index for policy metrics
CREATE INDEX idx_policy_metrics_date ON policy_performance_metrics(metric_date);
CREATE INDEX idx_policy_metrics_policy ON policy_performance_metrics(policy_id);

-- Update triggers for timestamp management
CREATE TRIGGER update_permission_hierarchy_updated_at 
    BEFORE UPDATE ON permission_hierarchy
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_dynamic_policies_updated_at 
    BEFORE UPDATE ON dynamic_policies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add constraints to ensure data integrity
ALTER TABLE dynamic_policies 
ADD CONSTRAINT chk_valid_policy_type 
CHECK (policy_type IN ('time_based', 'location_based', 'risk_based', 'device_based', 'behavioral', 'compliance', 'custom'));

ALTER TABLE policy_evaluations 
ADD CONSTRAINT chk_valid_decision 
CHECK (decision IN ('allow', 'deny', 'require_mfa', 'require_approval', 'restricted_access'));

ALTER TABLE policy_approval_queue 
ADD CONSTRAINT chk_valid_approval_status 
CHECK (current_status IN ('pending', 'approved', 'rejected', 'expired'));

-- Functions for permission hierarchy resolution
CREATE OR REPLACE FUNCTION resolve_inherited_permissions(user_permissions TEXT[])
RETURNS TEXT[] AS $$
DECLARE
    result TEXT[];
    perm TEXT;
    inherited_perm TEXT;
BEGIN
    -- Start with direct permissions
    result := user_permissions;
    
    -- For each user permission, find inherited permissions
    FOREACH perm IN ARRAY user_permissions LOOP
        -- Find child permissions that should be inherited
        FOR inherited_perm IN 
            SELECT child_permission 
            FROM permission_hierarchy 
            WHERE parent_permission = perm 
            AND is_active = true
            AND inheritance_type = 'automatic'
        LOOP
            -- Add to result if not already present
            IF NOT (inherited_perm = ANY(result)) THEN
                result := array_append(result, inherited_perm);
            END IF;
        END LOOP;
    END LOOP;
    
    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Function to check if permissions create circular inheritance
CREATE OR REPLACE FUNCTION check_circular_inheritance(parent_perm TEXT, child_perm TEXT)
RETURNS BOOLEAN AS $$
DECLARE
    current_perm TEXT;
    visited_perms TEXT[] := ARRAY[]::TEXT[];
BEGIN
    current_perm := child_perm;
    
    -- Follow the chain to see if we get back to parent
    WHILE current_perm IS NOT NULL LOOP
        -- If we've seen this permission before, we have a cycle
        IF current_perm = ANY(visited_perms) THEN
            RETURN true;
        END IF;
        
        -- If we reached the parent permission, we have a cycle
        IF current_perm = parent_perm THEN
            RETURN true;
        END IF;
        
        -- Mark as visited
        visited_perms := array_append(visited_perms, current_perm);
        
        -- Get the next parent in the chain
        SELECT parent_permission INTO current_perm
        FROM permission_hierarchy
        WHERE child_permission = current_perm
        AND is_active = true
        LIMIT 1;
    END LOOP;
    
    RETURN false;
END;
$$ LANGUAGE plpgsql;

-- Constraint to prevent circular inheritance
ALTER TABLE permission_hierarchy 
ADD CONSTRAINT chk_no_circular_inheritance 
CHECK (NOT check_circular_inheritance(parent_permission, child_permission));

-- Insert default permission hierarchy for EPSX
INSERT INTO permission_hierarchy (parent_permission, child_permission, inheritance_type, created_by) VALUES
-- Analytics hierarchy
('epsx:analytics:*', 'epsx:analytics:view', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('epsx:analytics:*', 'epsx:analytics:export', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('epsx:analytics:*', 'epsx:analytics:advanced', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),

-- Trading hierarchy
('epsx:trading:*', 'epsx:trading:basic', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('epsx:trading:*', 'epsx:trading:advanced', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('epsx:trading:*', 'epsx:trading:execute', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('epsx:trading:advanced', 'epsx:trading:basic', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),

-- Portfolio hierarchy
('epsx:portfolio:*', 'epsx:portfolio:view', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('epsx:portfolio:*', 'epsx:portfolio:manage', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('epsx:portfolio:*', 'epsx:portfolio:history', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('epsx:portfolio:manage', 'epsx:portfolio:view', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),

-- Admin hierarchy  
('admin:*:*', 'admin:users:*', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('admin:*:*', 'admin:permissions:*', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('admin:*:*', 'admin:system:*', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('admin:users:*', 'admin:users:view', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('admin:users:*', 'admin:users:modify', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1)),
('admin:users:*', 'admin:users:delete', 'automatic', (SELECT id FROM users WHERE email = 'system@epsx.io' LIMIT 1));

-- Insert default policy templates
INSERT INTO policy_templates (name, description, category, template_data) VALUES
('High-Value Transaction Control', 'Require approval for transactions above specified threshold', 'financial', 
'{"conditions": {"trade_amount": {"operator": ">", "value": "$THRESHOLD", "currency": "USD"}}, "actions": {"primary": "require_approval", "secondary": ["log_audit", "notify_risk_team"]}, "configurable_params": ["THRESHOLD"]}'),

('After-Hours Trading Restriction', 'Control trading access outside market hours', 'financial',
'{"conditions": {"time": {"operator": "not_between", "start": "09:30", "end": "16:00", "timezone": "EST"}}, "actions": {"primary": "require_approval", "message": "After-hours trading requires approval"}, "configurable_params": ["start", "end", "timezone"]}'),

('Geographic Data Access (GDPR)', 'Restrict EU users to EU data only', 'compliance',
'{"conditions": {"user_location": "EU", "data_jurisdiction": {"operator": "!=", "value": "EU"}}, "actions": {"primary": "deny", "message": "GDPR compliance: EU users cannot access non-EU data"}}'),

('Device Trust Verification', 'Require additional verification for untrusted devices', 'security',
'{"conditions": {"device_trust_score": {"operator": "<", "value": "$TRUST_THRESHOLD"}}, "actions": {"primary": "require_mfa", "message": "Additional verification required for untrusted device"}, "configurable_params": ["TRUST_THRESHOLD"]}'),

('Bulk Export Rate Limiting', 'Limit bulk data exports during peak hours', 'analytics',
'{"conditions": {"action": "epsx:analytics:export", "time": {"operator": "between", "start": "09:00", "end": "17:00"}, "export_size": {"operator": ">", "value": "$SIZE_LIMIT"}}, "actions": {"primary": "restricted_access", "restrictions": {"max_records": "$REDUCED_LIMIT"}}, "configurable_params": ["SIZE_LIMIT", "REDUCED_LIMIT"]}');