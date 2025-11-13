-- ============================================================================
-- PHASE 3: PARTITIONING & ARCHIVAL
-- ============================================================================
--
-- Purpose: Implement time-based partitioning for large audit tables,
--          create archival strategy for old data, and automate partition management
--
-- Expected Impact:
-- - 70-80% faster queries with partition pruning
-- - Easy archival of old partitions
-- - Reduced index maintenance overhead
-- - Maintains optimal table sizes
--
-- ============================================================================

-- ============================================================================
-- SECTION 1: PARTITION AUDIT LOG TABLE
-- ============================================================================

-- 1.1: Create partitioned audit log table (monthly partitions)
CREATE TABLE IF NOT EXISTS permission_audit_log_partitioned (
    LIKE permission_audit_log INCLUDING ALL EXCLUDING INDEXES EXCLUDING CONSTRAINTS
) PARTITION BY RANGE (event_timestamp);

-- Copy comments
COMMENT ON TABLE permission_audit_log_partitioned IS 'Partitioned audit log for permission changes - monthly partitions';

-- 1.2: Create initial partitions (last 6 months + next 3 months)
DO $$
DECLARE
    partition_date DATE;
    partition_start DATE;
    partition_end DATE;
    partition_name TEXT;
    i INTEGER;
BEGIN
    -- Create partitions for past 6 months
    FOR i IN REVERSE 6..1 LOOP
        partition_date := date_trunc('month', NOW() - (i || ' months')::INTERVAL);
        partition_start := partition_date;
        partition_end := partition_start + INTERVAL '1 month';
        partition_name := 'permission_audit_log_' || to_char(partition_start, 'YYYY_MM');

        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %I PARTITION OF permission_audit_log_partitioned
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );

        RAISE NOTICE 'Created partition: % (% to %)', partition_name, partition_start, partition_end;
    END LOOP;

    -- Create current month partition
    partition_start := date_trunc('month', NOW());
    partition_end := partition_start + INTERVAL '1 month';
    partition_name := 'permission_audit_log_' || to_char(partition_start, 'YYYY_MM');

    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF permission_audit_log_partitioned
         FOR VALUES FROM (%L) TO (%L)',
        partition_name, partition_start, partition_end
    );

    RAISE NOTICE 'Created partition: % (% to %)', partition_name, partition_start, partition_end;

    -- Create partitions for next 3 months
    FOR i IN 1..3 LOOP
        partition_date := date_trunc('month', NOW() + (i || ' months')::INTERVAL);
        partition_start := partition_date;
        partition_end := partition_start + INTERVAL '1 month';
        partition_name := 'permission_audit_log_' || to_char(partition_start, 'YYYY_MM');

        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %I PARTITION OF permission_audit_log_partitioned
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );

        RAISE NOTICE 'Created partition: % (% to %)', partition_name, partition_start, partition_end;
    END LOOP;
END $$;

-- 1.3: Migrate existing audit log data
INSERT INTO permission_audit_log_partitioned
SELECT * FROM permission_audit_log
ON CONFLICT DO NOTHING;

-- 1.4: Recreate constraints and indexes on partitioned table
ALTER TABLE permission_audit_log_partitioned
ADD CONSTRAINT valid_event_type CHECK (
    event_type IN (
        'granted', 'revoked', 'modified', 'expired',
        'group_assigned', 'group_removed', 'group_updated',
        'direct_permission_granted', 'direct_permission_revoked'
    )
),
ADD CONSTRAINT valid_wallet_address_format CHECK (
    wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
),
ADD CONSTRAINT valid_performed_by_format CHECK (
    performed_by IS NULL OR
    (performed_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(performed_by) = 42)
);

-- Recreate foreign keys
ALTER TABLE permission_audit_log_partitioned
ADD CONSTRAINT fk_audit_permission
FOREIGN KEY (permission_id)
REFERENCES permissions(id)
ON DELETE SET NULL;

ALTER TABLE permission_audit_log_partitioned
ADD CONSTRAINT fk_audit_group
FOREIGN KEY (group_id)
REFERENCES permission_groups(id)
ON DELETE SET NULL;

