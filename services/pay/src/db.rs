//! Read-only schema boundary and provider construction for epsx-pay-svc.
//!
//! Versioned migration execution is deliberately owned outside the service.
//! Startup only checks that the exact `public` schema required by the current
//! SQLx models is present before any provider or listener is constructed.

use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum PaySchemaError {
    #[error("pay schema compatibility query failed")]
    Query(#[from] sqlx::Error),
    #[error("pay schema is absent or incompatible; run the reviewed migration before startup")]
    Incompatible,
}

/// Exact, read-only compatibility probe for the four pay candidate tables.
///
/// The query intentionally validates the complete public table, column,
/// constraint, and index inventory. It rejects hostile `search_path` shadowing,
/// unexpected inheritance/partitioning, RLS/policies, constraints owned by
/// the four candidate tables, standalone/partial/expression/INCLUDE indexes,
/// non-default type collations, and opclass name/namespace drift. Inbound
/// references from the service-owned admin evidence tables are permitted and
/// must not be mistaken for constraints or duplicate indexes on the target.
const PAY_SCHEMA_COMPATIBILITY_QUERY: &str = r#"
WITH expected_tables (table_name) AS (
    VALUES
        ('pay_intents'::text),
        ('escrows'::text),
        ('pay_links'::text),
        ('pay_webhook_events'::text)
),
target_tables AS (
    SELECT
        expected.table_name,
        relation_record.oid AS relation_oid,
        relation_record.relkind,
        relation_record.relpersistence,
        relation_record.relispartition,
        relation_record.relrowsecurity,
        relation_record.relforcerowsecurity
    FROM expected_tables AS expected
    LEFT JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.nspname = 'public'
    LEFT JOIN pg_catalog.pg_class AS relation_record
      ON relation_record.relnamespace = namespace_record.oid
     AND relation_record.relname = expected.table_name
),
expected_columns (
    table_name,
    ordinal_position,
    column_name,
    formatted_type,
    not_null,
    default_expression
) AS (
    VALUES
        ('pay_intents', 1, 'id', 'character varying(66)', true, NULL::text),
        ('pay_intents', 2, 'chain_id', 'character varying(20)', true, NULL::text),
        ('pay_intents', 3, 'payer', 'character varying(42)', true, NULL::text),
        ('pay_intents', 4, 'payee', 'character varying(42)', true, NULL::text),
        ('pay_intents', 5, 'amount', 'character varying(78)', true, NULL::text),
        ('pay_intents', 6, 'token_address', 'character varying(42)', true, NULL::text),
        ('pay_intents', 7, 'status', 'character varying(30)', true, '''pending''::character varying'),
        ('pay_intents', 8, 'escrow_id', 'character varying(66)', false, NULL::text),
        ('pay_intents', 9, 'tx_hash', 'character varying(66)', false, NULL::text),
        ('pay_intents', 10, 'description', 'text', false, NULL::text),
        ('pay_intents', 11, 'expires_at', 'timestamp with time zone', false, NULL::text),
        ('pay_intents', 12, 'created_at', 'timestamp with time zone', true, 'now()'),
        ('pay_intents', 13, 'updated_at', 'timestamp with time zone', true, 'now()'),
        ('escrows', 1, 'id', 'character varying(66)', true, NULL::text),
        ('escrows', 2, 'chain_id', 'character varying(20)', true, NULL::text),
        ('escrows', 3, 'payer', 'character varying(42)', true, NULL::text),
        ('escrows', 4, 'payee', 'character varying(42)', true, NULL::text),
        ('escrows', 5, 'amount', 'character varying(78)', true, NULL::text),
        ('escrows', 6, 'token_address', 'character varying(42)', true, NULL::text),
        ('escrows', 7, 'fee_amount', 'character varying(78)', true, '''0''::character varying'),
        ('escrows', 8, 'status', 'character varying(30)', true, '''active''::character varying'),
        ('escrows', 9, 'on_chain_id', 'character varying(78)', false, NULL::text),
        ('escrows', 10, 'tx_hash', 'character varying(66)', false, NULL::text),
        ('escrows', 11, 'dispute_reason', 'text', false, NULL::text),
        ('escrows', 12, 'created_at', 'timestamp with time zone', true, 'now()'),
        ('escrows', 13, 'updated_at', 'timestamp with time zone', true, 'now()'),
        ('pay_links', 1, 'id', 'character varying(66)', true, NULL::text),
        ('pay_links', 2, 'slug', 'character varying(32)', true, NULL::text),
        ('pay_links', 3, 'intent_id', 'character varying(66)', true, NULL::text),
        ('pay_links', 4, 'max_uses', 'integer', true, '1'),
        ('pay_links', 5, 'current_uses', 'integer', true, '0'),
        ('pay_links', 6, 'expires_at', 'timestamp with time zone', false, NULL::text),
        ('pay_links', 7, 'created_at', 'timestamp with time zone', true, 'now()'),
        ('pay_webhook_events', 1, 'event_id', 'character varying(128)', true, NULL::text),
        ('pay_webhook_events', 2, 'intent_id', 'character varying(66)', false, NULL::text),
        ('pay_webhook_events', 3, 'escrow_id', 'character varying(66)', false, NULL::text),
        ('pay_webhook_events', 4, 'event_type', 'character varying(64)', true, NULL::text),
        ('pay_webhook_events', 5, 'payload', 'jsonb', true, NULL::text),
        ('pay_webhook_events', 6, 'received_at', 'timestamp with time zone', true, 'now()')
),
actual_columns AS (
    SELECT
        target.table_name,
        attribute_record.attnum::integer AS ordinal_position,
        attribute_record.attname AS column_name,
        pg_catalog.format_type(attribute_record.atttypid, attribute_record.atttypmod) AS formatted_type,
        attribute_record.attnotnull AS not_null,
        pg_catalog.pg_get_expr(default_record.adbin, default_record.adrelid) AS default_expression,
        attribute_record.attcollation AS attribute_collation,
        type_record.typcollation AS type_default_collation
    FROM target_tables AS target
    JOIN pg_catalog.pg_attribute AS attribute_record
      ON attribute_record.attrelid = target.relation_oid
     AND attribute_record.attnum > 0
     AND NOT attribute_record.attisdropped
    LEFT JOIN pg_catalog.pg_attrdef AS default_record
      ON default_record.adrelid = attribute_record.attrelid
     AND default_record.adnum = attribute_record.attnum
    JOIN pg_catalog.pg_type AS type_record
      ON type_record.oid = attribute_record.atttypid
),
table_compatibility AS (
    SELECT
        COUNT(relation_oid) = 4
        AND COALESCE(bool_and(
            relkind = 'r'
            AND relpersistence = 'p'
            AND NOT relispartition
            AND NOT relrowsecurity
            AND NOT relforcerowsecurity
        ), false) AS compatible
    FROM target_tables
),
inheritance_compatibility AS (
    SELECT NOT EXISTS (
        SELECT 1
        FROM pg_catalog.pg_inherits AS inheritance_record
        WHERE inheritance_record.inhrelid IN (
            SELECT relation_oid FROM target_tables WHERE relation_oid IS NOT NULL
        )
           OR inheritance_record.inhparent IN (
            SELECT relation_oid FROM target_tables WHERE relation_oid IS NOT NULL
        )
    ) AS compatible
),
policy_compatibility AS (
    SELECT NOT EXISTS (
        SELECT 1
        FROM pg_catalog.pg_policy AS policy_record
        WHERE policy_record.polrelid IN (
            SELECT relation_oid FROM target_tables WHERE relation_oid IS NOT NULL
        )
    ) AS compatible
),
column_compatibility AS (
    SELECT
        (SELECT COUNT(*) FROM expected_columns) = 39
        AND (SELECT COUNT(*) FROM actual_columns) = 39
        AND COUNT(actual.column_name) = 39
        AND COALESCE(bool_and(
            actual.ordinal_position = expected.ordinal_position
            AND actual.column_name = expected.column_name
            AND actual.formatted_type = expected.formatted_type
            AND actual.not_null = expected.not_null
            AND COALESCE(
                CASE
                    WHEN expected.default_expression IS NULL
                        THEN actual.default_expression IS NULL
                    ELSE actual.default_expression = expected.default_expression
                END,
                false
            )
        ), false) AS compatible
    FROM expected_columns AS expected
    LEFT JOIN actual_columns AS actual
      ON actual.table_name = expected.table_name
     AND actual.ordinal_position = expected.ordinal_position
),
column_collation_compatibility AS (
    SELECT
        COUNT(*) = 28
        AND COALESCE(bool_and(attribute_collation = type_default_collation), false) AS compatible
    FROM actual_columns
    WHERE formatted_type LIKE 'character varying(%'
       OR formatted_type = 'text'
),
expected_structural_constraints (table_name, column_name, constraint_type) AS (
    VALUES
        ('pay_intents', 'id', 'p'::"char"),
        ('escrows', 'id', 'p'::"char"),
        ('pay_links', 'id', 'p'::"char"),
        ('pay_links', 'slug', 'u'::"char"),
        ('pay_webhook_events', 'event_id', 'p'::"char")
),
actual_structural_constraints AS (
    SELECT
        target.table_name,
        key_attribute.attname AS column_name,
        constraint_record.contype AS constraint_type,
        constraint_record.oid AS constraint_oid,
        constraint_record.conindid AS index_oid,
        constraint_record.convalidated,
        constraint_record.condeferrable,
        constraint_record.condeferred,
        constraint_record.conkey,
        constraint_record.confkey
    FROM target_tables AS target
    JOIN pg_catalog.pg_constraint AS constraint_record
      ON constraint_record.conrelid = target.relation_oid
     AND constraint_record.contype <> 'n'
    LEFT JOIN pg_catalog.pg_attribute AS key_attribute
      ON key_attribute.attrelid = constraint_record.conrelid
     AND key_attribute.attnum = constraint_record.conkey[1]
),
structural_constraint_boundary AS (
    SELECT constraint_record.oid
    FROM pg_catalog.pg_constraint AS constraint_record
    WHERE constraint_record.conrelid IN (
        SELECT relation_oid FROM target_tables WHERE relation_oid IS NOT NULL
    )
      AND constraint_record.contype <> 'n'
),
structural_constraint_compatibility AS (
    SELECT
        (SELECT COUNT(*) FROM expected_structural_constraints) = 5
        AND (SELECT COUNT(*) FROM actual_structural_constraints) = 5
        AND (SELECT COUNT(*) FROM structural_constraint_boundary) = 5
        AND NOT EXISTS (
            SELECT 1
            FROM expected_structural_constraints AS expected
            WHERE (
                SELECT COUNT(*)
                FROM actual_structural_constraints AS actual
                WHERE actual.table_name = expected.table_name
                  AND actual.column_name = expected.column_name
                  AND actual.constraint_type = expected.constraint_type
                  AND actual.convalidated
                  AND NOT actual.condeferrable
                  AND NOT actual.condeferred
                  AND cardinality(actual.conkey) = 1
                  AND actual.confkey IS NULL
                  AND actual.index_oid <> 0
            ) <> 1
        )
        AND NOT EXISTS (
            SELECT 1
            FROM actual_structural_constraints AS actual
            WHERE NOT EXISTS (
                SELECT 1
                FROM expected_structural_constraints AS expected
                WHERE expected.table_name = actual.table_name
                  AND expected.column_name = actual.column_name
                  AND expected.constraint_type = actual.constraint_type
            )
        ) AS compatible
),
actual_not_null_constraints AS (
    SELECT
        target.table_name,
        key_attribute.attname AS column_name,
        constraint_record.convalidated,
        constraint_record.condeferrable,
        constraint_record.condeferred,
        constraint_record.conkey,
        constraint_record.confkey,
        constraint_record.conindid
    FROM target_tables AS target
    JOIN pg_catalog.pg_constraint AS constraint_record
      ON constraint_record.conrelid = target.relation_oid
     AND constraint_record.contype = 'n'
    LEFT JOIN pg_catalog.pg_attribute AS key_attribute
      ON key_attribute.attrelid = constraint_record.conrelid
     AND key_attribute.attnum = constraint_record.conkey[1]
),
not_null_constraint_compatibility AS (
    SELECT CASE
        WHEN current_setting('server_version_num')::integer >= 180000 THEN
            (SELECT COUNT(*) FROM expected_columns WHERE not_null) = 29
            AND (SELECT COUNT(*) FROM actual_not_null_constraints) = 29
            AND NOT EXISTS (
                SELECT 1
                FROM expected_columns AS expected
                WHERE expected.not_null
                  AND (
                    SELECT COUNT(*)
                    FROM actual_not_null_constraints AS actual
                    WHERE actual.table_name = expected.table_name
                      AND actual.column_name = expected.column_name
                      AND actual.convalidated
                      AND NOT actual.condeferrable
                      AND NOT actual.condeferred
                      AND cardinality(actual.conkey) = 1
                      AND actual.confkey IS NULL
                      AND actual.conindid = 0
                  ) <> 1
            )
            AND NOT EXISTS (
                SELECT 1
                FROM actual_not_null_constraints AS actual
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM expected_columns AS expected
                    WHERE expected.table_name = actual.table_name
                      AND expected.column_name = actual.column_name
                      AND expected.not_null
                )
            )
        ELSE (SELECT COUNT(*) FROM actual_not_null_constraints) = 0
    END AS compatible
),
expected_indexes (table_name, key_signature, is_unique, is_primary, constraint_type) AS (
    VALUES
        ('pay_intents', 'id', true, true, 'p'::text),
        ('pay_intents', 'payer,status', false, false, NULL::text),
        ('pay_intents', 'payee,status', false, false, NULL::text),
        ('escrows', 'id', true, true, 'p'::text),
        ('escrows', 'status', false, false, NULL::text),
        ('pay_links', 'id', true, true, 'p'::text),
        ('pay_links', 'slug', true, false, 'u'::text),
        ('pay_links', 'slug', false, false, NULL::text),
        ('pay_links', 'intent_id', false, false, NULL::text),
        ('pay_webhook_events', 'event_id', true, true, 'p'::text),
        ('pay_webhook_events', 'intent_id', false, false, NULL::text)
),
actual_indexes AS (
    SELECT
        target.table_name,
        key_inventory.key_signature,
        index_record.indisunique AS is_unique,
        index_record.indisprimary AS is_primary,
        constraint_record.contype::text AS constraint_type,
        index_record.indisvalid,
        index_record.indisready,
        index_record.indimmediate,
        index_record.indisclustered,
        index_record.indisreplident,
        -- `indnullsnotdistinct` was added in PostgreSQL 15. Reading the
        -- catalog row through JSON keeps this compatibility guard valid on
        -- PostgreSQL 14 while still enforcing the flag when it is exposed by
        -- newer servers.
        COALESCE(
            (to_jsonb(index_record) ->> 'indnullsnotdistinct')::boolean,
            false
        ) AS indnullsnotdistinct,
        index_record.indnkeyatts,
        index_record.indnatts,
        index_record.indpred,
        index_record.indexprs,
        access_method.amname AS access_method,
        key_inventory.opclasses_compatible,
        key_inventory.collations_compatible,
        key_inventory.options_compatible
    FROM target_tables AS target
    JOIN pg_catalog.pg_index AS index_record
      ON index_record.indrelid = target.relation_oid
    JOIN pg_catalog.pg_class AS index_relation
      ON index_relation.oid = index_record.indexrelid
    JOIN pg_catalog.pg_am AS access_method
      ON access_method.oid = index_relation.relam
    LEFT JOIN pg_catalog.pg_constraint AS constraint_record
      ON constraint_record.conindid = index_record.indexrelid
     AND constraint_record.conrelid = target.relation_oid
     AND constraint_record.contype IN ('p', 'u')
    JOIN LATERAL (
        SELECT
            string_agg(attribute_record.attname, ',' ORDER BY key_column.ordinality) AS key_signature,
            bool_and(
                opclass_record.opcname = 'text_ops'
                AND opclass_namespace.nspname = 'pg_catalog'
            ) AS opclasses_compatible,
            bool_and(collation_column.collation_oid = attribute_record.attcollation) AS collations_compatible,
            bool_and(option_column.option_value = 0) AS options_compatible
        FROM unnest(index_record.indkey::smallint[]) WITH ORDINALITY
            AS key_column(attribute_number, ordinality)
        JOIN unnest(index_record.indclass::oid[]) WITH ORDINALITY
            AS opclass_column(opclass_oid, ordinality)
          USING (ordinality)
        JOIN unnest(index_record.indcollation::oid[]) WITH ORDINALITY
            AS collation_column(collation_oid, ordinality)
          USING (ordinality)
        JOIN unnest(index_record.indoption::smallint[]) WITH ORDINALITY
            AS option_column(option_value, ordinality)
          USING (ordinality)
        JOIN pg_catalog.pg_attribute AS attribute_record
          ON attribute_record.attrelid = index_record.indrelid
         AND attribute_record.attnum = key_column.attribute_number
        JOIN pg_catalog.pg_opclass AS opclass_record
          ON opclass_record.oid = opclass_column.opclass_oid
        JOIN pg_catalog.pg_namespace AS opclass_namespace
          ON opclass_namespace.oid = opclass_record.opcnamespace
        WHERE key_column.ordinality <= index_record.indnkeyatts
    ) AS key_inventory ON true
),
index_compatibility AS (
    SELECT
        (SELECT COUNT(*) FROM expected_indexes) = 11
        AND (SELECT COUNT(*) FROM actual_indexes) = 11
        AND COALESCE((
            SELECT bool_and(
                actual.indisvalid
                AND actual.indisready
                AND actual.indimmediate
                AND NOT actual.indisclustered
                AND NOT actual.indisreplident
                AND NOT actual.indnullsnotdistinct
                AND actual.indnkeyatts = actual.indnatts
                AND actual.indpred IS NULL
                AND actual.indexprs IS NULL
                AND actual.access_method = 'btree'
                AND actual.opclasses_compatible
                AND actual.collations_compatible
                AND actual.options_compatible
            )
            FROM actual_indexes AS actual
        ), false)
        AND NOT EXISTS (
            SELECT 1
            FROM expected_indexes AS expected
            WHERE (
                SELECT COUNT(*)
                FROM actual_indexes AS actual
                WHERE actual.table_name = expected.table_name
                  AND actual.key_signature = expected.key_signature
                  AND actual.is_unique = expected.is_unique
                  AND actual.is_primary = expected.is_primary
                  AND actual.constraint_type IS NOT DISTINCT FROM expected.constraint_type
            ) <> 1
        )
        AND NOT EXISTS (
            SELECT 1
            FROM actual_indexes AS actual
            WHERE NOT EXISTS (
                SELECT 1
                FROM expected_indexes AS expected
                WHERE expected.table_name = actual.table_name
                  AND expected.key_signature = actual.key_signature
                  AND expected.is_unique = actual.is_unique
                  AND expected.is_primary = actual.is_primary
                  AND expected.constraint_type IS NOT DISTINCT FROM actual.constraint_type
            )
        ) AS compatible
)
SELECT
    COALESCE((SELECT compatible FROM table_compatibility), false)
    AND COALESCE((SELECT compatible FROM inheritance_compatibility), false)
    AND COALESCE((SELECT compatible FROM policy_compatibility), false)
    AND COALESCE((SELECT compatible FROM column_compatibility), false)
    AND COALESCE((SELECT compatible FROM column_collation_compatibility), false)
    AND COALESCE((SELECT compatible FROM structural_constraint_compatibility), false)
    AND COALESCE((SELECT compatible FROM not_null_constraint_compatibility), false)
    AND COALESCE((SELECT compatible FROM index_compatibility), false)
    AND to_regclass('public.pay_intents') IS NOT NULL
    AND to_regclass('public.escrows') IS NOT NULL
    AND to_regclass('public.pay_links') IS NOT NULL
    AND to_regclass('public.pay_webhook_events') IS NOT NULL
"#;

pub async fn verify_schema_compatibility(db: &PgPool) -> Result<(), PaySchemaError> {
    let compatible = sqlx::query_scalar::<_, bool>(PAY_SCHEMA_COMPATIBILITY_QUERY)
        .fetch_one(db)
        .await?;
    if compatible {
        Ok(())
    } else {
        Err(PaySchemaError::Incompatible)
    }
}

/// Compute the 0.3% escrow fee from a U256-formatted amount string.
/// Pulled out of main.rs as part of the modularization so it can
/// be unit-tested independently and reused by any future handler
/// that needs fee math.
pub fn compute_fee(amount: &str) -> String {
    use std::str::FromStr;
    if let Ok(amt) = alloy::primitives::U256::from_str(amount) {
        // 0.3% fee
        let fee = amt / alloy::primitives::U256::from(333u64);
        fee.to_string()
    } else {
        "0".to_string()
    }
}

/// Build the alloy provider state — `Arc<RwLock<Option<…>>>` so the
/// provider can be lazily initialized per-chain (the alloy crate's
/// `Provider` impl is not `Send` in every flavor; we wrap with
/// `RwLock` so handlers can re-acquire).
pub fn build_provider(
    chain_id: u64,
) -> Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>> {
    let provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>> =
        Arc::new(RwLock::new(None));
    if let Ok(p) = epsx_web3::provider_for_chain(epsx_kernel::ChainId(chain_id)) {
        // Best-effort: try_lock avoids the poison-future footgun.
        // If the lock is held elsewhere (it shouldn't be at startup),
        // we silently skip — the handler will return 503 if it
        // needs the provider.
        if let Ok(mut guard) = provider.try_write() {
            *guard = Some(Arc::from(p));
        }
    }
    provider
}

/// Re-export the response/request types that handlers need
/// without importing `crate::types::*` everywhere. Keeps the
/// module-public API surface small.
pub mod prelude {
    pub use crate::types::{
        CreatePayIntentRequest, CreatePayLinkRequest, DisputeEscrowRequest, EscrowListResponse,
        EscrowRecord, PayHistoryResponse, PayIntent, PayIntentListResponse, PayIntentResponse,
        PayLink, PayLinkResponse, RedeemPayLinkRequest, RedeemPayLinkResponse, RefundEscrowRequest,
        ReleaseEscrowRequest, ResolveDisputeRequest, WebhookAck,
    };
}
