-- Casbin policy storage tables
-- This migration creates the tables needed for Casbin policy-based access control

-- Create the main casbin_rule table for storing all policy rules
CREATE TABLE IF NOT EXISTS casbin_rule (
    id SERIAL PRIMARY KEY,
    ptype VARCHAR(100) NOT NULL,  -- Policy type: p (policy), g (role inheritance)
    v0 VARCHAR(100),              -- Subject (user/role)
    v1 VARCHAR(100),              -- Object (resource/role)
    v2 VARCHAR(100),              -- Action (permission)
    v3 VARCHAR(100),              -- Additional field
    v4 VARCHAR(100),              -- Additional field
    v5 VARCHAR(100),              -- Additional field
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for efficient policy queries
CREATE INDEX IF NOT EXISTS idx_casbin_rule_ptype ON casbin_rule(ptype);
CREATE INDEX IF NOT EXISTS idx_casbin_rule_v0 ON casbin_rule(v0);
CREATE INDEX IF NOT EXISTS idx_casbin_rule_v1 ON casbin_rule(v1);
CREATE INDEX IF NOT EXISTS idx_casbin_rule_v0_v1 ON casbin_rule(v0, v1);
CREATE INDEX IF NOT EXISTS idx_casbin_rule_ptype_v0 ON casbin_rule(ptype, v0);

-- Create a unique constraint for policy rules to prevent duplicates
CREATE UNIQUE INDEX IF NOT EXISTS idx_casbin_rule_unique ON casbin_rule(ptype, COALESCE(v0, ''), COALESCE(v1, ''), COALESCE(v2, ''), COALESCE(v3, ''), COALESCE(v4, ''), COALESCE(v5, ''));

-- Insert default RBAC policies for EPSX system
-- Role hierarchy: admin > moderator > premium_user > basic_user

-- Define role inheritance (g policies)
INSERT INTO casbin_rule (ptype, v0, v1) VALUES
-- Role inheritance
('g', 'admin', 'moderator'),
('g', 'moderator', 'premium_user'),  
('g', 'premium_user', 'basic_user')
ON CONFLICT DO NOTHING;

-- Define permission policies (p policies)
INSERT INTO casbin_rule (ptype, v0, v1, v2) VALUES
-- Admin permissions (full access)
('p', 'admin', '/api/v1/admin/*', '*'),
('p', 'admin', '/api/v1/iam/*', '*'),
('p', 'admin', '/api/v1/users/*', '*'),
('p', 'admin', '/api/v1/trading/*', '*'),
('p', 'admin', '/api/v1/portfolio/*', '*'),
('p', 'admin', '/api/v1/market-data/*', '*'),
('p', 'admin', '/api/v1/trading-signals/*', '*'),

-- Moderator permissions (user management + trading)
('p', 'moderator', '/api/v1/users/*', 'GET'),
('p', 'moderator', '/api/v1/users/*', 'PUT'),
('p', 'moderator', '/api/v1/trading/*', '*'),
('p', 'moderator', '/api/v1/portfolio/*', '*'),
('p', 'moderator', '/api/v1/market-data/*', '*'),
('p', 'moderator', '/api/v1/trading-signals/*', '*'),

-- Premium user permissions (advanced trading features)
('p', 'premium_user', '/api/v1/users/profile', '*'),
('p', 'premium_user', '/api/v1/trading/*', '*'),
('p', 'premium_user', '/api/v1/portfolio/*', '*'),
('p', 'premium_user', '/api/v1/market-data/*', '*'),
('p', 'premium_user', '/api/v1/trading-signals/*', '*'),

-- Basic user permissions (limited trading)
('p', 'basic_user', '/api/v1/users/profile', 'GET'),
('p', 'basic_user', '/api/v1/users/profile', 'PUT'),
('p', 'basic_user', '/api/v1/trading/orders', 'GET'),
('p', 'basic_user', '/api/v1/trading/positions', 'GET'),
('p', 'basic_user', '/api/v1/portfolio/summary', 'GET'),
('p', 'basic_user', '/api/v1/market-data/quotes', 'GET')
ON CONFLICT DO NOTHING;

-- Create a view for easier policy querying
CREATE OR REPLACE VIEW casbin_policies AS
SELECT 
    id,
    ptype,
    v0 as subject,
    v1 as object,
    v2 as action,
    v3, v4, v5
FROM casbin_rule
WHERE ptype = 'p';

-- Create a view for role inheritances
CREATE OR REPLACE VIEW casbin_roles AS
SELECT 
    id,
    v0 as user_or_role,
    v1 as role
FROM casbin_rule
WHERE ptype = 'g';