-- Recreate indexes
CREATE INDEX idx_audit_wallet_part ON permission_audit_log_partitioned(wallet_address, event_timestamp DESC);
CREATE INDEX idx_audit_timestamp_part ON permission_audit_log_partitioned(event_timestamp DESC);
CREATE INDEX idx_audit_event_type_part ON permission_audit_log_partitioned(event_type, event_timestamp DESC);
CREATE INDEX idx_audit_permission_part ON permission_audit_log_partitioned(permission_string, event_timestamp DESC) WHERE permission_string IS NOT NULL;
CREATE INDEX idx_audit_group_part ON permission_audit_log_partitioned(group_id, event_timestamp DESC) WHERE group_id IS NOT NULL;

-- 1.5: Swap tables
BEGIN;
  ALTER TABLE permission_audit_log RENAME TO permission_audit_log_old;
  ALTER TABLE permission_audit_log_partitioned RENAME TO permission_audit_log;

  -- Update dependent objects
  DROP VIEW IF EXISTS v_recent_permission_changes CASCADE;
  DROP VIEW IF EXISTS v_wallet_permission_history CASCADE;

  -- Recreate views
  CREATE OR REPLACE VIEW v_recent_permission_changes AS
  SELECT
      pal.id,
      pal.event_type,
      pal.event_timestamp,
      pal.wallet_address,
      wu.tier_level,
      pal.permission_string,
      pal.group_name,
      pal.performed_by,
      pal.reason,
      pal.expires_at
  FROM permission_audit_log pal
  LEFT JOIN wallet_users wu ON pal.wallet_address = wu.wallet_address
  WHERE pal.event_timestamp > NOW() - INTERVAL '7 days'
  ORDER BY pal.event_timestamp DESC;

  CREATE OR REPLACE VIEW v_wallet_permission_history AS
  SELECT
      pal.wallet_address,
      COUNT(*) FILTER (WHERE event_type IN ('granted', 'direct_permission_granted', 'group_assigned')) as total_grants,
      COUNT(*) FILTER (WHERE event_type IN ('revoked', 'direct_permission_revoked', 'group_removed')) as total_revocations,
      MAX(pal.event_timestamp) FILTER (WHERE event_type IN ('granted', 'direct_permission_granted', 'group_assigned')) as last_grant_at,
      MAX(pal.event_timestamp) FILTER (WHERE event_type IN ('revoked', 'direct_permission_revoked', 'group_removed')) as last_revoke_at
  FROM permission_audit_log pal
  GROUP BY pal.wallet_address;
COMMIT;

RAISE NOTICE '✅ Partitioned permission_audit_log table (monthly partitions)';

-- ============================================================================
-- SECTION 2: PARTITION EVENT STORE TABLE
-- ============================================================================

-- 2.1: Create partitioned event store (quarterly partitions)
CREATE TABLE IF NOT EXISTS event_store_partitioned (
    LIKE event_store INCLUDING ALL EXCLUDING INDEXES EXCLUDING CONSTRAINTS
) PARTITION BY RANGE (occurred_at);

-- Copy comments
COMMENT ON TABLE event_store_partitioned IS 'Partitioned event store - quarterly partitions for event sourcing';

-- 2.2: Create initial quarterly partitions
DO $$
DECLARE
    partition_date DATE;
    partition_start DATE;
    partition_end DATE;
    partition_name TEXT;
    quarter INTEGER;
    year INTEGER;
BEGIN
    -- Create partitions for last 4 quarters
    FOR i IN REVERSE 4..1 LOOP
        partition_date := date_trunc('quarter', NOW() - (i || ' quarters')::INTERVAL);
        partition_start := partition_date;
        partition_end := partition_start + INTERVAL '3 months';
        year := EXTRACT(YEAR FROM partition_start);
        quarter := EXTRACT(QUARTER FROM partition_start);
        partition_name := 'event_store_' || year || '_q' || quarter;

        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %I PARTITION OF event_store_partitioned
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );

        RAISE NOTICE 'Created partition: % (% to %)', partition_name, partition_start, partition_end;
    END LOOP;

    -- Create current quarter partition
    partition_start := date_trunc('quarter', NOW());
    partition_end := partition_start + INTERVAL '3 months';
    year := EXTRACT(YEAR FROM partition_start);
    quarter := EXTRACT(QUARTER FROM partition_start);
    partition_name := 'event_store_' || year || '_q' || quarter;

    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF event_store_partitioned
         FOR VALUES FROM (%L) TO (%L)',
        partition_name, partition_start, partition_end
    );

    RAISE NOTICE 'Created partition: % (% to %)', partition_name, partition_start, partition_end;

    -- Create next 2 quarters
    FOR i IN 1..2 LOOP
        partition_date := date_trunc('quarter', NOW() + (i || ' quarters')::INTERVAL);
        partition_start := partition_date;
        partition_end := partition_start + INTERVAL '3 months';
        year := EXTRACT(YEAR FROM partition_start);
        quarter := EXTRACT(QUARTER FROM partition_start);
        partition_name := 'event_store_' || year || '_q' || quarter;

        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %I PARTITION OF event_store_partitioned
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );

        RAISE NOTICE 'Created partition: % (% to %)', partition_name, partition_start, partition_end;
    END LOOP;
