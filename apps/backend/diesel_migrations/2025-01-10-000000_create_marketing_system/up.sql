-- EPSX Marketing System: Plans, Promotions, and Affiliates
-- Migration: 2025-01-10-000000_create_marketing_system

-- =====================================================
-- PRICING PLANS SYSTEM
-- =====================================================

-- Core pricing plans with affiliate support
CREATE TABLE pricing_plans (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    plan_type VARCHAR(50) NOT NULL CHECK (plan_type IN ('personal', 'api')),
    base_price DECIMAL(10,2) NOT NULL CHECK (base_price >= 0),
    current_price DECIMAL(10,2) NOT NULL CHECK (current_price >= 0),
    currency VARCHAR(10) NOT NULL DEFAULT 'USDT',
    features JSONB NOT NULL DEFAULT '[]',
    affiliate_commission_rate DECIMAL(5,2) DEFAULT 10.00 CHECK (affiliate_commission_rate >= 0 AND affiliate_commission_rate <= 100),
    display_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    is_highlighted BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for pricing plans
CREATE INDEX idx_pricing_plans_type_active ON pricing_plans(plan_type, is_active);
CREATE INDEX idx_pricing_plans_display_order ON pricing_plans(display_order);
CREATE INDEX idx_pricing_plans_updated_at ON pricing_plans(updated_at);

-- =====================================================
-- PROMOTIONAL CAMPAIGNS SYSTEM  
-- =====================================================

-- Promotional campaigns
CREATE TABLE promotional_campaigns (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    campaign_type VARCHAR(50) NOT NULL CHECK (campaign_type IN ('discount', 'flash_sale', 'seasonal', 'new_user', 'affiliate_bonus')),
    is_active BOOLEAN DEFAULT true,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    priority INTEGER DEFAULT 0,
    affiliate_eligible BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT valid_campaign_dates CHECK (end_date IS NULL OR end_date > start_date)
);

-- Plan-specific promotions
CREATE TABLE plan_promotions (
    id SERIAL PRIMARY KEY,
    campaign_id INTEGER REFERENCES promotional_campaigns(id) ON DELETE CASCADE,
    plan_id INTEGER REFERENCES pricing_plans(id) ON DELETE CASCADE,
    discount_type VARCHAR(20) NOT NULL CHECK (discount_type IN ('percentage', 'fixed_amount', 'new_price')),
    discount_value DECIMAL(10,2) NOT NULL CHECK (discount_value >= 0),
    max_discount_amount DECIMAL(10,2),
    promotional_badge TEXT,
    promotional_message TEXT,
    affects_affiliate_commission BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(campaign_id, plan_id)
);

-- Discount codes with affiliate tracking
CREATE TABLE discount_codes (
    id SERIAL PRIMARY KEY,
    code VARCHAR(50) UNIQUE NOT NULL,
    campaign_id INTEGER REFERENCES promotional_campaigns(id) ON DELETE CASCADE,
    created_by_affiliate_id INTEGER, -- Will reference affiliates(id), added later due to dependency
    discount_type VARCHAR(20) NOT NULL CHECK (discount_type IN ('percentage', 'fixed_amount', 'new_price')),
    discount_value DECIMAL(10,2) NOT NULL CHECK (discount_value >= 0),
    max_uses INTEGER,
    current_uses INTEGER DEFAULT 0 CHECK (current_uses >= 0),
    user_limit INTEGER DEFAULT 1 CHECK (user_limit > 0),
    is_active BOOLEAN DEFAULT true,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_until TIMESTAMPTZ,
    applicable_plans JSONB, -- Array of plan IDs, null = all plans
    affiliate_bonus_rate DECIMAL(5,2), -- Extra commission for this code
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT valid_discount_dates CHECK (valid_until IS NULL OR valid_until > valid_from),
    CONSTRAINT valid_max_uses CHECK (max_uses IS NULL OR max_uses >= current_uses)
);

-- A/B Testing for pricing
CREATE TABLE pricing_experiments (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    plan_id INTEGER REFERENCES pricing_plans(id) ON DELETE CASCADE,
    variant_a_price DECIMAL(10,2) NOT NULL CHECK (variant_a_price >= 0),
    variant_b_price DECIMAL(10,2) NOT NULL CHECK (variant_b_price >= 0),
    traffic_split INTEGER DEFAULT 50 CHECK (traffic_split >= 0 AND traffic_split <= 100),
    is_active BOOLEAN DEFAULT false,
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT valid_experiment_dates CHECK (end_date IS NULL OR end_date > start_date)
);

-- =====================================================
-- AFFILIATE SYSTEM
-- =====================================================

-- Affiliate accounts
CREATE TABLE affiliates (
    id SERIAL PRIMARY KEY,
    user_id INTEGER,
    affiliate_code VARCHAR(50) UNIQUE NOT NULL,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'suspended', 'banned')),
    commission_rate DECIMAL(5,2) DEFAULT 15.00 CHECK (commission_rate >= 0 AND commission_rate <= 100),
    payment_method VARCHAR(50) CHECK (payment_method IN ('bank_transfer', 'crypto', 'paypal', 'other')),
    payment_details JSONB,
    tax_id VARCHAR(100),
    company_name VARCHAR(255),
    website VARCHAR(255),
    social_media JSONB,
    marketing_materials JSONB,
    total_referrals INTEGER DEFAULT 0 CHECK (total_referrals >= 0),
    total_sales DECIMAL(12,2) DEFAULT 0 CHECK (total_sales >= 0),
    total_commissions DECIMAL(12,2) DEFAULT 0 CHECK (total_commissions >= 0),
    lifetime_earnings DECIMAL(12,2) DEFAULT 0 CHECK (lifetime_earnings >= 0),
    approved_at TIMESTAMPTZ,
    last_activity TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Note: Foreign key constraints to users table will be added later when users table is available

