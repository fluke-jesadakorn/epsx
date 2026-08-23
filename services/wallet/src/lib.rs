use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::{header, HeaderMap, HeaderName, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Json, Router,
};
use epsx_service_auth::{
    authenticate_headers, AccessTokenVerifier, JwksVerifier, JwksVerifierConfig, VerifiedPrincipal,
    ADMIN_AUDIENCE, FRONTEND_AUDIENCE,
};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

pub const WALLET_JSON_BODY_LIMIT_BYTES: usize = 8 * 1024;
pub const WALLETS_READ_PERMISSION: &str = "admin:wallets:read";
pub const WALLETS_MANAGE_PERMISSION: &str = "admin:wallets:manage";
pub const CREDITS_READ_PERMISSION: &str = "admin:credits:read";
pub const CREDITS_MANAGE_PERMISSION: &str = "admin:credits:manage";

const WALLET_SCHEMA_COMPATIBILITY_QUERY: &str = r#"
WITH expected_tables (table_name, column_count) AS (
    VALUES
        ('accounts', 6::bigint),
        ('nonces', 4::bigint),
        ('signed_transactions', 7::bigint)
),
expected_columns (
    table_name,
    ordinal_position,
    column_name,
    data_type,
    udt_name,
    is_nullable,
    character_maximum_length,
    datetime_precision,
    collation_kind,
    default_kind
) AS (
    VALUES
        ('accounts', 1, 'address', 'character varying', 'varchar', 'NO', 42::bigint, NULL::bigint, 'database-default', 'none'),
        ('accounts', 2, 'chain_id', 'character varying', 'varchar', 'NO', 10::bigint, NULL::bigint, 'database-default', 'none'),
        ('accounts', 3, 'label', 'text', 'text', 'YES', NULL::bigint, NULL::bigint, 'database-default', 'none'),
        ('accounts', 4, 'role', 'character varying', 'varchar', 'YES', 50::bigint, NULL::bigint, 'database-default', 'user'),
        ('accounts', 5, 'encrypted_pk', 'text', 'text', 'YES', NULL::bigint, NULL::bigint, 'database-default', 'none'),
        ('accounts', 6, 'created_at', 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 6::bigint, 'none', 'now'),
        ('nonces', 1, 'address', 'character varying', 'varchar', 'NO', 42::bigint, NULL::bigint, 'database-default', 'none'),
        ('nonces', 2, 'chain_id', 'character varying', 'varchar', 'NO', 10::bigint, NULL::bigint, 'database-default', 'none'),
        ('nonces', 3, 'nonce', 'bigint', 'int8', 'NO', NULL::bigint, NULL::bigint, 'none', 'zero'),
        ('nonces', 4, 'updated_at', 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 6::bigint, 'none', 'now'),
        ('signed_transactions', 1, 'id', 'integer', 'int4', 'NO', NULL::bigint, NULL::bigint, 'none', 'serial'),
        ('signed_transactions', 2, 'chain_id', 'character varying', 'varchar', 'NO', 10::bigint, NULL::bigint, 'database-default', 'none'),
        ('signed_transactions', 3, 'sender', 'character varying', 'varchar', 'NO', 42::bigint, NULL::bigint, 'database-default', 'none'),
        ('signed_transactions', 4, 'recipient', 'character varying', 'varchar', 'YES', 42::bigint, NULL::bigint, 'database-default', 'none'),
        ('signed_transactions', 5, 'value', 'character varying', 'varchar', 'YES', 78::bigint, NULL::bigint, 'database-default', 'none'),
        ('signed_transactions', 6, 'data_hash', 'character varying', 'varchar', 'YES', 66::bigint, NULL::bigint, 'database-default', 'none'),
        ('signed_transactions', 7, 'created_at', 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 6::bigint, 'none', 'now')
),
column_compatibility AS (
    SELECT COALESCE(
        bool_and(COALESCE(
            actual.column_name IS NOT NULL
            AND actual.ordinal_position = expected.ordinal_position
            AND actual.data_type = expected.data_type
            AND actual.udt_name = expected.udt_name
            AND actual.is_nullable = expected.is_nullable
            AND actual.is_identity = 'NO'
            AND actual.is_generated = 'NEVER'
            AND actual.character_maximum_length IS NOT DISTINCT FROM expected.character_maximum_length
            AND actual.datetime_precision IS NOT DISTINCT FROM expected.datetime_precision
            AND COALESCE(
                CASE expected.collation_kind
                    WHEN 'database-default' THEN actual.collation_name IS NULL
                    ELSE actual.collation_name IS NULL
                END,
                false
            )
            AND COALESCE(
                CASE expected.default_kind
                    WHEN 'user' THEN actual.column_default = '''user''::character varying'
                    WHEN 'zero' THEN actual.column_default = '0'
                    WHEN 'now' THEN actual.column_default = 'now()'
                    WHEN 'serial' THEN
                        actual.column_default IN (
                            'nextval(''signed_transactions_id_seq''::regclass)',
                            'nextval(''public.signed_transactions_id_seq''::regclass)'
                        )
                        AND pg_catalog.to_regclass(pg_catalog.pg_get_serial_sequence(
                            'public.signed_transactions', 'id'
                        )) = pg_catalog.to_regclass('public.signed_transactions_id_seq')
                    ELSE actual.column_default IS NULL
                END,
                false
            ),
            false
        )),
        false
    ) AS compatible
    FROM expected_columns AS expected
    LEFT JOIN information_schema.columns AS actual
      ON actual.table_schema = 'public'
     AND actual.table_name = expected.table_name
     AND actual.column_name = expected.column_name
),
column_count_compatibility AS (
    SELECT COALESCE(bool_and(
        (
            SELECT COUNT(*)
            FROM information_schema.columns AS actual
            WHERE actual.table_schema = 'public'
              AND actual.table_name = expected.table_name
        ) = expected.column_count
    ), false) AS compatible
    FROM expected_tables AS expected
),
not_null_catalog_exposure AS (
    SELECT
        pg_catalog.current_setting('server_version_num')::integer >= 180000
        OR EXISTS (
            SELECT 1
            FROM pg_catalog.pg_constraint AS constraint_record
            JOIN pg_catalog.pg_class AS table_record
              ON table_record.oid = constraint_record.conrelid
            JOIN pg_catalog.pg_namespace AS namespace_record
              ON namespace_record.oid = table_record.relnamespace
            WHERE namespace_record.nspname = 'public'
              AND table_record.relname IN ('accounts', 'nonces', 'signed_transactions')
              AND constraint_record.contype = 'n'
        ) AS exposed
),
not_null_constraint_compatibility AS (
    SELECT
        NOT exposure.exposed
        OR (
            (
                SELECT COUNT(*)
                FROM expected_columns AS expected
                WHERE expected.is_nullable = 'NO'
            ) = (
                SELECT COUNT(*)
                FROM pg_catalog.pg_constraint AS constraint_record
                JOIN pg_catalog.pg_class AS table_record
                  ON table_record.oid = constraint_record.conrelid
                JOIN pg_catalog.pg_namespace AS namespace_record
                  ON namespace_record.oid = table_record.relnamespace
                WHERE namespace_record.nspname = 'public'
                  AND table_record.relname IN ('accounts', 'nonces', 'signed_transactions')
                  AND constraint_record.contype = 'n'
            )
            AND NOT EXISTS (
                SELECT 1
                FROM pg_catalog.pg_constraint AS constraint_record
                JOIN pg_catalog.pg_class AS table_record
                  ON table_record.oid = constraint_record.conrelid
                JOIN pg_catalog.pg_namespace AS namespace_record
                  ON namespace_record.oid = table_record.relnamespace
                LEFT JOIN pg_catalog.pg_attribute AS attribute_record
                  ON attribute_record.attrelid = table_record.oid
                 AND attribute_record.attnum = constraint_record.conkey[1]
                LEFT JOIN expected_columns AS expected
                  ON expected.table_name = table_record.relname
                 AND expected.column_name = attribute_record.attname
                WHERE namespace_record.nspname = 'public'
                  AND table_record.relname IN ('accounts', 'nonces', 'signed_transactions')
                  AND constraint_record.contype = 'n'
                  AND (
                      cardinality(constraint_record.conkey) IS DISTINCT FROM 1
                      OR expected.column_name IS NULL
                      OR expected.is_nullable <> 'NO'
                      OR constraint_record.condeferrable
                      OR constraint_record.condeferred
                      OR NOT constraint_record.convalidated
                      OR NOT COALESCE(
                          (pg_catalog.to_jsonb(constraint_record) ->> 'conenforced')::boolean,
                          false
                      )
                  )
            )
            AND NOT EXISTS (
                SELECT 1
                FROM expected_columns AS expected
                WHERE expected.is_nullable = 'YES'
                  AND EXISTS (
                      SELECT 1
                      FROM pg_catalog.pg_constraint AS constraint_record
                      JOIN pg_catalog.pg_class AS table_record
                        ON table_record.oid = constraint_record.conrelid
                      JOIN pg_catalog.pg_namespace AS namespace_record
                        ON namespace_record.oid = table_record.relnamespace
                      JOIN pg_catalog.pg_attribute AS attribute_record
                        ON attribute_record.attrelid = table_record.oid
                       AND attribute_record.attnum = constraint_record.conkey[1]
                      WHERE namespace_record.nspname = 'public'
                        AND table_record.relname = expected.table_name
                        AND attribute_record.attname = expected.column_name
                        AND constraint_record.contype = 'n'
                  )
            )
            AND NOT EXISTS (
                SELECT 1
                FROM expected_columns AS expected
                WHERE expected.is_nullable = 'NO'
                  AND NOT EXISTS (
                      SELECT 1
                      FROM pg_catalog.pg_constraint AS constraint_record
                      JOIN pg_catalog.pg_class AS table_record
                        ON table_record.oid = constraint_record.conrelid
                      JOIN pg_catalog.pg_namespace AS namespace_record
                        ON namespace_record.oid = table_record.relnamespace
                      JOIN pg_catalog.pg_attribute AS attribute_record
                        ON attribute_record.attrelid = table_record.oid
                       AND attribute_record.attnum = constraint_record.conkey[1]
                      WHERE namespace_record.nspname = 'public'
                        AND table_record.relname = expected.table_name
                        AND attribute_record.attname = expected.column_name
                        AND constraint_record.contype = 'n'
                        AND cardinality(constraint_record.conkey) = 1
                        AND NOT constraint_record.condeferrable
                        AND NOT constraint_record.condeferred
                        AND constraint_record.convalidated
                        AND COALESCE(
                            (pg_catalog.to_jsonb(constraint_record) ->> 'conenforced')::boolean,
                            false
                        )
                  )
            )
        ) AS compatible
    FROM not_null_catalog_exposure AS exposure
),
relation_compatibility AS (
    SELECT COALESCE(bool_and(COALESCE((
        SELECT
            relation_record.relkind = 'r'
            AND relation_record.relpersistence = 'p'
            AND NOT relation_record.relispartition
            AND NOT relation_record.relrowsecurity
            AND NOT relation_record.relforcerowsecurity
            AND relation_record.relreplident = 'd'
            AND NOT EXISTS (
                SELECT 1
                FROM pg_catalog.pg_inherits AS inheritance_record
                WHERE (
                    inheritance_record.inhrelid = relation_record.oid
                    OR inheritance_record.inhparent = relation_record.oid
                )
            )
        FROM pg_catalog.pg_class AS relation_record
        JOIN pg_catalog.pg_namespace AS namespace_record
          ON namespace_record.oid = relation_record.relnamespace
        WHERE namespace_record.nspname = 'public'
          AND relation_record.relname = expected.table_name
    ), false)), false) AS compatible
    FROM expected_tables AS expected
),
expected_primary_keys (table_name, key_columns) AS (
    VALUES
        ('accounts', ARRAY['address', 'chain_id']::text[]),
        ('nonces', ARRAY['address', 'chain_id']::text[]),
        ('signed_transactions', ARRAY['id']::text[])
),
constraint_compatibility AS (
    SELECT COALESCE(bool_and(COALESCE((
        SELECT
            COUNT(*) = 1
            AND COALESCE(bool_and(
                constraint_record.contype = 'p'
                AND NOT constraint_record.condeferrable
                AND NOT constraint_record.condeferred
                AND constraint_record.convalidated
                AND constraint_record.conindid = constraint_index.indexrelid
                AND constraint_index.indisprimary
                AND constraint_index.indimmediate
                AND cardinality(constraint_record.conkey) = cardinality(expected.key_columns)
                AND ARRAY(
                    SELECT attribute_record.attname::text
                    FROM unnest(constraint_record.conkey) WITH ORDINALITY
                        AS key_record(attnum, ordinal_position)
                    JOIN pg_catalog.pg_attribute AS attribute_record
                      ON attribute_record.attrelid = relation_record.oid
                     AND attribute_record.attnum = key_record.attnum
                    ORDER BY key_record.ordinal_position
                ) = expected.key_columns
            ), false)
        FROM pg_catalog.pg_constraint AS constraint_record
        JOIN pg_catalog.pg_class AS relation_record
          ON relation_record.oid = constraint_record.conrelid
        JOIN pg_catalog.pg_namespace AS namespace_record
          ON namespace_record.oid = relation_record.relnamespace
        LEFT JOIN pg_catalog.pg_index AS constraint_index
          ON constraint_index.indrelid = relation_record.oid
         AND constraint_index.indexrelid = constraint_record.conindid
        WHERE namespace_record.nspname = 'public'
          AND relation_record.relname = expected.table_name
          AND constraint_record.contype <> 'n'
    ), false)), false) AS compatible
    FROM expected_primary_keys AS expected
),
expected_indexes (table_name, key_columns, operator_classes, operator_class_namespaces) AS (
    VALUES
        ('accounts', ARRAY['address', 'chain_id']::text[], ARRAY['text_ops', 'text_ops']::text[], ARRAY['pg_catalog', 'pg_catalog']::text[]),
        ('nonces', ARRAY['address', 'chain_id']::text[], ARRAY['text_ops', 'text_ops']::text[], ARRAY['pg_catalog', 'pg_catalog']::text[]),
        ('signed_transactions', ARRAY['id']::text[], ARRAY['int4_ops']::text[], ARRAY['pg_catalog']::text[])
),
index_compatibility AS (
    SELECT COALESCE(bool_and(COALESCE((
        SELECT
            COUNT(*) = 1
            AND COALESCE(bool_and(
                index_record.indisprimary
                AND index_record.indisunique
                AND index_record.indisvalid
                AND index_record.indisready
                AND index_record.indislive
                AND index_record.indimmediate
                AND index_record.indpred IS NULL
                AND index_record.indexprs IS NULL
                AND index_record.indnkeyatts = cardinality(expected.key_columns)
                AND index_record.indnatts = index_record.indnkeyatts
                AND access_method.amname = 'btree'
                AND NOT EXISTS (
                    SELECT 1
                    FROM unnest(index_record.indoption) AS option_record(option_value)
                    WHERE option_record.option_value <> 0
                )
                AND ARRAY(
                    SELECT attribute_record.attname::text
                    FROM unnest(index_record.indkey) WITH ORDINALITY
                        AS key_record(attnum, ordinal_position)
                    JOIN pg_catalog.pg_attribute AS attribute_record
                      ON attribute_record.attrelid = table_record.oid
                     AND attribute_record.attnum = key_record.attnum
                    ORDER BY key_record.ordinal_position
                ) = expected.key_columns
                AND ARRAY(
                    SELECT collation_record.collation_oid
                    FROM unnest(index_record.indcollation) WITH ORDINALITY
                        AS collation_record(collation_oid, ordinal_position)
                    ORDER BY collation_record.ordinal_position
                ) = ARRAY(
                    SELECT attribute_record.attcollation
                    FROM unnest(index_record.indkey) WITH ORDINALITY
                        AS key_record(attnum, ordinal_position)
                    JOIN pg_catalog.pg_attribute AS attribute_record
                      ON attribute_record.attrelid = table_record.oid
                     AND attribute_record.attnum = key_record.attnum
                    ORDER BY key_record.ordinal_position
                )
                AND ARRAY(
                    SELECT operator_class.opcname::text
                    FROM unnest(index_record.indclass) WITH ORDINALITY
                        AS class_record(operator_class_oid, ordinal_position)
                    JOIN pg_catalog.pg_opclass AS operator_class
                      ON operator_class.oid = class_record.operator_class_oid
                    ORDER BY class_record.ordinal_position
                ) = expected.operator_classes
                AND ARRAY(
                    SELECT operator_namespace.nspname::text
                    FROM unnest(index_record.indclass) WITH ORDINALITY
                        AS class_record(operator_class_oid, ordinal_position)
                    JOIN pg_catalog.pg_opclass AS operator_class
                      ON operator_class.oid = class_record.operator_class_oid
                    JOIN pg_catalog.pg_namespace AS operator_namespace
                      ON operator_namespace.oid = operator_class.opcnamespace
                    ORDER BY class_record.ordinal_position
                ) = expected.operator_class_namespaces
            ), false)
        FROM pg_catalog.pg_index AS index_record
        JOIN pg_catalog.pg_class AS table_record
          ON table_record.oid = index_record.indrelid
        JOIN pg_catalog.pg_namespace AS namespace_record
          ON namespace_record.oid = table_record.relnamespace
        JOIN pg_catalog.pg_class AS index_relation
          ON index_relation.oid = index_record.indexrelid
        JOIN pg_catalog.pg_am AS access_method
          ON access_method.oid = index_relation.relam
        WHERE namespace_record.nspname = 'public'
          AND table_record.relname = expected.table_name
    ), false)), false) AS compatible
    FROM expected_indexes AS expected
),
serial_default_dependency_compatibility AS (
    SELECT COALESCE((
        SELECT
            COUNT(*) = 1
            AND COALESCE(bool_and(
                pg_catalog.pg_get_expr(default_record.adbin, default_record.adrelid) IN (
                    'nextval(''signed_transactions_id_seq''::regclass)',
                    'nextval(''public.signed_transactions_id_seq''::regclass)'
                )
            ), false)
            AND COALESCE(bool_and(NOT EXISTS (
                    SELECT 1
                    FROM pg_catalog.pg_depend AS other_dependency
                    JOIN pg_catalog.pg_class AS other_sequence
                      ON other_sequence.oid = other_dependency.refobjid
                     AND other_sequence.relkind = 'S'
                    WHERE other_dependency.classid = 'pg_catalog.pg_attrdef'::pg_catalog.regclass
                      AND other_dependency.objid = default_record.oid
                      AND other_dependency.objsubid = 0
                      AND other_dependency.refclassid = 'pg_catalog.pg_class'::pg_catalog.regclass
                      AND other_dependency.refobjid <> pg_catalog.to_regclass(
                          'public.signed_transactions_id_seq'
                      )
                )), false)
        FROM pg_catalog.pg_attrdef AS default_record
        JOIN pg_catalog.pg_class AS table_record
          ON table_record.oid = default_record.adrelid
        JOIN pg_catalog.pg_namespace AS table_namespace
          ON table_namespace.oid = table_record.relnamespace
        JOIN pg_catalog.pg_attribute AS attribute_record
          ON attribute_record.attrelid = table_record.oid
         AND attribute_record.attnum = default_record.adnum
        JOIN pg_catalog.pg_depend AS default_dependency
          ON default_dependency.classid = 'pg_catalog.pg_attrdef'::pg_catalog.regclass
         AND default_dependency.objid = default_record.oid
         AND default_dependency.objsubid = 0
         AND default_dependency.refclassid = 'pg_catalog.pg_class'::pg_catalog.regclass
         AND default_dependency.refobjid = pg_catalog.to_regclass(
             'public.signed_transactions_id_seq'
         )
         AND default_dependency.deptype = 'n'
        WHERE table_namespace.nspname = 'public'
          AND table_record.relname = 'signed_transactions'
          AND attribute_record.attname = 'id'
    ), false) AS compatible
),
serial_sequence_compatibility AS (
    SELECT COALESCE((
        SELECT
            sequence_record.seqtypid = 'integer'::pg_catalog.regtype
            AND sequence_record.seqstart = 1
            AND sequence_record.seqincrement = 1
            AND sequence_record.seqmax = 2147483647
            AND sequence_record.seqmin = 1
            AND sequence_record.seqcache = 1
            AND NOT sequence_record.seqcycle
        FROM pg_catalog.pg_sequence AS sequence_record
        JOIN pg_catalog.pg_class AS relation_record
          ON relation_record.oid = sequence_record.seqrelid
        JOIN pg_catalog.pg_namespace AS namespace_record
          ON namespace_record.oid = relation_record.relnamespace
        WHERE namespace_record.nspname = 'public'
          AND relation_record.relname = 'signed_transactions_id_seq'
    ), false) AS compatible
)
SELECT
    COALESCE((SELECT compatible FROM column_compatibility), false)
    AND COALESCE((SELECT compatible FROM column_count_compatibility), false)
    AND COALESCE((SELECT compatible FROM not_null_constraint_compatibility), false)
    AND COALESCE((SELECT compatible FROM relation_compatibility), false)
    AND COALESCE((SELECT compatible FROM constraint_compatibility), false)
    AND COALESCE((SELECT compatible FROM index_compatibility), false)
    AND COALESCE((SELECT compatible FROM serial_default_dependency_compatibility), false)
    AND COALESCE((SELECT compatible FROM serial_sequence_compatibility), false)
"#;

#[derive(Debug, Error)]
pub enum WalletSchemaError {
    #[error("wallet schema compatibility query failed")]
    Query(#[source] sqlx::Error),
    #[error("wallet schema is incompatible; run the reviewed wallet migration before startup")]
    Incompatible,
}

pub async fn verify_schema_compatibility(db: &sqlx::PgPool) -> Result<(), WalletSchemaError> {
    let compatible = sqlx::query_scalar::<_, bool>(WALLET_SCHEMA_COMPATIBILITY_QUERY)
        .fetch_one(db)
        .await
        .map_err(WalletSchemaError::Query)?;
    if !compatible {
        return Err(WalletSchemaError::Incompatible);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum WalletConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, WalletConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-wallet/1")
        .build()?;
    let config =
        JwksVerifierConfig::new(issuer, jwks_url, Duration::from_secs(5 * 60), production)?;
    Ok(Arc::new(JwksVerifier::new(config, client)))
}

#[derive(Clone)]
struct AuthState {
    verifier: Arc<dyn AccessTokenVerifier>,
}

pub fn protect_router(router: Router, verifier: Arc<dyn AccessTokenVerifier>) -> Router {
    router
        .layer(DefaultBodyLimit::max(WALLET_JSON_BODY_LIMIT_BYTES))
        .layer(middleware::from_fn_with_state(
            AuthState { verifier },
            authorize_request,
        ))
}

/// Resolve the only account owner key handlers may use. A compatibility path
/// address may agree case-insensitively, but it can never select a different
/// account. The verifier binds subject and wallet; this helper additionally
/// rejects non-canonical EVM identities before any SQL predicate is built.
pub fn canonical_owner(
    principal: &VerifiedPrincipal,
    claimed_address: Option<&str>,
) -> Result<String, StatusCode> {
    let owner = normalize_address(&principal.wallet_address).ok_or(StatusCode::FORBIDDEN)?;
    if claimed_address
        .is_some_and(|claimed| normalize_address(claimed).is_none_or(|claimed| claimed != owner))
    {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(owner)
}

fn normalize_address(address: &str) -> Option<String> {
    let bytes = address.as_bytes();
    if bytes.len() != 42
        || bytes[0] != b'0'
        || !matches!(bytes[1], b'x' | b'X')
        || !bytes[2..].iter().all(u8::is_ascii_hexdigit)
    {
        return None;
    }
    Some(format!("0x{}", address[2..].to_ascii_lowercase()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessPolicy {
    Public,
    OwnerRead,
    AdminPermission(&'static str),
    UnsafeProjection,
    UnsafeCustodyMutation,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if !normalized_path(path) {
        return AccessPolicy::Blocked;
    }
    if path == "/api/v1/admin/wallets"
        || path.starts_with("/api/v1/admin/wallets/")
        || path == "/api/admin/wallets"
        || path.starts_with("/api/admin/wallets/")
    {
        let tail = path
            .strip_prefix("/api/v1/admin/wallets")
            .or_else(|| path.strip_prefix("/api/admin/wallets"))
            .unwrap_or_default();
        let segments: Vec<_> = tail
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        return match (method, segments.as_slice()) {
            (&Method::GET, []) | (&Method::GET, ["stats"]) => {
                AccessPolicy::AdminPermission(WALLETS_READ_PERMISSION)
            }
            (&Method::GET, [address]) if safe_dynamic_segment(address) => {
                AccessPolicy::AdminPermission(WALLETS_READ_PERMISSION)
            }
            (&Method::POST, [address, "disable" | "enable"]) if safe_dynamic_segment(address) => {
                AccessPolicy::AdminPermission(WALLETS_MANAGE_PERMISSION)
            }
            (&Method::PATCH, [address, "metadata"]) if safe_dynamic_segment(address) => {
                AccessPolicy::AdminPermission(WALLETS_MANAGE_PERMISSION)
            }
            _ => AccessPolicy::Blocked,
        };
    }
    if path == "/api/v1/admin/credits"
        || path.starts_with("/api/v1/admin/credits/")
        || path == "/api/admin/credits"
        || path.starts_with("/api/admin/credits/")
    {
        let tail = path
            .strip_prefix("/api/v1/admin/credits")
            .or_else(|| path.strip_prefix("/api/admin/credits"))
            .unwrap_or_default();
        let segments: Vec<_> = tail
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        return match (method, segments.as_slice()) {
            (&Method::GET, []) => AccessPolicy::AdminPermission(CREDITS_READ_PERMISSION),
            (&Method::GET, [address]) if safe_dynamic_segment(address) => {
                AccessPolicy::AdminPermission(CREDITS_READ_PERMISSION)
            }
            (&Method::POST, [address, "grant" | "revoke"]) if safe_dynamic_segment(address) => {
                AccessPolicy::AdminPermission(CREDITS_MANAGE_PERMISSION)
            }
            _ => AccessPolicy::Blocked,
        };
    }
    let Some(tail) = path.strip_prefix("/api/v1/wallet/") else {
        return AccessPolicy::Blocked;
    };
    let segments: Vec<_> = tail.split('/').collect();

    match (method, segments.as_slice()) {
        (&Method::GET, ["accounts"]) => AccessPolicy::OwnerRead,
        (&Method::GET, ["accounts", address]) if safe_dynamic_segment(address) => {
            AccessPolicy::OwnerRead
        }
        (&Method::POST, ["verify-message"]) => AccessPolicy::Public,
        (&Method::GET, ["balance", chain, address])
            if safe_dynamic_segment(chain) && safe_dynamic_segment(address) =>
        {
            AccessPolicy::UnsafeProjection
        }
        (&Method::POST, ["accounts" | "send" | "sign-message"]) => {
            AccessPolicy::UnsafeCustodyMutation
        }
        (&Method::POST, ["estimate-gas"]) => AccessPolicy::UnsafeProjection,
        _ => AccessPolicy::Blocked,
    }
}

fn normalized_path(path: &str) -> bool {
    path.starts_with('/')
        && path.len() <= 2048
        && !path.contains('%')
        && !path.contains('\\')
        && !path.contains("//")
        && !path.ends_with('/')
}

fn safe_dynamic_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        && !matches!(
            segment,
            "." | ".."
                | "health"
                | "accounts"
                | "balance"
                | "send"
                | "sign-message"
                | "verify-message"
                | "estimate-gas"
        )
}

async fn authorize_request(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Response {
    strip_spoofable_identity_headers(request.headers_mut());
    match classify(request.method(), request.uri().path()) {
        AccessPolicy::Public => {
            request.headers_mut().remove(header::AUTHORIZATION);
        }
        AccessPolicy::OwnerRead => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != FRONTEND_AUDIENCE && principal.audience != ADMIN_AUDIENCE {
                return auth_error(StatusCode::FORBIDDEN);
            }
            if normalize_address(&principal.wallet_address).is_none() {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::AdminPermission(required) => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE || !principal.has_permission(required) {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::UnsafeProjection
        | AccessPolicy::UnsafeCustodyMutation
        | AccessPolicy::Blocked => return StatusCode::NOT_FOUND.into_response(),
    }
    next.run(request).await
}

fn auth_error(status: StatusCode) -> Response {
    let code = if status == StatusCode::FORBIDDEN {
        "forbidden"
    } else {
        "unauthorized"
    };
    (status, Json(serde_json::json!({ "error": code }))).into_response()
}

fn strip_spoofable_identity_headers(headers: &mut HeaderMap) {
    let names: Vec<HeaderName> = headers
        .keys()
        .filter(|name| {
            let name = name.as_str();
            name.starts_with("x-user-")
                || name.starts_with("x-wallet-")
                || name.starts_with("x-auth-")
                || name.starts_with("x-epsx-")
                || matches!(
                    name,
                    "x-user"
                        | "x-subject"
                        | "x-principal"
                        | "x-wallet"
                        | "x-address"
                        | "x-chain-id"
                        | "x-client-id"
                        | "x-permissions"
                        | "x-role"
                        | "x-roles"
                        | "x-scope"
                        | "x-forwarded-user"
                )
        })
        .cloned()
        .collect();
    for name in names {
        headers.remove(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::{body::Body, routing::any};
    use epsx_service_auth::{VerifiedPrincipal, VerifyError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    const OWNER: &str = "0x1111111111111111111111111111111111111111";

    #[test]
    fn schema_probe_is_read_only_and_null_safe() {
        let upper = WALLET_SCHEMA_COMPATIBILITY_QUERY.to_ascii_uppercase();
        assert!(upper.trim_start().starts_with("WITH "));
        for token in [
            " INSERT ",
            " UPDATE ",
            " DELETE ",
            " CREATE ",
            " ALTER ",
            " DROP ",
            " TRUNCATE ",
        ] {
            assert!(!upper.contains(token), "mutation token returned: {token}");
        }
        assert!(WALLET_SCHEMA_COMPATIBILITY_QUERY.contains("bool_and(COALESCE("));
        assert!(WALLET_SCHEMA_COMPATIBILITY_QUERY
            .contains("COALESCE((SELECT compatible FROM index_compatibility), false)"));
    }

    #[test]
    fn schema_probe_pins_public_relations_constraints_indexes_and_sequence() {
        for anchor in [
            "actual.table_schema = 'public'",
            "actual.datetime_precision IS NOT DISTINCT FROM expected.datetime_precision",
            "actual.collation_name IS NULL",
            "pg_catalog.current_setting('server_version_num')::integer >= 180000",
            "NOT exposure.exposed",
            "expected.is_nullable = 'YES'",
            "expected.is_nullable = 'NO'",
            "pg_catalog.to_jsonb(constraint_record) ->> 'conenforced'",
            "namespace_record.nspname = 'public'",
            "COUNT(*) = 1",
            "constraint_record.contype = 'p'",
            "NOT constraint_record.condeferrable",
            "NOT constraint_record.condeferred",
            "constraint_record.convalidated",
            "constraint_record.conindid = constraint_index.indexrelid",
            "LEFT JOIN pg_catalog.pg_index AS constraint_index",
            "index_record.indisprimary",
            "index_record.indimmediate",
            "unnest(index_record.indcollation) WITH ORDINALITY",
            "index_record.indpred IS NULL",
            "index_record.indexprs IS NULL",
            "operator_class.opcnamespace",
            "pg_catalog.pg_get_serial_sequence(",
            "FROM pg_catalog.pg_attrdef AS default_record",
            "pg_catalog.pg_get_expr(default_record.adbin, default_record.adrelid) IN (",
            "JOIN pg_catalog.pg_depend AS default_dependency",
            "default_dependency.refobjid = pg_catalog.to_regclass(",
            "relation_record.relname = 'signed_transactions_id_seq'",
            "FROM pg_catalog.pg_inherits AS inheritance_record",
            "COALESCE((SELECT compatible FROM not_null_constraint_compatibility), false)",
        ] {
            assert!(
                WALLET_SCHEMA_COMPATIBILITY_QUERY.contains(anchor),
                "missing schema anchor: {anchor}"
            );
        }
        assert!(!WALLET_SCHEMA_COMPATIBILITY_QUERY.contains("actual.column_default LIKE"));
        assert!(!WALLET_SCHEMA_COMPATIBILITY_QUERY
            .contains("\n        JOIN pg_catalog.pg_index AS constraint_index"));
        let normalized_query = WALLET_SCHEMA_COMPATIBILITY_QUERY
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            !normalized_query.contains("AND NOT EXISTS ( AND NOT EXISTS ("),
            "nested duplicate NOT EXISTS opener makes the compatibility query invalid"
        );
        let relation_compatibility = WALLET_SCHEMA_COMPATIBILITY_QUERY
            .split_once("relation_compatibility AS (")
            .expect("relation compatibility CTE must exist")
            .1
            .split_once("expected_primary_keys (table_name, key_columns) AS (")
            .expect("relation compatibility CTE must end before primary-key expectations")
            .0;
        assert_eq!(
            relation_compatibility.matches("AND NOT EXISTS (").count(),
            1,
            "relation compatibility must have exactly one inheritance NOT EXISTS guard"
        );
        assert_eq!(
            relation_compatibility
                .matches("FROM pg_catalog.pg_inherits AS inheritance_record")
                .count(),
            1,
            "relation compatibility must inspect inheritance exactly once"
        );
        let allowed_serial_defaults = [
            "nextval('signed_transactions_id_seq'::regclass)",
            "nextval('public.signed_transactions_id_seq'::regclass)",
        ];
        let double_nextval = "nextval('signed_transactions_id_seq'::regclass) + nextval('signed_transactions_id_seq'::regclass)";
        assert!(!allowed_serial_defaults.contains(&double_nextval));
    }

    #[derive(Default)]
    struct FakeVerifier {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let (wallet, audience, permissions) = match token {
                "frontend-owner" => (OWNER, FRONTEND_AUDIENCE, vec![]),
                "admin-owner" => (OWNER, ADMIN_AUDIENCE, vec![]),
                "admin-wallets-read" => {
                    (OWNER, ADMIN_AUDIENCE, vec![WALLETS_READ_PERMISSION.into()])
                }
                "admin-domain-wildcard" => (OWNER, ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "admin-global-wildcard" => (OWNER, ADMIN_AUDIENCE, vec!["*:*:*".into()]),
                "other-audience" => (OWNER, "epsx-other", vec![]),
                "malformed-wallet" => ("0xabc", FRONTEND_AUDIENCE, vec![]),
                _ => return Err(VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: wallet.into(),
                wallet_address: wallet.into(),
                audience: audience.into(),
                permissions,
            })
        }
    }

    #[derive(Default)]
    struct Downstream {
        hits: AtomicUsize,
        authorization_seen: AtomicUsize,
        spoofed_identity_seen: AtomicUsize,
        principal_seen: AtomicUsize,
    }

    fn app() -> (Router, Arc<Downstream>, Arc<FakeVerifier>) {
        let downstream = Arc::new(Downstream::default());
        let observed = downstream.clone();
        let router = Router::new().fallback(any(move |request: Request| {
            let observed = observed.clone();
            async move {
                observed.hits.fetch_add(1, Ordering::SeqCst);
                if request.headers().contains_key(header::AUTHORIZATION) {
                    observed.authorization_seen.fetch_add(1, Ordering::SeqCst);
                }
                if request.headers().contains_key("x-user-id")
                    || request.headers().contains_key("x-wallet-address")
                    || request.headers().contains_key("x-permissions")
                {
                    observed
                        .spoofed_identity_seen
                        .fetch_add(1, Ordering::SeqCst);
                }
                if request.extensions().get::<VerifiedPrincipal>().is_some() {
                    observed.principal_seen.fetch_add(1, Ordering::SeqCst);
                }
                StatusCode::OK
            }
        }));
        let verifier = Arc::new(FakeVerifier::default());
        (
            protect_router(router, verifier.clone()),
            downstream,
            verifier,
        )
    }

    fn request(method: Method, path: &str, bearer: Option<&str>) -> axum::http::Request<Body> {
        let mut builder = axum::http::Request::builder().method(method).uri(path);
        if let Some(bearer) = bearer {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {bearer}"));
        }
        builder.body(Body::empty()).unwrap()
    }

    async fn status(app: &Router, request: axum::http::Request<Body>) -> StatusCode {
        app.clone().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn health_and_message_verification_are_the_only_anonymous_surfaces() {
        let (app, downstream, verifier) = app();
        for method in [Method::GET, Method::HEAD] {
            let mut health = request(method, "/health", Some("frontend-owner"));
            health
                .headers_mut()
                .insert("x-user-id", "attacker".parse().unwrap());
            assert_eq!(status(&app, health).await, StatusCode::OK);
        }
        let mut verify = request(
            Method::POST,
            "/api/v1/wallet/verify-message",
            Some("frontend-owner"),
        );
        verify
            .headers_mut()
            .insert("x-wallet-address", "attacker".parse().unwrap());
        assert_eq!(status(&app, verify).await, StatusCode::OK);

        assert_eq!(downstream.hits.load(Ordering::SeqCst), 3);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn public_message_verification_rejects_an_oversized_json_body_before_handler() {
        let hits = Arc::new(AtomicUsize::new(0));
        let observed = hits.clone();
        let router = Router::new().route(
            "/api/v1/wallet/verify-message",
            axum::routing::post(move |Json(_): Json<serde_json::Value>| {
                let observed = observed.clone();
                async move {
                    observed.fetch_add(1, Ordering::SeqCst);
                    StatusCode::OK
                }
            }),
        );
        let app = protect_router(router, Arc::new(FakeVerifier::default()));
        let oversized = serde_json::json!({
            "message": "a".repeat(WALLET_JSON_BODY_LIMIT_BYTES),
            "signature": "0x00",
            "expected_address": OWNER,
        })
        .to_string();
        let req = axum::http::Request::builder()
            .method(Method::POST)
            .uri("/api/v1/wallet/verify-message")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(oversized))
            .unwrap();
        assert_eq!(status(&app, req).await, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn account_reads_require_an_exact_supported_audience_and_principal() {
        let routes = [
            "/api/v1/wallet/accounts",
            "/api/v1/wallet/accounts/0x1111111111111111111111111111111111111111",
        ];
        let (app, downstream, _) = app();
        for path in routes {
            assert_eq!(
                status(&app, request(Method::GET, path, None)).await,
                StatusCode::UNAUTHORIZED
            );
            assert_eq!(
                status(&app, request(Method::GET, path, Some("invalid"))).await,
                StatusCode::UNAUTHORIZED
            );
            for denied in ["other-audience", "malformed-wallet"] {
                assert_eq!(
                    status(&app, request(Method::GET, path, Some(denied))).await,
                    StatusCode::FORBIDDEN
                );
            }
            for allowed in ["frontend-owner", "admin-owner"] {
                assert_eq!(
                    status(&app, request(Method::GET, path, Some(allowed))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 4);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn admin_wallet_reads_accept_canonical_wildcards() {
        let (app, downstream, _) = app();
        let path = "/api/v1/admin/wallets";
        for bearer in [None, Some("invalid")] {
            assert_eq!(
                status(&app, request(Method::GET, path, bearer)).await,
                StatusCode::UNAUTHORIZED
            );
        }
        for bearer in [Some("admin-owner"), Some("other-audience")] {
            assert_eq!(
                status(&app, request(Method::GET, path, bearer)).await,
                StatusCode::FORBIDDEN
            );
        }
        for bearer in [
            "admin-wallets-read",
            "admin-domain-wildcard",
            "admin-global-wildcard",
        ] {
            assert_eq!(
                status(&app, request(Method::GET, path, Some(bearer))).await,
                StatusCode::OK
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 3);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn spoofable_headers_never_replace_the_verified_owner() {
        let (app, downstream, _) = app();
        let mut req = request(
            Method::GET,
            "/api/v1/wallet/accounts",
            Some("frontend-owner"),
        );
        req.headers_mut()
            .insert("x-wallet-address", "0xattacker".parse().unwrap());
        req.headers_mut()
            .insert("x-permissions", "*:*".parse().unwrap());
        assert_eq!(status(&app, req).await, StatusCode::OK);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 1);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn owner_is_canonical_and_cross_owner_or_invalid_claims_are_hidden() {
        let principal = VerifiedPrincipal {
            subject: OWNER.into(),
            wallet_address: "0x111111111111111111111111111111111111AaAa".into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: vec![],
        };
        assert_eq!(
            canonical_owner(&principal, None).unwrap(),
            "0x111111111111111111111111111111111111aaaa"
        );
        assert_eq!(
            canonical_owner(
                &principal,
                Some("0X111111111111111111111111111111111111AAAA")
            )
            .unwrap(),
            "0x111111111111111111111111111111111111aaaa"
        );
        assert_eq!(
            canonical_owner(
                &principal,
                Some("0x2222222222222222222222222222222222222222")
            ),
            Err(StatusCode::NOT_FOUND)
        );
        let invalid = VerifiedPrincipal {
            wallet_address: "0xabc".into(),
            ..principal
        };
        assert_eq!(canonical_owner(&invalid, None), Err(StatusCode::FORBIDDEN));
    }

    #[test]
    fn admin_resource_prefixes_are_strict_boundaries() {
        assert!(matches!(
            classify(&Method::GET, "/api/v1/admin/wallets"),
            AccessPolicy::AdminPermission(WALLETS_READ_PERMISSION)
        ));
        assert!(matches!(
            classify(&Method::POST, "/api/v1/admin/wallets/0xabc/disable"),
            AccessPolicy::AdminPermission(WALLETS_MANAGE_PERMISSION)
        ));
        assert!(matches!(
            classify(&Method::GET, "/api/v1/admin/credits"),
            AccessPolicy::AdminPermission(CREDITS_READ_PERMISSION)
        ));
        assert!(matches!(
            classify(&Method::POST, "/api/v1/admin/credits/0xabc/grant"),
            AccessPolicy::AdminPermission(CREDITS_MANAGE_PERMISSION)
        ));
        for (method, path) in [
            (Method::GET, "/api/v1/admin/walletsfoo"),
            (Method::POST, "/api/v1/admin/wallets/../disable"),
            (Method::POST, "/api/v1/admin/wallets/%2e%2e/disable"),
            (Method::POST, "/api/v1/admin/wallets/0xabc//disable"),
            (Method::GET, "/api/v1/admin/creditsfoo"),
            (Method::POST, "/api/v1/admin/credits/../grant"),
            (Method::POST, "/api/v1/admin/credits/%2e%2e/grant"),
        ] {
            assert_eq!(classify(&method, path), AccessPolicy::Blocked, "{path}");
        }
    }

    #[tokio::test]
    async fn unsafe_projection_and_custody_routes_fail_before_auth_or_handlers() {
        let routes = [
            (Method::POST, "/api/v1/wallet/accounts"),
            (
                Method::GET,
                "/api/v1/wallet/balance/56/0x1111111111111111111111111111111111111111",
            ),
            (Method::POST, "/api/v1/wallet/send"),
            (Method::POST, "/api/v1/wallet/sign-message"),
            (Method::POST, "/api/v1/wallet/estimate-gas"),
        ];
        let (app, downstream, verifier) = app();
        for (method, path) in routes {
            for bearer in [None, Some("invalid"), Some("frontend-owner")] {
                assert_eq!(
                    status(&app, request(method.clone(), path, bearer)).await,
                    StatusCode::NOT_FOUND,
                    "{method} {path}"
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unknown_methods_arities_encoded_and_reserved_paths_fail_before_auth() {
        let routes = [
            (Method::POST, "/health"),
            (Method::GET, "/health/"),
            (Method::HEAD, "/api/v1/wallet/verify-message"),
            (Method::POST, "/api/v1/wallet/accounts/owner"),
            (Method::DELETE, "/api/v1/wallet/accounts/owner"),
            (Method::GET, "/api/v1/wallet/accounts/accounts"),
            (Method::GET, "/api/v1/wallet/accounts/../send"),
            (Method::GET, "/api/v1/wallet/accounts/%2e%2e"),
            (Method::GET, "/api/v1/wallet/balance/56"),
            (Method::GET, "/api/v1/wallet/balance/56/address/extra"),
            (Method::GET, "/api/v1/wallet//accounts"),
            (Method::GET, "/api/v1/wallet/unknown"),
            (Method::GET, "/unknown"),
        ];
        let (app, downstream, verifier) = app();
        for (method, path) in routes {
            assert_eq!(
                status(&app, request(method.clone(), path, Some("frontend-owner"))).await,
                StatusCode::NOT_FOUND,
                "{method} {path}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn narrowed_runtime_mount_returns_404_for_wrong_method_and_unknown_path() {
        let verifier = Arc::new(FakeVerifier::default());
        let router = Router::new().route(
            "/api/v1/wallet/accounts",
            axum::routing::get(|| async { StatusCode::OK }),
        );
        let app = protect_router(router, verifier.clone());
        assert_eq!(
            status(
                &app,
                request(
                    Method::DELETE,
                    "/api/v1/wallet/accounts",
                    Some("frontend-owner")
                )
            )
            .await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status(
                &app,
                request(Method::GET, "/api/v1/wallet/nope", Some("frontend-owner"))
            )
            .await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn production_verifier_rejects_insecure_or_local_configuration() {
        assert!(matches!(
            build_auth_verifier(
                "https://identity.example.com",
                "http://identity.example.com/.well-known/jwks.json",
                true,
            ),
            Err(WalletConfigError::Auth(_))
        ));
        assert!(matches!(
            build_auth_verifier(
                "https://localhost:8443",
                "https://localhost:8443/.well-known/jwks.json",
                true,
            ),
            Err(WalletConfigError::Auth(_))
        ));
    }
}
