-- Rollback EPSX Marketing System
-- Migration: 2025-01-10-000000_create_marketing_system

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS affiliate_materials CASCADE;
DROP TABLE IF EXISTS affiliate_tiers CASCADE;
DROP TABLE IF EXISTS affiliate_payouts CASCADE;
DROP TABLE IF EXISTS commissions CASCADE;
DROP TABLE IF EXISTS referrals CASCADE;
DROP TABLE IF EXISTS affiliates CASCADE;
DROP TABLE IF EXISTS pricing_experiments CASCADE;
DROP TABLE IF EXISTS discount_codes CASCADE;
DROP TABLE IF EXISTS plan_promotions CASCADE;
DROP TABLE IF EXISTS promotional_campaigns CASCADE;
DROP TABLE IF EXISTS pricing_plans CASCADE;

-- Drop indexes (automatically dropped with tables, but explicit for clarity)
-- Indexes will be automatically dropped when tables are dropped