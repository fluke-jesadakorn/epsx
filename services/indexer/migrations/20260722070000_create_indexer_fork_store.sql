DO $indexer_fork_store_preflight$
DECLARE
    collision_names TEXT;
BEGIN
    SELECT string_agg(pg_catalog.quote_ident(rel.relname), ', ' ORDER BY rel.relname)
    INTO collision_names
    FROM pg_catalog.pg_class rel
    JOIN pg_catalog.pg_namespace ns ON ns.oid = rel.relnamespace
    WHERE ns.nspname = 'public'
      AND rel.relname = ANY (ARRAY[
          'indexer_block_candidates',
          'indexer_transaction_inclusions',
          'indexer_receipts',
          'indexer_raw_logs',
          'indexer_selected_blocks',
          'indexer_chain_state',
          'indexer_mutation_journal',
          'indexer_mutation_blocks',
          'idx_indexer_block_candidates_parent',
          'idx_indexer_transaction_inclusions_hash'
      ]::TEXT[]);

    IF collision_names IS NOT NULL THEN
        RAISE EXCEPTION USING
            ERRCODE = '42P07',
            MESSAGE = 'indexer fork-store fresh-create collision in public: ' || collision_names,
            DETAIL = 'all eight fork-store table names and both explicit index names are reserved regardless of relation kind',
            HINT = 'refusing baseline adoption; inspect and reconcile the existing relations before running a reviewed migration';
    END IF;
END
$indexer_fork_store_preflight$;

CREATE TABLE IF NOT EXISTS public.indexer_block_candidates (
    chain_id BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,
    number BIGINT NOT NULL,
    parent_hash BYTEA NOT NULL,
    block_timestamp TIMESTAMPTZ NOT NULL,
    beneficiary BYTEA,
    gas_used BIGINT NOT NULL,
    gas_limit BIGINT NOT NULL,
    stored_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT indexer_block_candidates_pkey PRIMARY KEY (chain_id, block_hash),
    CONSTRAINT indexer_block_candidates_chain_number_hash_key UNIQUE (chain_id, number, block_hash),
    CONSTRAINT indexer_block_candidates_chain_id_check CHECK (chain_id BETWEEN 1 AND 9999999999),
    CONSTRAINT indexer_block_candidates_number_check CHECK (number >= 0),
    CONSTRAINT indexer_block_candidates_block_hash_check CHECK (
        octet_length(block_hash) = 32
        AND block_hash <> decode(repeat('00', 32), 'hex')
    ),
    CONSTRAINT indexer_block_candidates_parent_hash_check CHECK (octet_length(parent_hash) = 32),
    CONSTRAINT indexer_block_candidates_beneficiary_check CHECK (
        beneficiary IS NULL OR octet_length(beneficiary) = 20
    ),
    CONSTRAINT indexer_block_candidates_gas_used_check CHECK (gas_used >= 0),
    CONSTRAINT indexer_block_candidates_gas_limit_check CHECK (gas_limit >= 0),
    CONSTRAINT indexer_block_candidates_gas_bounds_check CHECK (gas_used <= gas_limit)
);

CREATE INDEX IF NOT EXISTS idx_indexer_block_candidates_parent
    ON public.indexer_block_candidates USING btree (chain_id, parent_hash, number, block_hash);

