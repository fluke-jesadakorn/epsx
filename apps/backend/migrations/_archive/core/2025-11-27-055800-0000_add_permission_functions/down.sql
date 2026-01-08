-- ================================================================================================
-- REMOVE PERMISSION FUNCTIONS MIGRATION
-- ================================================================================================
-- Version: 2025-11-27-055800-0000
-- Description: Remove PostgreSQL functions for permission system
--
-- Functions Removed:
-- - wallet_has_permission
-- - get_wallet_permissions_detailed_working
-- - get_wallet_effective_permissions
-- - get_wallet_permission_stats
-- - wallet_has_permissions_batch
-- ================================================================================================

DROP FUNCTION IF EXISTS wallet_has_permissions_batch(VARCHAR(42), VARCHAR(255)[]);
DROP FUNCTION IF EXISTS get_wallet_permission_stats(VARCHAR(42));
DROP FUNCTION IF EXISTS get_wallet_effective_permissions(VARCHAR(42));
DROP FUNCTION IF EXISTS get_wallet_permissions_detailed_working(VARCHAR(42));
DROP FUNCTION IF EXISTS wallet_has_permission(VARCHAR(42), VARCHAR(255));

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'PERMISSION FUNCTIONS REMOVED SUCCESSFULLY! 🗑️';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Functions Removed:';
    RAISE NOTICE '  ❌ wallet_has_permission()';
    RAISE NOTICE '  ❌ get_wallet_permissions_detailed_working()';
    RAISE NOTICE '  ❌ get_wallet_effective_permissions()';
    RAISE NOTICE '  ❌ get_wallet_permission_stats()';
    RAISE NOTICE '  ❌ wallet_has_permissions_batch()';
    RAISE NOTICE '';
    RAISE NOTICE 'Migration rolled back successfully.';
    RAISE NOTICE '=================================================================================';
END $$;