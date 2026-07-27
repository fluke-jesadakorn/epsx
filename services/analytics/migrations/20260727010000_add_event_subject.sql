-- Preserve wallet subjects separately from the legacy UUID compatibility
-- column. Existing rows remain untouched; new authenticated events retain
-- the backend-verified subject without coercion.

ALTER TABLE public.events
    ADD COLUMN IF NOT EXISTS subject VARCHAR(128);

CREATE INDEX IF NOT EXISTS events_subject_created_at_idx
    ON public.events (subject, created_at DESC)
    WHERE subject IS NOT NULL;

