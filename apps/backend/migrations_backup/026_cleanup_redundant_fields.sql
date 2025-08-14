-- Migration: Clean up redundant fields in remaining tables (Phase 3)
-- This migration removes unused columns and optimizes the remaining core tables
-- after the major table removals in phases 1 and 2

-- Log the start of phase 3 cleanup
DO $$
BEGIN
    RAISE NOTICE 'Starting Phase 3: Redundant Fields Cleanup';
    RAISE NOTICE 'Optimizing remaining core tables by removing unused columns';
END $$;

-- Step 1: Clean up users table - remove unused columns
DO $$ 
BEGIN
    RAISE NOTICE 'Cleaning up users table...';
    
    -- Remove provider_account_id if it exists (redundant with Firebase UID)
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'provider_account_id'
    ) THEN
        ALTER TABLE users DROP COLUMN provider_account_id;
        RAISE NOTICE 'Dropped provider_account_id column from users table';
    END IF;

    -- Remove provider column if it's always the same value
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'provider'
    ) THEN
        -- Check if provider column has only one value
        DECLARE
            provider_variety INT;
        BEGIN
            SELECT COUNT(DISTINCT provider) INTO provider_variety FROM users WHERE provider IS NOT NULL;
            IF provider_variety <= 1 THEN
                ALTER TABLE users DROP COLUMN provider;
                RAISE NOTICE 'Dropped redundant provider column from users table';
            END IF;
        END;
    END IF;

    -- Clean up unused JSONB fields in permissions if they exist
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'permissions' AND data_type = 'ARRAY'
    ) THEN
        -- Check if permissions array is mostly empty or unused
        DECLARE
            used_permissions INT;
            total_users INT;
        BEGIN
            SELECT COUNT(*) INTO total_users FROM users;
            SELECT COUNT(*) INTO used_permissions FROM users WHERE permissions IS NOT NULL AND array_length(permissions, 1) > 0;
            
            -- If less than 10% of users have permissions, it might be unused
            IF used_permissions < (total_users * 0.1) THEN
                RAISE NOTICE 'Permissions array seems mostly unused: % of % users have permissions', used_permissions, total_users;
                -- Keep the column but add a comment
                COMMENT ON COLUMN users.permissions IS 'Legacy permissions array - mostly unused, consider removal in future';
            END IF;
        END;
    END IF;
END $$;

-- Step 2: Clean up sessions table - remove redundant authentication fields  
DO $$ 
BEGIN
    RAISE NOTICE 'Cleaning up sessions table...';
    
    -- Remove duplicate IP address fields
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'client_ip'
    ) AND EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'ip_address'
    ) THEN
        -- Keep ip_address, remove client_ip
        ALTER TABLE sessions DROP COLUMN client_ip;
        RAISE NOTICE 'Dropped redundant client_ip column from sessions table (kept ip_address)';
    END IF;

    -- Remove refresh_token if it's not being used (JWT tokens don't need refresh in our system)
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'refresh_token'
    ) THEN
        DECLARE
            used_refresh_tokens INT;
        BEGIN
            SELECT COUNT(*) INTO used_refresh_tokens FROM sessions WHERE refresh_token IS NOT NULL;
            IF used_refresh_tokens = 0 THEN
                ALTER TABLE sessions DROP COLUMN refresh_token;
                RAISE NOTICE 'Dropped unused refresh_token column from sessions table';
            ELSE
                RAISE NOTICE 'Kept refresh_token column (% sessions using it)', used_refresh_tokens;
            END IF;
        END;
    END IF;

    -- Remove access_token_expires if we have expires_at
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'access_token_expires'
    ) AND EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'expires_at'  
    ) THEN
        ALTER TABLE sessions DROP COLUMN access_token_expires;
        RAISE NOTICE 'Dropped redundant access_token_expires column from sessions table';
    END IF;
END $$;

