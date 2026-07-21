CREATE TABLE IF NOT EXISTS public.identity_users (
    user_id UUID PRIMARY KEY,
    wallet_address VARCHAR(42) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_authenticated_at TIMESTAMPTZ,
    CONSTRAINT identity_users_wallet_address_unique UNIQUE (wallet_address),
    CONSTRAINT identity_users_wallet_address_format CHECK (wallet_address ~ '^0x[0-9a-f]{40}$'),
    CONSTRAINT identity_users_updated_order CHECK (updated_at >= created_at),
    CONSTRAINT identity_users_last_authenticated_order CHECK (
        last_authenticated_at IS NULL OR last_authenticated_at >= created_at
    )
);

CREATE TABLE IF NOT EXISTS public.identity_siwe_challenges (
    challenge_id UUID PRIMARY KEY,
    wallet_address VARCHAR(42) NOT NULL,
    client_id VARCHAR(64) NOT NULL,
    chain_id VARCHAR(20) NOT NULL,
    domain VARCHAR(255) NOT NULL,
    nonce_hash BYTEA NOT NULL,
    message_hash BYTEA NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    CONSTRAINT identity_siwe_challenges_nonce_hash_unique UNIQUE (nonce_hash),
    CONSTRAINT identity_siwe_challenges_wallet_address_format CHECK (
        wallet_address ~ '^0x[0-9a-f]{40}$'
    ),
    CONSTRAINT identity_siwe_challenges_client_id CHECK (
        client_id IN ('epsx-frontend', 'epsx-admin')
    ),
    CONSTRAINT identity_siwe_challenges_chain_id_format CHECK (
        CASE
            WHEN chain_id ~ '^(0|[1-9][0-9]{0,19})$'
                THEN chain_id::NUMERIC <= 18446744073709551615
            ELSE FALSE
        END
    ),
    CONSTRAINT identity_siwe_challenges_domain_normalized CHECK (
        domain = LOWER(domain)
        AND domain ~ '^[a-z0-9.-]+(:[0-9]{1,5})?$'
        AND LENGTH(domain) BETWEEN 1 AND 255
    ),
    CONSTRAINT identity_siwe_challenges_nonce_hash_size CHECK (OCTET_LENGTH(nonce_hash) = 32),
    CONSTRAINT identity_siwe_challenges_message_hash_size CHECK (OCTET_LENGTH(message_hash) = 32),
    CONSTRAINT identity_siwe_challenges_expiry_order CHECK (expires_at > issued_at),
    CONSTRAINT identity_siwe_challenges_consumed_order CHECK (
        consumed_at IS NULL OR consumed_at >= issued_at
    )
);

CREATE INDEX IF NOT EXISTS identity_siwe_challenges_active_lookup_idx
    ON public.identity_siwe_challenges (wallet_address, client_id, nonce_hash)
    WHERE consumed_at IS NULL;

CREATE INDEX IF NOT EXISTS identity_siwe_challenges_expiry_idx
    ON public.identity_siwe_challenges (expires_at);

CREATE TABLE IF NOT EXISTS public.identity_refresh_families (
    family_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    client_id VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,
    CONSTRAINT identity_refresh_families_user_fk
        FOREIGN KEY (user_id) REFERENCES public.identity_users (user_id) ON DELETE RESTRICT,
    CONSTRAINT identity_refresh_families_ownership_unique UNIQUE (family_id, user_id, client_id),
    CONSTRAINT identity_refresh_families_client_id CHECK (
        client_id IN ('epsx-frontend', 'epsx-admin')
    ),
    CONSTRAINT identity_refresh_families_revoked_order CHECK (
        revoked_at IS NULL OR revoked_at >= created_at
    )
);

CREATE TABLE IF NOT EXISTS public.identity_refresh_sessions (
    session_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    family_id UUID NOT NULL,
    parent_session_id UUID,
    client_id VARCHAR(64) NOT NULL,
    token_hash BYTEA NOT NULL,
    hash_key_id VARCHAR(64) NOT NULL,
    hash_version SMALLINT NOT NULL DEFAULT 1,
    generation INTEGER NOT NULL DEFAULT 0,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    CONSTRAINT identity_refresh_sessions_user_fk
        FOREIGN KEY (user_id) REFERENCES public.identity_users (user_id) ON DELETE RESTRICT,
    CONSTRAINT identity_refresh_sessions_family_fk
        FOREIGN KEY (family_id, user_id, client_id)
        REFERENCES public.identity_refresh_families (family_id, user_id, client_id)
        ON DELETE RESTRICT,
    CONSTRAINT identity_refresh_sessions_lineage_parent_key
        UNIQUE (session_id, user_id, family_id, client_id),
    CONSTRAINT identity_refresh_sessions_parent_lineage_fk
        FOREIGN KEY (parent_session_id, user_id, family_id, client_id)
        REFERENCES public.identity_refresh_sessions (session_id, user_id, family_id, client_id)
        ON DELETE RESTRICT,
    CONSTRAINT identity_refresh_sessions_parent_unique UNIQUE (parent_session_id),
    CONSTRAINT identity_refresh_sessions_not_self_parent CHECK (
        parent_session_id IS NULL OR parent_session_id <> session_id
    ),
    CONSTRAINT identity_refresh_sessions_token_hash_unique UNIQUE (hash_key_id, token_hash),
    CONSTRAINT identity_refresh_sessions_client_id CHECK (
        client_id IN ('epsx-frontend', 'epsx-admin')
    ),
    CONSTRAINT identity_refresh_sessions_token_hash_size CHECK (OCTET_LENGTH(token_hash) = 32),
    CONSTRAINT identity_refresh_sessions_hash_key_id_nonempty CHECK (
        hash_key_id ~ '^[A-Za-z0-9._:-]{1,64}$'
    ),
    CONSTRAINT identity_refresh_sessions_hash_version_positive CHECK (hash_version > 0),
    CONSTRAINT identity_refresh_sessions_generation_shape CHECK (
        (parent_session_id IS NULL AND generation = 0)
        OR (parent_session_id IS NOT NULL AND generation > 0)
    ),
    CONSTRAINT identity_refresh_sessions_expiry_order CHECK (expires_at > issued_at),
    CONSTRAINT identity_refresh_sessions_consumed_order CHECK (
        consumed_at IS NULL OR consumed_at >= issued_at
    ),
    CONSTRAINT identity_refresh_sessions_revoked_order CHECK (
        revoked_at IS NULL OR revoked_at >= issued_at
    )
);

CREATE INDEX IF NOT EXISTS identity_refresh_sessions_active_user_idx
    ON public.identity_refresh_sessions (user_id, client_id, expires_at)
    WHERE consumed_at IS NULL AND revoked_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS identity_refresh_sessions_one_root_per_family_idx
    ON public.identity_refresh_sessions (family_id)
    WHERE parent_session_id IS NULL;

CREATE INDEX IF NOT EXISTS identity_refresh_sessions_family_idx
    ON public.identity_refresh_sessions (family_id, generation DESC);

CREATE INDEX IF NOT EXISTS identity_refresh_sessions_expiry_idx
    ON public.identity_refresh_sessions (expires_at);