-- Add the affiliate foreign key constraint for discount codes
ALTER TABLE discount_codes ADD CONSTRAINT fk_discount_codes_affiliate
    FOREIGN KEY (created_by_affiliate_id) REFERENCES affiliates(id) ON DELETE SET NULL;

-- Referral tracking
CREATE TABLE referrals (
    id SERIAL PRIMARY KEY,
    affiliate_id INTEGER REFERENCES affiliates(id) ON DELETE CASCADE,
    referred_user_id INTEGER,
    referral_source VARCHAR(100),
    referral_medium VARCHAR(100),
    referral_campaign VARCHAR(100),
    ip_address INET,
    user_agent TEXT,
    conversion_status VARCHAR(20) DEFAULT 'pending' CHECK (conversion_status IN ('pending', 'converted', 'expired')),
    converted_at TIMESTAMPTZ,
    attribution_window INTEGER DEFAULT 30 CHECK (attribution_window > 0),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(affiliate_id, referred_user_id)
);

-- Commission tracking
CREATE TABLE commissions (
    id SERIAL PRIMARY KEY,
    affiliate_id INTEGER REFERENCES affiliates(id) ON DELETE CASCADE,
    referral_id INTEGER REFERENCES referrals(id) ON DELETE SET NULL,
    order_id VARCHAR(255),
    plan_id INTEGER REFERENCES pricing_plans(id) ON DELETE SET NULL,
    commission_type VARCHAR(20) DEFAULT 'sale' CHECK (commission_type IN ('sale', 'recurring', 'bonus')),
    gross_amount DECIMAL(10,2) NOT NULL CHECK (gross_amount >= 0),
    commission_rate DECIMAL(5,2) NOT NULL CHECK (commission_rate >= 0),
    commission_amount DECIMAL(10,2) NOT NULL CHECK (commission_amount >= 0),
    currency VARCHAR(10) DEFAULT 'USDT',
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'paid', 'cancelled')),
    approved_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    payment_reference VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Affiliate payouts
CREATE TABLE affiliate_payouts (
    id SERIAL PRIMARY KEY,
    affiliate_id INTEGER REFERENCES affiliates(id) ON DELETE CASCADE,
    payout_period_start TIMESTAMPTZ NOT NULL,
    payout_period_end TIMESTAMPTZ NOT NULL,
    total_commissions DECIMAL(12,2) NOT NULL CHECK (total_commissions >= 0),
    payout_amount DECIMAL(12,2) NOT NULL CHECK (payout_amount >= 0),
    currency VARCHAR(10) DEFAULT 'USDT',
    payment_method VARCHAR(50),
    payment_details JSONB,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    transaction_reference VARCHAR(255),
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT valid_payout_period CHECK (payout_period_end > payout_period_start)
);

