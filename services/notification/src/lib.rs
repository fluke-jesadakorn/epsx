use axum::{
    extract::{Request, State},
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

pub const NOTIFICATIONS_MANAGE_PERMISSION: &str = "admin:notifications:manage";

const NOTIFICATION_SCHEMA_COMPATIBILITY_QUERY: &str = r#"
WITH expected_columns (
    table_name,
    column_name,
    ordinal_position,
    data_type,
    udt_name,
    is_nullable,
    character_maximum_length,
    datetime_precision,
    default_kind
) AS (
    VALUES
        ('templates', 'id', 1, 'character varying', 'varchar', 'NO', 66::bigint, NULL::bigint, 'none'),
        ('templates', 'name', 2, 'character varying', 'varchar', 'NO', 100::bigint, NULL::bigint, 'none'),
        ('templates', 'channel', 3, 'character varying', 'varchar', 'NO', 20::bigint, NULL::bigint, 'none'),
        ('templates', 'subject', 4, 'text', 'text', 'YES', NULL::bigint, NULL::bigint, 'none'),
        ('templates', 'body', 5, 'text', 'text', 'NO', NULL::bigint, NULL::bigint, 'none'),
        ('templates', 'variables', 6, 'jsonb', 'jsonb', 'NO', NULL::bigint, NULL::bigint, 'empty_object'),
        ('templates', 'active', 7, 'boolean', 'bool', 'NO', NULL::bigint, NULL::bigint, 'true'),
        ('templates', 'created_at', 8, 'timestamp with time zone', 'timestamptz', 'NO', NULL::bigint, 6::bigint, 'now'),
        ('templates', 'updated_at', 9, 'timestamp with time zone', 'timestamptz', 'NO', NULL::bigint, 6::bigint, 'now'),
        ('notifications', 'id', 1, 'character varying', 'varchar', 'NO', 66::bigint, NULL::bigint, 'none'),
        ('notifications', 'user_id', 2, 'character varying', 'varchar', 'YES', 66::bigint, NULL::bigint, 'none'),
        ('notifications', 'channel', 3, 'character varying', 'varchar', 'NO', 20::bigint, NULL::bigint, 'none'),
        ('notifications', 'recipient', 4, 'character varying', 'varchar', 'NO', 255::bigint, NULL::bigint, 'none'),
        ('notifications', 'template_id', 5, 'character varying', 'varchar', 'YES', 66::bigint, NULL::bigint, 'none'),
        ('notifications', 'subject', 6, 'text', 'text', 'YES', NULL::bigint, NULL::bigint, 'none'),
        ('notifications', 'body', 7, 'text', 'text', 'NO', NULL::bigint, NULL::bigint, 'none'),
        ('notifications', 'data', 8, 'jsonb', 'jsonb', 'YES', NULL::bigint, NULL::bigint, 'none'),
        ('notifications', 'status', 9, 'character varying', 'varchar', 'NO', 20::bigint, NULL::bigint, 'pending'),
        ('notifications', 'error', 10, 'text', 'text', 'YES', NULL::bigint, NULL::bigint, 'none'),
        ('notifications', 'sent_at', 11, 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 6::bigint, 'none'),
        ('notifications', 'created_at', 12, 'timestamp with time zone', 'timestamptz', 'NO', NULL::bigint, 6::bigint, 'now'),
        ('notifications', 'read_at', 13, 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 6::bigint, 'none'),
        ('notifications', 'title', 14, 'text', 'text', 'YES', NULL::bigint, NULL::bigint, 'none'),
        ('notifications', 'notification_type', 15, 'character varying', 'varchar', 'YES', 50::bigint, NULL::bigint, 'none'),
        ('notifications', 'priority', 16, 'character varying', 'varchar', 'YES', 20::bigint, NULL::bigint, 'none'),
        ('notifications', 'action_url', 17, 'text', 'text', 'YES', NULL::bigint, NULL::bigint, 'none')
),
expected_not_null (table_name, column_name) AS (
    VALUES
        ('templates', 'id'),
        ('templates', 'name'),
        ('templates', 'channel'),
        ('templates', 'body'),
        ('templates', 'variables'),
        ('templates', 'active'),
        ('templates', 'created_at'),
        ('templates', 'updated_at'),
        ('notifications', 'id'),
        ('notifications', 'channel'),
        ('notifications', 'recipient'),
        ('notifications', 'body'),
        ('notifications', 'status'),
        ('notifications', 'created_at')
),
expected_key_constraints (table_name, constraint_name, constraint_type, column_name) AS (
    VALUES
        ('templates', 'templates_pkey', 'p', 'id'),
        ('templates', 'templates_name_key', 'u', 'name'),
        ('notifications', 'notifications_pkey', 'p', 'id')
),
expected_indexes (
    table_name,
    index_name,
    index_kind,
    first_column,
    second_column,
    first_option,
    second_option
) AS (
    VALUES
        ('templates', 'templates_pkey', 'p', 'id', NULL::text, 0, NULL::integer),
        ('templates', 'templates_name_key', 'u', 'name', NULL::text, 0, NULL::integer),
        ('notifications', 'notifications_pkey', 'p', 'id', NULL::text, 0, NULL::integer),
        ('notifications', 'idx_notif_user', 'i', 'user_id', 'created_at', 0, 3),
        ('notifications', 'idx_notif_status', 'i', 'status', NULL::text, 0, NULL::integer)
),
column_compatibility AS (
    SELECT COALESCE(bool_and(
        actual.column_name IS NOT NULL
        AND actual.ordinal_position = expected.ordinal_position
        AND actual.data_type = expected.data_type
        AND actual.udt_schema = 'pg_catalog'
        AND actual.udt_name = expected.udt_name
        AND actual.is_nullable = expected.is_nullable
        AND actual.character_maximum_length IS NOT DISTINCT FROM expected.character_maximum_length
        AND actual.datetime_precision IS NOT DISTINCT FROM expected.datetime_precision
        AND actual.collation_name IS NULL
        AND actual.is_identity = 'NO'
        AND actual.is_generated = 'NEVER'
        AND COALESCE(
            CASE expected.default_kind
                WHEN 'empty_object' THEN actual.column_default = '''{}''::jsonb'
                WHEN 'true' THEN actual.column_default = 'true'
                WHEN 'pending' THEN actual.column_default IN (
                    '''pending''::character varying',
                    '''pending''::text',
                    '''pending'''
                )
                WHEN 'now' THEN actual.column_default IN ('now()', 'CURRENT_TIMESTAMP')
                ELSE actual.column_default IS NULL
            END,
            false
        )
    ), false) AS compatible
    FROM expected_columns AS expected
    LEFT JOIN information_schema.columns AS actual
      ON actual.table_schema = 'public'
     AND actual.table_name = expected.table_name
     AND actual.column_name = expected.column_name
),
column_inventory_compatibility AS (
    SELECT COUNT(*) = 26
       AND COUNT(*) FILTER (WHERE table_name = 'templates') = 9
       AND COUNT(*) FILTER (WHERE table_name = 'notifications') = 17 AS compatible
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name IN ('templates', 'notifications')
),
relation_compatibility AS (
    SELECT COUNT(*) = 2
       AND COALESCE(bool_and(
            table_record.relkind = 'r'
            AND table_record.relpersistence = 'p'
            AND table_record.relreplident = 'd'
            AND NOT table_record.relrowsecurity
            AND NOT table_record.relforcerowsecurity
            AND NOT table_record.relispartition
       ), false) AS compatible
    FROM pg_catalog.pg_class AS table_record
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('templates', 'notifications')
),
inheritance_compatibility AS (
    SELECT COUNT(*) = 0 AS compatible
    FROM pg_catalog.pg_inherits AS inheritance_record
    JOIN pg_catalog.pg_class AS child_table
      ON child_table.oid = inheritance_record.inhrelid
    JOIN pg_catalog.pg_namespace AS child_namespace
      ON child_namespace.oid = child_table.relnamespace
    JOIN pg_catalog.pg_class AS parent_table
      ON parent_table.oid = inheritance_record.inhparent
    JOIN pg_catalog.pg_namespace AS parent_namespace
      ON parent_namespace.oid = parent_table.relnamespace
    WHERE (child_namespace.nspname = 'public' AND child_table.relname IN ('templates', 'notifications'))
       OR (parent_namespace.nspname = 'public' AND parent_table.relname IN ('templates', 'notifications'))
),
policy_compatibility AS (
    SELECT COUNT(*) = 0 AS compatible
    FROM pg_catalog.pg_policy AS policy_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = policy_record.polrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('templates', 'notifications')
),
key_constraint_compatibility AS (
    SELECT COUNT(*) = 3
       AND COALESCE(bool_and(
            expected.table_name IS NOT NULL
            AND constraint_record.conname = expected.constraint_name
            AND constraint_record.contype::text = expected.constraint_type
            AND cardinality(constraint_record.conkey) = 1
            AND attribute_record.attname = expected.column_name
            AND constraint_record.convalidated
            AND NOT constraint_record.condeferrable
            AND NOT constraint_record.condeferred
            AND constraint_record.conparentid = 0
            AND constraint_record.coninhcount = 0
            AND constraint_record.conislocal
            AND constraint_record.connoinherit
            AND (
                NOT (to_jsonb(constraint_record) ? 'conperiod')
                OR COALESCE((to_jsonb(constraint_record) ->> 'conperiod')::boolean, false) = false
            )
            AND (
                NOT (to_jsonb(constraint_record) ? 'conenforced')
                OR COALESCE((to_jsonb(constraint_record) ->> 'conenforced')::boolean, false)
            )
            AND index_record.indisunique
            AND index_record.indisprimary = (constraint_record.contype = 'p')
            AND index_record.indisvalid
            AND index_record.indisready
            AND index_record.indimmediate
            AND index_record.indnkeyatts = 1
            AND index_record.indnatts = 1
            AND index_record.indpred IS NULL
            AND index_record.indexprs IS NULL
       ), false) AS compatible
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    LEFT JOIN pg_catalog.pg_attribute AS attribute_record
      ON attribute_record.attrelid = table_record.oid
     AND attribute_record.attnum = constraint_record.conkey[1]
     AND NOT attribute_record.attisdropped
    LEFT JOIN expected_key_constraints AS expected
      ON expected.table_name = table_record.relname
     AND expected.constraint_type = constraint_record.contype::text
     AND expected.column_name = attribute_record.attname
    LEFT JOIN pg_catalog.pg_index AS index_record
      ON index_record.indexrelid = constraint_record.conindid
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('templates', 'notifications')
      AND constraint_record.contype IN ('p', 'u')
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
              AND table_record.relname IN ('templates', 'notifications')
              AND constraint_record.contype = 'n'
        ) AS exposed
),
scoped_not_null_constraints AS (
    SELECT
        constraint_record.*,
        table_record.relname::text AS table_name,
        attribute_record.attname::text AS column_name
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    LEFT JOIN pg_catalog.pg_attribute AS attribute_record
      ON attribute_record.attrelid = table_record.oid
     AND attribute_record.attnum = constraint_record.conkey[1]
     AND NOT attribute_record.attisdropped
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('templates', 'notifications')
      AND constraint_record.contype = 'n'
),
not_null_constraint_compatibility AS (
    SELECT
        (
            NOT exposure.exposed
            AND COUNT(constraint_record.oid) = 0
        )
        OR (
            exposure.exposed
            AND COUNT(constraint_record.oid) = 14
            AND COUNT(DISTINCT (constraint_record.table_name, constraint_record.column_name)) = 14
            AND COALESCE(bool_and(
                expected.table_name IS NOT NULL
                AND cardinality(constraint_record.conkey) = 1
                AND constraint_record.convalidated
                AND NOT constraint_record.condeferrable
                AND NOT constraint_record.condeferred
                AND constraint_record.conparentid = 0
                AND constraint_record.coninhcount = 0
                AND constraint_record.conislocal
                AND NOT constraint_record.connoinherit
                AND COALESCE(
                    (to_jsonb(constraint_record) ->> 'conenforced')::boolean,
                    false
                )
            ), false)
        ) AS compatible
    FROM not_null_catalog_exposure AS exposure
    LEFT JOIN scoped_not_null_constraints AS constraint_record
      ON true
    LEFT JOIN expected_not_null AS expected
      ON expected.table_name = constraint_record.table_name
     AND expected.column_name = constraint_record.column_name
    GROUP BY exposure.exposed
),
foreign_key_compatibility AS (
    SELECT COUNT(*) = 0 AS compatible
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS source_table
      ON source_table.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS source_namespace
      ON source_namespace.oid = source_table.relnamespace
    JOIN pg_catalog.pg_class AS target_table
      ON target_table.oid = constraint_record.confrelid
    JOIN pg_catalog.pg_namespace AS target_namespace
      ON target_namespace.oid = target_table.relnamespace
    WHERE constraint_record.contype = 'f'
      AND (
          (source_namespace.nspname = 'public' AND source_table.relname IN ('templates', 'notifications'))
          OR (target_namespace.nspname = 'public' AND target_table.relname IN ('templates', 'notifications'))
      )
),
check_constraint_compatibility AS (
    SELECT COUNT(*) = 0 AS compatible
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('templates', 'notifications')
      AND constraint_record.contype = 'c'
),
other_constraint_compatibility AS (
    SELECT COUNT(*) = 0 AS compatible
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('templates', 'notifications')
      AND constraint_record.contype NOT IN ('p', 'u', 'n', 'f', 'c')
),
index_inventory_compatibility AS (
    SELECT COUNT(*) = 5
       AND COALESCE(bool_and(
            expected.table_name IS NOT NULL
            AND index_namespace.nspname = 'public'
            AND index_relation.relkind = 'i'
            AND index_relation.relpersistence = 'p'
            AND access_method.amname = 'btree'
            AND index_record.indisunique = (expected.index_kind IN ('p', 'u'))
            AND index_record.indisprimary = (expected.index_kind = 'p')
            AND NOT index_record.indisexclusion
            AND index_record.indisvalid
            AND index_record.indisready
            AND index_record.indislive
            AND index_record.indimmediate
            AND NOT index_record.indisclustered
            AND NOT index_record.indisreplident
            AND COALESCE((to_jsonb(index_record) ->> 'indnullsnotdistinct')::boolean, false) = false
            AND index_record.indnkeyatts = CASE WHEN expected.second_column IS NULL THEN 1 ELSE 2 END
            AND index_record.indnatts = index_record.indnkeyatts
            AND index_record.indpred IS NULL
            AND index_record.indexprs IS NULL
            AND first_attribute.attname = expected.first_column
            AND index_record.indoption[0] = expected.first_option
            AND index_record.indcollation[0] = first_attribute.attcollation
            AND first_type.typname = 'varchar'
            AND first_opclass.opcname = 'text_ops'
            AND first_opclass.opcdefault
            AND (
                (expected.second_column IS NULL
                 AND second_attribute.attname IS NULL
                 AND second_opclass.oid IS NULL)
                OR
                (second_attribute.attname = expected.second_column
                 AND index_record.indoption[1] = expected.second_option
                 AND index_record.indcollation[1] = second_attribute.attcollation
                 AND second_attribute.attcollation = 0
                 AND second_type.typname = 'timestamptz'
                 AND second_opclass.opcname = 'timestamptz_ops'
                 AND second_opclass.opcdefault)
            )
            AND (
                (expected.index_kind IN ('p', 'u')
                 AND constraint_record.oid IS NOT NULL
                 AND constraint_record.contype::text = expected.index_kind
                 AND constraint_record.conindid = index_record.indexrelid)
                OR
                (expected.index_kind = 'i' AND constraint_record.oid IS NULL)
            )
       ), false) AS compatible
    FROM pg_catalog.pg_index AS index_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = index_record.indrelid
    JOIN pg_catalog.pg_namespace AS table_namespace
      ON table_namespace.oid = table_record.relnamespace
    JOIN pg_catalog.pg_class AS index_relation
      ON index_relation.oid = index_record.indexrelid
    JOIN pg_catalog.pg_namespace AS index_namespace
      ON index_namespace.oid = index_relation.relnamespace
    LEFT JOIN expected_indexes AS expected
      ON expected.table_name = table_record.relname
     AND expected.index_name = index_relation.relname
    LEFT JOIN pg_catalog.pg_constraint AS constraint_record
      ON constraint_record.conrelid = table_record.oid
     AND constraint_record.conindid = index_record.indexrelid
     AND constraint_record.contype IN ('p', 'u', 'x')
    LEFT JOIN pg_catalog.pg_attribute AS first_attribute
      ON first_attribute.attrelid = table_record.oid
     AND first_attribute.attnum = index_record.indkey[0]
     AND NOT first_attribute.attisdropped
    LEFT JOIN pg_catalog.pg_attribute AS second_attribute
      ON second_attribute.attrelid = table_record.oid
     AND second_attribute.attnum = index_record.indkey[1]
     AND NOT second_attribute.attisdropped
    LEFT JOIN pg_catalog.pg_type AS first_type
      ON first_type.oid = first_attribute.atttypid
    LEFT JOIN pg_catalog.pg_type AS second_type
      ON second_type.oid = second_attribute.atttypid
    LEFT JOIN pg_catalog.pg_opclass AS first_opclass
      ON first_opclass.oid = index_record.indclass[0]
    LEFT JOIN pg_catalog.pg_opclass AS second_opclass
      ON second_opclass.oid = index_record.indclass[1]
    LEFT JOIN pg_catalog.pg_am AS access_method
      ON access_method.oid = index_relation.relam
    WHERE table_namespace.nspname = 'public'
      AND table_record.relname IN ('templates', 'notifications')
)
SELECT
    to_regclass('public.templates') IS NOT NULL
    AND to_regclass('public.notifications') IS NOT NULL
    AND COALESCE((SELECT compatible FROM column_compatibility), false)
    AND COALESCE((SELECT compatible FROM column_inventory_compatibility), false)
    AND COALESCE((SELECT compatible FROM relation_compatibility), false)
    AND COALESCE((SELECT compatible FROM inheritance_compatibility), false)
    AND COALESCE((SELECT compatible FROM policy_compatibility), false)
    AND COALESCE((SELECT compatible FROM key_constraint_compatibility), false)
    AND COALESCE((SELECT compatible FROM not_null_constraint_compatibility), false)
    AND COALESCE((SELECT compatible FROM foreign_key_compatibility), false)
    AND COALESCE((SELECT compatible FROM check_constraint_compatibility), false)
    AND COALESCE((SELECT compatible FROM other_constraint_compatibility), false)
    AND COALESCE((SELECT compatible FROM index_inventory_compatibility), false)
"#;

#[derive(Debug, Error)]
pub enum NotificationSchemaError {
    #[error("notification schema compatibility query failed")]
    Query(#[source] sqlx::Error),
    #[error(
        "notification schema is incompatible; run the reviewed notification migration before startup"
    )]
    Incompatible,
}

pub async fn verify_schema_compatibility(db: &sqlx::PgPool) -> Result<(), NotificationSchemaError> {
    let compatible = sqlx::query_scalar::<_, bool>(NOTIFICATION_SCHEMA_COMPATIBILITY_QUERY)
        .fetch_one(db)
        .await
        .map_err(NotificationSchemaError::Query)?;
    if !compatible {
        return Err(NotificationSchemaError::Incompatible);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum NotificationConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, NotificationConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-notification/1")
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

/// Resolve the only owner key the candidate notification schema can safely
/// accept: the wallet identity proven by the access token. Compatibility
/// `user_id` inputs may agree with that identity but can never select another
/// owner's records.
pub fn canonical_owner(
    principal: &VerifiedPrincipal,
    claimed_user_id: Option<&str>,
) -> Result<String, StatusCode> {
    if claimed_user_id.is_some_and(|claimed| claimed != principal.wallet_address) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(principal.wallet_address.clone())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessPolicy {
    Public,
    Owner,
    NotificationsAdmin,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if path.contains('%') || path.contains('\\') {
        return AccessPolicy::Blocked;
    }
    let Some(tail) = path.strip_prefix("/api/v1/notification/") else {
        return AccessPolicy::Blocked;
    };
    let segments: Vec<_> = tail.split('/').collect();
    if segments.iter().any(|segment| segment.is_empty()) {
        return AccessPolicy::Blocked;
    }

    match (method, segments.as_slice()) {
        (&Method::GET | &Method::POST, ["templates"]) | (&Method::POST, ["send"]) => {
            AccessPolicy::NotificationsAdmin
        }
        (&Method::GET | &Method::DELETE, ["templates", id]) if safe_notification_id(id) => {
            AccessPolicy::NotificationsAdmin
        }
        (&Method::GET, ["list" | "unread-count"])
        | (&Method::POST, ["mark-all-read" | "clear-all"]) => AccessPolicy::Owner,
        (&Method::GET | &Method::DELETE, [id]) if safe_notification_id(id) => AccessPolicy::Owner,
        (&Method::POST, [id, "read" | "unread"]) if safe_notification_id(id) => AccessPolicy::Owner,
        _ => AccessPolicy::Blocked,
    }
}

fn safe_notification_id(id: &str) -> bool {
    !id.is_empty()
        && !matches!(id, "." | "..")
        && !matches!(
            id,
            "templates" | "send" | "list" | "unread-count" | "mark-all-read" | "clear-all"
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
        AccessPolicy::Owner => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != FRONTEND_AUDIENCE && principal.audience != ADMIN_AUDIENCE {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::NotificationsAdmin => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(NOTIFICATIONS_MANAGE_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
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
    use axum::{body::Body, extract::Extension, routing::any};
    use epsx_service_auth::{VerifiedPrincipal, VerifyError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    struct FakeVerifier;

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            let (subject, audience, permissions) = match token {
                "frontend-owner" => ("0xabc", FRONTEND_AUDIENCE, vec![]),
                "admin-owner" => ("0xabc", ADMIN_AUDIENCE, vec![]),
                "admin-manage" => (
                    "0xadmin",
                    ADMIN_AUDIENCE,
                    vec![NOTIFICATIONS_MANAGE_PERMISSION.into()],
                ),
                "admin-resource-wildcard" => (
                    "0xadmin",
                    ADMIN_AUDIENCE,
                    vec!["admin:notifications:*".into()],
                ),
                "admin-domain-wildcard" => ("0xadmin", ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "admin-invalid-wildcard" => {
                    ("0xadmin", ADMIN_AUDIENCE, vec!["admin:*:manage".into()])
                }
                "frontend-manage" => (
                    "0xabc",
                    FRONTEND_AUDIENCE,
                    vec![NOTIFICATIONS_MANAGE_PERMISSION.into()],
                ),
                "other-audience" => ("0xabc", "epsx-other", vec![]),
                _ => return Err(VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: subject.into(),
                wallet_address: subject.into(),
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

    fn app() -> (Router, Arc<Downstream>) {
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
        (protect_router(router, Arc::new(FakeVerifier)), downstream)
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
        let (app, downstream) = app();
        for method in [Method::GET, Method::HEAD] {
            let mut health = request(method, "/health", Some("admin-manage"));
            health
                .headers_mut()
                .insert("x-user-id", "attacker".parse().unwrap());
            assert_eq!(status(&app, health).await, StatusCode::OK);
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 2);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn owner_routes_require_an_exact_allowed_audience_and_insert_the_principal() {
        let routes = [
            (Method::GET, "/api/v1/notification/list"),
            (Method::GET, "/api/v1/notification/unread-count"),
            (Method::POST, "/api/v1/notification/mark-all-read"),
            (Method::POST, "/api/v1/notification/clear-all"),
            (Method::GET, "/api/v1/notification/notification-id"),
            (Method::DELETE, "/api/v1/notification/notification-id"),
            (Method::POST, "/api/v1/notification/notification-id/read"),
            (Method::POST, "/api/v1/notification/notification-id/unread"),
        ];
        let (app, downstream) = app();
        for bearer in ["frontend-owner", "admin-owner"] {
            for (method, path) in &routes {
                assert_eq!(
                    status(&app, request(method.clone(), path, Some(bearer))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 16);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 16);

        for bearer in [None, Some("invalid"), Some("other-audience")] {
            let expected = if bearer == Some("other-audience") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::UNAUTHORIZED
            };
            assert_eq!(
                status(
                    &app,
                    request(Method::GET, "/api/v1/notification/list", bearer),
                )
                .await,
                expected
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 16);
    }

    #[tokio::test]
    async fn admin_routes_use_the_canonical_backend_permission_grammar() {
        let routes = [
            (Method::GET, "/api/v1/notification/templates"),
            (Method::POST, "/api/v1/notification/templates"),
            (Method::GET, "/api/v1/notification/templates/template-id"),
            (Method::DELETE, "/api/v1/notification/templates/template-id"),
            (Method::POST, "/api/v1/notification/send"),
        ];
        let (app, downstream) = app();
        for bearer in [
            "admin-manage",
            "admin-resource-wildcard",
            "admin-domain-wildcard",
        ] {
            for (method, path) in &routes {
                assert_eq!(
                    status(&app, request(method.clone(), path, Some(bearer))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 15);

        for bearer in [
            None,
            Some("invalid"),
            Some("admin-owner"),
            Some("admin-invalid-wildcard"),
            Some("frontend-manage"),
            Some("other-audience"),
        ] {
            let expected = if bearer.is_none() || bearer == Some("invalid") {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::FORBIDDEN
            };
            assert_eq!(
                status(
                    &app,
                    request(Method::POST, "/api/v1/notification/send", bearer),
                )
                .await,
                expected
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 15);
    }

    #[tokio::test]
    async fn spoofed_headers_never_grant_admin_or_replace_owner_identity() {
        let (app, downstream) = app();
        let mut admin = request(
            Method::POST,
            "/api/v1/notification/send",
            Some("admin-owner"),
        );
        admin.headers_mut().insert(
            "x-permissions",
            NOTIFICATIONS_MANAGE_PERMISSION.parse().unwrap(),
        );
        assert_eq!(status(&app, admin).await, StatusCode::FORBIDDEN);

        let mut owner = request(
            Method::GET,
            "/api/v1/notification/list",
            Some("frontend-owner"),
        );
        owner
            .headers_mut()
            .insert("x-user-id", "0xattacker".parse().unwrap());
        assert_eq!(status(&app, owner).await, StatusCode::OK);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 1);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn caller_selected_owner_is_rejected_and_missing_owner_is_derived() {
        let principal = VerifiedPrincipal {
            subject: "0xabc".into(),
            wallet_address: "0xabc".into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: vec![],
        };
        assert_eq!(canonical_owner(&principal, None).unwrap(), "0xabc");
        assert_eq!(canonical_owner(&principal, Some("0xabc")).unwrap(), "0xabc");
        assert_eq!(
            canonical_owner(&principal, Some("0xdef")),
            Err(StatusCode::FORBIDDEN)
        );
        assert_eq!(
            canonical_owner(&principal, Some("")),
            Err(StatusCode::FORBIDDEN)
        );
    }

    #[tokio::test]
    async fn strict_arity_unknown_and_unapproved_methods_are_404_before_downstream() {
        let (app, downstream) = app();
        for (method, path) in [
            (Method::POST, "/health"),
            (Method::PUT, "/api/v1/notification/templates"),
            (Method::POST, "/api/v1/notification/templates/template-id"),
            (Method::GET, "/api/v1/notification/send"),
            (Method::POST, "/api/v1/notification/list"),
            (Method::DELETE, "/api/v1/notification/templates"),
            (Method::GET, "/api/v1/notification/templates/a/b"),
            (Method::GET, "/api/v1/notification/templates/.."),
            (Method::POST, "/api/v1/notification/a/read/extra"),
            (Method::GET, "/api/v1/notification/%2e%2e"),
            (Method::GET, "/api/v1/notification/unknown/shape"),
            (Method::GET, "/api/v1/notification/"),
            (Method::GET, "/unknown"),
        ] {
            assert_eq!(
                status(&app, request(method, path, Some("admin-manage"))).await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn production_verifier_rejects_local_or_insecure_identity_endpoints() {
        assert!(build_auth_verifier(
            "http://127.0.0.1:8080",
            "http://127.0.0.1:8080/.well-known/jwks.json",
            true,
        )
        .is_err());
        assert!(build_auth_verifier(
            "https://identity.example",
            "https://identity.example/.well-known/jwks.json",
            true,
        )
        .is_ok());
    }

    #[tokio::test]
    async fn owner_helper_is_available_to_no_database_handlers() {
        async fn owner_handler(
            Extension(principal): Extension<VerifiedPrincipal>,
        ) -> Result<&'static str, StatusCode> {
            canonical_owner(&principal, Some("0xdef"))?;
            Ok("unreachable")
        }

        let app = protect_router(
            Router::new().route(
                "/api/v1/notification/list",
                axum::routing::get(owner_handler),
            ),
            Arc::new(FakeVerifier),
        );
        assert_eq!(
            status(
                &app,
                request(
                    Method::GET,
                    "/api/v1/notification/list",
                    Some("frontend-owner"),
                ),
            )
            .await,
            StatusCode::FORBIDDEN
        );
    }
}
