-- Authentication Performance Optimization Migration
-- Optimizes database schema for high-performance authentication operations

-- ============================================================================
-- ENHANCED INDEXES FOR AUTHENTICATION PERFORMANCE
-- ============================================================================

-- Drop existing suboptimal indexes
DROP INDEX IF EXISTS idx_user_permissions_permission;
DROP INDEX IF EXISTS idx_user_permissions_user_id;

-- Composite index for permission lookups (most common query pattern)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_permissions_lookup_optimized
ON user_permissions (user_id, is_active, expires_at, permission)
WHERE is_active = true;

-- Partial index for permanent permissions (80% of use cases)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_permissions_permanent_optimized
ON user_permissions (user_id, permission)
WHERE is_active = true AND expires_at IS NULL;

-- Partial index for temporary permissions only
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_permissions_temporary_optimized
ON user_permissions (user_id, expires_at, permission)
WHERE is_active = true AND expires_at IS NOT NULL;

-- Covering index for permission counting queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_permissions_count_optimized
ON user_permissions (user_id) 
INCLUDE (permission, expires_at, is_active)
WHERE is_active = true;

-- ============================================================================
-- SESSION MANAGEMENT OPTIMIZATION
-- ============================================================================

-- Composite index for active session lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_active_lookup
ON sessions (user_id, is_active, expires_at)
WHERE is_active = true;

-- Index for session cleanup operations
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_cleanup
ON sessions (expires_at, is_active)
WHERE expires_at < CURRENT_TIMESTAMP OR is_active = false;

-- Index for session token lookups (hash prefix for performance)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_token_prefix
ON sessions (left(access_token, 16), expires_at)
WHERE is_active = true;

-- ============================================================================
-- JWT TOKEN REVOCATION OPTIMIZATION
-- ============================================================================

-- Optimized index for active revoked token checks
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_revoked_tokens_active_lookup
ON revoked_tokens (jti, expires_at)
WHERE expires_at > CURRENT_TIMESTAMP;

-- Index for expired token cleanup
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_revoked_tokens_cleanup
ON revoked_tokens (expires_at)
WHERE expires_at <= CURRENT_TIMESTAMP;

-- ============================================================================
-- USER AUTHENTICATION OPTIMIZATION
-- ============================================================================

-- Composite index for user authentication
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_auth_lookup
ON users (email, is_active, firebase_uid)
WHERE is_active = true;

-- Index for Firebase UID lookups with active status
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_firebase_active
ON users (firebase_uid, is_active, last_login_at)
WHERE is_active = true;

-- ============================================================================
-- REFRESH TOKEN OPTIMIZATION
-- ============================================================================

-- Composite index for refresh token validation
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_refresh_tokens_validation
ON refresh_tokens (user_id, token_hash, expires_at, is_revoked)
WHERE is_revoked = false;

-- Index for token family rotation
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_refresh_tokens_family
ON refresh_tokens (family_id, expires_at, is_revoked)
WHERE is_revoked = false;

-- Index for cleanup of expired refresh tokens
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_refresh_tokens_cleanup
ON refresh_tokens (expires_at, is_revoked)
WHERE expires_at <= CURRENT_TIMESTAMP OR is_revoked = true;

-- ============================================================================
-- AUDIT AND MONITORING OPTIMIZATION
-- ============================================================================

-- Partitioned table for policy evaluations (high volume)
CREATE TABLE IF NOT EXISTS policy_evaluations_partitioned (
    LIKE policy_evaluations INCLUDING ALL
) PARTITION BY RANGE (evaluated_at);

-- Create monthly partitions for the next 12 months
DO $$
DECLARE
    start_date DATE;
    end_date DATE;
    partition_name TEXT;
BEGIN
    FOR i IN 0..11 LOOP
        start_date := DATE_TRUNC('month', CURRENT_DATE) + (i || ' months')::INTERVAL;
        end_date := start_date + INTERVAL '1 month';
        partition_name := 'policy_evaluations_' || TO_CHAR(start_date, 'YYYY_MM');
        
        EXECUTE format('
            CREATE TABLE IF NOT EXISTS %I 
            PARTITION OF policy_evaluations_partitioned
            FOR VALUES FROM (%L) TO (%L)',
            partition_name, start_date, end_date
        );
        
        -- Create indexes on each partition
        EXECUTE format('
            CREATE INDEX IF NOT EXISTS %I 
            ON %I (user_id, evaluated_at)',
            partition_name || '_user_time_idx', partition_name
        );
        
        EXECUTE format('
            CREATE INDEX IF NOT EXISTS %I 
            ON %I (policy_id, decision)',
            partition_name || '_policy_decision_idx', partition_name
        );
    END LOOP;
END $$;

-- ============================================================================
-- DATA TYPE OPTIMIZATIONS
-- ============================================================================

-- Optimize frequently accessed columns for better performance
ALTER TABLE user_permissions 
    ALTER COLUMN permission TYPE VARCHAR(255),
    ALTER COLUMN is_active SET DEFAULT true;