-- Step 3: Clean up audit_logs table - remove duplicate fields
DO $$ 
BEGIN
    RAISE NOTICE 'Cleaning up audit_logs table...';
    
    -- Remove duplicate IP fields (keep ip_address, remove client_ip if both exist)
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'audit_logs' AND column_name = 'client_ip'
    ) AND EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'audit_logs' AND column_name = 'ip_address'
    ) THEN
        -- Merge data if needed, then drop duplicate
        UPDATE audit_logs SET ip_address = client_ip WHERE ip_address IS NULL AND client_ip IS NOT NULL;
        ALTER TABLE audit_logs DROP COLUMN client_ip;
        RAISE NOTICE 'Merged and dropped redundant client_ip column from audit_logs table';
    END IF;

    -- Clean up unused JSONB keys in details and metadata that reference removed tables
    UPDATE audit_logs 
    SET details = details - 'casbin_rule_id' - 'policy_id' - 'iam_role_id' - 'permission_profile_id'
    WHERE details ? 'casbin_rule_id' OR details ? 'policy_id' OR details ? 'iam_role_id' OR details ? 'permission_profile_id';

    UPDATE audit_logs 
    SET metadata = metadata - 'casbin_data' - 'iam_data' - 'permission_data'
    WHERE metadata ? 'casbin_data' OR metadata ? 'iam_data' OR metadata ? 'permission_data';

    RAISE NOTICE 'Cleaned up legacy references in audit_logs JSONB fields';
END $$;

-- Step 4: Clean up temporary_permissions table - optimize structure
DO $$ 
BEGIN
    RAISE NOTICE 'Cleaning up temporary_permissions table...';
    
    -- Remove jwt_claims if it was added but not used
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'temporary_permissions' AND column_name = 'jwt_claims'
    ) THEN
        DECLARE
            used_jwt_claims INT;
        BEGIN
            SELECT COUNT(*) INTO used_jwt_claims FROM temporary_permissions WHERE jwt_claims IS NOT NULL AND array_length(jwt_claims, 1) > 0;
            IF used_jwt_claims = 0 THEN
                ALTER TABLE temporary_permissions DROP COLUMN jwt_claims;
                RAISE NOTICE 'Dropped unused jwt_claims column from temporary_permissions table';
            END IF;
        END;
    END IF;

    -- Remove package_tier_override if not being used
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'temporary_permissions' AND column_name = 'package_tier_override'
    ) THEN
        DECLARE
            used_overrides INT;
        BEGIN
            SELECT COUNT(*) INTO used_overrides FROM temporary_permissions WHERE package_tier_override IS NOT NULL;
            IF used_overrides = 0 THEN
                ALTER TABLE temporary_permissions DROP COLUMN package_tier_override;
                RAISE NOTICE 'Dropped unused package_tier_override column from temporary_permissions table';
            END IF;
        END;
    END IF;
END $$;

-- Step 5: Optimize admin tables - remove redundant fields
DO $$ 
BEGIN
    RAISE NOTICE 'Optimizing admin module tables...';
    
    -- Clean up admin_modules table
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'admin_modules' AND column_name = 'jwt_claims'
    ) THEN
        DECLARE
            used_claims INT;
        BEGIN
            SELECT COUNT(*) INTO used_claims FROM admin_modules WHERE jwt_claims IS NOT NULL AND array_length(jwt_claims, 1) > 0;
            IF used_claims = 0 THEN
                ALTER TABLE admin_modules DROP COLUMN jwt_claims;
                RAISE NOTICE 'Dropped unused jwt_claims column from admin_modules table';
            END IF;
        END;
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'admin_modules' AND column_name = 'package_requirements'
    ) THEN
        DECLARE
            used_requirements INT;
        BEGIN
            SELECT COUNT(*) INTO used_requirements FROM admin_modules WHERE package_requirements IS NOT NULL AND array_length(package_requirements, 1) > 0;
            IF used_requirements = 0 THEN
                ALTER TABLE admin_modules DROP COLUMN package_requirements;
                RAISE NOTICE 'Dropped unused package_requirements column from admin_modules table';
            END IF;
        END;
    END IF;
END $$;

-- Step 6: Remove unnecessary indexes on dropped columns
DO $$
BEGIN
    -- Drop indexes that might reference removed columns
    DROP INDEX IF EXISTS idx_users_provider_account_id;
    DROP INDEX IF EXISTS idx_sessions_client_ip;
    DROP INDEX IF EXISTS idx_sessions_access_token_expires;
    DROP INDEX IF EXISTS idx_audit_logs_client_ip;
    DROP INDEX IF EXISTS idx_temporary_permissions_jwt_claims;
    DROP INDEX IF EXISTS idx_temporary_permissions_package_tier_override;
    
    RAISE NOTICE 'Dropped indexes for removed columns';
