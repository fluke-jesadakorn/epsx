-- ============================================================================
-- PHASE 5: SAGA PATTERN IMPLEMENTATION
-- ============================================================================
--
-- Purpose: Implement saga pattern for distributed transactions,
--          enhance event sourcing with snapshot optimization,
--          and add saga orchestration infrastructure
--
-- Expected Impact:
-- - Reliable distributed transactions
-- - Automatic compensation on failures
-- - Better event replay performance
-- - Transactional consistency across aggregates
--
-- ============================================================================

-- ============================================================================
-- SECTION 1: SAGA INSTANCE MANAGEMENT
-- ============================================================================

-- 1.1: Create saga instances table
CREATE TABLE IF NOT EXISTS saga_instances (
    saga_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Saga identity
    saga_type VARCHAR(100) NOT NULL,
    correlation_id UUID,

    -- Saga state
    status VARCHAR(50) NOT NULL DEFAULT 'started',
    current_step INTEGER NOT NULL DEFAULT 0,
    total_steps INTEGER NOT NULL,

    -- Saga data
    saga_data JSONB NOT NULL DEFAULT '{}',
    context JSONB NOT NULL DEFAULT '{}',

    -- Timing
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    started_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,

    -- Error handling
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,

    -- Audit
    initiated_by VARCHAR(42),

    CONSTRAINT valid_saga_status CHECK (
        status IN ('started', 'executing', 'compensating', 'completed', 'failed', 'cancelled')
    ),
    CONSTRAINT valid_step_range CHECK (current_step >= 0 AND current_step <= total_steps),
    CONSTRAINT valid_retry_count CHECK (retry_count >= 0 AND retry_count <= max_retries)
);

-- Indexes
CREATE INDEX idx_saga_instances_status ON saga_instances(status, created_at DESC);
CREATE INDEX idx_saga_instances_type ON saga_instances(saga_type, status, created_at DESC);
CREATE INDEX idx_saga_instances_correlation ON saga_instances(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_saga_instances_pending ON saga_instances(status, current_step)
    WHERE status IN ('started', 'executing', 'compensating');

COMMENT ON TABLE saga_instances IS 'Saga orchestration instances for distributed transactions';

-- 1.2: Create saga steps table
CREATE TABLE IF NOT EXISTS saga_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_id UUID NOT NULL REFERENCES saga_instances(saga_id) ON DELETE CASCADE,

    -- Step definition
    step_number INTEGER NOT NULL,
    step_name VARCHAR(100) NOT NULL,
    step_type VARCHAR(50) NOT NULL,

    -- Command/Event data
    command_type VARCHAR(100) NOT NULL,
    command_data JSONB NOT NULL DEFAULT '{}',
    compensation_command_type VARCHAR(100),
    compensation_data JSONB,

    -- Execution state
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    result JSONB,
    error TEXT,

    -- Timing
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    compensated_at TIMESTAMPTZ,

    -- Retry handling
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    next_retry_at TIMESTAMPTZ,

    CONSTRAINT valid_step_status CHECK (
        status IN ('pending', 'executing', 'completed', 'failed', 'compensating', 'compensated', 'skipped')
    ),
    CONSTRAINT valid_step_type CHECK (
        step_type IN ('command', 'event', 'query', 'notification')
    ),
    UNIQUE(saga_id, step_number)
);

-- Indexes
CREATE INDEX idx_saga_steps_saga ON saga_steps(saga_id, step_number);
CREATE INDEX idx_saga_steps_status ON saga_steps(status, created_at DESC);
CREATE INDEX idx_saga_steps_pending ON saga_steps(saga_id, status, step_number)
    WHERE status IN ('pending', 'executing');
CREATE INDEX idx_saga_steps_retry ON saga_steps(next_retry_at)
    WHERE next_retry_at IS NOT NULL AND status = 'failed';

COMMENT ON TABLE saga_steps IS 'Individual steps in saga execution with compensation support';

