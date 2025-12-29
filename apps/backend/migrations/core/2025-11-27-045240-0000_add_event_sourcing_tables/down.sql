-- ================================================================================================
-- REMOVE EVENT SOURCING TABLES MIGRATION ROLLBACK
-- ================================================================================================
-- Version: 2025-11-27-045240-0000 (Rollback)
-- Created: 2025-11-27
-- Description: Remove event sourcing infrastructure tables
--
-- Tables Removed:
-- - aggregate_snapshots: Performance optimization snapshots
-- - outbox_events: Transactional outbox for reliable event publishing
-- - event_store: Immutable event log for event sourcing
--
-- Order: Drop in reverse order of creation to handle foreign key dependencies
-- ================================================================================================

-- ================================================================================================
-- SECTION 1: REMOVE AGGREGATE SNAPSHOTS
-- ================================================================================================

DROP TABLE IF EXISTS aggregate_snapshots;

-- ================================================================================================
-- SECTION 2: REMOVE OUTBOX EVENTS
-- ================================================================================================

DROP TABLE IF EXISTS outbox_events;

-- ================================================================================================
-- SECTION 3: REMOVE EVENT STORE
-- ================================================================================================

DROP TABLE IF EXISTS event_store;

-- ================================================================================================
-- COMPLETION MESSAGE
-- ================================================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'EVENT SOURCING TABLES REMOVED SUCCESSFULLY! 🗑️';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Event Sourcing Tables Removed:';
    RAISE NOTICE '  ✅ aggregate_snapshots (performance optimization)';
    RAISE NOTICE '  ✅ outbox_events (reliable event publishing)';
    RAISE NOTICE '  ✅ event_store (immutable event log)';
    RAISE NOTICE '';
    RAISE NOTICE 'Rollback completed successfully!';
    RAISE NOTICE 'Database schema returned to previous state.';
    RAISE NOTICE '=================================================================================';
END $$;
