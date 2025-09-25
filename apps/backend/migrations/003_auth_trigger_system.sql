-- ================================================================================================
-- EPSX Authentication Trigger System Migration
-- ================================================================================================
-- This migration adds the required columns for the authentication trigger system
-- Adds trigger configuration to wallet_identities and trigger tracking to wallet_auth_log
-- ================================================================================================

-- Set timezone for consistent timestamp handling
SET timezone = 'UTC';

-- ================================================================================================
-- 1. ADD TRIGGER CONFIGURATION TO WALLET_IDENTITIES
-- ================================================================================================

-- Add trigger_config column to wallet_identities table
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_identities' AND column_name = 'trigger_config') THEN
        ALTER TABLE wallet_identities ADD COLUMN trigger_config JSONB DEFAULT '{"auto_enabled": true, "frequency": "on_auth", "cache_duration_minutes": 60, "max_assignments_per_trigger": 10, "fallback_on_failure": "continue_auth"}';
        
        RAISE NOTICE '✅ Added trigger_config column to wallet_identities';
    ELSE
        RAISE NOTICE 'ℹ️ trigger_config column already exists in wallet_identities';
    END IF;
END $$;

-- ================================================================================================
-- 2. ADD TRIGGER TRACKING TO WALLET_AUTH_LOG
-- ================================================================================================

-- Add trigger_assignments_made column to wallet_auth_log table
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'wallet_auth_log' AND column_name = 'trigger_assignments_made') THEN
        ALTER TABLE wallet_auth_log ADD COLUMN trigger_assignments_made INTEGER;
        
        RAISE NOTICE '✅ Added trigger_assignments_made column to wallet_auth_log';
    ELSE
        RAISE NOTICE 'ℹ️ trigger_assignments_made column already exists in wallet_auth_log';
    END IF;
END $$;

-- ================================================================================================
-- 3. CREATE INDEXES FOR TRIGGER PERFORMANCE
-- ================================================================================================

-- Index on wallet_identities trigger_config for fast lookups
CREATE INDEX IF NOT EXISTS idx_wallet_identities_trigger_config 
ON wallet_identities USING GIN (trigger_config);

-- Index on wallet_auth_log for trigger analytics
CREATE INDEX IF NOT EXISTS idx_wallet_auth_log_trigger 
ON wallet_auth_log (wallet_address, auth_method, created_at) 
WHERE auth_method = 'trigger';

-- Index on wallet_auth_log for trigger assignments tracking
CREATE INDEX IF NOT EXISTS idx_wallet_auth_log_trigger_assignments 
ON wallet_auth_log (trigger_assignments_made, created_at)
WHERE trigger_assignments_made IS NOT NULL;

-- ================================================================================================
-- 4. CREATE HELPER VIEWS FOR TRIGGER ANALYTICS
-- ================================================================================================

-- View for trigger execution statistics
CREATE OR REPLACE VIEW trigger_execution_stats AS
SELECT 
    wallet_address,
    COUNT(*) as total_executions,
    COUNT(CASE WHEN success = true THEN 1 END) as successful_executions,
    COALESCE(SUM(trigger_assignments_made), 0) as total_assignments_made,
    AVG(verification_time_ms) as avg_execution_time_ms,
    MAX(created_at) as last_execution,
    (COUNT(CASE WHEN success = true THEN 1 END)::float / COUNT(*)::float * 100) as success_rate
FROM wallet_auth_log 
WHERE auth_method = 'trigger'
GROUP BY wallet_address
ORDER BY total_executions DESC;

-- View for trigger configuration summary
CREATE OR REPLACE VIEW trigger_config_summary AS
SELECT 
    wallet_address,
    display_name,
    trigger_config->>'auto_enabled' as auto_enabled,
    trigger_config->>'frequency' as frequency,
    trigger_config->>'cache_duration_minutes' as cache_duration_minutes,
    trigger_config->>'max_assignments_per_trigger' as max_assignments_per_trigger,
    trigger_config->>'fallback_on_failure' as fallback_on_failure,
    created_at,
    updated_at
FROM wallet_identities 
WHERE trigger_config IS NOT NULL
ORDER BY updated_at DESC;

-- ================================================================================================
-- 5. UPDATE EXISTING WALLET_IDENTITIES WITH DEFAULT TRIGGER CONFIG
-- ================================================================================================

-- Set default trigger configuration for existing wallets that don't have it
UPDATE wallet_identities 
SET trigger_config = '{"auto_enabled": true, "frequency": "on_auth", "cache_duration_minutes": 60, "max_assignments_per_trigger": 10, "fallback_on_failure": "continue_auth"}'::jsonb
WHERE trigger_config IS NULL;

-- ================================================================================================
-- 6. CREATE TRIGGER CLEANUP FUNCTION
-- ================================================================================================

-- Function to clean up old trigger execution logs
CREATE OR REPLACE FUNCTION cleanup_trigger_execution_logs()
RETURNS INTEGER AS $$
DECLARE
    cleaned_count INTEGER := 0;
BEGIN
    -- Clean up trigger logs older than 30 days
    DELETE FROM wallet_auth_log 
    WHERE auth_method = 'trigger' 
      AND created_at < NOW() - INTERVAL '30 days';
    
    GET DIAGNOSTICS cleaned_count = ROW_COUNT;
    
    RETURN cleaned_count;
END;
$$ LANGUAGE plpgsql;

-- ================================================================================================
-- 7. SUCCESS MESSAGE
-- ================================================================================================

DO $$ 
BEGIN 
    RAISE NOTICE '✅ EPSX Authentication Trigger System Migration Complete!';
    RAISE NOTICE '📊 Schema Updates:';
    RAISE NOTICE '  - Added trigger_config column to wallet_identities';
    RAISE NOTICE '  - Added trigger_assignments_made column to wallet_auth_log';
    RAISE NOTICE '  - Created performance indexes for trigger operations';
    RAISE NOTICE '  - Created helper views for trigger analytics';
    RAISE NOTICE '  - Updated existing wallets with default trigger configuration';
    RAISE NOTICE '🚀 Authentication trigger system is now ready!';
END $$;