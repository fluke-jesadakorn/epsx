-- Remove disable_info column from wallet_users
ALTER TABLE wallet_users DROP COLUMN IF EXISTS disable_info;

-- Drop wallet_activity_logs table
DROP TABLE IF EXISTS wallet_activity_logs;
