-- Older production databases placed these tables in public. The consolidated
-- v3 baseline moved their definitions to infra_logs without a forward cutover.
-- Move the existing relations, preserving rows, indexes, constraints and owned
-- sequences. Keep simple, updatable views for legacy unqualified SQL readers.
CREATE SCHEMA IF NOT EXISTS infra_logs;

DO $$
DECLARE
    relation_name TEXT;
    source_kind "char";
    child RECORD;
BEGIN
    FOREACH relation_name IN ARRAY ARRAY[
        'api_key_usage_logs', 'event_store', 'outbox_events',
        'aggregate_snapshots', 'unified_audit_log', 'permission_audit_log',
        'payment_audit_log', 'assignment_audit_log', 'wallet_activity_logs',
        'audit_logs', 'analytics_events'
    ] LOOP
        SELECT c.relkind INTO source_kind FROM pg_class c
        JOIN pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relname=relation_name;
        IF source_kind IN ('r','p') THEN
            IF to_regclass(format('infra_logs.%I',relation_name)) IS NOT NULL THEN
                RAISE EXCEPTION 'Both public.% and infra_logs.% exist; reconcile before moving data', relation_name, relation_name;
            END IF;
            EXECUTE format('ALTER TABLE public.%I SET SCHEMA infra_logs',relation_name);
            FOR child IN
                SELECT c.relname FROM pg_inherits i
                JOIN pg_class c ON c.oid=i.inhrelid
                JOIN pg_namespace n ON n.oid=c.relnamespace
                WHERE i.inhparent=to_regclass(format('infra_logs.%I',relation_name)) AND n.nspname='public'
            LOOP
                EXECUTE format('ALTER TABLE public.%I SET SCHEMA infra_logs',child.relname);
            END LOOP;
        ELSIF source_kind IS NOT NULL AND source_kind <> 'v' THEN
            RAISE EXCEPTION 'Unexpected public relation kind for %', relation_name;
        END IF;
        IF to_regclass(format('infra_logs.%I',relation_name)) IS NULL THEN
            RAISE EXCEPTION 'Missing analytics relation %; refusing empty replacement',relation_name;
        END IF;
        IF to_regclass(format('public.%I',relation_name)) IS NULL THEN
            EXECUTE format('CREATE VIEW public.%I AS SELECT * FROM infra_logs.%I',relation_name,relation_name);
        END IF;
    END LOOP;
END
$$;
