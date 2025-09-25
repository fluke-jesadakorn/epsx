-- Migration: Remove Unrelated/Legacy Tables
-- Purpose: Clean up database by removing tables not related to Web3-first architecture
-- Date: December 24, 2024

-- ===============================
-- LEGACY USER SYSTEM CLEANUP
-- ===============================

-- Drop legacy user-related tables (replaced by wallet-based system)
DROP TABLE IF EXISTS user_group_memberships CASCADE;
DROP TABLE IF EXISTS user_permission_cache CASCADE;  
DROP TABLE IF EXISTS user_subscription_activations CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- Drop legacy user views
DROP VIEW IF EXISTS active_user_groups CASCADE;

-- ===============================  
-- DIESEL/ORM MIGRATION CLEANUP
-- ===============================

-- Remove Diesel migration table (using SQLx now)
DROP TABLE IF EXISTS __diesel_schema_migrations CASCADE;

-- ===============================
-- UNRELATED/OBSOLETE FEATURES
-- ===============================

-- Remove device fingerprinting (not part of core Web3 auth)
DROP TABLE IF EXISTS device_fingerprints CASCADE;

-- Remove API key system (using Web3 signatures now)
DROP TABLE IF EXISTS api_keys CASCADE;
DROP TABLE IF EXISTS api_requests CASCADE;

-- Remove permission delegation system (not implemented in current architecture)
DROP TABLE IF EXISTS permission_delegations CASCADE;

-- Remove user behavior profiling (not part of core system)
DROP TABLE IF EXISTS user_behavior_profiles CASCADE;

-- ===============================
-- ENTERPRISE FEATURE CLEANUP  
-- ===============================

-- Remove enterprise teams (keeping DAO governance instead)
DROP TABLE IF EXISTS enterprise_team_members CASCADE;
DROP TABLE IF EXISTS enterprise_teams CASCADE;

-- Remove enterprise compliance cache (keeping main compliance tables)
DROP TABLE IF EXISTS enterprise_compliance_cache CASCADE;

-- ===============================
-- DUPLICATE/REDUNDANT TABLES
-- ===============================

-- Remove duplicate Web3 auth nonces (using request_nonces)
DROP TABLE IF EXISTS web3_auth_nonces CASCADE;

-- Remove wallet migrations tracking (not needed)
DROP TABLE IF EXISTS wallet_migrations CASCADE;

-- Remove client tier mappings (using permission groups instead)
DROP TABLE IF EXISTS client_tier_mappings CASCADE;

-- ===============================
-- VERIFICATION ATTEMPT CLEANUP
-- ===============================

-- Keep payment_verification_attempts but remove view if redundant
-- (payment_verification_attempts table is kept as it's used for blockchain verification)

-- ===============================
-- UPDATE MIGRATION TRACKER
-- ===============================

-- Record this cleanup in migration log
INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
VALUES 
    (20241224000009, 'remove_unrelated_legacy_tables', NOW(), true, encode(sha256('remove_unrelated_legacy_tables'::bytea), 'hex'), 0);

-- ===============================
-- VERIFICATION QUERIES
-- ===============================

-- Count remaining tables (should be around 50 core Web3 tables)
DO $$
DECLARE
    table_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO table_count
    FROM information_schema.tables 
    WHERE table_schema = 'public' AND table_type = 'BASE TABLE';
    
    RAISE NOTICE 'Remaining tables after cleanup: %', table_count;
    
    -- Verify core Web3 tables still exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'wallet_identities') THEN
        RAISE EXCEPTION 'Critical error: wallet_identities table was dropped!';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'wallet_group_memberships') THEN
        RAISE EXCEPTION 'Critical error: wallet_group_memberships table was dropped!';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'permission_groups') THEN
        RAISE EXCEPTION 'Critical error: permission_groups table was dropped!';
    END IF;
    
    RAISE NOTICE 'Core Web3 tables verified as present ✓';
END $$;