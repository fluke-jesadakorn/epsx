-- ============================================================================
-- Restore `notification_subscriptions` table (rollback path for the wave10
-- / track C drop migration).
-- ============================================================================
--
-- The schema mirrors the original `00000000000001_consolidated_baseline_v2/
-- up.sql` definition verbatim. The only change vs. the original is
-- the `COMMENT ON TABLE` — it's updated to note the wave-10 decision
-- and the upstream audit reference.
--
-- Safety notes:
--   - The down migration runs at the operator's explicit request;
--     it is *not* a rollback that ships with the production
--     rollout. The original forward migration
--     (`00000000000001_consolidated_baseline_v2`) is *not* edited —
--     the wave-10 drop is a *new* migration that layers on top.
--   - `IF NOT EXISTS` guards on the indexes and the CREATE TABLE.
--   - `uuid-ossp` extension is created on the spot so the rollback
--     works against a clean DB (the original migration does the
--     same).
--
-- ============================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

CREATE TABLE IF NOT EXISTS notification_subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    instance_id VARCHAR(100) NOT NULL,
    connection_id VARCHAR(100) NOT NULL,
    connected_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_ping_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    disconnected_at TIMESTAMP WITH TIME ZONE,
    user_agent TEXT,
    ip_address INET,
    redis_channel VARCHAR(200),

    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address = 'all' OR
        (wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND length(wallet_address) = 42)
    ),
    CONSTRAINT unique_connection UNIQUE (instance_id, connection_id)
);

CREATE INDEX IF NOT EXISTS idx_subs_wallet_active
    ON notification_subscriptions(wallet_address, connected_at)
    WHERE disconnected_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_subs_instance_active
    ON notification_subscriptions(instance_id, connected_at)
    WHERE disconnected_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_subs_stale
    ON notification_subscriptions(last_ping_at, disconnected_at)
    WHERE disconnected_at IS NULL;

COMMENT ON TABLE notification_subscriptions IS
    'Tracks active SSE connections for multi-instance Redis pub/sub '
    '(restored by wave10/track-c down migration; the table was dropped '
    'in the up migration because no INSERT path existed in the read-path — '
    'see docs/wave8-service-boundary/audit-notifications.md §4c).';

SELECT 'EPSX NOTIFICATIONS: notification_subscriptions table restored.' AS success_message;
