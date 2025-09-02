-- OIDC Database Schema
-- Creates tables for OpenID Connect token management and audit

-- Create OIDC refresh tokens table
CREATE TABLE oidc_refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jti VARCHAR(255) NOT NULL UNIQUE,  -- JWT ID for token identification
    user_id VARCHAR(255) NOT NULL,     -- Firebase UID or internal user ID
    token_hash VARCHAR(255) NOT NULL,  -- Hashed token for security
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMP WITH TIME ZONE,
    revoked_reason TEXT,
    client_id VARCHAR(255),            -- Optional client identification
    scope TEXT,                        -- Token scope
    metadata JSONB                     -- Additional token metadata
);

-- Indexes for performance
CREATE INDEX idx_oidc_refresh_tokens_user_id ON oidc_refresh_tokens(user_id);
CREATE INDEX idx_oidc_refresh_tokens_jti ON oidc_refresh_tokens(jti);
CREATE INDEX idx_oidc_refresh_tokens_expires_at ON oidc_refresh_tokens(expires_at);
CREATE INDEX idx_oidc_refresh_tokens_revoked ON oidc_refresh_tokens(revoked) WHERE NOT revoked;

-- Create OIDC token audit log table
CREATE TABLE oidc_token_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation VARCHAR(50) NOT NULL,    -- INSERT, UPDATE, DELETE, VALIDATE
    table_name VARCHAR(100) NOT NULL,
    record_id UUID,                    -- ID of the affected record
    jti VARCHAR(255),                  -- JWT ID
    user_id VARCHAR(255),              -- User associated with the token
    old_values JSONB,                  -- Previous values (for UPDATE/DELETE)
    new_values JSONB,                  -- New values (for INSERT/UPDATE)
    changed_by VARCHAR(255),           -- User who made the change
    changed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address INET,                   -- IP address of the request
    user_agent TEXT,                   -- User agent string
    session_id VARCHAR(255),           -- Session identifier
    additional_data JSONB              -- Additional audit data
);

-- Indexes for audit log
CREATE INDEX idx_oidc_token_audit_operation ON oidc_token_audit(operation);
CREATE INDEX idx_oidc_token_audit_table_name ON oidc_token_audit(table_name);
CREATE INDEX idx_oidc_token_audit_user_id ON oidc_token_audit(user_id);
CREATE INDEX idx_oidc_token_audit_changed_at ON oidc_token_audit(changed_at);
CREATE INDEX idx_oidc_token_audit_jti ON oidc_token_audit(jti);

-- Create audit trigger function
CREATE OR REPLACE FUNCTION audit_oidc_token_changes()
RETURNS TRIGGER AS $$
DECLARE
    audit_data JSONB;
    user_context VARCHAR(255);
    ip_context INET;
    ua_context TEXT;
BEGIN
    -- Get context from application (set via SET LOCAL)
    BEGIN
        user_context := current_setting('app.current_user_id');
    EXCEPTION
        WHEN OTHERS THEN user_context := 'system';
    END;
    
    BEGIN
        ip_context := current_setting('app.client_ip')::INET;
    EXCEPTION
        WHEN OTHERS THEN ip_context := NULL;
    END;
    
    BEGIN
        ua_context := current_setting('app.user_agent');
    EXCEPTION
        WHEN OTHERS THEN ua_context := NULL;
    END;

    -- Build audit data based on operation
    audit_data := CASE TG_OP
        WHEN 'INSERT' THEN jsonb_build_object(
            'action', 'token_created',
            'token_type', 'refresh',
            'expires_at', NEW.expires_at,
            'scope', NEW.scope
        )
        WHEN 'UPDATE' THEN jsonb_build_object(
            'action', 'token_updated',
            'changes', jsonb_build_object(
                'revoked_changed', (OLD.revoked IS DISTINCT FROM NEW.revoked),
                'last_used_updated', (OLD.last_used_at IS DISTINCT FROM NEW.last_used_at)
            )
        )
        WHEN 'DELETE' THEN jsonb_build_object(
            'action', 'token_deleted',
            'was_revoked', OLD.revoked,
            'original_expires_at', OLD.expires_at
        )
    END;

    -- Insert audit record
    INSERT INTO oidc_token_audit (
        operation,
        table_name,
        record_id,
        jti,
        user_id,
        old_values,
        new_values,
        changed_by,
        ip_address,
        user_agent,
        additional_data
    ) VALUES (
        TG_OP,
        TG_TABLE_NAME,
        COALESCE(NEW.id, OLD.id),
        COALESCE(NEW.jti, OLD.jti),
        COALESCE(NEW.user_id, OLD.user_id),
        CASE WHEN TG_OP IN ('UPDATE', 'DELETE') THEN row_to_json(OLD)::JSONB ELSE NULL END,
        CASE WHEN TG_OP IN ('INSERT', 'UPDATE') THEN row_to_json(NEW)::JSONB ELSE NULL END,
        user_context,
        ip_context,
        ua_context,
        audit_data
    );

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create triggers for oidc_refresh_tokens table
CREATE TRIGGER oidc_refresh_tokens_audit_trigger
    AFTER INSERT OR UPDATE OR DELETE ON oidc_refresh_tokens
    FOR EACH ROW EXECUTE FUNCTION audit_oidc_token_changes();