CREATE TABLE IF NOT EXISTS public.indexer_transaction_inclusions (
    chain_id BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,
    transaction_index INTEGER NOT NULL,
    transaction_hash BYTEA NOT NULL,
    from_address BYTEA NOT NULL,
    to_address BYTEA,
    value NUMERIC(78, 0) NOT NULL,
    input_data BYTEA NOT NULL,
    CONSTRAINT indexer_transaction_inclusions_pkey PRIMARY KEY (chain_id, block_hash, transaction_index),
    CONSTRAINT indexer_transaction_inclusions_chain_block_tx_hash_key UNIQUE (chain_id, block_hash, transaction_hash),
    CONSTRAINT indexer_transaction_inclusions_block_fkey FOREIGN KEY (chain_id, block_hash)
        REFERENCES public.indexer_block_candidates (chain_id, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_transaction_inclusions_chain_id_check CHECK (chain_id BETWEEN 1 AND 9999999999),
    CONSTRAINT indexer_transaction_inclusions_index_check CHECK (transaction_index >= 0),
    CONSTRAINT indexer_transaction_inclusions_hash_check CHECK (
        octet_length(transaction_hash) = 32
        AND transaction_hash <> decode(repeat('00', 32), 'hex')
    ),
    CONSTRAINT indexer_transaction_inclusions_from_check CHECK (octet_length(from_address) = 20),
    CONSTRAINT indexer_transaction_inclusions_to_check CHECK (
        to_address IS NULL OR octet_length(to_address) = 20
    ),
    CONSTRAINT indexer_transaction_inclusions_value_check CHECK (
        value >= 0
        AND value <= NUMERIC '115792089237316195423570985008687907853269984665640564039457584007913129639935'
    )
);

CREATE INDEX IF NOT EXISTS idx_indexer_transaction_inclusions_hash
    ON public.indexer_transaction_inclusions USING btree
    (chain_id, transaction_hash, block_hash, transaction_index);

CREATE TABLE IF NOT EXISTS public.indexer_receipts (
    chain_id BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,
    transaction_index INTEGER NOT NULL,
    outcome SMALLINT NOT NULL,
    post_state_root BYTEA,
    gas_used BIGINT NOT NULL,
    cumulative_gas_used BIGINT NOT NULL,
    CONSTRAINT indexer_receipts_pkey PRIMARY KEY (chain_id, block_hash, transaction_index),
    CONSTRAINT indexer_receipts_transaction_fkey FOREIGN KEY (chain_id, block_hash, transaction_index)
        REFERENCES public.indexer_transaction_inclusions (chain_id, block_hash, transaction_index)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_receipts_chain_id_check CHECK (chain_id BETWEEN 1 AND 9999999999),
    CONSTRAINT indexer_receipts_index_check CHECK (transaction_index >= 0),
    CONSTRAINT indexer_receipts_outcome_check CHECK (outcome BETWEEN 0 AND 2),
    CONSTRAINT indexer_receipts_post_state_root_check CHECK (
        (outcome IN (0, 1) AND post_state_root IS NULL)
        OR (
            outcome = 2
            AND post_state_root IS NOT NULL
            AND octet_length(post_state_root) = 32
        )
    ),
    CONSTRAINT indexer_receipts_gas_used_check CHECK (gas_used >= 0),
    CONSTRAINT indexer_receipts_cumulative_gas_check CHECK (cumulative_gas_used >= 0),
    CONSTRAINT indexer_receipts_gas_bounds_check CHECK (gas_used <= cumulative_gas_used)
);

CREATE TABLE IF NOT EXISTS public.indexer_raw_logs (
    chain_id BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,
    log_index INTEGER NOT NULL,
    transaction_index INTEGER NOT NULL,
    address BYTEA NOT NULL,
    topic0 BYTEA,
    topic1 BYTEA,
    topic2 BYTEA,
    topic3 BYTEA,
    data BYTEA NOT NULL,
    CONSTRAINT indexer_raw_logs_pkey PRIMARY KEY (chain_id, block_hash, log_index),
    CONSTRAINT indexer_raw_logs_receipt_fkey FOREIGN KEY (chain_id, block_hash, transaction_index)
        REFERENCES public.indexer_receipts (chain_id, block_hash, transaction_index)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_raw_logs_chain_id_check CHECK (chain_id BETWEEN 1 AND 9999999999),
    CONSTRAINT indexer_raw_logs_log_index_check CHECK (log_index >= 0),
    CONSTRAINT indexer_raw_logs_transaction_index_check CHECK (transaction_index >= 0),
    CONSTRAINT indexer_raw_logs_address_check CHECK (octet_length(address) = 20),
    CONSTRAINT indexer_raw_logs_topic0_check CHECK (topic0 IS NULL OR octet_length(topic0) = 32),
    CONSTRAINT indexer_raw_logs_topic1_check CHECK (topic1 IS NULL OR octet_length(topic1) = 32),
    CONSTRAINT indexer_raw_logs_topic2_check CHECK (topic2 IS NULL OR octet_length(topic2) = 32),
    CONSTRAINT indexer_raw_logs_topic3_check CHECK (topic3 IS NULL OR octet_length(topic3) = 32),
    CONSTRAINT indexer_raw_logs_topic_prefix_check CHECK (
        (topic1 IS NULL OR topic0 IS NOT NULL)
        AND (topic2 IS NULL OR topic1 IS NOT NULL)
        AND (topic3 IS NULL OR topic2 IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS public.indexer_selected_blocks (
    chain_id BIGINT NOT NULL,
    number BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,
    selected_revision BIGINT NOT NULL,
    CONSTRAINT indexer_selected_blocks_pkey PRIMARY KEY (chain_id, number),
    CONSTRAINT indexer_selected_blocks_chain_number_hash_key UNIQUE (chain_id, number, block_hash),
    CONSTRAINT indexer_selected_blocks_candidate_fkey FOREIGN KEY (chain_id, number, block_hash)
        REFERENCES public.indexer_block_candidates (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_selected_blocks_chain_id_check CHECK (chain_id BETWEEN 1 AND 9999999999),
    CONSTRAINT indexer_selected_blocks_number_check CHECK (number >= 0),
    CONSTRAINT indexer_selected_blocks_hash_check CHECK (
        octet_length(block_hash) = 32
        AND block_hash <> decode(repeat('00', 32), 'hex')
    ),
    CONSTRAINT indexer_selected_blocks_revision_check CHECK (
        selected_revision BETWEEN 1 AND 9223372036854775807
    )
);

CREATE TABLE IF NOT EXISTS public.indexer_chain_state (
    chain_id BIGINT NOT NULL,
    revision BIGINT NOT NULL DEFAULT 0,
    selected_head_number BIGINT,
    selected_head_hash BYTEA,
    finalized_selection_number BIGINT,
    finalized_selection_hash BYTEA,
    lease_owner VARCHAR(128),
    lease_fence BIGINT NOT NULL DEFAULT 0,
    lease_expires_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT indexer_chain_state_pkey PRIMARY KEY (chain_id),
    CONSTRAINT indexer_chain_state_selected_head_fkey
        FOREIGN KEY (chain_id, selected_head_number, selected_head_hash)
        REFERENCES public.indexer_selected_blocks (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT indexer_chain_state_finalized_selection_fkey
        FOREIGN KEY (chain_id, finalized_selection_number, finalized_selection_hash)
        REFERENCES public.indexer_selected_blocks (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT indexer_chain_state_chain_id_check CHECK (chain_id BETWEEN 1 AND 9999999999),
    CONSTRAINT indexer_chain_state_revision_check CHECK (
        revision BETWEEN 0 AND 9223372036854775807
    ),
    CONSTRAINT indexer_chain_state_selected_head_pair_check CHECK (
        (selected_head_number IS NULL) = (selected_head_hash IS NULL)
    ),
    CONSTRAINT indexer_chain_state_selected_head_shape_check CHECK (
        selected_head_number IS NULL
        OR (
            selected_head_number >= 0
            AND octet_length(selected_head_hash) = 32
            AND selected_head_hash <> decode(repeat('00', 32), 'hex')
        )
    ),
    CONSTRAINT indexer_chain_state_finalized_pair_check CHECK (
        (finalized_selection_number IS NULL) = (finalized_selection_hash IS NULL)
    ),
    CONSTRAINT indexer_chain_state_finalized_shape_check CHECK (
        finalized_selection_number IS NULL
        OR (
            finalized_selection_number >= 0
            AND octet_length(finalized_selection_hash) = 32
            AND finalized_selection_hash <> decode(repeat('00', 32), 'hex')
        )
    ),
    CONSTRAINT indexer_chain_state_finalized_bounds_check CHECK (
        finalized_selection_number IS NULL
        OR (
            selected_head_number IS NOT NULL
            AND finalized_selection_number <= selected_head_number
        )
    ),
    CONSTRAINT indexer_chain_state_lease_pair_check CHECK (
        (lease_owner IS NULL) = (lease_expires_at IS NULL)
    ),
    CONSTRAINT indexer_chain_state_lease_owner_check CHECK (
        lease_owner IS NULL OR lease_owner ~ '^[A-Za-z0-9._:-]{1,128}$'
    ),
    CONSTRAINT indexer_chain_state_lease_fence_check CHECK (
        lease_fence BETWEEN 0 AND 9223372036854775807
    ),
    CONSTRAINT indexer_chain_state_live_lease_fence_check CHECK (
        lease_owner IS NULL OR lease_fence >= 1
    )
);

CREATE TABLE IF NOT EXISTS public.indexer_mutation_journal (
    chain_id BIGINT NOT NULL,
    mutation_id BYTEA NOT NULL,
    kind SMALLINT NOT NULL,
    expected_revision BIGINT NOT NULL,
    expected_selected_head_number BIGINT,
    expected_selected_head_hash BYTEA,
    expected_finalized_selection_number BIGINT,
    expected_finalized_selection_hash BYTEA,
    lease_owner VARCHAR(128) NOT NULL,
    lease_fence BIGINT NOT NULL,
    common_ancestor_number BIGINT,
    common_ancestor_hash BYTEA,
    finality_target_number BIGINT,
    finality_target_hash BYTEA,
    result_revision BIGINT NOT NULL,
    result_selected_head_number BIGINT,
    result_selected_head_hash BYTEA,
    result_finalized_selection_number BIGINT,
    result_finalized_selection_hash BYTEA,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT indexer_mutation_journal_pkey PRIMARY KEY (chain_id, mutation_id),
    CONSTRAINT indexer_mutation_journal_chain_result_revision_key UNIQUE (chain_id, result_revision),
    CONSTRAINT indexer_mutation_journal_chain_fkey FOREIGN KEY (chain_id)
        REFERENCES public.indexer_chain_state (chain_id)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_mutation_journal_expected_head_fkey
        FOREIGN KEY (chain_id, expected_selected_head_number, expected_selected_head_hash)
        REFERENCES public.indexer_block_candidates (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_mutation_journal_expected_finalized_fkey
        FOREIGN KEY (chain_id, expected_finalized_selection_number, expected_finalized_selection_hash)
        REFERENCES public.indexer_block_candidates (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_mutation_journal_common_ancestor_fkey
        FOREIGN KEY (chain_id, common_ancestor_number, common_ancestor_hash)
        REFERENCES public.indexer_block_candidates (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_mutation_journal_finality_target_fkey
        FOREIGN KEY (chain_id, finality_target_number, finality_target_hash)
        REFERENCES public.indexer_block_candidates (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_mutation_journal_result_head_fkey
        FOREIGN KEY (chain_id, result_selected_head_number, result_selected_head_hash)
        REFERENCES public.indexer_block_candidates (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_mutation_journal_result_finalized_fkey
        FOREIGN KEY (chain_id, result_finalized_selection_number, result_finalized_selection_hash)
        REFERENCES public.indexer_block_candidates (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_mutation_journal_chain_id_check CHECK (chain_id BETWEEN 1 AND 9999999999),
    CONSTRAINT indexer_mutation_journal_mutation_id_check CHECK (
        octet_length(mutation_id) = 32
        AND mutation_id <> decode(repeat('00', 32), 'hex')
    ),
    CONSTRAINT indexer_mutation_journal_kind_check CHECK (kind BETWEEN 0 AND 3),
    CONSTRAINT indexer_mutation_journal_expected_revision_check CHECK (
        expected_revision BETWEEN 0 AND 9223372036854775806
    ),
    CONSTRAINT indexer_mutation_journal_lease_owner_check CHECK (
        lease_owner ~ '^[A-Za-z0-9._:-]{1,128}$'
    ),
    CONSTRAINT indexer_mutation_journal_lease_fence_check CHECK (
        lease_fence BETWEEN 1 AND 9223372036854775807
    ),
    CONSTRAINT indexer_mutation_journal_result_revision_check CHECK (
        result_revision BETWEEN 1 AND 9223372036854775807
        AND result_revision = expected_revision + 1
    ),
    CONSTRAINT indexer_mutation_journal_expected_head_pair_check CHECK (
        (expected_selected_head_number IS NULL) = (expected_selected_head_hash IS NULL)
    ),
    CONSTRAINT indexer_mutation_journal_expected_head_shape_check CHECK (
        expected_selected_head_number IS NULL
        OR (
            expected_selected_head_number >= 0
            AND octet_length(expected_selected_head_hash) = 32
            AND expected_selected_head_hash <> decode(repeat('00', 32), 'hex')
        )
    ),
    CONSTRAINT indexer_mutation_journal_expected_finalized_pair_check CHECK (
        (expected_finalized_selection_number IS NULL) = (expected_finalized_selection_hash IS NULL)
    ),
    CONSTRAINT indexer_mutation_journal_expected_finalized_shape_check CHECK (
        expected_finalized_selection_number IS NULL
        OR (
            expected_finalized_selection_number >= 0
            AND octet_length(expected_finalized_selection_hash) = 32
            AND expected_finalized_selection_hash <> decode(repeat('00', 32), 'hex')
        )
    ),
    CONSTRAINT indexer_mutation_journal_common_ancestor_pair_check CHECK (
        (common_ancestor_number IS NULL) = (common_ancestor_hash IS NULL)
    ),
    CONSTRAINT indexer_mutation_journal_common_ancestor_shape_check CHECK (
        common_ancestor_number IS NULL
        OR (
            common_ancestor_number >= 0
            AND octet_length(common_ancestor_hash) = 32
            AND common_ancestor_hash <> decode(repeat('00', 32), 'hex')
        )
    ),
    CONSTRAINT indexer_mutation_journal_finality_target_pair_check CHECK (
        (finality_target_number IS NULL) = (finality_target_hash IS NULL)
    ),
    CONSTRAINT indexer_mutation_journal_finality_target_shape_check CHECK (
        finality_target_number IS NULL
        OR (
            finality_target_number >= 0
            AND octet_length(finality_target_hash) = 32
            AND finality_target_hash <> decode(repeat('00', 32), 'hex')
        )
    ),
    CONSTRAINT indexer_mutation_journal_result_head_pair_check CHECK (
        (result_selected_head_number IS NULL) = (result_selected_head_hash IS NULL)
    ),
    CONSTRAINT indexer_mutation_journal_result_head_shape_check CHECK (
        result_selected_head_number IS NULL
        OR (
            result_selected_head_number >= 0
            AND octet_length(result_selected_head_hash) = 32
            AND result_selected_head_hash <> decode(repeat('00', 32), 'hex')
        )
    ),
    CONSTRAINT indexer_mutation_journal_result_finalized_pair_check CHECK (
        (result_finalized_selection_number IS NULL) = (result_finalized_selection_hash IS NULL)
    ),
    CONSTRAINT indexer_mutation_journal_result_finalized_shape_check CHECK (
        result_finalized_selection_number IS NULL
        OR (
            result_finalized_selection_number >= 0
            AND octet_length(result_finalized_selection_hash) = 32
            AND result_finalized_selection_hash <> decode(repeat('00', 32), 'hex')
        )
    ),
    CONSTRAINT indexer_mutation_journal_expected_finalized_bounds_check CHECK (
        expected_finalized_selection_number IS NULL
        OR (
            expected_selected_head_number IS NOT NULL
            AND expected_finalized_selection_number <= expected_selected_head_number
        )
    ),
    CONSTRAINT indexer_mutation_journal_result_finalized_bounds_check CHECK (
        result_finalized_selection_number IS NULL
        OR (
            result_selected_head_number IS NOT NULL
            AND result_finalized_selection_number <= result_selected_head_number
        )
    ),
    CONSTRAINT indexer_mutation_journal_kind_shape_check CHECK (
        (
            kind = 0
            AND expected_revision = 0
            AND expected_selected_head_number IS NULL
            AND expected_finalized_selection_number IS NULL
            AND common_ancestor_number IS NULL
            AND finality_target_number IS NULL
            AND result_selected_head_number IS NOT NULL
            AND result_finalized_selection_number IS NULL
        )
        OR (
            kind = 1
            AND expected_selected_head_number IS NOT NULL
            AND result_selected_head_number IS NOT NULL
            AND common_ancestor_number IS NULL
            AND finality_target_number IS NULL
            AND result_selected_head_number > expected_selected_head_number
            AND result_finalized_selection_number
                IS NOT DISTINCT FROM expected_finalized_selection_number
            AND result_finalized_selection_hash
                IS NOT DISTINCT FROM expected_finalized_selection_hash
        )
        OR (
            kind = 2
            AND expected_selected_head_number IS NOT NULL
            AND result_selected_head_number IS NOT NULL
            AND common_ancestor_number IS NOT NULL
            AND finality_target_number IS NULL
            AND common_ancestor_number <= expected_selected_head_number
            AND result_selected_head_number > common_ancestor_number
            AND (
                expected_finalized_selection_number IS NULL
                OR common_ancestor_number >= expected_finalized_selection_number
            )
            AND result_finalized_selection_number
                IS NOT DISTINCT FROM expected_finalized_selection_number
            AND result_finalized_selection_hash
                IS NOT DISTINCT FROM expected_finalized_selection_hash
        )
        OR (
            kind = 3
            AND expected_selected_head_number IS NOT NULL
            AND common_ancestor_number IS NULL
            AND finality_target_number IS NOT NULL
            AND result_selected_head_number
                IS NOT DISTINCT FROM expected_selected_head_number
            AND result_selected_head_hash
                IS NOT DISTINCT FROM expected_selected_head_hash
            AND result_finalized_selection_number IS NOT NULL
            AND result_finalized_selection_number
                IS NOT DISTINCT FROM finality_target_number
            AND result_finalized_selection_hash
                IS NOT DISTINCT FROM finality_target_hash
            AND finality_target_number <= expected_selected_head_number
            AND (
                expected_finalized_selection_number IS NULL
                OR finality_target_number > expected_finalized_selection_number
            )
        )
    )
);

CREATE TABLE IF NOT EXISTS public.indexer_mutation_blocks (
    chain_id BIGINT NOT NULL,
    mutation_id BYTEA NOT NULL,
    role SMALLINT NOT NULL,
    ordinal BIGINT NOT NULL,
    number BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,
    CONSTRAINT indexer_mutation_blocks_pkey PRIMARY KEY (chain_id, mutation_id, role, ordinal),
    CONSTRAINT indexer_mutation_blocks_chain_mutation_role_number_key
        UNIQUE (chain_id, mutation_id, role, number),
    CONSTRAINT indexer_mutation_blocks_journal_fkey FOREIGN KEY (chain_id, mutation_id)
        REFERENCES public.indexer_mutation_journal (chain_id, mutation_id)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_mutation_blocks_candidate_fkey FOREIGN KEY (chain_id, number, block_hash)
        REFERENCES public.indexer_block_candidates (chain_id, number, block_hash)
        ON UPDATE NO ACTION ON DELETE NO ACTION,
    CONSTRAINT indexer_mutation_blocks_chain_id_check CHECK (chain_id BETWEEN 1 AND 9999999999),
    CONSTRAINT indexer_mutation_blocks_mutation_id_check CHECK (
        octet_length(mutation_id) = 32
        AND mutation_id <> decode(repeat('00', 32), 'hex')
    ),
    CONSTRAINT indexer_mutation_blocks_role_check CHECK (role IN (0, 1)),
    CONSTRAINT indexer_mutation_blocks_ordinal_check CHECK (ordinal >= 0),
    CONSTRAINT indexer_mutation_blocks_number_check CHECK (number >= 0),
    CONSTRAINT indexer_mutation_blocks_hash_check CHECK (
        octet_length(block_hash) = 32
        AND block_hash <> decode(repeat('00', 32), 'hex')
    )
);