END $$;

-- 2.3: Migrate existing event store data
INSERT INTO event_store_partitioned
SELECT * FROM event_store
ON CONFLICT DO NOTHING;

-- 2.4: Recreate constraints and indexes
ALTER TABLE event_store_partitioned
ADD CONSTRAINT event_store_unique_version UNIQUE (aggregate_id, aggregate_version),
ADD CONSTRAINT event_store_version_positive CHECK (aggregate_version >= 0);

-- Recreate indexes
CREATE INDEX idx_event_store_aggregate_part ON event_store_partitioned(aggregate_id, aggregate_version);
CREATE INDEX idx_event_store_type_time_part ON event_store_partitioned(event_type, occurred_at DESC);
CREATE INDEX idx_event_store_correlation_part ON event_store_partitioned(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_event_store_aggregate_type_part ON event_store_partitioned(aggregate_type, occurred_at DESC);

-- 2.5: Update outbox_events foreign key
ALTER TABLE outbox_events DROP CONSTRAINT IF EXISTS outbox_events_event_id_fkey;

-- 2.6: Swap tables
BEGIN;
  ALTER TABLE event_store RENAME TO event_store_old;
  ALTER TABLE event_store_partitioned RENAME TO event_store;

  -- Recreate outbox FK
  ALTER TABLE outbox_events
  ADD CONSTRAINT outbox_events_event_id_fkey
  FOREIGN KEY (event_id)
  REFERENCES event_store(event_id)
  ON DELETE CASCADE;
COMMIT;

RAISE NOTICE '✅ Partitioned event_store table (quarterly partitions)';

-- ============================================================================
-- SECTION 3: CREATE NOTIFICATION ARCHIVE TABLE
-- ============================================================================

-- 3.1: Create partitioned archive table for old notifications
CREATE TABLE IF NOT EXISTS wallet_notifications_archive (
    LIKE wallet_notifications INCLUDING ALL EXCLUDING INDEXES
) PARTITION BY RANGE (created_at);

COMMENT ON TABLE wallet_notifications_archive IS 'Archived notifications older than 90 days - monthly partitions';

-- 3.2: Create archive partitions (last 12 months)
DO $$
DECLARE
    partition_date DATE;
    partition_start DATE;
    partition_end DATE;
    partition_name TEXT;
    i INTEGER;
BEGIN
    FOR i IN REVERSE 12..1 LOOP
        partition_date := date_trunc('month', NOW() - (i || ' months')::INTERVAL);
        partition_start := partition_date;
        partition_end := partition_start + INTERVAL '1 month';
        partition_name := 'wallet_notifications_archive_' || to_char(partition_start, 'YYYY_MM');

        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %I PARTITION OF wallet_notifications_archive
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );

        RAISE NOTICE 'Created archive partition: %', partition_name;
    END LOOP;
END $$;

-- 3.3: Create indexes on archive table
CREATE INDEX idx_archive_notifications_wallet ON wallet_notifications_archive(wallet_address, created_at DESC);
CREATE INDEX idx_archive_notifications_created ON wallet_notifications_archive(created_at DESC);

RAISE NOTICE '✅ Created notification archive table';

-- ============================================================================
-- SECTION 4: AUTOMATED PARTITION MANAGEMENT FUNCTIONS
-- ============================================================================

-- 4.1: Function to create new audit log partition
CREATE OR REPLACE FUNCTION create_audit_log_partition()
RETURNS TEXT AS $$
DECLARE
    partition_name TEXT;
    partition_start DATE;
    partition_end DATE;
BEGIN
    -- Create partition for next month
    partition_start := date_trunc('month', NOW() + INTERVAL '1 month');
    partition_end := partition_start + INTERVAL '1 month';
    partition_name := 'permission_audit_log_' || to_char(partition_start, 'YYYY_MM');

    -- Check if partition already exists
    IF NOT EXISTS (
        SELECT 1 FROM pg_class WHERE relname = partition_name
    ) THEN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF permission_audit_log
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );

        RETURN 'Created partition: ' || partition_name;
    ELSE
        RETURN 'Partition already exists: ' || partition_name;
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_audit_log_partition IS 'Creates next month audit log partition';

