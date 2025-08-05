-- Create Casbin policy table
CREATE TABLE casbin_rule (
    id SERIAL PRIMARY KEY,
    ptype VARCHAR(255) NOT NULL,
    v0 VARCHAR(255),
    v1 VARCHAR(255),
    v2 VARCHAR(255),
    v3 VARCHAR(255),
    v4 VARCHAR(255),
    v5 VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_casbin_rule_ptype ON casbin_rule (ptype);
CREATE INDEX idx_casbin_rule_v0 ON casbin_rule (v0);
CREATE INDEX idx_casbin_rule_v1 ON casbin_rule (v1);
CREATE INDEX idx_casbin_rule_v2 ON casbin_rule (v2);

-- Insert initial policies based on current IAM profiles
INSERT INTO casbin_rule (ptype, v0, v1, v2) VALUES
-- Admin roles
('p', 'admin-full-004', '*', '*'),
('p', 'moderator-standard-003', 'users', 'manage'),
('p', 'moderator-standard-003', 'content', 'moderate'),

-- User roles  
('p', 'user-premium-002', 'analytics', 'read'),
('p', 'user-premium-002', 'trading', 'advanced'),
('p', 'user-basic-001', 'trading', 'basic'),
('p', 'user-basic-001', 'market-data', 'read');