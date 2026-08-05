CREATE TABLE IF NOT EXISTS public.e2e_baseline (
    id INTEGER PRIMARY KEY,
    marker TEXT NOT NULL
);

INSERT INTO public.e2e_baseline (id, marker)
VALUES (1, 'epsx-migration-baseline-v1')
ON CONFLICT (id) DO UPDATE SET marker = EXCLUDED.marker;

CREATE SCHEMA IF NOT EXISTS infra_logs;

CREATE TABLE IF NOT EXISTS infra_logs.outbox_events (
    id TEXT PRIMARY KEY,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS public.provider_callback_fixtures (
    id TEXT PRIMARY KEY,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS public.sse_cursors (
    owner_id TEXT PRIMARY KEY,
    cursor_value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS public.worker_leases (
    lease_key TEXT PRIMARY KEY,
    holder TEXT NOT NULL
);
