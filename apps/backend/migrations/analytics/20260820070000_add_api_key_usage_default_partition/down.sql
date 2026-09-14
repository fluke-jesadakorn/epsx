-- Preserve every usage row during rollback. Detaching removes the partition
-- from routing without dropping its table or data; reapplying the up migration
-- attaches the same table again.

DO $$
BEGIN
    IF to_regclass('infra_logs.api_key_usage_logs') IS NOT NULL
       AND to_regclass('infra_logs.api_key_usage_logs_default') IS NOT NULL
       AND EXISTS (
           SELECT 1
           FROM pg_catalog.pg_inherits
           WHERE inhrelid = 'infra_logs.api_key_usage_logs_default'::regclass
             AND inhparent = 'infra_logs.api_key_usage_logs'::regclass
       ) THEN
        ALTER TABLE infra_logs.api_key_usage_logs
            DETACH PARTITION infra_logs.api_key_usage_logs_default;
    END IF;
END
$$;
