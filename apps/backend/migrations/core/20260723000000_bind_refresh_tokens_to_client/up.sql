ALTER TABLE public.openid_refresh_tokens
    ADD COLUMN IF NOT EXISTS client_id VARCHAR(32),
    ADD COLUMN IF NOT EXISTS family_id UUID;

DO $$
DECLARE
    column_data_type TEXT;
    column_maximum_length INTEGER;
    column_is_nullable TEXT;
    column_default TEXT;
    column_is_identity TEXT;
    column_is_generated TEXT;
BEGIN
    SELECT
        data_type,
        character_maximum_length,
        is_nullable,
        column_default,
        is_identity,
        is_generated
    INTO
        column_data_type,
        column_maximum_length,
        column_is_nullable,
        column_default,
        column_is_identity,
        column_is_generated
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'openid_refresh_tokens'
      AND column_name = 'client_id';

    IF column_data_type IS DISTINCT FROM 'character varying'
       OR column_maximum_length IS DISTINCT FROM 32
       OR column_is_nullable IS DISTINCT FROM 'YES' THEN
        RAISE EXCEPTION
            'openid_refresh_tokens.client_id drift: expected nullable VARCHAR(32), got type %, length %, nullable %',
            column_data_type, column_maximum_length, column_is_nullable;
    END IF;
    IF column_default IS NOT NULL
       OR column_is_identity IS DISTINCT FROM 'NO'
       OR column_is_generated IS DISTINCT FROM 'NEVER' THEN
        RAISE EXCEPTION
            'openid_refresh_tokens.client_id drift: defaults, identity, and generated values are forbidden';
    END IF;

    SELECT
        data_type,
        character_maximum_length,
        is_nullable,
        column_default,
        is_identity,
        is_generated
    INTO
        column_data_type,
        column_maximum_length,
        column_is_nullable,
        column_default,
        column_is_identity,
        column_is_generated
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'openid_refresh_tokens'
      AND column_name = 'family_id';

    IF column_data_type IS DISTINCT FROM 'uuid'
       OR column_maximum_length IS NOT NULL
       OR column_is_nullable IS DISTINCT FROM 'YES'
       OR column_default IS NOT NULL
       OR column_is_identity IS DISTINCT FROM 'NO'
       OR column_is_generated IS DISTINCT FROM 'NEVER' THEN
        RAISE EXCEPTION
            'openid_refresh_tokens.family_id drift: expected nullable UUID without default, identity, or generation';
    END IF;
END
$$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'openid_refresh_tokens_client_id_check'
          AND conrelid = 'public.openid_refresh_tokens'::regclass
    ) THEN
        RAISE EXCEPTION
            'pre-existing openid_refresh_tokens_client_id_check is refused; reconcile catalog drift explicitly';
    END IF;
END
$$;

ALTER TABLE public.openid_refresh_tokens
    ADD CONSTRAINT openid_refresh_tokens_client_id_check
    CHECK (
        client_id IS NULL
        OR client_id IN ('epsx-frontend', 'epsx-admin')
    )
    NOT VALID;

ALTER TABLE public.openid_refresh_tokens
    VALIDATE CONSTRAINT openid_refresh_tokens_client_id_check;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'openid_refresh_tokens_client_id_check'
          AND conrelid = 'public.openid_refresh_tokens'::regclass
          AND contype = 'c'
          AND convalidated
    ) THEN
        RAISE EXCEPTION 'openid_refresh_tokens_client_id_check is not validated';
    END IF;
END
$$;

CREATE INDEX openid_refresh_tokens_family_id_idx
    ON public.openid_refresh_tokens (family_id)
    WHERE family_id IS NOT NULL;