-- Add constraint to ensure permission format
ALTER TABLE user_permissions 
    ADD CONSTRAINT chk_permission_format 
    CHECK (
        permission ~ '^[a-zA-Z0-9_-]+:[a-zA-Z0-9_*-]+:[a-zA-Z0-9_*-]+(:[0-9]+)?$'
    );

-- Add constraint for reasonable timestamp values
ALTER TABLE user_permissions 
    ADD CONSTRAINT chk_expires_at_reasonable 
    CHECK (
        expires_at IS NULL OR 
        (expires_at > CURRENT_TIMESTAMP - INTERVAL '1 day' AND 
         expires_at < CURRENT_TIMESTAMP + INTERVAL '10 years')
    );

-- ============================================================================
-- MATERIALIZED VIEWS FOR PERFORMANCE
-- ============================================================================

-- Materialized view for user permission summary (for dashboard queries)
CREATE MATERIALIZED VIEW IF NOT EXISTS user_permission_summary AS
SELECT 
    u.id as user_id,
    u.email,
    u.package_tier,
    COUNT(up.permission) as total_permissions,
    COUNT(CASE WHEN up.expires_at IS NULL THEN 1 END) as permanent_permissions,
    COUNT(CASE WHEN up.expires_at IS NOT NULL AND up.expires_at > CURRENT_TIMESTAMP THEN 1 END) as temporary_permissions,
    COUNT(CASE WHEN up.expires_at IS NOT NULL AND up.expires_at <= CURRENT_TIMESTAMP THEN 1 END) as expired_permissions,
    ARRAY_AGG(up.permission ORDER BY up.permission) FILTER (WHERE up.is_active = true) as active_permissions,
    MAX(up.updated_at) as last_permission_update
FROM users u
LEFT JOIN user_permissions up ON u.id = up.user_id
WHERE u.is_active = true
GROUP BY u.id, u.email, u.package_tier;

-- Index on materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_permission_summary_user_id 
ON user_permission_summary (user_id);

CREATE INDEX IF NOT EXISTS idx_user_permission_summary_email 
ON user_permission_summary (email);

-- ============================================================================
-- STORED PROCEDURES FOR COMMON OPERATIONS
-- ============================================================================

-- Optimized function to get user permissions with caching hints
CREATE OR REPLACE FUNCTION get_user_permissions_optimized(p_user_id UUID)
RETURNS TABLE(permission VARCHAR, expires_at TIMESTAMPTZ) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        up.permission,
        up.expires_at
    FROM user_permissions up
    WHERE up.user_id = p_user_id
      AND up.is_active = true
      AND (up.expires_at IS NULL OR up.expires_at > CURRENT_TIMESTAMP)
    ORDER BY up.permission;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to check specific permission efficiently
CREATE OR REPLACE FUNCTION check_user_permission(
    p_user_id UUID,
    p_permission VARCHAR
) RETURNS BOOLEAN AS $$
DECLARE
    permission_exists BOOLEAN := FALSE;
BEGIN
    -- Check exact match first (fastest)
    SELECT EXISTS(
        SELECT 1 FROM user_permissions 
        WHERE user_id = p_user_id 
          AND permission = p_permission
          AND is_active = true
          AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
    ) INTO permission_exists;
    
    IF permission_exists THEN
        RETURN TRUE;
    END IF;
    
    -- Check wildcard matches
    SELECT EXISTS(
        SELECT 1 FROM user_permissions 
        WHERE user_id = p_user_id 
          AND is_active = true
          AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
          AND (
              permission = 'admin:*:*' OR
              permission = SPLIT_PART(p_permission, ':', 1) || ':*:*' OR
              permission = SPLIT_PART(p_permission, ':', 1) || ':' || SPLIT_PART(p_permission, ':', 2) || ':*'
          )
    ) INTO permission_exists;
    
    RETURN permission_exists;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function for batch permission checking
CREATE OR REPLACE FUNCTION check_user_permissions_batch(
    p_user_id UUID,
    p_permissions VARCHAR[]
) RETURNS BOOLEAN[] AS $$
DECLARE
    result BOOLEAN[];
    permission VARCHAR;
BEGIN
    -- Initialize result array
    result := ARRAY[]::BOOLEAN[];
    
    -- Check each permission
    FOREACH permission IN ARRAY p_permissions LOOP
        result := array_append(result, check_user_permission(p_user_id, permission));
    END LOOP;
    
    RETURN result;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- MAINTENANCE PROCEDURES
-- ============================================================================

-- Procedure to clean up expired tokens and sessions
CREATE OR REPLACE FUNCTION cleanup_expired_auth_data()
RETURNS TABLE(
    deleted_sessions INTEGER,
    deleted_revoked_tokens INTEGER,
    deleted_refresh_tokens INTEGER
) AS $$
DECLARE
    session_count INTEGER;
    revoked_count INTEGER;
    refresh_count INTEGER;