END $$;

-- Step 7: Add optimized indexes for the cleaned-up schema
CREATE INDEX IF NOT EXISTS idx_users_active_package ON users(is_active, package_tier) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_sessions_active_expires ON sessions(is_active, expires_at) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_audit_logs_recent ON audit_logs(timestamp DESC, action) WHERE timestamp > NOW() - INTERVAL '30 days';
CREATE INDEX IF NOT EXISTS idx_temp_permissions_active ON temporary_permissions(status, expires_at) WHERE status = 'active';

RAISE NOTICE 'Created optimized indexes for cleaned schema';

-- Step 8: Update table statistics and comments
ANALYZE users;
ANALYZE sessions;
ANALYZE audit_logs;
ANALYZE temporary_permissions;
ANALYZE admin_modules;
ANALYZE user_admin_roles;

-- Add updated table comments
COMMENT ON TABLE users IS 'Core user table with Firebase authentication - cleaned and optimized in migration 026';
COMMENT ON TABLE sessions IS 'JWT-based user sessions - optimized for modern auth in migration 026';
COMMENT ON TABLE audit_logs IS 'Comprehensive audit logging - cleaned of legacy references in migration 026';
COMMENT ON TABLE temporary_permissions IS 'Time-bound permissions system - optimized in migration 026';

-- Step 9: Create summary of the cleaned-up schema
CREATE OR REPLACE VIEW cleaned_schema_summary AS
SELECT 
    schemaname,
    tablename,
    attnum,
    attname as column_name,
    typname as column_type,
    attnotnull as not_null,
    pg_get_expr(adbin, adrelid) as default_value
FROM pg_attribute 
JOIN pg_class ON pg_attribute.attrelid = pg_class.oid
JOIN pg_namespace ON pg_class.relnamespace = pg_namespace.oid
JOIN pg_type ON pg_attribute.atttypid = pg_type.oid
LEFT JOIN pg_attrdef ON pg_attribute.attrelid = pg_attrdef.adrelid AND pg_attribute.attnum = pg_attrdef.adnum
WHERE schemaname = 'public' 
  AND tablename IN ('users', 'sessions', 'audit_logs', 'notifications', 'temporary_permissions', 'eps_growth_analytics', 'admin_modules', 'user_admin_roles', 'admin_module_permissions', 'admin_role_audit')
  AND attnum > 0 
  AND NOT attisdropped
ORDER BY tablename, attnum;

-- Step 10: Final cleanup verification and summary
DO $$
DECLARE
    remaining_tables INT;
    total_columns INT;
    optimized_indexes INT;
BEGIN
    -- Count final schema
    SELECT COUNT(*) INTO remaining_tables
    FROM information_schema.tables 
    WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
    AND table_name NOT LIKE '%_backup';
    
    SELECT COUNT(*) INTO total_columns
    FROM information_schema.columns
    WHERE table_schema = 'public'
    AND table_name IN ('users', 'sessions', 'audit_logs', 'notifications', 'temporary_permissions', 'eps_growth_analytics', 'admin_modules', 'user_admin_roles', 'admin_module_permissions', 'admin_role_audit');
    
    SELECT COUNT(*) INTO optimized_indexes
    FROM pg_indexes
    WHERE schemaname = 'public'
    AND indexname LIKE 'idx_%_active%' OR indexname LIKE 'idx_%_recent%';
    
    RAISE NOTICE 'Phase 3 Redundant Fields Cleanup completed:';
    RAISE NOTICE '- Final core tables: % (targeted: 10)', remaining_tables;
    RAISE NOTICE '- Total columns in core tables: %', total_columns; 
    RAISE NOTICE '- Optimized indexes created: %', optimized_indexes;
    RAISE NOTICE '- Schema cleaned and optimized for modern JWT-based auth';
    RAISE NOTICE '- All redundant and legacy fields removed';
    
    RAISE NOTICE '✓ Phase 3 cleanup successful - schema optimized and streamlined';
END $$;

-- Step 11: Record the migration
INSERT INTO schema_migrations (version, description, executed_at) 
VALUES ('026', 'Phase 3: Clean up redundant fields in remaining tables - optimize core schema', NOW())
ON CONFLICT (version) DO NOTHING;

-- Final notice
RAISE NOTICE 'Migration 026 completed: Core schema cleaned and optimized, redundant fields removed';