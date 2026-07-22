use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderName, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Json, Router,
};
use epsx_service_auth::{
    authenticate_headers, AccessTokenVerifier, JwksVerifier, JwksVerifierConfig, ADMIN_AUDIENCE,
};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

pub const INDEXER_MANAGE_PERMISSION: &str = "admin:indexer:manage";

pub const INDEXER_SCHEMA_COMPATIBILITY_QUERY: &str = r#"
WITH expected_relations(table_name) AS (
    VALUES ('blocks'), ('transactions'), ('token_transfers')
),
expected_columns(table_name, ordinal_position, column_name, data_type, udt_name, character_maximum_length, is_nullable, default_kind, datetime_precision) AS (
    VALUES
        ('blocks', 1, 'chain_id', 'character varying', 'varchar', 10, 'NO', 'none', NULL::integer),
        ('blocks', 2, 'number', 'bigint', 'int8', NULL::integer, 'NO', 'none', NULL::integer),
        ('blocks', 3, 'hash', 'character varying', 'varchar', 66, 'NO', 'none', NULL::integer),
        ('blocks', 4, 'parent_hash', 'character varying', 'varchar', 66, 'NO', 'none', NULL::integer),
        ('blocks', 5, 'timestamp', 'timestamp with time zone', 'timestamptz', NULL::integer, 'NO', 'none', 6),
        ('blocks', 6, 'miner', 'character varying', 'varchar', 42, 'YES', 'none', NULL::integer),
        ('blocks', 7, 'gas_used', 'bigint', 'int8', NULL::integer, 'NO', 'none', NULL::integer),
        ('blocks', 8, 'gas_limit', 'bigint', 'int8', NULL::integer, 'NO', 'none', NULL::integer),
        ('blocks', 9, 'tx_count', 'integer', 'int4', NULL::integer, 'NO', 'zero', NULL::integer),
        ('transactions', 1, 'chain_id', 'character varying', 'varchar', 10, 'NO', 'none', NULL::integer),
        ('transactions', 2, 'hash', 'character varying', 'varchar', 66, 'NO', 'none', NULL::integer),
        ('transactions', 3, 'from_address', 'character varying', 'varchar', 42, 'NO', 'none', NULL::integer),
        ('transactions', 4, 'to_address', 'character varying', 'varchar', 42, 'YES', 'none', NULL::integer),
        ('transactions', 5, 'value', 'character varying', 'varchar', 78, 'NO', 'none', NULL::integer),
        ('transactions', 6, 'block_number', 'bigint', 'int8', NULL::integer, 'NO', 'none', NULL::integer),
        ('transactions', 7, 'status', 'integer', 'int4', NULL::integer, 'YES', 'none', NULL::integer),
        ('transactions', 8, 'timestamp', 'timestamp with time zone', 'timestamptz', NULL::integer, 'NO', 'none', 6),
        ('transactions', 9, 'input_data', 'bytea', 'bytea', NULL::integer, 'NO', 'none', NULL::integer),
        ('token_transfers', 1, 'chain_id', 'character varying', 'varchar', 10, 'NO', 'none', NULL::integer),
        ('token_transfers', 2, 'tx_hash', 'character varying', 'varchar', 66, 'NO', 'none', NULL::integer),
        ('token_transfers', 3, 'log_index', 'integer', 'int4', NULL::integer, 'NO', 'none', NULL::integer),
        ('token_transfers', 4, 'token_address', 'character varying', 'varchar', 42, 'NO', 'none', NULL::integer),
        ('token_transfers', 5, 'from_address', 'character varying', 'varchar', 42, 'NO', 'none', NULL::integer),
        ('token_transfers', 6, 'to_address', 'character varying', 'varchar', 42, 'NO', 'none', NULL::integer),
        ('token_transfers', 7, 'value', 'character varying', 'varchar', 78, 'NO', 'none', NULL::integer),
        ('token_transfers', 8, 'block_number', 'bigint', 'int8', NULL::integer, 'NO', 'none', NULL::integer),
        ('token_transfers', 9, 'timestamp', 'timestamp with time zone', 'timestamptz', NULL::integer, 'NO', 'none', 6)
),
resolved_relations AS (
    SELECT e.table_name, to_regclass(format('public.%I', e.table_name)) AS relation_oid
    FROM expected_relations e
),
relation_shape AS (
    SELECT count(*) = 3
       AND bool_and(r.relation_oid IS NOT NULL)
       AND bool_and(c.relnamespace = 'public'::regnamespace)
       AND bool_and(c.relkind = 'r')
       AND bool_and(c.relpersistence = 'p')
       AND bool_and(NOT c.relispartition)
       AND bool_and(NOT c.relrowsecurity AND NOT c.relforcerowsecurity)
       AND bool_and(c.relreplident = 'd') AS ok
    FROM resolved_relations r
    LEFT JOIN pg_catalog.pg_class c ON c.oid = r.relation_oid
),
column_shape AS (
    SELECT count(*) = 27
       AND bool_and(c.column_name IS NOT NULL)
       AND bool_and(c.data_type = e.data_type)
       AND bool_and(c.udt_name = e.udt_name)
       AND bool_and(c.character_maximum_length IS NOT DISTINCT FROM e.character_maximum_length)
       AND bool_and(c.is_nullable = e.is_nullable)
       AND bool_and(c.datetime_precision IS NOT DISTINCT FROM e.datetime_precision)
       AND bool_and(COALESCE(
            CASE e.default_kind
                WHEN 'none' THEN c.column_default IS NULL
                WHEN 'zero' THEN c.column_default IN ('0', '0::integer')
                ELSE false
            END,
            false
       ))
       AND bool_and(c.is_identity = 'NO')
       AND bool_and(c.is_generated = 'NEVER') AS ok
    FROM expected_columns e
    LEFT JOIN information_schema.columns c
      ON c.table_schema = 'public'
     AND c.table_name = e.table_name
     AND c.ordinal_position = e.ordinal_position
     AND c.column_name = e.column_name
),
column_count AS (
    SELECT count(*) = 27 AS ok
    FROM information_schema.columns c
    WHERE c.table_schema = 'public'
      AND c.table_name IN (SELECT table_name FROM expected_relations)
),
attribute_shape AS (
    SELECT count(*) = 27
       AND bool_and(a.attidentity = '')
       AND bool_and(a.attgenerated = '')
       AND bool_and(a.attisdropped = false)
       AND bool_and(
            CASE WHEN t.typcollation = 0 THEN a.attcollation = 0
                 ELSE a.attcollation = t.typcollation END
       ) AS ok
    FROM expected_columns e
    JOIN resolved_relations r USING (table_name)
    JOIN pg_catalog.pg_attribute a
      ON a.attrelid = r.relation_oid
     AND a.attnum = e.ordinal_position
     AND a.attname = e.column_name
    JOIN pg_catalog.pg_type t ON t.oid = a.atttypid
),
expected_structural_constraints(constraint_name, table_name, constraint_type, key_columns, referenced_table, referenced_columns) AS (
    VALUES
      ('blocks_pkey', 'blocks', 'p', ARRAY['chain_id','number']::text[], NULL::text, NULL::text[]),
      ('blocks_chain_hash_key', 'blocks', 'u', ARRAY['chain_id','hash']::text[], NULL::text, NULL::text[]),
      ('transactions_pkey', 'transactions', 'p', ARRAY['chain_id','hash']::text[], NULL::text, NULL::text[]),
      ('transactions_chain_hash_block_key', 'transactions', 'u', ARRAY['chain_id','hash','block_number']::text[], NULL::text, NULL::text[]),
      ('transactions_block_fkey', 'transactions', 'f', ARRAY['chain_id','block_number']::text[], 'blocks', ARRAY['chain_id','number']::text[]),
      ('token_transfers_pkey', 'token_transfers', 'p', ARRAY['chain_id','tx_hash','log_index']::text[], NULL::text, NULL::text[]),
      ('token_transfers_transaction_fkey', 'token_transfers', 'f', ARRAY['chain_id','tx_hash','block_number']::text[], 'transactions', ARRAY['chain_id','hash','block_number']::text[])
),
actual_structural_constraints AS (
    SELECT con.conname AS constraint_name,
           rel.relname AS table_name,
           con.contype::text AS constraint_type,
           ARRAY(
             SELECT att.attname::text
             FROM unnest(con.conkey) WITH ORDINALITY AS key(attnum, ord)
             JOIN pg_catalog.pg_attribute att ON att.attrelid = con.conrelid AND att.attnum = key.attnum
             ORDER BY key.ord
           ) AS key_columns,
           ref.relname AS referenced_table,
           CASE WHEN con.contype = 'f' THEN ARRAY(
             SELECT att.attname::text
             FROM unnest(con.confkey) WITH ORDINALITY AS key(attnum, ord)
             JOIN pg_catalog.pg_attribute att ON att.attrelid = con.confrelid AND att.attnum = key.attnum
             ORDER BY key.ord
           ) ELSE NULL::text[] END AS referenced_columns,
           con.condeferrable, con.condeferred, con.convalidated,
           COALESCE((to_jsonb(con)->>'conenforced')::boolean, true) AS enforced,
           con.confupdtype, con.confdeltype, con.confmatchtype,
           con.conindid
    FROM pg_catalog.pg_constraint con
    JOIN pg_catalog.pg_class rel ON rel.oid = con.conrelid
    JOIN pg_catalog.pg_namespace ns ON ns.oid = rel.relnamespace AND ns.nspname = 'public'
    LEFT JOIN pg_catalog.pg_class ref ON ref.oid = con.confrelid
    WHERE rel.relname IN (SELECT table_name FROM expected_relations)
      AND con.contype IN ('p','u','f')
),
structural_constraint_shape AS (
    SELECT count(*) = 7
       AND bool_and(a.constraint_name IS NOT NULL)
       AND bool_and(a.constraint_type = e.constraint_type)
       AND bool_and(a.key_columns = e.key_columns)
       AND bool_and(a.referenced_table IS NOT DISTINCT FROM e.referenced_table)
       AND bool_and(a.referenced_columns IS NOT DISTINCT FROM e.referenced_columns)
       AND bool_and(NOT a.condeferrable AND NOT a.condeferred AND a.convalidated AND a.enforced)
       AND bool_and(CASE WHEN e.constraint_type = 'f'
                         THEN a.confupdtype = 'a' AND a.confdeltype = 'a' AND a.confmatchtype = 's'
                         ELSE a.conindid <> 0 END) AS ok
    FROM expected_structural_constraints e
    LEFT JOIN actual_structural_constraints a
      ON a.constraint_name = e.constraint_name AND a.table_name = e.table_name
),
structural_constraint_count AS (
    SELECT count(*) = 7 AS ok FROM actual_structural_constraints
),
expected_checks(constraint_name, table_name, expected_definition) AS (
    VALUES
      ('blocks_chain_id_check','blocks','check(chain_id~''^[1-9][0-9]{0,9}$'')'),
      ('blocks_number_check','blocks','check(number>=0)'),
      ('blocks_hash_check','blocks','check(hash~''^0x[0-9a-f]{64}$'')'),
      ('blocks_parent_hash_check','blocks','check(parent_hash~''^0x[0-9a-f]{64}$'')'),
      ('blocks_miner_check','blocks','check(minerisnullorminer~''^0x[0-9a-f]{40}$'')'),
      ('blocks_gas_used_check','blocks','check(gas_used>=0)'),
      ('blocks_gas_limit_check','blocks','check(gas_limit>=0)'),
      ('blocks_gas_bounds_check','blocks','check(gas_used<=gas_limit)'),
      ('blocks_tx_count_check','blocks','check(tx_count>=0)'),
      ('transactions_chain_id_check','transactions','check(chain_id~''^[1-9][0-9]{0,9}$'')'),
      ('transactions_hash_check','transactions','check(hash~''^0x[0-9a-f]{64}$'')'),
      ('transactions_from_address_check','transactions','check(from_address~''^0x[0-9a-f]{40}$'')'),
      ('transactions_to_address_check','transactions','check(to_addressisnullorto_address~''^0x[0-9a-f]{40}$'')'),
      ('transactions_value_check','transactions','check(casewhenvalue~''^(0|[1-9][0-9]{0,77})$''thenvalue<=''115792089237316195423570985008687907853269984665640564039457584007913129639935''elsefalseend)'),
      ('transactions_block_number_check','transactions','check(block_number>=0)'),
      ('transactions_status_check','transactions','check(statusisnullor(status=any(array[0,1])))'),
      ('token_transfers_chain_id_check','token_transfers','check(chain_id~''^[1-9][0-9]{0,9}$'')'),
      ('token_transfers_tx_hash_check','token_transfers','check(tx_hash~''^0x[0-9a-f]{64}$'')'),
      ('token_transfers_log_index_check','token_transfers','check(log_index>=0)'),
      ('token_transfers_token_address_check','token_transfers','check(token_address~''^0x[0-9a-f]{40}$'')'),
      ('token_transfers_from_address_check','token_transfers','check(from_address~''^0x[0-9a-f]{40}$'')'),
      ('token_transfers_to_address_check','token_transfers','check(to_address~''^0x[0-9a-f]{40}$'')'),
      ('token_transfers_value_check','token_transfers','check(casewhenvalue~''^(0|[1-9][0-9]{0,77})$''thenvalue<=''115792089237316195423570985008687907853269984665640564039457584007913129639935''elsefalseend)'),
      ('token_transfers_block_number_check','token_transfers','check(block_number>=0)')
),
actual_checks AS (
    SELECT con.conname AS constraint_name, rel.relname AS table_name,
           regexp_replace(
               regexp_replace(
                   regexp_replace(
                       lower(pg_catalog.pg_get_constraintdef(con.oid, true)),
                       '[[:space:]]', '', 'g'
                   ),
                   '\(([a-z_][a-z0-9_]*)\)::(text|numeric)', '\1::\2', 'g'
               ),
               '::(text|numeric)', '', 'g'
           ) AS normalized_definition,
           con.condeferrable, con.condeferred, con.convalidated,
           COALESCE((to_jsonb(con)->>'conenforced')::boolean, true) AS enforced
    FROM pg_catalog.pg_constraint con
    JOIN pg_catalog.pg_class rel ON rel.oid = con.conrelid
    JOIN pg_catalog.pg_namespace ns ON ns.oid = rel.relnamespace AND ns.nspname = 'public'
    WHERE rel.relname IN (SELECT table_name FROM expected_relations)
      AND con.contype = 'c'
),
check_shape AS (
    SELECT count(*) = 24
       AND bool_and(a.constraint_name IS NOT NULL)
       AND bool_and(a.normalized_definition = e.expected_definition)
       AND bool_and(NOT a.condeferrable AND NOT a.condeferred AND a.convalidated AND a.enforced) AS ok
    FROM expected_checks e
    LEFT JOIN actual_checks a
      ON a.constraint_name = e.constraint_name AND a.table_name = e.table_name
),
check_count AS (
    SELECT count(*) = 24 AS ok FROM actual_checks
),
foreign_key_boundary AS (
    SELECT count(*) = 2 AS ok
    FROM pg_catalog.pg_constraint con
    WHERE con.contype = 'f'
      AND (con.conrelid IN (SELECT relation_oid FROM resolved_relations)
           OR con.confrelid IN (SELECT relation_oid FROM resolved_relations))
),
expected_indexes(index_name, table_name, unique_index, primary_index, expected_definition) AS (
    VALUES
      ('blocks_pkey','blocks',true,true,'USING btree (chain_id, number)'),
      ('blocks_chain_hash_key','blocks',true,false,'USING btree (chain_id, hash)'),
      ('idx_blocks_timestamp','blocks',false,false,'USING btree (chain_id, "timestamp" DESC, number DESC)'),
      ('transactions_pkey','transactions',true,true,'USING btree (chain_id, hash)'),
      ('transactions_chain_hash_block_key','transactions',true,false,'USING btree (chain_id, hash, block_number)'),
      ('idx_transactions_block','transactions',false,false,'USING btree (chain_id, block_number DESC, hash DESC)'),
      ('token_transfers_pkey','token_transfers',true,true,'USING btree (chain_id, tx_hash, log_index)'),
      ('idx_transfers_token','token_transfers',false,false,'USING btree (chain_id, token_address, block_number DESC, tx_hash DESC, log_index DESC)'),
      ('idx_transfers_from','token_transfers',false,false,'USING btree (chain_id, from_address, block_number DESC, tx_hash DESC, log_index DESC)'),
      ('idx_transfers_to','token_transfers',false,false,'USING btree (chain_id, to_address, block_number DESC, tx_hash DESC, log_index DESC)')
),
actual_indexes AS (
    SELECT idx.relname AS index_name, rel.relname AS table_name,
           i.indisunique AS unique_index, i.indisprimary AS primary_index,
           i.indisvalid, i.indisready, i.indislive, i.indimmediate,
           i.indnkeyatts, i.indnatts, i.indexprs, i.indpred,
           am.amname,
           pg_catalog.pg_get_indexdef(i.indexrelid) AS definition,
           i.indexrelid, i.indrelid, i.indkey, i.indcollation, i.indclass
    FROM pg_catalog.pg_index i
    JOIN pg_catalog.pg_class idx ON idx.oid = i.indexrelid
    JOIN pg_catalog.pg_class rel ON rel.oid = i.indrelid
    JOIN pg_catalog.pg_namespace ns ON ns.oid = rel.relnamespace AND ns.nspname = 'public'
    JOIN pg_catalog.pg_am am ON am.oid = idx.relam
    WHERE rel.relname IN (SELECT table_name FROM expected_relations)
),
index_shape AS (
    SELECT count(*) = 10
       AND bool_and(a.index_name IS NOT NULL)
       AND bool_and(a.unique_index = e.unique_index AND a.primary_index = e.primary_index)
       AND bool_and(a.indisvalid AND a.indisready AND a.indislive AND a.indimmediate)
       AND bool_and(a.indnkeyatts = a.indnatts)
       AND bool_and(a.indexprs IS NULL AND a.indpred IS NULL)
       AND bool_and(a.amname = 'btree')
       AND bool_and(substring(regexp_replace(a.definition, '\s+', ' ', 'g') from 'USING btree.*$') = e.expected_definition) AS ok
    FROM expected_indexes e
    LEFT JOIN actual_indexes a
      ON a.index_name = e.index_name AND a.table_name = e.table_name
),
index_count AS (
    SELECT count(*) = 10 AS ok FROM actual_indexes
),
index_catalog_shape AS (
    SELECT NOT EXISTS (
        SELECT 1
        FROM actual_indexes i
        CROSS JOIN LATERAL unnest(i.indclass) AS class_oid
        JOIN pg_catalog.pg_opclass opc ON opc.oid = class_oid
        JOIN pg_catalog.pg_namespace ns ON ns.oid = opc.opcnamespace
        WHERE ns.nspname <> 'pg_catalog'
    ) AND NOT EXISTS (
        SELECT 1
        FROM actual_indexes i
        CROSS JOIN LATERAL unnest(i.indkey, i.indcollation) AS keys(attnum, collation_oid)
        LEFT JOIN pg_catalog.pg_attribute a
          ON a.attrelid = i.indrelid AND a.attnum = keys.attnum
        WHERE keys.attnum <= 0 OR keys.collation_oid <> a.attcollation
    ) AS ok
),
inheritance_shape AS (
    SELECT NOT EXISTS (
        SELECT 1 FROM pg_catalog.pg_inherits inh
        WHERE inh.inhrelid IN (SELECT relation_oid FROM resolved_relations)
           OR inh.inhparent IN (SELECT relation_oid FROM resolved_relations)
    ) AS ok
)
SELECT COALESCE(
    (SELECT ok FROM relation_shape)
    AND (SELECT ok FROM column_shape)
    AND (SELECT ok FROM column_count)
    AND (SELECT ok FROM attribute_shape)
    AND (SELECT ok FROM structural_constraint_shape)
    AND (SELECT ok FROM structural_constraint_count)
    AND (SELECT ok FROM check_shape)
    AND (SELECT ok FROM check_count)
    AND (SELECT ok FROM foreign_key_boundary)
    AND (SELECT ok FROM index_shape)
    AND (SELECT ok FROM index_count)
    AND (SELECT ok FROM index_catalog_shape)
    AND (SELECT ok FROM inheritance_shape),
    false
)
"#;

