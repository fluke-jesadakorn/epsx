-- Add rate limit columns to plans table
-- These define the rate limits for each subscription plan

ALTER TABLE plans ADD COLUMN IF NOT EXISTS rate_limit_per_minute INTEGER NOT NULL DEFAULT 60;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS rate_limit_per_hour INTEGER NOT NULL DEFAULT 1000;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS rate_limit_per_day INTEGER NOT NULL DEFAULT 10000;
ALTER TABLE plans ADD COLUMN IF NOT EXISTS burst_capacity INTEGER NOT NULL DEFAULT 10;

-- Add comments for documentation
COMMENT ON COLUMN plans.rate_limit_per_minute IS 'Maximum API requests per minute for this plan';
COMMENT ON COLUMN plans.rate_limit_per_hour IS 'Maximum API requests per hour for this plan';
COMMENT ON COLUMN plans.rate_limit_per_day IS 'Maximum API requests per day for this plan';
COMMENT ON COLUMN plans.burst_capacity IS 'Maximum burst capacity (extra requests allowed in short bursts)';

-- Update existing plans with sensible defaults based on their tier
-- Free tier
UPDATE plans SET 
    rate_limit_per_minute = 60,
    rate_limit_per_hour = 1000,
    rate_limit_per_day = 10000,
    burst_capacity = 10
WHERE LOWER(name) LIKE '%free%' OR price_amount = 0;

-- Basic tier
UPDATE plans SET 
    rate_limit_per_minute = 120,
    rate_limit_per_hour = 3000,
    rate_limit_per_day = 50000,
    burst_capacity = 20
WHERE LOWER(name) LIKE '%basic%' OR LOWER(name) LIKE '%starter%';

-- Premium tier
UPDATE plans SET 
    rate_limit_per_minute = 300,
    rate_limit_per_hour = 10000,
    rate_limit_per_day = 200000,
    burst_capacity = 50
WHERE LOWER(name) LIKE '%premium%' OR LOWER(name) LIKE '%pro%';

-- Enterprise tier
UPDATE plans SET 
    rate_limit_per_minute = 1000,
    rate_limit_per_hour = 50000,
    rate_limit_per_day = 1000000,
    burst_capacity = 200
WHERE LOWER(name) LIKE '%enterprise%' OR LOWER(name) LIKE '%unlimited%';
