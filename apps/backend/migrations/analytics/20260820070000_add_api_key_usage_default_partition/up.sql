-- Keep API-key usage writes available after the last explicit monthly partition.
-- The default partition is intentionally additive. If a detached table from a
-- rollback already exists, reattach it so any rows retained by the rollback are
-- visible through the partitioned parent again.

DO $$
BEGIN
    IF to_regclass('infra_logs.api_key_usage_logs') IS NULL THEN
        RAISE EXCEPTION
            'infra_logs.api_key_usage_logs must exist before adding its default partition';
    END IF;

    IF to_regclass('infra_logs.api_key_usage_logs_default') IS NULL THEN
        CREATE TABLE infra_logs.api_key_usage_logs_default
            PARTITION OF infra_logs.api_key_usage_logs DEFAULT;
    ELSIF EXISTS (
        SELECT 1
        FROM pg_catalog.pg_inherits
        WHERE inhrelid = 'infra_logs.api_key_usage_logs_default'::regclass
          AND inhparent <> 'infra_logs.api_key_usage_logs'::regclass
    ) THEN
        RAISE EXCEPTION
            'infra_logs.api_key_usage_logs_default is attached to an unexpected parent';
    ELSIF NOT EXISTS (
        SELECT 1
        FROM pg_catalog.pg_inherits
        WHERE inhrelid = 'infra_logs.api_key_usage_logs_default'::regclass
          AND inhparent = 'infra_logs.api_key_usage_logs'::regclass
    ) THEN
        ALTER TABLE infra_logs.api_key_usage_logs
            ATTACH PARTITION infra_logs.api_key_usage_logs_default DEFAULT;
    END IF;
END
$$;

COMMENT ON TABLE infra_logs.api_key_usage_logs_default IS
    'Catch-all API-key usage partition; retain on rollback and reattach on reapply.';
