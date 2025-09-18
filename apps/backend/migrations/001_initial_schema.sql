-- SQLx migration: Initial database schema
-- Replaces Diesel schema with native PostgreSQL for Cloud Run compatibility

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Core Users table - Primary authentication table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    firebase_uid VARCHAR(128) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255),
    name VARCHAR(255),
    avatar_url TEXT,
    package_tier VARCHAR(50),
    email_verified BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    primary_platform_id UUID
);

-- Create indexes for performance (especially for login queries)
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX idx_users_is_active ON users(is_active);

-- User Sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    provider VARCHAR(50),
    session_token TEXT,
    user_agent TEXT,
    ip_address INET,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_access_token ON sessions(access_token);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_is_active ON sessions(is_active);

-- User Permissions table - Structured permissions system
CREATE TABLE user_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(255) NOT NULL, -- Format: platform:resource:action
    granted_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    granted_by UUID REFERENCES users(id),
    expires_at TIMESTAMPTZ, -- NULL for permanent permissions
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- GIN index for performance optimization (from CLAUDE.md notes)
CREATE INDEX idx_user_permissions_user_id ON user_permissions(user_id);
CREATE INDEX idx_user_permissions_permission ON user_permissions USING GIN (permission);
CREATE INDEX idx_user_permissions_expires_at ON user_permissions(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_user_permissions_active ON user_permissions(user_id, is_active) WHERE is_active = true;

-- Refresh Tokens table - OAuth token management
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id TEXT NOT NULL, -- Firebase UID reference
    token_hash TEXT NOT NULL,
    family_id UUID NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMPTZ,
    device_info JSONB,
    ip_address INET,
    user_agent TEXT,
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    revoked_at TIMESTAMPTZ,
    revoked_reason TEXT
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_family_id ON refresh_tokens(family_id);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
CREATE INDEX idx_refresh_tokens_revoked ON refresh_tokens(is_revoked);

-- Revoked Tokens table - Token revocation tracking
CREATE TABLE revoked_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    jti TEXT NOT NULL UNIQUE, -- JWT ID
    user_id TEXT NOT NULL, -- Firebase UID reference
    token_type TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked_by TEXT,
    revoked_reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_revoked_tokens_jti ON revoked_tokens(jti);
CREATE INDEX idx_revoked_tokens_user_id ON revoked_tokens(user_id);
CREATE INDEX idx_revoked_tokens_expires_at ON revoked_tokens(expires_at);

-- Pricing Plans table
CREATE TABLE pricing_plans (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    plan_type VARCHAR NOT NULL,
    base_price NUMERIC NOT NULL,
    current_price NUMERIC NOT NULL,
    currency VARCHAR NOT NULL,
    features JSONB NOT NULL,
    affiliate_commission_rate NUMERIC,
    display_order INTEGER,
    is_active BOOLEAN DEFAULT true,
    is_highlighted BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Promotional Campaigns table
CREATE TABLE promotional_campaigns (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    description TEXT,
    campaign_type VARCHAR NOT NULL,
    is_active BOOLEAN DEFAULT true,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    priority INTEGER,
    affiliate_eligible BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Plan Promotions table
CREATE TABLE plan_promotions (
    id SERIAL PRIMARY KEY,
    campaign_id INTEGER REFERENCES promotional_campaigns(id),
    plan_id INTEGER REFERENCES pricing_plans(id),
    discount_type VARCHAR NOT NULL,
    discount_value NUMERIC NOT NULL,
    max_discount_amount NUMERIC,
    promotional_badge TEXT,
    promotional_message TEXT,
    affects_affiliate_commission BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Discount Codes table
CREATE TABLE discount_codes (
    id SERIAL PRIMARY KEY,
    code VARCHAR NOT NULL UNIQUE,
    campaign_id INTEGER REFERENCES promotional_campaigns(id),
    created_by_affiliate_id INTEGER,
    discount_type VARCHAR NOT NULL,
    discount_value NUMERIC NOT NULL,
    max_uses INTEGER,
    current_uses INTEGER DEFAULT 0,
    user_limit INTEGER,
    is_active BOOLEAN DEFAULT true,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_until TIMESTAMPTZ,
    applicable_plans JSONB,
    affiliate_bonus_rate NUMERIC,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Pricing Experiments table
CREATE TABLE pricing_experiments (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    description TEXT,
    plan_id INTEGER REFERENCES pricing_plans(id),
    variant_a_price NUMERIC NOT NULL,
    variant_b_price NUMERIC NOT NULL,
    traffic_split INTEGER DEFAULT 50,
    is_active BOOLEAN DEFAULT false,
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Affiliates table
CREATE TABLE affiliates (
    id SERIAL PRIMARY KEY,
    user_id INTEGER,
    affiliate_code VARCHAR NOT NULL UNIQUE,
    status VARCHAR DEFAULT 'pending',
    commission_rate NUMERIC,
    payment_method VARCHAR,
    payment_details JSONB,
    tax_id VARCHAR,
    company_name VARCHAR,
    website VARCHAR,
    social_media JSONB,
    marketing_materials JSONB,
    total_referrals INTEGER DEFAULT 0,
    total_sales NUMERIC DEFAULT 0,
    total_commissions NUMERIC DEFAULT 0,
    lifetime_earnings NUMERIC DEFAULT 0,
    approved_at TIMESTAMPTZ,
    last_activity TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Referrals table
CREATE TABLE referrals (
    id SERIAL PRIMARY KEY,
    affiliate_id INTEGER REFERENCES affiliates(id),
    referreduser_id INTEGER,
    referral_source VARCHAR,
    referral_medium VARCHAR,
    referral_campaign VARCHAR,
    ip_address INET,
    user_agent TEXT,
    conversion_status VARCHAR DEFAULT 'pending',
    converted_at TIMESTAMPTZ,
    attribution_window INTEGER DEFAULT 30,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Commissions table
CREATE TABLE commissions (
    id SERIAL PRIMARY KEY,
    affiliate_id INTEGER REFERENCES affiliates(id),
    referral_id INTEGER REFERENCES referrals(id),
    order_id VARCHAR,
    plan_id INTEGER REFERENCES pricing_plans(id),
    commission_type VARCHAR DEFAULT 'standard',
    gross_amount NUMERIC NOT NULL,
    commission_rate NUMERIC NOT NULL,
    commission_amount NUMERIC NOT NULL,
    currency VARCHAR DEFAULT 'USD',
    status VARCHAR DEFAULT 'pending',
    approved_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    payment_reference VARCHAR,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Affiliate Payouts table
CREATE TABLE affiliate_payouts (
    id SERIAL PRIMARY KEY,
    affiliate_id INTEGER REFERENCES affiliates(id),
    payout_period_start TIMESTAMPTZ NOT NULL,
    payout_period_end TIMESTAMPTZ NOT NULL,
    total_commissions NUMERIC NOT NULL,
    payout_amount NUMERIC NOT NULL,
    currency VARCHAR DEFAULT 'USD',
    payment_method VARCHAR,
    payment_details JSONB,
    status VARCHAR DEFAULT 'pending',
    transaction_reference VARCHAR,
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Affiliate Materials table
CREATE TABLE affiliate_materials (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    material_type VARCHAR NOT NULL, -- 'type' is a reserved word, using material_type
    category VARCHAR,
    content TEXT,
    image_url VARCHAR,
    dimensions VARCHAR,
    target_plan_id INTEGER REFERENCES pricing_plans(id),
    click_tracking BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    usage_count INTEGER DEFAULT 0,
    conversion_rate NUMERIC,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Affiliate Tiers table
CREATE TABLE affiliate_tiers (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    min_referrals INTEGER DEFAULT 0,
    min_sales NUMERIC DEFAULT 0,
    commission_rate NUMERIC NOT NULL,
    bonus_rate NUMERIC,
    perks JSONB,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Add foreign key constraints for affiliates
ALTER TABLE discount_codes ADD CONSTRAINT fk_discount_codes_affiliate 
    FOREIGN KEY (created_by_affiliate_id) REFERENCES affiliates(id);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply updated_at triggers to relevant tables
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_permissions_updated_at BEFORE UPDATE ON user_permissions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_refresh_tokens_updated_at BEFORE UPDATE ON refresh_tokens
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_pricing_plans_updated_at BEFORE UPDATE ON pricing_plans
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_promotional_campaigns_updated_at BEFORE UPDATE ON promotional_campaigns
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_pricing_experiments_updated_at BEFORE UPDATE ON pricing_experiments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_affiliates_updated_at BEFORE UPDATE ON affiliates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_commissions_updated_at BEFORE UPDATE ON commissions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_affiliate_payouts_updated_at BEFORE UPDATE ON affiliate_payouts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_affiliate_materials_updated_at BEFORE UPDATE ON affiliate_materials
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_affiliate_tiers_updated_at BEFORE UPDATE ON affiliate_tiers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();