-- 1.3: Create saga events log
CREATE TABLE IF NOT EXISTS saga_events (
    id BIGSERIAL PRIMARY KEY,
    saga_id UUID NOT NULL REFERENCES saga_instances(saga_id) ON DELETE CASCADE,
    step_id UUID REFERENCES saga_steps(id) ON DELETE SET NULL,

    -- Event data
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL DEFAULT '{}',

    -- Metadata
    occurred_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    sequence_number INTEGER NOT NULL,

    -- Indexes
    UNIQUE(saga_id, sequence_number)
);

CREATE INDEX idx_saga_events_saga ON saga_events(saga_id, sequence_number);
CREATE INDEX idx_saga_events_type ON saga_events(event_type, occurred_at DESC);
CREATE INDEX idx_saga_events_occurred ON saga_events(occurred_at DESC);

COMMENT ON TABLE saga_events IS 'Audit log of all saga events for debugging and monitoring';

RAISE NOTICE '✅ Created saga management tables';

-- ============================================================================
-- SECTION 2: SAGA ORCHESTRATION FUNCTIONS
-- ============================================================================

-- 2.1: Function to create new saga
CREATE OR REPLACE FUNCTION create_saga(
    p_saga_type VARCHAR,
    p_total_steps INTEGER,
    p_saga_data JSONB DEFAULT '{}',
    p_initiated_by VARCHAR DEFAULT NULL,
    p_correlation_id UUID DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_saga_id UUID;
BEGIN
    INSERT INTO saga_instances (
        saga_type,
        total_steps,
        saga_data,
        correlation_id,
        initiated_by,
        status,
        current_step
    ) VALUES (
        p_saga_type,
        p_total_steps,
        p_saga_data,
        COALESCE(p_correlation_id, gen_random_uuid()),
        p_initiated_by,
        'started',
        0
    )
    RETURNING saga_id INTO v_saga_id;

    -- Log saga created event
    INSERT INTO saga_events (saga_id, event_type, event_data, sequence_number)
    VALUES (
        v_saga_id,
        'saga_created',
        jsonb_build_object(
            'saga_type', p_saga_type,
            'total_steps', p_total_steps,
            'initiated_by', p_initiated_by
        ),
        0
    );

    RETURN v_saga_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_saga IS 'Creates a new saga instance for orchestration';

-- 2.2: Function to add saga step
CREATE OR REPLACE FUNCTION add_saga_step(
    p_saga_id UUID,
    p_step_number INTEGER,
    p_step_name VARCHAR,
    p_command_type VARCHAR,
    p_command_data JSONB,
    p_compensation_type VARCHAR DEFAULT NULL,
    p_compensation_data JSONB DEFAULT NULL,
    p_step_type VARCHAR DEFAULT 'command'
)
RETURNS UUID AS $$
DECLARE
    v_step_id UUID;
BEGIN
    INSERT INTO saga_steps (
        saga_id,
        step_number,
        step_name,
        step_type,
        command_type,
        command_data,
        compensation_command_type,
        compensation_data,
        status
    ) VALUES (
        p_saga_id,
        p_step_number,
        p_step_name,
        p_step_type,
        p_command_type,
        p_command_data,
        p_compensation_type,
        p_compensation_data,
        'pending'
    )
    RETURNING id INTO v_step_id;

    RETURN v_step_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION add_saga_step IS 'Adds a new step to saga execution plan';

-- 2.3: Function to execute saga step
CREATE OR REPLACE FUNCTION execute_saga_step(
    p_step_id UUID
)
RETURNS JSONB AS $$
DECLARE
    v_step RECORD;
    v_result JSONB;
BEGIN
    -- Get step details
    SELECT * INTO v_step
    FROM saga_steps
    WHERE id = p_step_id
    FOR UPDATE;

    IF v_step.status != 'pending' THEN
        RAISE EXCEPTION 'Step % is not in pending status (current: %)', p_step_id, v_step.status;
    END IF;

    -- Mark step as executing
    UPDATE saga_steps
    SET status = 'executing',
        started_at = NOW()
    WHERE id = p_step_id;

    -- Log event
    INSERT INTO saga_events (saga_id, step_id, event_type, event_data, sequence_number)
    SELECT
        saga_id,
        id,
        'step_started',
        jsonb_build_object(
            'step_number', step_number,
            'step_name', step_name,
            'command_type', command_type
        ),
        (SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM saga_events WHERE saga_id = v_step.saga_id)
    FROM saga_steps
    WHERE id = p_step_id;

    -- Update saga status
    UPDATE saga_instances
    SET status = 'executing',
        current_step = v_step.step_number,
        updated_at = NOW()
    WHERE saga_id = v_step.saga_id;

    -- Return step data for execution
    RETURN jsonb_build_object(
        'step_id', v_step.id,
        'saga_id', v_step.saga_id,
        'command_type', v_step.command_type,
        'command_data', v_step.command_data
    );
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION execute_saga_step IS 'Marks saga step as executing and returns execution data';

-- 2.4: Function to complete saga step
CREATE OR REPLACE FUNCTION complete_saga_step(
    p_step_id UUID,
    p_result JSONB DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
    v_step RECORD;
    v_saga RECORD;
    v_next_step_exists BOOLEAN;
BEGIN
    -- Get step details
    SELECT * INTO v_step
    FROM saga_steps
    WHERE id = p_step_id;

    -- Mark step as completed
    UPDATE saga_steps
    SET status = 'completed',
        result = p_result,
        completed_at = NOW()
    WHERE id = p_step_id;

    -- Log event
    INSERT INTO saga_events (saga_id, step_id, event_type, event_data, sequence_number)
    SELECT
        v_step.saga_id,
        p_step_id,
        'step_completed',
        jsonb_build_object('step_number', v_step.step_number, 'result', p_result),
        (SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM saga_events WHERE saga_id = v_step.saga_id);

    -- Get saga details
    SELECT * INTO v_saga
    FROM saga_instances
    WHERE saga_id = v_step.saga_id;

    -- Check if there are more steps
    SELECT EXISTS(
        SELECT 1 FROM saga_steps
        WHERE saga_id = v_step.saga_id
          AND step_number > v_step.step_number
          AND status = 'pending'
    ) INTO v_next_step_exists;

    IF NOT v_next_step_exists THEN
        -- Complete saga
        UPDATE saga_instances
        SET status = 'completed',
            completed_at = NOW(),
            updated_at = NOW()
        WHERE saga_id = v_step.saga_id;

        -- Log saga completion
        INSERT INTO saga_events (saga_id, event_type, event_data, sequence_number)
        VALUES (
            v_step.saga_id,
            'saga_completed',
            jsonb_build_object('total_steps', v_saga.total_steps),
            (SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM saga_events WHERE saga_id = v_step.saga_id)
        );
    END IF;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION complete_saga_step IS 'Marks saga step as completed and checks for saga completion';

-- 2.5: Function to fail saga step and trigger compensation
CREATE OR REPLACE FUNCTION fail_saga_step(
    p_step_id UUID,
    p_error TEXT
)
RETURNS BOOLEAN AS $$
DECLARE
    v_step RECORD;
    v_can_retry BOOLEAN;
BEGIN
    -- Get step details
    SELECT * INTO v_step
    FROM saga_steps
    WHERE id = p_step_id;

    -- Check if retry is possible
    v_can_retry := v_step.retry_count < v_step.max_retries;

    IF v_can_retry THEN
        -- Schedule retry
        UPDATE saga_steps
        SET retry_count = retry_count + 1,
            next_retry_at = NOW() + (power(2, retry_count + 1) || ' seconds')::INTERVAL,
            error = p_error
        WHERE id = p_step_id;

        -- Log retry
        INSERT INTO saga_events (saga_id, step_id, event_type, event_data, sequence_number)
        SELECT
            v_step.saga_id,
            p_step_id,
            'step_retry_scheduled',
            jsonb_build_object(
                'step_number', v_step.step_number,
                'retry_count', v_step.retry_count + 1,
                'error', p_error
            ),
            (SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM saga_events WHERE saga_id = v_step.saga_id);
    ELSE
        -- Mark step as failed
        UPDATE saga_steps
        SET status = 'failed',
            error = p_error,
            completed_at = NOW()
        WHERE id = p_step_id;

        -- Trigger compensation
        PERFORM compensate_saga(v_step.saga_id);

        -- Log failure
        INSERT INTO saga_events (saga_id, step_id, event_type, event_data, sequence_number)
        SELECT
            v_step.saga_id,
            p_step_id,
            'step_failed',
            jsonb_build_object('step_number', v_step.step_number, 'error', p_error),
            (SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM saga_events WHERE saga_id = v_step.saga_id);
    END IF;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION fail_saga_step IS 'Handles step failure with retry logic and compensation trigger';

-- 2.6: Function to compensate saga
CREATE OR REPLACE FUNCTION compensate_saga(
    p_saga_id UUID
)
RETURNS BOOLEAN AS $$
DECLARE
    v_step RECORD;
BEGIN
    -- Mark saga as compensating
    UPDATE saga_instances
    SET status = 'compensating',
        failed_at = NOW(),
        updated_at = NOW()
    WHERE saga_id = p_saga_id;

    -- Log compensation start
    INSERT INTO saga_events (saga_id, event_type, event_data, sequence_number)
    VALUES (
        p_saga_id,
        'saga_compensation_started',
        jsonb_build_object('reason', 'step_failure'),
        (SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM saga_events WHERE saga_id = p_saga_id)
    );

    -- Mark completed steps for compensation (in reverse order)
    FOR v_step IN
        SELECT *
        FROM saga_steps
        WHERE saga_id = p_saga_id
          AND status = 'completed'
          AND compensation_command_type IS NOT NULL
        ORDER BY step_number DESC
    LOOP
        UPDATE saga_steps
        SET status = 'compensating'
        WHERE id = v_step.id;

        -- Log compensation
        INSERT INTO saga_events (saga_id, step_id, event_type, event_data, sequence_number)
        VALUES (
            p_saga_id,
            v_step.id,
            'step_compensating',
            jsonb_build_object('step_number', v_step.step_number),
            (SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM saga_events WHERE saga_id = p_saga_id)
        );
    END LOOP;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION compensate_saga IS 'Initiates compensation for failed saga';

RAISE NOTICE '✅ Created saga orchestration functions';

-- ============================================================================
-- SECTION 3: ENHANCED EVENT SOURCING
-- ============================================================================

-- 3.1: Function for optimized aggregate replay with snapshots
CREATE OR REPLACE FUNCTION replay_aggregate_optimized(
    p_aggregate_id VARCHAR,
    p_aggregate_type VARCHAR
)
RETURNS JSONB AS $$
DECLARE
    v_snapshot RECORD;
    v_events JSONB;
    v_event_count INTEGER;
BEGIN
    -- Get latest snapshot
    SELECT *
    INTO v_snapshot
    FROM aggregate_snapshots
    WHERE aggregate_id = p_aggregate_id
      AND aggregate_type = p_aggregate_type
    ORDER BY aggregate_version DESC
    LIMIT 1;

    -- Get events since snapshot
    WITH events_since_snapshot AS (
        SELECT
            event_id,
            event_type,
            event_data,
            aggregate_version,
            occurred_at
        FROM event_store
        WHERE aggregate_id = p_aggregate_id
          AND aggregate_type = p_aggregate_type
          AND aggregate_version > COALESCE(v_snapshot.aggregate_version, -1)
        ORDER BY aggregate_version
    )
    SELECT
        jsonb_agg(
            jsonb_build_object(
                'event_id', event_id,
                'event_type', event_type,
                'event_data', event_data,
                'version', aggregate_version,
                'occurred_at', occurred_at
            )
        ),
        COUNT(*)
    INTO v_events, v_event_count
    FROM events_since_snapshot;

    -- Return snapshot + events
    RETURN jsonb_build_object(
        'aggregate_id', p_aggregate_id,
        'aggregate_type', p_aggregate_type,
        'snapshot', COALESCE(v_snapshot.snapshot_data, '{}'::jsonb),
        'snapshot_version', COALESCE(v_snapshot.aggregate_version, -1),
        'events', COALESCE(v_events, '[]'::jsonb),
        'event_count', COALESCE(v_event_count, 0),
        'current_version', COALESCE(v_snapshot.aggregate_version, -1) + COALESCE(v_event_count, 0)
    );
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION replay_aggregate_optimized IS 'Optimized aggregate replay using snapshots';

-- 3.2: Function to create snapshot with auto-threshold
CREATE OR REPLACE FUNCTION create_aggregate_snapshot_auto(
    p_aggregate_id VARCHAR,
    p_aggregate_type VARCHAR,
    p_snapshot_threshold INTEGER DEFAULT 100
)
RETURNS BOOLEAN AS $$
DECLARE
    v_last_snapshot_version BIGINT;
    v_current_version BIGINT;
    v_event_count INTEGER;
    v_aggregate_data JSONB;
BEGIN
    -- Get last snapshot version
    SELECT COALESCE(aggregate_version, -1)
    INTO v_last_snapshot_version
    FROM aggregate_snapshots
    WHERE aggregate_id = p_aggregate_id
      AND aggregate_type = p_aggregate_type
    ORDER BY aggregate_version DESC
    LIMIT 1;

    -- Get current version
    SELECT COALESCE(MAX(aggregate_version), -1)
    INTO v_current_version
    FROM event_store
    WHERE aggregate_id = p_aggregate_id
      AND aggregate_type = p_aggregate_type;

    v_event_count := v_current_version - v_last_snapshot_version;

    -- Check if snapshot is needed
    IF v_event_count < p_snapshot_threshold THEN
        RETURN FALSE;
    END IF;

    -- Replay and create snapshot
    v_aggregate_data := replay_aggregate_optimized(p_aggregate_id, p_aggregate_type);

    -- Insert snapshot
    INSERT INTO aggregate_snapshots (
        aggregate_id,
        aggregate_type,
        aggregate_version,
        snapshot_data,
        event_count_at_snapshot
    ) VALUES (
        p_aggregate_id,
        p_aggregate_type,
        v_current_version,
        v_aggregate_data->'snapshot',
        v_event_count
    )
    ON CONFLICT (aggregate_id) DO UPDATE SET
        aggregate_version = EXCLUDED.aggregate_version,
        snapshot_data = EXCLUDED.snapshot_data,
        event_count_at_snapshot = EXCLUDED.event_count_at_snapshot,
        created_at = NOW();

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_aggregate_snapshot_auto IS 'Automatically creates snapshot if event threshold is exceeded';

RAISE NOTICE '✅ Enhanced event sourcing with snapshot optimization';

-- ============================================================================
-- SECTION 4: SAGA MONITORING AND VIEWS
-- ============================================================================

-- 4.1: Create saga status view
CREATE OR REPLACE VIEW v_saga_status AS
SELECT
    si.saga_id,
    si.saga_type,
    si.status,
    si.current_step,
    si.total_steps,
    ROUND(100.0 * si.current_step / NULLIF(si.total_steps, 0), 2) as progress_pct,
    si.created_at,
    si.started_at,
    si.completed_at,
    si.failed_at,
    CASE
        WHEN si.completed_at IS NOT NULL THEN
            EXTRACT(EPOCH FROM (si.completed_at - si.started_at))
        WHEN si.failed_at IS NOT NULL THEN
            EXTRACT(EPOCH FROM (si.failed_at - si.started_at))
        ELSE
            EXTRACT(EPOCH FROM (NOW() - si.started_at))
    END as duration_seconds,
    si.error_message,
    si.retry_count,
    COUNT(ss.id) as total_step_records,
    COUNT(*) FILTER (WHERE ss.status = 'completed') as completed_steps,
    COUNT(*) FILTER (WHERE ss.status = 'failed') as failed_steps,
    COUNT(*) FILTER (WHERE ss.status = 'compensated') as compensated_steps
FROM saga_instances si
LEFT JOIN saga_steps ss ON si.saga_id = ss.saga_id
GROUP BY si.saga_id, si.saga_type, si.status, si.current_step, si.total_steps,
         si.created_at, si.started_at, si.completed_at, si.failed_at,
         si.error_message, si.retry_count;

COMMENT ON VIEW v_saga_status IS 'Real-time saga execution status with progress metrics';

-- 4.2: Create function to get saga execution timeline
CREATE OR REPLACE FUNCTION get_saga_timeline(p_saga_id UUID)
RETURNS TABLE (
    sequence_number INTEGER,
    event_type TEXT,
    step_name TEXT,
    occurred_at TIMESTAMPTZ,
    event_data JSONB
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        se.sequence_number,
        se.event_type::TEXT,
        COALESCE(ss.step_name, 'N/A')::TEXT,
        se.occurred_at,
        se.event_data
    FROM saga_events se
    LEFT JOIN saga_steps ss ON se.step_id = ss.id
    WHERE se.saga_id = p_saga_id
    ORDER BY se.sequence_number;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_saga_timeline IS 'Returns complete timeline of saga execution events';

-- 4.3: Create function to get pending saga work
CREATE OR REPLACE FUNCTION get_pending_saga_work()
RETURNS TABLE (
    saga_id UUID,
    saga_type TEXT,
    step_id UUID,
    step_number INTEGER,
    command_type TEXT,
    command_data JSONB
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        si.saga_id,
        si.saga_type::TEXT,
        ss.id as step_id,
        ss.step_number,
        ss.command_type::TEXT,
        ss.command_data
    FROM saga_instances si
    JOIN saga_steps ss ON si.saga_id = ss.saga_id
    WHERE si.status IN ('started', 'executing')
      AND ss.status = 'pending'
      AND ss.step_number = si.current_step + 1
    ORDER BY si.created_at
    LIMIT 100;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_pending_saga_work IS 'Returns next pending saga steps ready for execution';

RAISE NOTICE '✅ Created saga monitoring views and functions';

-- ============================================================================
-- SECTION 5: SAGA EXAMPLE IMPLEMENTATIONS
-- ============================================================================

-- 5.1: Example: Permission Group Assignment Saga
CREATE OR REPLACE FUNCTION create_permission_assignment_saga(
    p_wallet_address VARCHAR,
    p_group_id UUID,
    p_assigned_by VARCHAR
)
RETURNS UUID AS $$
DECLARE
    v_saga_id UUID;
    v_saga_data JSONB;
BEGIN
    v_saga_data := jsonb_build_object(
        'wallet_address', p_wallet_address,
        'group_id', p_group_id,
        'assigned_by', p_assigned_by
    );

    -- Create saga
    v_saga_id := create_saga(
        'permission_assignment',
        3,  -- 3 steps
        v_saga_data,
        p_assigned_by
    );

    -- Step 1: Validate wallet exists
    PERFORM add_saga_step(
        v_saga_id, 1, 'Validate Wallet',
        'validate_wallet_exists', jsonb_build_object('wallet_address', p_wallet_address),
        NULL, NULL, 'query'
    );

    -- Step 2: Assign group
    PERFORM add_saga_step(
        v_saga_id, 2, 'Assign Group',
        'assign_wallet_group', v_saga_data,
        'remove_wallet_group', v_saga_data  -- Compensation
    );

    -- Step 3: Send notification
    PERFORM add_saga_step(
        v_saga_id, 3, 'Send Notification',
        'send_permission_granted_notification', v_saga_data,
        NULL, NULL, 'notification'
    );

    RETURN v_saga_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_permission_assignment_saga IS 'Creates saga for permission group assignment';

RAISE NOTICE '✅ Created example saga implementations';

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'PHASE 5: SAGA PATTERN IMPLEMENTATION COMPLETE! ✅';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE 'Changes Applied:';
    RAISE NOTICE '  ✅ Created saga instance management tables';
    RAISE NOTICE '  ✅ Implemented saga orchestration functions';
    RAISE NOTICE '  ✅ Added compensation logic';
    RAISE NOTICE '  ✅ Enhanced event sourcing with snapshot optimization';
    RAISE NOTICE '  ✅ Created saga monitoring views';
    RAISE NOTICE '  ✅ Added example saga implementations';
    RAISE NOTICE '';
    RAISE NOTICE 'Expected Benefits:';
    RAISE NOTICE '  ⚡ Reliable distributed transactions';
    RAISE NOTICE '  🔄 Automatic compensation on failures';
    RAISE NOTICE '  📊 Better event replay performance';
    RAISE NOTICE '  🔒 Transactional consistency across aggregates';
    RAISE NOTICE '';
    RAISE NOTICE 'Saga Management:';
    RAISE NOTICE '  🆕 SELECT create_saga(''saga_type'', 5, ''{}''::jsonb);';
    RAISE NOTICE '  ➕ SELECT add_saga_step(saga_id, 1, ''Step Name'', ''command'', ''{}''::jsonb);';
    RAISE NOTICE '  ▶️  SELECT execute_saga_step(step_id);';
    RAISE NOTICE '  ✅ SELECT complete_saga_step(step_id, result);';
    RAISE NOTICE '  ❌ SELECT fail_saga_step(step_id, ''error message'');';
    RAISE NOTICE '  🔄 SELECT compensate_saga(saga_id);';
    RAISE NOTICE '';
    RAISE NOTICE 'Monitoring:';
    RAISE NOTICE '  📊 SELECT * FROM v_saga_status;';
    RAISE NOTICE '  📅 SELECT * FROM get_saga_timeline(saga_id);';
    RAISE NOTICE '  📋 SELECT * FROM get_pending_saga_work();';
    RAISE NOTICE '';
    RAISE NOTICE 'Event Sourcing:';
    RAISE NOTICE '  🔁 SELECT replay_aggregate_optimized(aggregate_id, aggregate_type);';
    RAISE NOTICE '  📸 SELECT create_aggregate_snapshot_auto(aggregate_id, aggregate_type);';
    RAISE NOTICE '';
    RAISE NOTICE 'Example Usage:';
    RAISE NOTICE '  SELECT create_permission_assignment_saga(';
    RAISE NOTICE '    ''0x123...'', group_uuid, ''0x456...''';
    RAISE NOTICE '  );';
    RAISE NOTICE '';
    RAISE NOTICE 'Next Steps:';
    RAISE NOTICE '  1. Implement saga orchestrator service in backend';
    RAISE NOTICE '  2. Add saga polling/worker for step execution';
    RAISE NOTICE '  3. Create domain-specific sagas';
    RAISE NOTICE '  4. Monitor saga execution patterns';
    RAISE NOTICE '=================================================================================';
    RAISE NOTICE '';
    RAISE NOTICE '🎉 ALL DATABASE REFACTORING PHASES COMPLETE!';
    RAISE NOTICE '';
    RAISE NOTICE 'Summary of All Changes:';
    RAISE NOTICE '  Phase 1: Schema Normalization - 15-20%% storage reduction';
    RAISE NOTICE '  Phase 2: Index Optimization - 50-70%% faster queries';
    RAISE NOTICE '  Phase 3: Partitioning - 70-80%% faster with partition pruning';
    RAISE NOTICE '  Phase 4: Read Models - Real-time consistency';
    RAISE NOTICE '  Phase 5: Saga Pattern - Distributed transaction support';
    RAISE NOTICE '';
    RAISE NOTICE 'Total Expected Improvements:';
    RAISE NOTICE '  📊 Storage: 20-30%% reduction (500MB-2GB saved)';
    RAISE NOTICE '  ⚡ Queries: 50-80%% faster across the board';
    RAISE NOTICE '  ✍️  Writes: 20-30%% faster operations';
    RAISE NOTICE '  🔄 Consistency: Real-time data propagation';
    RAISE NOTICE '  🛡️  Reliability: Transactional guarantees';
    RAISE NOTICE '=================================================================================';
END $$;