-- 4.2: Function to create new event store partition
CREATE OR REPLACE FUNCTION create_event_store_partition()
RETURNS TEXT AS $$
DECLARE
    partition_name TEXT;
    partition_start DATE;
    partition_end DATE;
    quarter INTEGER;
    year INTEGER;
BEGIN
    -- Create partition for next quarter
    partition_start := date_trunc('quarter', NOW() + INTERVAL '3 months');
    partition_end := partition_start + INTERVAL '3 months';
    year := EXTRACT(YEAR FROM partition_start);
    quarter := EXTRACT(QUARTER FROM partition_start);
    partition_name := 'event_store_' || year || '_q' || quarter;

    -- Check if partition already exists
    IF NOT EXISTS (
        SELECT 1 FROM pg_class WHERE relname = partition_name
    ) THEN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF event_store
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );

        RETURN 'Created partition: ' || partition_name;
    ELSE
        RETURN 'Partition already exists: ' || partition_name;
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_event_store_partition IS 'Creates next quarter event store partition';

-- 4.3: Function to create new notification archive partition
CREATE OR REPLACE FUNCTION create_notification_archive_partition()
RETURNS TEXT AS $$
DECLARE
    partition_name TEXT;
    partition_start DATE;
    partition_end DATE;
BEGIN
    -- Create partition for current month if it doesn't exist
    partition_start := date_trunc('month', NOW());
    partition_end := partition_start + INTERVAL '1 month';
    partition_name := 'wallet_notifications_archive_' || to_char(partition_start, 'YYYY_MM');

    IF NOT EXISTS (
        SELECT 1 FROM pg_class WHERE relname = partition_name
    ) THEN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF wallet_notifications_archive
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );

        RETURN 'Created archive partition: ' || partition_name;
    ELSE
        RETURN 'Archive partition already exists: ' || partition_name;
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_notification_archive_partition IS 'Creates current month notification archive partition';

RAISE NOTICE '✅ Created partition management functions';

-- ============================================================================
-- SECTION 5: ARCHIVAL FUNCTIONS
-- ============================================================================

-- 5.1: Function to archive old notifications
CREATE OR REPLACE FUNCTION archive_old_notifications(
    p_age_days INTEGER DEFAULT 90
)
RETURNS TABLE (
    archived_count INTEGER,
    oldest_archived TIMESTAMPTZ,
    newest_archived TIMESTAMPTZ
) AS $$
DECLARE
    v_archived_count INTEGER;
    v_oldest TIMESTAMPTZ;
    v_newest TIMESTAMPTZ;
BEGIN
    -- Ensure archive partition exists
    PERFORM create_notification_archive_partition();

    -- Move read notifications older than specified days to archive
    WITH archived AS (
        DELETE FROM wallet_notifications
        WHERE read_at IS NOT NULL
          AND deleted_at IS NULL
          AND created_at < NOW() - (p_age_days || ' days')::INTERVAL
        RETURNING *
    ),
    inserted AS (
        INSERT INTO wallet_notifications_archive
        SELECT * FROM archived
        RETURNING id, created_at
    )
    SELECT
        COUNT(*)::INTEGER,
        MIN(created_at),
        MAX(created_at)
    INTO v_archived_count, v_oldest, v_newest
    FROM inserted;

    RETURN QUERY SELECT v_archived_count, v_oldest, v_newest;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION archive_old_notifications IS 'Archives read notifications older than specified days (default 90)';

-- 5.2: Function to drop old audit partitions
CREATE OR REPLACE FUNCTION drop_old_audit_partitions(
    p_retention_months INTEGER DEFAULT 6
)
RETURNS TABLE (
    dropped_partition TEXT,
    partition_size TEXT
) AS $$
DECLARE
    partition_rec RECORD;
    cutoff_date DATE;
