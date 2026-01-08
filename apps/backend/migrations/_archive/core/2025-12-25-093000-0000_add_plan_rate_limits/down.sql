-- Remove rate limit columns from plans table

ALTER TABLE plans DROP COLUMN IF EXISTS rate_limit_per_minute;
ALTER TABLE plans DROP COLUMN IF EXISTS rate_limit_per_hour;
ALTER TABLE plans DROP COLUMN IF EXISTS rate_limit_per_day;
ALTER TABLE plans DROP COLUMN IF EXISTS burst_capacity;