-- Affiliate marketing materials
CREATE TABLE affiliate_materials (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('banner', 'text_link', 'email_template', 'landing_page', 'social_post')),
    category VARCHAR(100),
    content TEXT,
    image_url VARCHAR(500),
    dimensions VARCHAR(20),
    target_plan_id INTEGER REFERENCES pricing_plans(id) ON DELETE SET NULL,
    click_tracking BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    usage_count INTEGER DEFAULT 0 CHECK (usage_count >= 0),
    conversion_rate DECIMAL(5,2) DEFAULT 0 CHECK (conversion_rate >= 0),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Affiliate tiers for multi-level commissions
CREATE TABLE affiliate_tiers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    min_referrals INTEGER DEFAULT 0 CHECK (min_referrals >= 0),
    min_sales DECIMAL(12,2) DEFAULT 0 CHECK (min_sales >= 0),
    commission_rate DECIMAL(5,2) NOT NULL CHECK (commission_rate >= 0 AND commission_rate <= 100),
    bonus_rate DECIMAL(5,2) DEFAULT 0 CHECK (bonus_rate >= 0),
    perks JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =====================================================
-- INDEXES FOR PERFORMANCE
-- =====================================================

-- Promotional campaigns indexes
CREATE INDEX idx_promotional_campaigns_active ON promotional_campaigns(is_active, start_date, end_date);
CREATE INDEX idx_promotional_campaigns_dates ON promotional_campaigns(start_date, end_date);
CREATE INDEX idx_plan_promotions_plan_campaign ON plan_promotions(plan_id, campaign_id);

-- Discount codes indexes
CREATE INDEX idx_discount_codes_active ON discount_codes(is_active, valid_from, valid_until);
CREATE INDEX idx_discount_codes_affiliate ON discount_codes(created_by_affiliate_id);
CREATE INDEX idx_discount_codes_campaign ON discount_codes(campaign_id);

-- Affiliate indexes
CREATE INDEX idx_affiliates_code ON affiliates(affiliate_code);
CREATE INDEX idx_affiliates_status ON affiliates(status);
CREATE INDEX idx_affiliates_user ON affiliates(user_id);

-- Referral indexes
CREATE INDEX idx_referrals_affiliate ON referrals(affiliate_id);
CREATE INDEX idx_referrals_user ON referrals(referred_user_id);
CREATE INDEX idx_referrals_status ON referrals(conversion_status);
CREATE INDEX idx_referrals_expires ON referrals(expires_at);

-- Commission indexes
CREATE INDEX idx_commissions_affiliate ON commissions(affiliate_id);
CREATE INDEX idx_commissions_status ON commissions(status);
CREATE INDEX idx_commissions_created ON commissions(created_at);
CREATE INDEX idx_commissions_referral ON commissions(referral_id);

-- Payout indexes
CREATE INDEX idx_affiliate_payouts_affiliate ON affiliate_payouts(affiliate_id);
CREATE INDEX idx_affiliate_payouts_status ON affiliate_payouts(status);
CREATE INDEX idx_affiliate_payouts_period ON affiliate_payouts(payout_period_start, payout_period_end);

-- Marketing materials indexes
CREATE INDEX idx_affiliate_materials_active ON affiliate_materials(is_active, type);
CREATE INDEX idx_affiliate_materials_plan ON affiliate_materials(target_plan_id);

-- =====================================================
-- SAMPLE DATA FOR TESTING
-- =====================================================