BEGIN
    -- Clean up expired sessions
    DELETE FROM sessions 
    WHERE expires_at < CURRENT_TIMESTAMP - INTERVAL '1 day';
    GET DIAGNOSTICS session_count = ROW_COUNT;
    
    -- Clean up expired revoked tokens
    DELETE FROM revoked_tokens 
    WHERE expires_at < CURRENT_TIMESTAMP - INTERVAL '1 day';
    GET DIAGNOSTICS revoked_count = ROW_COUNT;
    
    -- Clean up expired refresh tokens
    DELETE FROM refresh_tokens 
    WHERE expires_at < CURRENT_TIMESTAMP - INTERVAL '7 days';
    GET DIAGNOSTICS refresh_count = ROW_COUNT;
    
    RETURN QUERY SELECT session_count, revoked_count, refresh_count;
END;
$$ LANGUAGE plpgsql;

-- Procedure to refresh materialized views
CREATE OR REPLACE FUNCTION refresh_auth_materialized_views()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_permission_summary;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- AUTOMATIC MAINTENANCE JOBS
-- ============================================================================

-- Create extension for background jobs if not exists
CREATE EXTENSION IF NOT EXISTS pg_cron;

-- Schedule daily cleanup of expired data (if pg_cron is available)
-- SELECT cron.schedule('auth-cleanup', '0 2 * * *', 'SELECT cleanup_expired_auth_data();');

-- Schedule hourly refresh of materialized views
-- SELECT cron.schedule('auth-views-refresh', '0 * * * *', 'SELECT refresh_auth_materialized_views();');

-- ============================================================================
-- PERFORMANCE MONITORING VIEWS
-- ============================================================================

-- View for authentication performance monitoring
CREATE OR REPLACE VIEW auth_performance_stats AS
SELECT 
    'user_permissions' as table_name,
    COUNT(*) as total_rows,
    COUNT(*) FILTER (WHERE is_active = true) as active_rows,
    COUNT(*) FILTER (WHERE expires_at IS NOT NULL) as temporary_permissions,
    AVG(CASE WHEN updated_at > created_at THEN EXTRACT(EPOCH FROM updated_at - created_at) END) as avg_update_delay_seconds
FROM user_permissions
UNION ALL
SELECT 
    'sessions' as table_name,
    COUNT(*) as total_rows,
    COUNT(*) FILTER (WHERE is_active = true) as active_rows,
    COUNT(*) FILTER (WHERE expires_at > CURRENT_TIMESTAMP) as valid_sessions,
    AVG(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP - created_at)) as avg_session_age_seconds
FROM sessions
UNION ALL
SELECT 
    'refresh_tokens' as table_name,
    COUNT(*) as total_rows,
    COUNT(*) FILTER (WHERE is_revoked = false) as active_rows,
    COUNT(*) FILTER (WHERE expires_at > CURRENT_TIMESTAMP) as valid_tokens,
    AVG(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP - created_at)) as avg_token_age_seconds
FROM refresh_tokens;

-- View for permission distribution analysis
CREATE OR REPLACE VIEW permission_distribution_stats AS
SELECT 
    SPLIT_PART(permission, ':', 1) as platform,
    SPLIT_PART(permission, ':', 2) as resource,
    SPLIT_PART(permission, ':', 3) as action,
    COUNT(*) as usage_count,
    COUNT(DISTINCT user_id) as unique_users,
    COUNT(*) FILTER (WHERE expires_at IS NOT NULL) as temporary_grants,
    MIN(created_at) as first_granted,
    MAX(updated_at) as last_updated
FROM user_permissions 
WHERE is_active = true
GROUP BY 
    SPLIT_PART(permission, ':', 1),
    SPLIT_PART(permission, ':', 2),
    SPLIT_PART(permission, ':', 3)
ORDER BY usage_count DESC;

-- ============================================================================
-- STATISTICS UPDATE
-- ============================================================================

-- Update table statistics for better query planning
ANALYZE users;
ANALYZE user_permissions;
ANALYZE sessions;
ANALYZE refresh_tokens;
ANALYZE revoked_tokens;

-- ============================================================================
-- VALIDATION CHECKS
-- ============================================================================

-- Verify all indexes were created successfully
DO $$
DECLARE
    missing_indexes TEXT[];
    expected_indexes TEXT[] := ARRAY[
        'idx_user_permissions_lookup_optimized',
        'idx_user_permissions_permanent_optimized',
        'idx_user_permissions_temporary_optimized',
        'idx_sessions_active_lookup',
        'idx_revoked_tokens_active_lookup',
        'idx_users_auth_lookup',
        'idx_refresh_tokens_validation'
    ];
    idx TEXT;
BEGIN
    missing_indexes := ARRAY[]::TEXT[];
    
    FOREACH idx IN ARRAY expected_indexes LOOP
        IF NOT EXISTS (
            SELECT 1 FROM pg_indexes 
            WHERE indexname = idx
        ) THEN
            missing_indexes := array_append(missing_indexes, idx);
        END IF;
    END LOOP;
    
    IF array_length(missing_indexes, 1) > 0 THEN
        RAISE WARNING 'Missing indexes: %', array_to_string(missing_indexes, ', ');
    ELSE
        RAISE NOTICE 'All authentication optimization indexes created successfully';
    END IF;
END $$;