#[derive(Debug, Error)]
pub enum IndexerSchemaError {
    #[error("indexer schema compatibility query failed")]
    Database(#[from] sqlx::Error),
    #[error("indexer schema is absent or incompatible")]
    Incompatible,
}

pub async fn verify_schema_compatibility(db: &sqlx::PgPool) -> Result<(), IndexerSchemaError> {
    let compatible = sqlx::query_scalar::<_, bool>(INDEXER_SCHEMA_COMPATIBILITY_QUERY)
        .fetch_one(db)
        .await?;
    if !compatible {
        return Err(IndexerSchemaError::Incompatible);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum IndexerConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, IndexerConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-indexer/1")
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
    router.layer(middleware::from_fn_with_state(
        AuthState { verifier },
        authorize_request,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessPolicy {
    Public,
    UnsafeProjection,
    UnsafeOperatorMutation,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if !normalized_path(path) {
        return AccessPolicy::Blocked;
    }
    let Some(tail) = path.strip_prefix("/api/v1/indexer/") else {
        return AccessPolicy::Blocked;
    };
    let segments: Vec<_> = tail.split('/').collect();

    match (method, segments.as_slice()) {
        (&Method::GET, ["status", chain]) if safe_dynamic_segment(chain) => {
            AccessPolicy::UnsafeProjection
        }
        (&Method::GET, ["block", chain, number])
            if safe_dynamic_segment(chain) && safe_dynamic_segment(number) =>
        {
            AccessPolicy::UnsafeProjection
        }
        (&Method::GET, ["tx", chain, hash])
            if safe_dynamic_segment(chain) && safe_dynamic_segment(hash) =>
        {
            AccessPolicy::UnsafeProjection
        }
        (&Method::GET, ["transfers", chain, address])
            if safe_dynamic_segment(chain) && safe_dynamic_segment(address) =>
        {
            AccessPolicy::UnsafeProjection
        }
        (&Method::POST, ["sync"]) => AccessPolicy::UnsafeOperatorMutation,
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
        && !matches!(
            segment,
            "." | ".." | "health" | "status" | "block" | "tx" | "transfers" | "sync"
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
        AccessPolicy::UnsafeProjection => return StatusCode::NOT_FOUND.into_response(),
        AccessPolicy::UnsafeOperatorMutation => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(INDEXER_MANAGE_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }

            // A verified operator still cannot dispatch sync. A12 has not
            // supplied canonical ingestion, a durable lease, finality, or
            // replay rules, and this service contains no ingestion worker.
            return StatusCode::NOT_FOUND.into_response();
        }
        AccessPolicy::Blocked => return StatusCode::NOT_FOUND.into_response(),
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
    use axum::{
        body::Body,
        routing::{any, post},
    };
    use epsx_service_auth::{VerifiedPrincipal, VerifyError, FRONTEND_AUDIENCE};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    #[derive(Default)]
    struct FakeVerifier {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let (audience, permissions) = match token {
                "admin-none" => (ADMIN_AUDIENCE, vec![]),
                "admin-manage" => (ADMIN_AUDIENCE, vec![INDEXER_MANAGE_PERMISSION.into()]),
                "admin-resource-wildcard" => (ADMIN_AUDIENCE, vec!["admin:indexer:*".into()]),
                "admin-domain-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "admin-invalid-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:manage".into()]),
                "frontend-manage" => (FRONTEND_AUDIENCE, vec![INDEXER_MANAGE_PERMISSION.into()]),
                "other-audience" => ("epsx-other", vec![INDEXER_MANAGE_PERMISSION.into()]),
                _ => return Err(VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: "0xabc".into(),
                wallet_address: "0xabc".into(),
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
    async fn health_is_the_only_anonymous_surface_and_strips_credentials() {
        let (app, downstream, verifier) = app();
        for method in [Method::GET, Method::HEAD] {
            let mut health = request(method, "/health", Some("admin-manage"));
            health
                .headers_mut()
                .insert("x-user-id", "attacker".parse().unwrap());
            assert_eq!(status(&app, health).await, StatusCode::OK);
        }
        assert_eq!(
            status(&app, request(Method::POST, "/health", None)).await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 2);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn current_read_projections_fail_closed_before_auth_or_handlers() {
        let (app, downstream, verifier) = app();
        for path in [
            "/api/v1/indexer/status/56",
            "/api/v1/indexer/block/56/100",
            "/api/v1/indexer/tx/56/0xabc",
            "/api/v1/indexer/transfers/56/0xabc",
        ] {
            assert_eq!(
                status(&app, request(Method::GET, path, Some("admin-manage"))).await,
                StatusCode::NOT_FOUND,
                "{path}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn sync_requires_a_verified_exact_admin_audience_and_permission() {
        let (app, downstream, _) = app();
        assert_eq!(
            status(&app, request(Method::POST, "/api/v1/indexer/sync", None)).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(
                &app,
                request(Method::POST, "/api/v1/indexer/sync", Some("invalid"))
            )
            .await,
            StatusCode::UNAUTHORIZED
        );
        for bearer in [
            "admin-none",
            "frontend-manage",
            "other-audience",
            "admin-invalid-wildcard",
        ] {
            assert_eq!(
                status(
                    &app,
                    request(Method::POST, "/api/v1/indexer/sync", Some(bearer))
                )
                .await,
                StatusCode::FORBIDDEN,
                "{bearer}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn authorized_sync_still_fails_closed_without_an_ingestion_worker() {
        let (app, downstream, _) = app();
        for bearer in [
            "admin-manage",
            "admin-resource-wildcard",
            "admin-domain-wildcard",
        ] {
            assert_eq!(
                status(
                    &app,
                    request(Method::POST, "/api/v1/indexer/sync", Some(bearer))
                )
                .await,
                StatusCode::NOT_FOUND,
                "{bearer}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn spoofable_headers_never_reach_health_or_sync_handlers() {
        let (app, downstream, _) = app();
        let mut sync = request(Method::POST, "/api/v1/indexer/sync", Some("admin-manage"));
        sync.headers_mut()
            .insert("x-wallet-address", "0xattacker".parse().unwrap());
        sync.headers_mut()
            .insert("x-permissions", "admin:*:*".parse().unwrap());
        assert_eq!(status(&app, sync).await, StatusCode::NOT_FOUND);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unknown_methods_and_path_arities_fail_before_auth_and_handlers() {
        let (app, downstream, verifier) = app();
        for (method, path) in [
            (Method::GET, "/api/v1/indexer/sync"),
            (Method::PUT, "/api/v1/indexer/sync"),
            (Method::POST, "/api/v1/indexer/status/56"),
            (Method::GET, "/api/v1/indexer/status"),
            (Method::GET, "/api/v1/indexer/status/56/extra"),
            (Method::GET, "/api/v1/indexer/block/56"),
            (Method::GET, "/api/v1/indexer/tx/56/hash/extra"),
            (Method::GET, "/api/v1/indexer/unknown"),
        ] {
            assert_eq!(
                status(&app, request(method, path, Some("admin-manage"))).await,
                StatusCode::NOT_FOUND,
                "{path}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn narrowed_runtime_mount_returns_404_instead_of_method_not_allowed() {
        let verifier = Arc::new(FakeVerifier::default());
        let router = Router::new().route("/api/v1/indexer/sync", post(|| async { StatusCode::OK }));
        let app = protect_router(router, verifier.clone());

        assert_eq!(
            status(
                &app,
                request(Method::GET, "/api/v1/indexer/sync", Some("admin-manage"))
            )
            .await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status(
                &app,
                request(Method::GET, "/api/v1/indexer/unknown", Some("admin-manage"))
            )
            .await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn encoded_ambiguous_and_reserved_paths_are_structurally_blocked() {
        for path in [
            "/api/v1/indexer/status/%2e%2e",
            "/api/v1/indexer/status/..",
            "/api/v1/indexer/status/sync",
            "/api/v1/indexer/block/56/status",
            "/api//v1/indexer/status/56",
            "/api/v1/indexer/status/56/",
            "/api/v1/indexer/tx/56\\hash",
        ] {
            assert_eq!(
                classify(&Method::GET, path),
                AccessPolicy::Blocked,
                "{path}"
            );
        }
    }

    #[test]
    fn the_locked_exact_policy_table_is_conservative() {
        assert_eq!(classify(&Method::GET, "/health"), AccessPolicy::Public);
        assert_eq!(
            classify(&Method::GET, "/api/v1/indexer/status/56"),
            AccessPolicy::UnsafeProjection
        );
        assert_eq!(
            classify(&Method::GET, "/api/v1/indexer/block/56/100"),
            AccessPolicy::UnsafeProjection
        );
        assert_eq!(
            classify(&Method::POST, "/api/v1/indexer/sync"),
            AccessPolicy::UnsafeOperatorMutation
        );
        assert_eq!(
            classify(&Method::GET, "/api/v1/indexer/sync"),
            AccessPolicy::Blocked
        );
    }

    #[test]
    fn schema_probe_is_read_only_and_pins_the_exact_catalog_boundary() {
        let query = INDEXER_SCHEMA_COMPATIBILITY_QUERY;
        assert!(query.trim_start().starts_with("WITH expected_relations"));
        for forbidden in [
            "INSERT ",
            "UPDATE ",
            "DELETE ",
            "CREATE ",
            "ALTER ",
            "DROP ",
            "TRUNCATE ",
            "GRANT ",
            "REVOKE ",
        ] {
            assert!(
                !query.to_ascii_uppercase().contains(forbidden),
                "{forbidden}"
            );
        }
        for anchor in [
            "to_regclass(format('public.%I', e.table_name))",
            "count(*) = 27",
            "count(*) = 24",
            "count(*) = 10",
            "transactions_pkey",
            "ARRAY['chain_id','hash']::text[]",
            "foreign_key_boundary",
            "pg_catalog.pg_inherits",
            "NOT c.relrowsecurity AND NOT c.relforcerowsecurity",
            "ns.nspname <> 'pg_catalog'",
            "keys.collation_oid <> a.attcollation",
            "a.indexprs IS NULL AND a.indpred IS NULL",
            "COALESCE(",
        ] {
            assert!(query.contains(anchor), "missing {anchor}");
        }
    }

    #[test]
    fn schema_probe_does_not_accept_the_legacy_global_transaction_key() {
        assert!(INDEXER_SCHEMA_COMPATIBILITY_QUERY.contains(
            "('transactions_pkey', 'transactions', 'p', ARRAY['chain_id','hash']::text[]"
        ));
        assert!(!INDEXER_SCHEMA_COMPATIBILITY_QUERY
            .contains("('transactions_pkey', 'transactions', 'p', ARRAY['hash']::text[]"));
    }

    #[test]
    fn schema_probe_rejects_weakened_checks_and_pins_the_u256_ceiling() {
        let query = INDEXER_SCHEMA_COMPATIBILITY_QUERY;
        const U256_MAX: &str =
            "115792089237316195423570985008687907853269984665640564039457584007913129639935";

        assert!(query.contains("a.normalized_definition = e.expected_definition"));
        assert_eq!(query.matches("SELECT att.attname::text").count(), 2);
        assert!(!query.contains("strpos(a.definition"));
        assert!(query.contains("('blocks_number_check','blocks','check(number>=0)')"));
        assert!(!query.contains("check((number>=0)ortrue)"));
        assert!(query.contains("check(statusisnullor(status=any(array[0,1])))"));
        assert!(query.contains("USING btree (chain_id, \"timestamp\" DESC, number DESC)"));
        assert_eq!(query.matches(U256_MAX).count(), 2);
        assert_eq!(query.matches("check(casewhenvalue~").count(), 2);

        let exact = "check(number>=0)";
        let weakened = "check((number>=0)ortrue)";
        assert_ne!(exact, weakened);
    }
}