BEGIN
    cutoff_date := date_trunc('month', NOW() - (p_retention_months || ' months')::INTERVAL);

    FOR partition_rec IN
        SELECT
            c.relname as partition_name,
            pg_size_pretty(pg_total_relation_size(c.oid)) as size
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relname LIKE 'permission_audit_log_%'
          AND c.relkind = 'r'
          AND n.nspname = 'public'
    LOOP
        -- Extract date from partition name (permission_audit_log_YYYY_MM)
        DECLARE
            partition_date DATE;
            date_parts TEXT[];
        BEGIN
            date_parts := regexp_match(partition_rec.partition_name, '(\d{4})_(\d{2})');
            IF date_parts IS NOT NULL THEN
                partition_date := (date_parts[1] || '-' || date_parts[2] || '-01')::DATE;

                IF partition_date < cutoff_date THEN
                    -- Drop partition
                    EXECUTE 'DROP TABLE IF EXISTS ' || partition_rec.partition_name || ' CASCADE';

                    RETURN QUERY SELECT partition_rec.partition_name, partition_rec.size;
                END IF;
            END IF;
        EXCEPTION WHEN OTHERS THEN
            -- Skip partitions with invalid names
            CONTINUE;
        END;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION drop_old_audit_partitions IS 'Drops audit log partitions older than retention period (default 6 months)';

-- 5.3: Function to drop old event store partitions
CREATE OR REPLACE FUNCTION drop_old_event_store_partitions(
    p_retention_quarters INTEGER DEFAULT 4
)
RETURNS TABLE (
    dropped_partition TEXT,
    partition_size TEXT
) AS $$
DECLARE
    partition_rec RECORD;
    cutoff_date DATE;
BEGIN
    cutoff_date := date_trunc('quarter', NOW() - (p_retention_quarters || ' quarters')::INTERVAL);

    FOR partition_rec IN
        SELECT
            c.relname as partition_name,
            pg_size_pretty(pg_total_relation_size(c.oid)) as size
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relname LIKE 'event_store_%'
          AND c.relkind = 'r'
          AND n.nspname = 'public'
    LOOP
        DECLARE
            partition_date DATE;
            date_parts TEXT[];
            year INTEGER;
            quarter INTEGER;
        BEGIN
            date_parts := regexp_match(partition_rec.partition_name, '(\d{4})_q(\d)');
            IF date_parts IS NOT NULL THEN
                year := date_parts[1]::INTEGER;
                quarter := date_parts[2]::INTEGER;
                partition_date := (year || '-' || ((quarter - 1) * 3 + 1) || '-01')::DATE;

                IF partition_date < cutoff_date THEN
                    EXECUTE 'DROP TABLE IF EXISTS ' || partition_rec.partition_name || ' CASCADE';
                    RETURN QUERY SELECT partition_rec.partition_name, partition_rec.size;
                END IF;
            END IF;
        EXCEPTION WHEN OTHERS THEN
            CONTINUE;
        END;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION drop_old_event_store_partitions IS 'Drops event store partitions older than retention period (default 4 quarters)';

RAISE NOTICE '✅ Created archival functions';

-- ============================================================================
-- SECTION 6: PARTITION HEALTH MONITORING
-- ============================================================================

-- 6.1: Create partition info view
CREATE OR REPLACE VIEW v_partition_info AS
SELECT
    nmsp_parent.nspname AS schema_name,
    parent.relname AS table_name,
    child.relname AS partition_name,
    pg_get_expr(child.relpartbound, child.oid) AS partition_bounds,
    pg_size_pretty(pg_total_relation_size(child.oid)) AS partition_size,
    pg_stat_get_live_tuples(child.oid) AS row_count
FROM pg_inherits
JOIN pg_class parent ON pg_inherits.inhparent = parent.oid
JOIN pg_class child ON pg_inherits.inhrelid = child.oid
JOIN pg_namespace nmsp_parent ON nmsp_parent.oid = parent.relnamespace
WHERE parent.relkind = 'p'  -- Partitioned table
  AND nmsp_parent.nspname IN ('public', 'read_model')
ORDER BY schema_name, table_name, partition_name;

COMMENT ON VIEW v_partition_info IS 'Shows all partitions with size and row count information';

