-- Optimize Embedded Timestamp Permissions - Database Indexes
-- Migration: Add indexes to optimize embedded timestamp permission queries

-- ============================================================================
-- EXTENSIONS SETUP
-- ============================================================================

-- Enable trigram extension for permission string similarity searches
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============================================================================
-- PERMISSION STRING PATTERN INDEXES
-- ============================================================================

-- Index for permissions containing timestamps (pattern matching)
-- This speeds up queries that filter permissions with embedded timestamps
CREATE INDEX IF NOT EXISTS idx_user_permissions_timestamped 
ON user_permissions (permission) 
WHERE permission ~ '.*:[0-9]{10}$';

-- Index for permission prefix searches (platform:resource:action part)
-- This optimizes parsing and base permission lookups
CREATE INDEX IF NOT EXISTS idx_user_permissions_base_pattern
ON user_permissions USING gin (permission gin_trgm_ops);

-- ============================================================================
-- EXPIRY OPTIMIZATION INDEXES  
-- ============================================================================

-- Partial index for permissions that have expires_at set
-- This optimizes queries for permissions with separate expiry columns
CREATE INDEX IF NOT EXISTS idx_user_permissions_expires_at_active
ON user_permissions (expires_at, user_id) 
WHERE expires_at IS NOT NULL AND is_active = true;

-- Index for finding soon-to-expire permissions (within 24 hours)
-- This optimizes health monitoring and expiry warning queries
CREATE INDEX IF NOT EXISTS idx_user_permissions_expiring_soon
ON user_permissions (expires_at, user_id, permission)
WHERE expires_at IS NOT NULL 
;

-- Index for finding expired permissions
-- This optimizes cleanup operations and expiry filtering
CREATE INDEX IF NOT EXISTS idx_user_permissions_expired
ON user_permissions (expires_at, user_id, permission)
WHERE expires_at IS NOT NULL ;

-- ============================================================================
-- USER-CENTRIC INDEXES
-- ============================================================================

-- Composite index for user permission lookups with expiry
-- This optimizes the common query pattern: get all valid permissions for a user
CREATE INDEX IF NOT EXISTS idx_user_permissions_user_active_expiry
ON user_permissions (user_id, is_active, expires_at);

-- Index for user permission counts and statistics
-- This optimizes admin dashboard permission statistics
CREATE INDEX IF NOT EXISTS idx_user_permissions_user_stats
ON user_permissions (user_id, is_active) 
WHERE is_active = true;

-- ============================================================================
-- PLATFORM-SPECIFIC INDEXES
-- ============================================================================

-- Index for platform-specific permission queries
-- This optimizes cross-platform permission isolation
CREATE INDEX IF NOT EXISTS idx_user_permissions_platform
ON user_permissions (user_id, permission) 
WHERE permission LIKE 'epsx:%' OR permission LIKE 'epsx-pay:%' OR permission LIKE 'epsx-token:%' OR permission LIKE 'admin:%';

-- Partial indexes for each major platform (for high-performance platform isolation)
CREATE INDEX IF NOT EXISTS idx_user_permissions_epsx_platform
ON user_permissions (user_id, permission, expires_at)
WHERE permission LIKE 'epsx:%';

CREATE INDEX IF NOT EXISTS idx_user_permissions_admin_platform  
ON user_permissions (user_id, permission, expires_at)
WHERE permission LIKE 'admin:%';

-- ============================================================================
-- AUDIT AND HISTORY INDEXES
-- ============================================================================

-- Index for permission grant/revoke operations (audit purposes)
-- This optimizes queries for permission history and audit logs
CREATE INDEX IF NOT EXISTS idx_user_permissions_granted_by_date
ON user_permissions (granted_by, granted_at, user_id);

-- Index for finding permissions granted within a time range
-- This optimizes audit queries and permission lifecycle tracking
CREATE INDEX IF NOT EXISTS idx_user_permissions_granted_at_range
ON user_permissions (granted_at, user_id, permission)
WHERE granted_at IS NOT NULL;

-- ============================================================================
-- CLEANUP OPTIMIZATION INDEXES
-- ============================================================================

-- Index for efficient batch cleanup operations
-- This optimizes the cleanup-expired-permissions endpoint
CREATE INDEX IF NOT EXISTS idx_user_permissions_cleanup_batch
ON user_permissions (expires_at, id)
WHERE expires_at IS NOT NULL  AND is_active = true;

-- Index for cleanup statistics and reporting
CREATE INDEX IF NOT EXISTS idx_user_permissions_cleanup_stats
ON user_permissions (expires_at, created_at)
WHERE expires_at IS NOT NULL;

-- ============================================================================
-- PERFORMANCE MONITORING INDEXES
-- ============================================================================

-- Index for permission health monitoring queries
-- This optimizes the permission health dashboard
CREATE INDEX IF NOT EXISTS idx_user_permissions_health_monitoring
ON user_permissions (user_id, expires_at, is_active, created_at)
WHERE is_active IS NOT NULL;

-- Index for system-wide permission statistics
-- This optimizes admin analytics and reporting
CREATE INDEX IF NOT EXISTS idx_user_permissions_system_stats
ON user_permissions (created_at, expires_at, is_active);

-- ============================================================================
-- FULL-TEXT SEARCH OPTIMIZATION
-- ============================================================================

-- GIN index for fast permission string searching and pattern matching
-- This optimizes admin search functionality
CREATE INDEX IF NOT EXISTS idx_user_permissions_fulltext_search
ON user_permissions USING gin (permission gin_trgm_ops);

-- ============================================================================
-- COMPOUND QUERY OPTIMIZATION
-- ============================================================================

-- Multi-column index for the most common query pattern:
-- "Get all active, non-expired permissions for a specific user"
CREATE INDEX IF NOT EXISTS idx_user_permissions_active_valid
ON user_permissions (user_id, is_active, expires_at, permission)
WHERE is_active = true;

-- Index for permission validation queries (checking if permission exists and is valid)
CREATE INDEX IF NOT EXISTS idx_user_permissions_validation
ON user_permissions (user_id, permission, is_active, expires_at);

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON INDEX idx_user_permissions_timestamped IS 
'Optimizes queries filtering permissions with embedded Unix timestamps';

COMMENT ON INDEX idx_user_permissions_base_pattern IS 
'Accelerates permission parsing and base permission lookups using trigrams';

COMMENT ON INDEX idx_user_permissions_expires_at_active IS 
'Optimizes queries for permissions with separate expiry columns';

COMMENT ON INDEX idx_user_permissions_expiring_soon IS 
'Speeds up health monitoring queries for permissions expiring within 24 hours';

COMMENT ON INDEX idx_user_permissions_expired IS 
'Accelerates cleanup operations and expired permission filtering';

COMMENT ON INDEX idx_user_permissions_user_active_expiry IS 
'Optimizes the common pattern: get valid permissions for user';

COMMENT ON INDEX idx_user_permissions_platform IS 
'Enables fast platform-specific permission queries with cross-platform isolation';

COMMENT ON INDEX idx_user_permissions_active_valid IS 
'Optimizes the critical query: get all active, non-expired permissions for user';

-- ============================================================================
-- STATISTICS UPDATE
-- ============================================================================

-- Update table statistics to help query planner use new indexes effectively
ANALYZE user_permissions;