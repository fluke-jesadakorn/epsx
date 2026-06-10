-- Plan Features & Categories Migration
-- Adds feature registry, plan-feature junction, and plan categories

-- 1. Feature definitions registry
CREATE TABLE features (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    feature_type VARCHAR(10) NOT NULL CHECK (feature_type IN ('boolean', 'numeric', 'text')),
    merge_strategy VARCHAR(10) NOT NULL CHECK (merge_strategy IN ('max', 'sum', 'override', 'any', 'exclusive')),
    default_value JSONB NOT NULL DEFAULT 'null',
    tags TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_features_key ON features(key);
CREATE INDEX idx_features_active ON features(is_active);

-- 2. Junction: plan -> feature values
CREATE TABLE plan_features (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plan_id UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    feature_id UUID NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    value JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(plan_id, feature_id)
);

CREATE INDEX idx_plan_features_plan ON plan_features(plan_id);
CREATE INDEX idx_plan_features_feature ON plan_features(feature_id);

-- 3. Add plan_category to plans
ALTER TABLE plans ADD COLUMN plan_category VARCHAR(20) NOT NULL DEFAULT 'base';
ALTER TABLE plans ADD CONSTRAINT valid_plan_category
    CHECK (plan_category IN ('base', 'addon', 'system', 'exclusive'));

CREATE INDEX idx_plans_category ON plans(plan_category);

-- 4. Seed features from current PlanFeatures struct defaults
INSERT INTO features (key, name, description, feature_type, merge_strategy, default_value, tags, sort_order) VALUES
    ('api_calls_limit', 'API Calls Limit', 'Maximum API calls allowed', 'numeric', 'max', '100', '{}', 1),
    ('rankings_limit', 'Rankings Limit', 'Maximum rankings accessible', 'numeric', 'max', '3', '{}', 2),
    ('ranking_offset', 'Ranking Offset', 'Starting offset for rankings', 'numeric', 'override', '50', '{}', 3),
    ('analytics_enabled', 'Analytics Access', 'Access to analytics features', 'boolean', 'any', 'false', '{}', 4),
    ('premium_support', 'Premium Support', 'Access to premium support', 'boolean', 'any', 'false', '{"premium"}', 5),
    ('rate_limit_per_min', 'Rate Limit (per min)', 'Requests per minute', 'numeric', 'max', '60', '{}', 6),
    ('rate_limit_per_hour', 'Rate Limit (per hour)', 'Requests per hour', 'numeric', 'max', '1000', '{}', 7),
    ('rate_limit_per_day', 'Rate Limit (per day)', 'Requests per day', 'numeric', 'max', '10000', '{}', 8),
    ('burst_capacity', 'Burst Capacity', 'Maximum burst request count', 'numeric', 'max', '10', '{}', 9);

-- 5. Migrate existing plan data into plan_features
-- Extract rate limits and metadata from plans into plan_features rows
INSERT INTO plan_features (plan_id, feature_id, value)
SELECT p.id, f.id, to_jsonb(p.rate_limit_per_minute)
FROM plans p CROSS JOIN features f
WHERE f.key = 'rate_limit_per_min' AND p.rate_limit_per_minute != 60;

INSERT INTO plan_features (plan_id, feature_id, value)
SELECT p.id, f.id, to_jsonb(p.rate_limit_per_hour)
FROM plans p CROSS JOIN features f
WHERE f.key = 'rate_limit_per_hour' AND p.rate_limit_per_hour != 1000;

INSERT INTO plan_features (plan_id, feature_id, value)
SELECT p.id, f.id, to_jsonb(p.rate_limit_per_day)
FROM plans p CROSS JOIN features f
WHERE f.key = 'rate_limit_per_day' AND p.rate_limit_per_day != 10000;

INSERT INTO plan_features (plan_id, feature_id, value)
SELECT p.id, f.id, to_jsonb(p.burst_capacity)
FROM plans p CROSS JOIN features f
WHERE f.key = 'burst_capacity' AND p.burst_capacity != 10;

-- Extract ranking_offset from plan_metadata if present
INSERT INTO plan_features (plan_id, feature_id, value)
SELECT p.id, f.id, p.plan_metadata->'ranking_offset'
FROM plans p CROSS JOIN features f
WHERE f.key = 'ranking_offset'
  AND p.plan_metadata ? 'ranking_offset'
  AND p.plan_metadata->'ranking_offset' IS NOT NULL
ON CONFLICT (plan_id, feature_id) DO NOTHING;