-- Create function for token cleanup
CREATE OR REPLACE FUNCTION cleanup_expired_oidc_tokens(older_than_hours INTEGER DEFAULT 24)
RETURNS TABLE(deleted_count BIGINT) AS $$
DECLARE
    cutoff_time TIMESTAMP WITH TIME ZONE;
    result_count BIGINT;
BEGIN
    cutoff_time := NOW() - (older_than_hours || ' hours')::INTERVAL;
    
    -- Delete expired tokens
    WITH deleted AS (
        DELETE FROM oidc_refresh_tokens 
        WHERE expires_at < cutoff_time
        RETURNING id
    )
    SELECT COUNT(*) INTO result_count FROM deleted;
    
    -- Log cleanup action
    INSERT INTO oidc_token_audit (
        operation,
        table_name,
        changed_by,
        additional_data
    ) VALUES (
        'CLEANUP',
        'oidc_refresh_tokens',
        'system_cleanup',
        jsonb_build_object(
            'action', 'expired_tokens_cleanup',
            'deleted_count', result_count,
            'cutoff_time', cutoff_time,
            'cleanup_hours', older_than_hours
        )
    );
    
    RETURN QUERY SELECT result_count;
END;
$$ LANGUAGE plpgsql;

-- Create function for token revocation by user
CREATE OR REPLACE FUNCTION revoke_user_oidc_tokens(target_user_id VARCHAR(255), reason TEXT DEFAULT 'user_requested')
RETURNS TABLE(revoked_count BIGINT) AS $$
DECLARE
    result_count BIGINT;
BEGIN
    -- Revoke all active tokens for user
    WITH revoked AS (
        UPDATE oidc_refresh_tokens 
        SET 
            revoked = TRUE,
            revoked_at = NOW(),
            revoked_reason = reason
        WHERE user_id = target_user_id 
          AND NOT revoked
          AND expires_at > NOW()
        RETURNING id
    )
    SELECT COUNT(*) INTO result_count FROM revoked;
    
    RETURN QUERY SELECT result_count;
END;
$$ LANGUAGE plpgsql;

-- Create view for active tokens
CREATE VIEW active_oidc_tokens AS
SELECT 
    id,
    jti,
    user_id,
    expires_at,
    created_at,
    last_used_at,
    scope,
    metadata,
    EXTRACT(EPOCH FROM (expires_at - NOW())) AS seconds_until_expiry
FROM oidc_refresh_tokens
WHERE NOT revoked 
  AND expires_at > NOW();

-- Create view for token statistics
CREATE VIEW oidc_token_stats AS
WITH token_counts AS (
    SELECT 
        COUNT(*) as total_tokens,
        COUNT(*) FILTER (WHERE NOT revoked AND expires_at > NOW()) as active_tokens,
        COUNT(*) FILTER (WHERE revoked) as revoked_tokens,
        COUNT(*) FILTER (WHERE expires_at <= NOW()) as expired_tokens,
        COUNT(DISTINCT user_id) as unique_users,
        AVG(EXTRACT(EPOCH FROM (expires_at - created_at))) as avg_token_lifetime_seconds,
        MIN(created_at) as oldest_token_created,
        MAX(created_at) as newest_token_created
    FROM oidc_refresh_tokens
),
recent_activity AS (
    SELECT 
        COUNT(*) FILTER (WHERE changed_at >= NOW() - INTERVAL '1 hour') as operations_last_hour,
        COUNT(*) FILTER (WHERE changed_at >= NOW() - INTERVAL '24 hours') as operations_last_day,
        COUNT(*) FILTER (WHERE operation = 'INSERT' AND changed_at >= NOW() - INTERVAL '24 hours') as tokens_created_today,
        COUNT(*) FILTER (WHERE operation = 'DELETE' AND changed_at >= NOW() - INTERVAL '24 hours') as tokens_deleted_today
    FROM oidc_token_audit
)
SELECT 
    tc.*,
    ra.operations_last_hour,
    ra.operations_last_day,
    ra.tokens_created_today,
    ra.tokens_deleted_today,
    NOW() as stats_generated_at
FROM token_counts tc
CROSS JOIN recent_activity ra;

-- Grant necessary permissions (adjust as needed for your setup)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON oidc_refresh_tokens TO epsx_backend;
-- GRANT SELECT, INSERT ON oidc_token_audit TO epsx_backend;
-- GRANT SELECT ON active_oidc_tokens TO epsx_backend;
-- GRANT SELECT ON oidc_token_stats TO epsx_backend;
-- GRANT EXECUTE ON FUNCTION cleanup_expired_oidc_tokens(INTEGER) TO epsx_backend;
-- GRANT EXECUTE ON FUNCTION revoke_user_oidc_tokens(VARCHAR(255), TEXT) TO epsx_backend;