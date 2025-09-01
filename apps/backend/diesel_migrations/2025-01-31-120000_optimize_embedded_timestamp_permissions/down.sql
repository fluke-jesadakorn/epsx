-- Rollback Embedded Timestamp Permissions Optimization
-- Migration: Remove indexes for embedded timestamp permission queries

-- ============================================================================
-- REMOVE COMPOUND QUERY INDEXES
-- ============================================================================

DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_active_valid;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_validation;

-- ============================================================================
-- REMOVE FULL-TEXT SEARCH INDEXES
-- ============================================================================

DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_fulltext_search;
-- Note: Not dropping pg_trgm extension as it may be used by other features

-- ============================================================================
-- REMOVE PERFORMANCE MONITORING INDEXES
-- ============================================================================

DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_health_monitoring;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_system_stats;

-- ============================================================================
-- REMOVE CLEANUP OPTIMIZATION INDEXES
-- ============================================================================

DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_cleanup_batch;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_cleanup_stats;

-- ============================================================================
-- REMOVE AUDIT AND HISTORY INDEXES
-- ============================================================================

DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_granted_by_date;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_granted_at_range;

-- ============================================================================
-- REMOVE PLATFORM-SPECIFIC INDEXES
-- ============================================================================

DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_epsx_platform;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_admin_platform;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_platform;

-- ============================================================================
-- REMOVE USER-CENTRIC INDEXES
-- ============================================================================

DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_user_stats;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_user_active_expiry;

-- ============================================================================
-- REMOVE EXPIRY OPTIMIZATION INDEXES
-- ============================================================================

DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_expired;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_expiring_soon;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_expires_at_active;

-- ============================================================================
-- REMOVE PERMISSION STRING PATTERN INDEXES
-- ============================================================================

DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_base_pattern;
DROP INDEX CONCURRENTLY IF EXISTS idx_user_permissions_timestamped;

-- ============================================================================
-- UPDATE STATISTICS
-- ============================================================================

-- Update table statistics after index removal
ANALYZE user_permissions;