-- ================================================================================================
-- MIGRATION 033: Create Permission Audit Log System
-- ================================================================================================
-- Purpose: Track all permission grants, revocations, and modifications for security and compliance
-- Benefits:
--   - Complete audit trail for permission changes
--   - Compliance with security requirements
--   - Forensic analysis capabilities
--   - User permission history tracking
-- ================================================================================================

-- ================================================================================================
-- STEP 1: Create permission_audit_log Table
-- ================================================================================================

CREATE TABLE IF NOT EXISTS permission_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Event identification
    event_type VARCHAR(50) NOT NULL,  -- 'granted', 'revoked', 'modified', 'expired', 'group_assigned', 'group_removed'
    event_timestamp TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    event_source VARCHAR(100) DEFAULT 'system' NOT NULL,  -- 'admin_ui', 'api', 'system', 'migration', 'cron'

    -- Subject (who was affected)
    wallet_address VARCHAR(42) NOT NULL,

    -- Permission details
    permission_string VARCHAR(255),
    permission_id UUID,
    group_id UUID,
    group_name VARCHAR(100),

    -- Actor (who performed the action)
    performed_by VARCHAR(42),  -- Wallet address of admin/system
    performed_by_name VARCHAR(255),  -- Display name if available

    -- Context
    reason TEXT,
    request_id VARCHAR(36),  -- Correlate with API request logs
    ip_address INET,
    user_agent TEXT,

    -- Before/After state
    previous_state JSONB,  -- Previous permission state
    new_state JSONB,  -- New permission state

    -- Temporal information
    expires_at TIMESTAMPTZ,  -- For temporary permissions
    valid_from TIMESTAMPTZ,  -- When permission became effective
    valid_until TIMESTAMPTZ,  -- When permission was revoked/expired

    -- Additional metadata
    metadata JSONB DEFAULT '{}'::JSONB,

    -- Constraints
    CONSTRAINT valid_event_type CHECK (
        event_type IN (
            'granted', 'revoked', 'modified', 'expired',
            'group_assigned', 'group_removed', 'group_updated',
            'direct_permission_granted', 'direct_permission_revoked'
        )
    ),
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    ),
    CONSTRAINT valid_performed_by_format CHECK (
        performed_by IS NULL OR
        (performed_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(performed_by) = 42)
    )
);

-- ================================================================================================
-- STEP 2: Create Indexes for Audit Log Performance
-- ================================================================================================

-- Primary query patterns
CREATE INDEX idx_audit_log_wallet ON permission_audit_log(wallet_address, event_timestamp DESC);
CREATE INDEX idx_audit_log_timestamp ON permission_audit_log(event_timestamp DESC);
CREATE INDEX idx_audit_log_event_type ON permission_audit_log(event_type, event_timestamp DESC);
CREATE INDEX idx_audit_log_permission ON permission_audit_log(permission_string, event_timestamp DESC) WHERE permission_string IS NOT NULL;
CREATE INDEX idx_audit_log_group ON permission_audit_log(group_id, event_timestamp DESC) WHERE group_id IS NOT NULL;
CREATE INDEX idx_audit_log_performed_by ON permission_audit_log(performed_by, event_timestamp DESC) WHERE performed_by IS NOT NULL;
CREATE INDEX idx_audit_log_request_id ON permission_audit_log(request_id) WHERE request_id IS NOT NULL;

-- JSONB indexes for metadata queries
CREATE INDEX idx_audit_log_previous_state_gin ON permission_audit_log USING gin(previous_state);
CREATE INDEX idx_audit_log_new_state_gin ON permission_audit_log USING gin(new_state);
CREATE INDEX idx_audit_log_metadata_gin ON permission_audit_log USING gin(metadata);

-- Partitioning support (for future optimization)
CREATE INDEX idx_audit_log_timestamp_month ON permission_audit_log((date_trunc('month', event_timestamp)));

COMMENT ON TABLE permission_audit_log IS 'Complete audit trail of all permission-related events';
COMMENT ON COLUMN permission_audit_log.event_type IS 'Type of permission event that occurred';
COMMENT ON COLUMN permission_audit_log.event_source IS 'Where the event originated from';
COMMENT ON COLUMN permission_audit_log.previous_state IS 'Permission state before change (for forensic analysis)';
COMMENT ON COLUMN permission_audit_log.new_state IS 'Permission state after change';

-- ================================================================================================
-- STEP 3: Create Audit Logging Functions
-- ================================================================================================

