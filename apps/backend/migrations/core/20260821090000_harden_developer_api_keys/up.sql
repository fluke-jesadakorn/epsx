-- Developer API keys are capabilities derived from live customer entitlements.
-- Nothing is assignable until it is explicitly opted in by the backend.
ALTER TABLE permissions
    ADD COLUMN IF NOT EXISTS api_assignable BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE permissions
SET api_assignable = TRUE,
    updated_at = NOW()
WHERE permission_string IN (
    'epsx:analytics:view',
    'epsx:analytics:advanced',
    'epsx:data:export'
)
  AND permission_string NOT LIKE 'admin:%';

CREATE INDEX IF NOT EXISTS idx_permissions_api_assignable
    ON permissions (permission_string)
    WHERE api_assignable = TRUE AND is_active = TRUE;

-- Durable mutation claims. The response secret is deliberately absent: a
-- replay can identify the previously-created resource but can never recover
-- or reveal the plaintext API key again.
CREATE TABLE IF NOT EXISTS developer_api_key_idempotency (
    principal VARCHAR(42) NOT NULL,
    operation VARCHAR(20) NOT NULL,
    idempotency_key VARCHAR(128) NOT NULL,
    payload_hash VARCHAR(64) NOT NULL,
    resource_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    PRIMARY KEY (principal, operation, idempotency_key),
    CONSTRAINT developer_api_key_idempotency_operation_check
        CHECK (operation IN ('create', 'revoke')),
    CONSTRAINT developer_api_key_idempotency_payload_hash_check
        CHECK (payload_hash ~ '^[0-9a-f]{64}$')
);

CREATE INDEX IF NOT EXISTS idx_developer_api_key_idempotency_resource
    ON developer_api_key_idempotency (resource_id)
    WHERE resource_id IS NOT NULL;

-- Core-local audit is inserted in the same transaction as the key mutation.
-- This makes audit failure a mutation failure instead of a best-effort log.
CREATE TABLE IF NOT EXISTS developer_api_key_audit (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    actor VARCHAR(42) NOT NULL,
    action VARCHAR(20) NOT NULL,
    api_key_id UUID NOT NULL REFERENCES api_keys(id),
    idempotency_key VARCHAR(128) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT developer_api_key_audit_action_check
        CHECK (action IN ('created', 'revoked')),
    UNIQUE (actor, action, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_developer_api_key_audit_key_created
    ON developer_api_key_audit (api_key_id, created_at DESC);

