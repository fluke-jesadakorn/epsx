ALTER TABLE public.openid_refresh_tokens
    ADD COLUMN IF NOT EXISTS token_digest BYTEA,
    ADD COLUMN IF NOT EXISTS digest_key_id VARCHAR(32),
    ADD COLUMN IF NOT EXISTS digest_version SMALLINT,
    ADD COLUMN IF NOT EXISTS storage_version SMALLINT,
    ADD COLUMN IF NOT EXISTS consumed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS replay_detected_at TIMESTAMPTZ;

DO $$
DECLARE
    expected_column TEXT;
    expected_data_type TEXT;
    expected_maximum_length INTEGER;
    observed_data_type TEXT;
    observed_maximum_length INTEGER;
    observed_is_nullable TEXT;
    observed_default TEXT;
    observed_is_identity TEXT;
    observed_is_generated TEXT;
BEGIN
    FOR expected_column, expected_data_type, expected_maximum_length IN
        SELECT *
        FROM (
            VALUES
                ('token_digest', 'bytea', NULL::INTEGER),
                ('digest_key_id', 'character varying', 32),
                ('digest_version', 'smallint', NULL::INTEGER),
                ('storage_version', 'smallint', NULL::INTEGER),
                ('consumed_at', 'timestamp with time zone', NULL::INTEGER),
                ('revoked_at', 'timestamp with time zone', NULL::INTEGER),
                ('replay_detected_at', 'timestamp with time zone', NULL::INTEGER)
        ) AS expected(column_name, data_type, maximum_length)
    LOOP
        SELECT
            columns.data_type,
            columns.character_maximum_length,
            columns.is_nullable,
            columns.column_default,
            columns.is_identity,
            columns.is_generated
        INTO
            observed_data_type,
            observed_maximum_length,
            observed_is_nullable,
            observed_default,
            observed_is_identity,
            observed_is_generated
        FROM information_schema.columns AS columns
        WHERE columns.table_schema = 'public'
          AND columns.table_name = 'openid_refresh_tokens'
          AND columns.column_name = expected_column;

        IF observed_data_type IS DISTINCT FROM expected_data_type
           OR observed_maximum_length IS DISTINCT FROM expected_maximum_length
           OR observed_is_nullable IS DISTINCT FROM 'YES'
           OR observed_default IS NOT NULL
           OR observed_is_identity IS DISTINCT FROM 'NO'
           OR observed_is_generated IS DISTINCT FROM 'NEVER' THEN
            RAISE EXCEPTION
                'openid_refresh_tokens.% drift: expected nullable, default-free, non-generated %(%), got type %, length %, nullable %, default %, identity %, generated %',
                expected_column,
                expected_data_type,
                expected_maximum_length,
                observed_data_type,
                observed_maximum_length,
                observed_is_nullable,
                observed_default,
                observed_is_identity,
                observed_is_generated;
        END IF;
    END LOOP;
END
$$;

DO $$
DECLARE
    constraint_name TEXT;
BEGIN
    FOREACH constraint_name IN ARRAY ARRAY[
        'openid_refresh_tokens_digest_shape_check',
        'openid_refresh_tokens_digest_size_check',
        'openid_refresh_tokens_digest_key_id_check',
        'openid_refresh_tokens_digest_version_check',
        'openid_refresh_tokens_terminal_state_check',
        'openid_refresh_tokens_consumed_order_check',
        'openid_refresh_tokens_revoked_order_check',
        'openid_refresh_tokens_replay_order_check'
    ]
    LOOP
        IF EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = constraint_name
              AND conrelid = 'public.openid_refresh_tokens'::regclass
        ) THEN
            RAISE EXCEPTION
                'pre-existing % is refused; reconcile catalog drift explicitly',
                constraint_name;
        END IF;
    END LOOP;

    IF EXISTS (
        SELECT 1
        FROM pg_class AS relation
        JOIN pg_namespace AS namespace ON namespace.oid = relation.relnamespace
        WHERE namespace.nspname = 'public'
          AND relation.relname = 'openid_refresh_tokens_digest_lookup_uq'
    ) THEN
        RAISE EXCEPTION
            'pre-existing openid_refresh_tokens_digest_lookup_uq is refused; reconcile catalog drift explicitly';
    END IF;