-- Function to log permission grant event
CREATE OR REPLACE FUNCTION log_permission_granted(
    p_wallet_address VARCHAR,
    p_permission_string VARCHAR,
    p_permission_id UUID,
    p_granted_by VARCHAR,
    p_reason TEXT DEFAULT NULL,
    p_expires_at TIMESTAMPTZ DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
    v_audit_id UUID;
BEGIN
    INSERT INTO permission_audit_log (
        event_type,
        wallet_address,
        permission_string,
        permission_id,
        performed_by,
        reason,
        expires_at,
        valid_from,
        new_state,
        metadata
    ) VALUES (
        'direct_permission_granted',
        p_wallet_address,
        p_permission_string,
        p_permission_id,
        p_granted_by,
        p_reason,
        p_expires_at,
        NOW(),
        jsonb_build_object(
            'permission', p_permission_string,
            'expires_at', p_expires_at
        ),
        p_metadata
    )
    RETURNING id INTO v_audit_id;

    RETURN v_audit_id;
END;
$$ LANGUAGE plpgsql;

-- Function to log permission revoke event
CREATE OR REPLACE FUNCTION log_permission_revoked(
    p_wallet_address VARCHAR,
    p_permission_string VARCHAR,
    p_permission_id UUID,
    p_revoked_by VARCHAR,
    p_reason TEXT DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
    v_audit_id UUID;
BEGIN
    INSERT INTO permission_audit_log (
        event_type,
        wallet_address,
        permission_string,
        permission_id,
        performed_by,
        reason,
        valid_until,
        previous_state,
        metadata
    ) VALUES (
        'direct_permission_revoked',
        p_wallet_address,
        p_permission_string,
        p_permission_id,
        p_revoked_by,
        p_reason,
        NOW(),
        jsonb_build_object(
            'permission', p_permission_string
        ),
        p_metadata
    )
    RETURNING id INTO v_audit_id;

    RETURN v_audit_id;
END;
$$ LANGUAGE plpgsql;

-- Function to log group assignment event
CREATE OR REPLACE FUNCTION log_group_assigned(
    p_wallet_address VARCHAR,
    p_group_id UUID,
    p_group_name VARCHAR,
    p_assigned_by VARCHAR,
    p_reason TEXT DEFAULT NULL,
    p_expires_at TIMESTAMPTZ DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
    v_audit_id UUID;
    v_group_permissions JSONB;
BEGIN
    -- Get all permissions in the group
    SELECT jsonb_agg(p.permission_string)
    INTO v_group_permissions
    FROM permission_group_memberships pgm
    JOIN permissions p ON pgm.permission_id = p.id
    WHERE pgm.group_id = p_group_id
      AND p.is_active = TRUE;

    INSERT INTO permission_audit_log (
        event_type,
        wallet_address,
        group_id,
        group_name,
        performed_by,
        reason,
        expires_at,
        valid_from,
        new_state,
        metadata
    ) VALUES (
        'group_assigned',
        p_wallet_address,
        p_group_id,
        p_group_name,
        p_assigned_by,
        p_reason,
        p_expires_at,
        NOW(),
        jsonb_build_object(
            'group_id', p_group_id,
            'group_name', p_group_name,
            'permissions', v_group_permissions,
            'expires_at', p_expires_at
        ),
        p_metadata
    )
    RETURNING id INTO v_audit_id;

    RETURN v_audit_id;
END;
$$ LANGUAGE plpgsql;

-- Function to log group removal event
CREATE OR REPLACE FUNCTION log_group_removed(
    p_wallet_address VARCHAR,
    p_group_id UUID,
    p_group_name VARCHAR,
    p_removed_by VARCHAR,
    p_reason TEXT DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
    v_audit_id UUID;
    v_group_permissions JSONB;
BEGIN
    -- Get all permissions in the group
    SELECT jsonb_agg(p.permission_string)
    INTO v_group_permissions
    FROM permission_group_memberships pgm
    JOIN permissions p ON pgm.permission_id = p.id
    WHERE pgm.group_id = p_group_id
      AND p.is_active = TRUE;

    INSERT INTO permission_audit_log (
        event_type,
        wallet_address,
        group_id,
        group_name,
        performed_by,
        reason,
        valid_until,
        previous_state,
        metadata
    ) VALUES (
        'group_removed',
        p_wallet_address,
        p_group_id,
        p_group_name,
        p_removed_by,
        p_reason,
        NOW(),
        jsonb_build_object(
            'group_id', p_group_id,
            'group_name', p_group_name,
            'permissions', v_group_permissions
        ),
        p_metadata
    )
    RETURNING id INTO v_audit_id;

    RETURN v_audit_id;
END;
$$ LANGUAGE plpgsql;

-- ================================================================================================
-- STEP 4: Create Triggers for Automatic Audit Logging
-- ================================================================================================

-- Trigger function for wallet_direct_permissions
CREATE OR REPLACE FUNCTION audit_wallet_direct_permissions()
RETURNS TRIGGER AS $$
DECLARE
    v_permission_string VARCHAR;
BEGIN
    -- Get permission string
    SELECT permission_string INTO v_permission_string
    FROM permissions WHERE id = COALESCE(NEW.permission_id, OLD.permission_id);

    IF TG_OP = 'INSERT' THEN
        PERFORM log_permission_granted(
            NEW.wallet_address,
            v_permission_string,
            NEW.permission_id,
            NEW.granted_by,
            NEW.grant_reason,
            NEW.expires_at,
            jsonb_build_object('source', 'trigger')
        );
    ELSIF TG_OP = 'DELETE' THEN
        PERFORM log_permission_revoked(
            OLD.wallet_address,
            v_permission_string,
            OLD.permission_id,
            CURRENT_SETTING('app.current_user', TRUE),  -- Get from session variable
            NULL,
            jsonb_build_object('source', 'trigger')
        );
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_audit_wallet_direct_permissions
AFTER INSERT OR DELETE ON wallet_direct_permissions
FOR EACH ROW EXECUTE FUNCTION audit_wallet_direct_permissions();

-- Trigger function for wallet_group_assignments
CREATE OR REPLACE FUNCTION audit_wallet_group_assignments()
RETURNS TRIGGER AS $$
DECLARE
    v_group_name VARCHAR;
BEGIN
    -- Get group name
    SELECT name INTO v_group_name
    FROM permission_groups WHERE id = COALESCE(NEW.group_id, OLD.group_id);

    IF TG_OP = 'INSERT' THEN
        PERFORM log_group_assigned(
            NEW.wallet_address,
            NEW.group_id,
            v_group_name,
            NEW.assigned_by,
            NEW.assignment_reason,
            NEW.expires_at,
            jsonb_build_object('source', 'trigger')
        );
    ELSIF TG_OP = 'DELETE' THEN
        PERFORM log_group_removed(
            OLD.wallet_address,
            OLD.group_id,
            v_group_name,
            CURRENT_SETTING('app.current_user', TRUE),
            NULL,
            jsonb_build_object('source', 'trigger')
        );
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_audit_wallet_group_assignments
AFTER INSERT OR DELETE ON wallet_group_assignments
FOR EACH ROW EXECUTE FUNCTION audit_wallet_group_assignments();

-- ================================================================================================
-- STEP 5: Create Audit Query Helper Views
-- ================================================================================================

-- View: Recent permission changes
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

-- View: Permission history by wallet
CREATE OR REPLACE VIEW v_wallet_permission_history AS
SELECT
    pal.wallet_address,
    COUNT(*) FILTER (WHERE event_type IN ('granted', 'direct_permission_granted', 'group_assigned')) as total_grants,
    COUNT(*) FILTER (WHERE event_type IN ('revoked', 'direct_permission_revoked', 'group_removed')) as total_revocations,
    MAX(pal.event_timestamp) FILTER (WHERE event_type IN ('granted', 'direct_permission_granted', 'group_assigned')) as last_grant_at,
    MAX(pal.event_timestamp) FILTER (WHERE event_type IN ('revoked', 'direct_permission_revoked', 'group_removed')) as last_revoke_at
FROM permission_audit_log pal
GROUP BY pal.wallet_address;

COMMENT ON VIEW v_recent_permission_changes IS 'Recent permission changes (last 7 days) for monitoring';
COMMENT ON VIEW v_wallet_permission_history IS 'Permission change statistics per wallet';

-- ================================================================================================
-- VERIFICATION QUERIES
-- ================================================================================================

-- Check audit log structure:
-- SELECT column_name, data_type, is_nullable
-- FROM information_schema.columns
-- WHERE table_name = 'permission_audit_log'
-- ORDER BY ordinal_position;

-- Check triggers:
-- SELECT trigger_name, event_manipulation, event_object_table
-- FROM information_schema.triggers
-- WHERE trigger_name LIKE '%audit%';

-- ================================================================================================
-- ROLLBACK SCRIPT (If needed)
-- ================================================================================================
-- DROP TRIGGER IF EXISTS trg_audit_wallet_direct_permissions ON wallet_direct_permissions;
-- DROP TRIGGER IF EXISTS trg_audit_wallet_group_assignments ON wallet_group_assignments;
-- DROP FUNCTION IF EXISTS audit_wallet_direct_permissions();
-- DROP FUNCTION IF EXISTS audit_wallet_group_assignments();
-- DROP FUNCTION IF EXISTS log_permission_granted(VARCHAR, VARCHAR, UUID, VARCHAR, TEXT, TIMESTAMPTZ, JSONB);
-- DROP FUNCTION IF EXISTS log_permission_revoked(VARCHAR, VARCHAR, UUID, VARCHAR, TEXT, JSONB);
-- DROP FUNCTION IF EXISTS log_group_assigned(VARCHAR, UUID, VARCHAR, VARCHAR, TEXT, TIMESTAMPTZ, JSONB);
-- DROP FUNCTION IF EXISTS log_group_removed(VARCHAR, UUID, VARCHAR, VARCHAR, TEXT, JSONB);
-- DROP VIEW IF EXISTS v_recent_permission_changes;
-- DROP VIEW IF EXISTS v_wallet_permission_history;
-- DROP TABLE IF EXISTS permission_audit_log CASCADE;