-- Insert sample pricing plans
INSERT INTO pricing_plans (name, plan_type, base_price, current_price, currency, features, affiliate_commission_rate, display_order, is_highlighted) VALUES
('Bronze Plan', 'personal', 0.00, 0.00, 'USDT', '["Limited access", "Basic features", "Community support"]', 10.00, 1, false),
('Silver Plan', 'personal', 1.00, 1.00, 'USDT', '["Full access for 1 month", "Priority support", "Advanced features"]', 15.00, 2, false),
('Gold Plan', 'personal', 9.90, 9.90, 'USDT', '["Extended access", "Premium features", "Priority support", "Early access to new features"]', 20.00, 3, true),
('Diamond Plan', 'personal', 99.90, 99.90, 'USDT', '["Enterprise features", "24/7 support", "White-label options", "Custom integrations", "Bulk operations"]', 25.00, 4, false),
('VIP Plan', 'personal', 499.90, 499.90, 'USDT', '["Unlimited features", "Personal account manager", "Custom solutions", "Priority development", "Dedicated infrastructure"]', 30.00, 5, false),
('API Personal', 'api', 999.00, 999.00, 'USDT', '["25 Data sets", "Country Selection", "Unlimited Accounts"]', 15.00, 6, false),
('API Company', 'api', 2999.00, 2999.00, 'USDT', '["100 Data sets", "Country Selection", "Unlimited Accounts", "Priority Support"]', 20.00, 7, true),
('API Partner', 'api', 0.00, 0.00, 'USDT', '["100 Data sets", "Country Selection", "Industry Selection", "15% Revenue Share", "Unlimited Accounts", "Custom Integration"]', 10.00, 8, false);

-- Insert sample affiliate tiers
INSERT INTO affiliate_tiers (name, min_referrals, min_sales, commission_rate, bonus_rate, perks) VALUES
('Bronze', 0, 0.00, 10.00, 0.00, '["Basic marketing materials", "Standard support"]'),
('Silver', 10, 1000.00, 15.00, 2.50, '["Enhanced marketing materials", "Priority support", "Monthly webinars"]'),
('Gold', 25, 5000.00, 20.00, 5.00, '["Premium marketing materials", "Dedicated support", "Custom materials", "Performance bonuses"]'),
('Platinum', 50, 15000.00, 25.00, 7.50, '["Exclusive materials", "Personal account manager", "Custom landing pages", "Early access to features"]');

-- Insert sample promotional campaign
INSERT INTO promotional_campaigns (name, description, campaign_type, is_active, start_date, end_date, priority, affiliate_eligible) VALUES
('New Year Sale 2025', 'Kick off the new year with amazing discounts on all plans!', 'seasonal', true, '2025-01-01 00:00:00+00', '2025-01-31 23:59:59+00', 10, true);

-- Insert sample plan promotions
INSERT INTO plan_promotions (campaign_id, plan_id, discount_type, discount_value, promotional_badge, promotional_message, affects_affiliate_commission) VALUES
(1, 2, 'percentage', 20.00, '🎉 20% OFF', 'New Year Special - Save 20%!', false),
(1, 3, 'percentage', 30.00, '🔥 30% OFF', 'Best Deal - Save 30% this January!', false),
(1, 4, 'fixed_amount', 20.00, '💰 $20 OFF', 'Premium savings - $20 off Diamond!', false);

-- Insert sample marketing materials
INSERT INTO affiliate_materials (name, type, category, content, image_url, dimensions, is_active) VALUES
('EPSX Banner - General', 'banner', 'general', '<img src="/banners/epsx-general-728x90.png" alt="EPSX - Advanced Analytics Platform">', '/banners/epsx-general-728x90.png', '728x90', true),
('Gold Plan Text Link', 'text_link', 'specific_plan', 'Get 30% off EPSX Gold Plan - Advanced analytics for serious traders!', null, null, true),
('New Year Email Template', 'email_template', 'seasonal', '<h2>🎉 New Year Sale at EPSX!</h2><p>Start 2025 with the best trading analytics platform...</p>', null, null, true);

-- Update pricing based on promotions (this would normally be handled by the application)
UPDATE pricing_plans SET current_price = base_price * 0.80 WHERE id = 2; -- Silver 20% off
UPDATE pricing_plans SET current_price = base_price * 0.70 WHERE id = 3; -- Gold 30% off  
UPDATE pricing_plans SET current_price = base_price - 20.00 WHERE id = 4; -- Diamond $20 off