END
$$;

ALTER TABLE public.openid_refresh_tokens
    ADD CONSTRAINT openid_refresh_tokens_digest_shape_check
        CHECK (
            (
                storage_version IS NULL
                AND token_digest IS NULL
                AND digest_key_id IS NULL
                AND digest_version IS NULL
            )
            OR (
                storage_version = 2
                AND token_digest IS NOT NULL
                AND digest_key_id IS NOT NULL
                AND digest_version IS NOT NULL
                AND client_id IS NOT NULL
                AND family_id IS NOT NULL
            )
        ) NOT VALID,
    ADD CONSTRAINT openid_refresh_tokens_digest_size_check
        CHECK (token_digest IS NULL OR OCTET_LENGTH(token_digest) = 32) NOT VALID,
    ADD CONSTRAINT openid_refresh_tokens_digest_key_id_check
        CHECK (
            digest_key_id IS NULL
            OR digest_key_id ~ '^[A-Za-z0-9_-]{1,32}$'
        ) NOT VALID,
    ADD CONSTRAINT openid_refresh_tokens_digest_version_check
        CHECK (digest_version IS NULL OR digest_version > 0) NOT VALID,
    ADD CONSTRAINT openid_refresh_tokens_terminal_state_check
        CHECK (
            (
                storage_version IS NULL
                AND token_digest IS NULL
                AND digest_key_id IS NULL
                AND digest_version IS NULL
                AND consumed_at IS NULL
                AND revoked_at IS NULL
                AND replay_detected_at IS NULL
            )
            OR (
                storage_version = 2
                AND token_digest IS NOT NULL
                AND digest_key_id IS NOT NULL
                AND digest_version IS NOT NULL
                AND (
                    (
                        is_revoked IS FALSE
                        AND consumed_at IS NULL
                        AND revoked_at IS NULL
                        AND replay_detected_at IS NULL
                    )
                    OR (
                        is_revoked IS TRUE
                        AND consumed_at IS NOT NULL
                        AND revoked_at IS NULL
                    )
                    OR (
                        is_revoked IS TRUE
                        AND consumed_at IS NULL
                        AND revoked_at IS NOT NULL
                        AND replay_detected_at IS NULL
                    )
                )
            )
        ) NOT VALID,
    ADD CONSTRAINT openid_refresh_tokens_consumed_order_check
        CHECK (consumed_at IS NULL OR consumed_at >= created_at) NOT VALID,
    ADD CONSTRAINT openid_refresh_tokens_revoked_order_check
        CHECK (revoked_at IS NULL OR revoked_at >= created_at) NOT VALID,
    ADD CONSTRAINT openid_refresh_tokens_replay_order_check
        CHECK (
            replay_detected_at IS NULL
            OR (
                consumed_at IS NOT NULL
                AND replay_detected_at >= consumed_at
            )
        ) NOT VALID;

ALTER TABLE public.openid_refresh_tokens
    VALIDATE CONSTRAINT openid_refresh_tokens_digest_shape_check,
    VALIDATE CONSTRAINT openid_refresh_tokens_digest_size_check,
    VALIDATE CONSTRAINT openid_refresh_tokens_digest_key_id_check,
    VALIDATE CONSTRAINT openid_refresh_tokens_digest_version_check,
    VALIDATE CONSTRAINT openid_refresh_tokens_terminal_state_check,
    VALIDATE CONSTRAINT openid_refresh_tokens_consumed_order_check,
    VALIDATE CONSTRAINT openid_refresh_tokens_revoked_order_check,
    VALIDATE CONSTRAINT openid_refresh_tokens_replay_order_check;

CREATE UNIQUE INDEX openid_refresh_tokens_digest_lookup_uq
    ON public.openid_refresh_tokens (digest_key_id, digest_version, token_digest)
    WHERE token_digest IS NOT NULL;