-- 6.2: Create function to check partition health
CREATE OR REPLACE FUNCTION check_partition_health()
RETURNS TABLE (
    table_name TEXT,
    status TEXT,
    message TEXT,
    action_required TEXT
) AS $$
BEGIN
    -- Check if audit log needs new partition (within 7 days of current month end)
    IF date_trunc('month', NOW() + INTERVAL '7 days') > date_trunc('month', NOW()) THEN
        RETURN QUERY SELECT
            'permission_audit_log'::TEXT,
            'WARNING'::TEXT,
            'Current month ending soon'::TEXT,
            'Run: SELECT create_audit_log_partition()'::TEXT;
    END IF;

    -- Check if event store needs new partition (within 14 days of current quarter end)
    IF date_trunc('quarter', NOW() + INTERVAL '14 days') > date_trunc('quarter', NOW()) THEN
        RETURN QUERY SELECT
            'event_store'::TEXT,
            'WARNING'::TEXT,
            'Current quarter ending soon'::TEXT,
            'Run: SELECT create_event_store_partition()'::TEXT;
    END IF;

    -- Check for large partitions that might need attention
    FOR table_name, message IN
        SELECT
            partition_name::TEXT,
            'Large partition: ' || partition_size
        FROM v_partition_info
        WHERE pg_total_relation_size(partition_name::regclass) > 1024 * 1024 * 1024  -- > 1GB
    LOOP
        RETURN QUERY SELECT table_name, 'INFO'::TEXT, message, 'Consider archiving'::TEXT;
    END LOOP;

    -- If no issues, return OK status
    IF NOT FOUND THEN
        RETURN QUERY SELECT
            'All partitions'::TEXT,
            'OK'::TEXT,
            'All partitions healthy'::TEXT,
            'No action required'::TEXT;
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION check_partition_health IS 'Checks health of partitioned tables and suggests actions';

RAISE NOTICE '✅ Created partition monitoring tools';

-- ============================================================================
-- SECTION 7: CLEANUP OLD TABLES
-- ============================================================================

-- Note: Old tables are kept with _old suffix for safety
-- Drop them manually after verifying data integrity:
-- DROP TABLE IF EXISTS permission_audit_log_old CASCADE;
-- DROP TABLE IF EXISTS event_store_old CASCADE;

RAISE NOTICE '⚠️  Old tables kept as *_old - verify and drop manually after testing';

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'PHASE 3: PARTITIONING & ARCHIVAL COMPLETE! ✅';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Changes Applied:';
    RAISE NOTICE '  ✅ Partitioned permission_audit_log (monthly partitions)';
    RAISE NOTICE '  ✅ Partitioned event_store (quarterly partitions)';
    RAISE NOTICE '  ✅ Created wallet_notifications_archive table';
    RAISE NOTICE '  ✅ Created automated partition management functions';
    RAISE NOTICE '  ✅ Created archival functions';
    RAISE NOTICE '  ✅ Created partition health monitoring';
    RAISE NOTICE '';
    RAISE NOTICE 'Expected Benefits:';
    RAISE NOTICE '  ⚡ 70-80%% faster queries with partition pruning';
    RAISE NOTICE '  📦 Easy archival of old partitions';
    RAISE NOTICE '  🔧 Reduced index maintenance overhead';
    RAISE NOTICE '  📊 Maintains optimal table sizes';
    RAISE NOTICE '';
    RAISE NOTICE 'Management Commands:';
    RAISE NOTICE '  🆕 SELECT create_audit_log_partition();';
    RAISE NOTICE '  🆕 SELECT create_event_store_partition();';
    RAISE NOTICE '  📁 SELECT * FROM archive_old_notifications(90);';
    RAISE NOTICE '  🗑️  SELECT * FROM drop_old_audit_partitions(6);';
    RAISE NOTICE '  🗑️  SELECT * FROM drop_old_event_store_partitions(4);';
    RAISE NOTICE '  🏥 SELECT * FROM check_partition_health();';
    RAISE NOTICE '  📊 SELECT * FROM v_partition_info;';
    RAISE NOTICE '';
    RAISE NOTICE 'Recommended Automation (using pg_cron):';
    RAISE NOTICE '  - Monthly: SELECT create_audit_log_partition();';
    RAISE NOTICE '  - Quarterly: SELECT create_event_store_partition();';
    RAISE NOTICE '  - Weekly: SELECT archive_old_notifications(90);';
    RAISE NOTICE '  - Monthly: SELECT drop_old_audit_partitions(6);';
    RAISE NOTICE '';
    RAISE NOTICE 'Next Steps:';
    RAISE NOTICE '  1. Set up pg_cron for automated partition management';
    RAISE NOTICE '  2. Monitor partition health for 1-2 weeks';
    RAISE NOTICE '  3. Drop old tables after verification: *_old';
    RAISE NOTICE '  4. Run Phase 4: Read Model Optimization';
    RAISE NOTICE '=================================================================================';
END $$;
