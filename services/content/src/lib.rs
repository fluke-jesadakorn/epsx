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

const CONTENT_MANAGE_PERMISSION: &str = "admin:content:manage";

const CONTENT_SCHEMA_COMPATIBILITY_QUERY: &str = r#"
WITH expected_columns (
    table_name,
    column_name,
    ordinal_position,
    data_type,
    udt_name,
    is_nullable,
    character_maximum_length,
    default_kind
) AS (
    VALUES
        ('pages', 'id', 1, 'uuid', 'uuid', 'NO', NULL::bigint, 'uuid'),
        ('pages', 'slug', 2, 'character varying', 'varchar', 'NO', 255::bigint, 'none'),
        ('pages', 'title', 3, 'character varying', 'varchar', 'NO', 255::bigint, 'none'),
        ('pages', 'locale', 4, 'character varying', 'varchar', 'NO', 10::bigint, 'en'),
        ('pages', 'status', 5, 'character varying', 'varchar', 'NO', 20::bigint, 'draft'),
        ('pages', 'blocks_json', 6, 'jsonb', 'jsonb', 'NO', NULL::bigint, 'empty_array'),
        ('pages', 'seo_json', 7, 'jsonb', 'jsonb', 'YES', NULL::bigint, 'empty_object'),
        ('pages', 'theme_id', 8, 'uuid', 'uuid', 'YES', NULL::bigint, 'none'),
        ('pages', 'created_at', 9, 'timestamp with time zone', 'timestamptz', 'NO', NULL::bigint, 'now'),
        ('pages', 'updated_at', 10, 'timestamp with time zone', 'timestamptz', 'NO', NULL::bigint, 'now'),
        ('pages', 'published_at', 11, 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 'none'),
        ('themes', 'id', 1, 'uuid', 'uuid', 'NO', NULL::bigint, 'uuid'),
        ('themes', 'name', 2, 'character varying', 'varchar', 'NO', 100::bigint, 'none'),
        ('themes', 'colors_json', 3, 'jsonb', 'jsonb', 'NO', NULL::bigint, 'empty_object'),
        ('themes', 'fonts_json', 4, 'jsonb', 'jsonb', 'NO', NULL::bigint, 'empty_object'),
        ('themes', 'spacing_json', 5, 'jsonb', 'jsonb', 'NO', NULL::bigint, 'empty_object'),
        ('themes', 'breakpoints_json', 6, 'jsonb', 'jsonb', 'NO', NULL::bigint, 'empty_object'),
        ('themes', 'radius_json', 7, 'jsonb', 'jsonb', 'YES', NULL::bigint, 'empty_object'),
        ('themes', 'is_default', 8, 'boolean', 'bool', 'NO', NULL::bigint, 'false'),
        ('block_types', 'id', 1, 'uuid', 'uuid', 'NO', NULL::bigint, 'uuid'),
        ('block_types', 'block_type', 2, 'character varying', 'varchar', 'NO', 50::bigint, 'none'),
        ('block_types', 'name', 3, 'character varying', 'varchar', 'NO', 100::bigint, 'none'),
        ('block_types', 'category', 4, 'character varying', 'varchar', 'NO', 50::bigint, 'none'),
        ('block_types', 'description', 5, 'text', 'text', 'YES', NULL::bigint, 'none'),
        ('block_types', 'schema_json', 6, 'jsonb', 'jsonb', 'NO', NULL::bigint, 'empty_object'),
        ('block_types', 'default_props_json', 7, 'jsonb', 'jsonb', 'NO', NULL::bigint, 'empty_object'),
        ('block_types', 'admin_only', 8, 'boolean', 'bool', 'NO', NULL::bigint, 'false'),
        ('block_types', 'updated_at', 9, 'timestamp with time zone', 'timestamptz', 'NO', NULL::bigint, 'now'),
        ('edit_sessions', 'id', 1, 'uuid', 'uuid', 'NO', NULL::bigint, 'uuid'),
        ('edit_sessions', 'page_id', 2, 'uuid', 'uuid', 'NO', NULL::bigint, 'none'),
        ('edit_sessions', 'user_id', 3, 'uuid', 'uuid', 'NO', NULL::bigint, 'none'),
        ('edit_sessions', 'status', 4, 'character varying', 'varchar', 'NO', 20::bigint, 'active'),
        ('edit_sessions', 'started_at', 5, 'timestamp with time zone', 'timestamptz', 'NO', NULL::bigint, 'now'),
        ('edit_sessions', 'ended_at', 6, 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 'none')
),
expected_tables (table_name, column_count) AS (
    VALUES
        ('pages', 11),
        ('themes', 8),
        ('block_types', 9),
        ('edit_sessions', 6)
),
column_compatibility AS (
    SELECT bool_and(
        actual.column_name IS NOT NULL
        AND actual.ordinal_position = expected.ordinal_position
        AND actual.data_type = expected.data_type
        AND actual.udt_name = expected.udt_name
        AND actual.is_nullable = expected.is_nullable
        AND actual.character_maximum_length IS NOT DISTINCT FROM expected.character_maximum_length
        AND COALESCE(
            CASE expected.default_kind
                WHEN 'uuid' THEN actual.column_default = 'gen_random_uuid()'
                WHEN 'en' THEN actual.column_default IN (
                    '''en''::character varying',
                    '''en''::text',
                    '''en'''
                )
                WHEN 'draft' THEN actual.column_default IN (
                    '''draft''::character varying',
                    '''draft''::text',
                    '''draft'''
                )
                WHEN 'active' THEN actual.column_default IN (
                    '''active''::character varying',
                    '''active''::text',
                    '''active'''
                )
                WHEN 'empty_array' THEN actual.column_default = '''[]''::jsonb'
                WHEN 'empty_object' THEN actual.column_default = '''{}''::jsonb'
                WHEN 'false' THEN actual.column_default = 'false'
                WHEN 'now' THEN actual.column_default IN ('now()', 'CURRENT_TIMESTAMP')
                ELSE actual.column_default IS NULL
            END,
            false
        )
    ) AS compatible
    FROM expected_columns AS expected
    LEFT JOIN information_schema.columns AS actual
      ON actual.table_schema = 'public'
     AND actual.table_name = expected.table_name
     AND actual.column_name = expected.column_name
),
column_inventory_compatibility AS (
    SELECT COUNT(*) = 34 AS compatible
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name IN ('pages', 'themes', 'block_types', 'edit_sessions')
),
relation_compatibility AS (
    SELECT COUNT(*) = 4
       AND COALESCE(bool_and(
            table_record.relkind = 'r'
            AND table_record.relpersistence = 'p'
       ), false) AS compatible
    FROM pg_catalog.pg_class AS table_record
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('pages', 'themes', 'block_types', 'edit_sessions')
),
primary_key_compatibility AS (
    SELECT COUNT(*) = 4
       AND COUNT(DISTINCT table_record.relname) = 4
       AND COALESCE(bool_and(
            constraint_record.convalidated
            AND NOT constraint_record.condeferrable
            AND NOT constraint_record.condeferred
            AND cardinality(constraint_record.conkey) = 1
            AND attribute_record.attname = 'id'
       ), false) AS compatible
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    JOIN pg_catalog.pg_attribute AS attribute_record
      ON attribute_record.attrelid = table_record.oid
     AND attribute_record.attnum = constraint_record.conkey[1]
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('pages', 'themes', 'block_types', 'edit_sessions')
      AND constraint_record.contype = 'p'
),
unique_key_compatibility AS (
    SELECT COUNT(*) = 3
       AND COUNT(DISTINCT (table_record.relname, attribute_record.attname)) = 3
       AND COALESCE(bool_and(
            constraint_record.convalidated
            AND NOT constraint_record.condeferrable
            AND NOT constraint_record.condeferred
            AND cardinality(constraint_record.conkey) = 1
            AND index_record.indisunique
            AND NOT index_record.indisprimary
            AND index_record.indisvalid
            AND index_record.indisready
            AND index_record.indimmediate
            AND index_record.indnkeyatts = 1
            AND index_record.indnatts = 1
            AND index_record.indpred IS NULL
            AND index_record.indexprs IS NULL
            AND (
                (table_record.relname = 'pages' AND attribute_record.attname = 'slug')
                OR (table_record.relname = 'themes' AND attribute_record.attname = 'name')
                OR (table_record.relname = 'block_types' AND attribute_record.attname = 'block_type')
            )
       ), false) AS compatible
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    JOIN pg_catalog.pg_attribute AS attribute_record
      ON attribute_record.attrelid = table_record.oid
     AND attribute_record.attnum = constraint_record.conkey[1]
    JOIN pg_catalog.pg_index AS index_record
      ON index_record.indexrelid = constraint_record.conindid
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('pages', 'themes', 'block_types', 'edit_sessions')
      AND constraint_record.contype = 'u'
),
foreign_key_compatibility AS (
    SELECT COUNT(*) = 1
       AND COALESCE(bool_and(
            source_namespace.nspname = 'public'
            AND source_table.relname = 'edit_sessions'
            AND source_column.attname = 'page_id'
            AND target_namespace.nspname = 'public'
            AND target_table.relname = 'pages'
            AND target_column.attname = 'id'
            AND constraint_record.convalidated
            AND cardinality(constraint_record.conkey) = 1
            AND cardinality(constraint_record.confkey) = 1
            AND constraint_record.confupdtype = 'a'
            AND constraint_record.confdeltype = 'c'
            AND constraint_record.confmatchtype = 's'
            AND NOT constraint_record.condeferrable
            AND NOT constraint_record.condeferred
       ), false) AS compatible
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS source_table
      ON source_table.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS source_namespace
      ON source_namespace.oid = source_table.relnamespace
    JOIN pg_catalog.pg_attribute AS source_column
      ON source_column.attrelid = source_table.oid
     AND source_column.attnum = constraint_record.conkey[1]
    JOIN pg_catalog.pg_class AS target_table
      ON target_table.oid = constraint_record.confrelid
    JOIN pg_catalog.pg_namespace AS target_namespace
      ON target_namespace.oid = target_table.relnamespace
    JOIN pg_catalog.pg_attribute AS target_column
      ON target_column.attrelid = target_table.oid
     AND target_column.attnum = constraint_record.confkey[1]
    WHERE constraint_record.contype = 'f'
      AND (
          (
              source_namespace.nspname = 'public'
              AND source_table.relname IN ('pages', 'themes', 'block_types', 'edit_sessions')
          )
          OR (
              target_namespace.nspname = 'public'
              AND target_table.relname IN ('pages', 'themes', 'block_types', 'edit_sessions')
          )
      )
),
unique_index_inventory_compatibility AS (
    SELECT COUNT(*) = 7
       AND COUNT(DISTINCT (
            table_record.relname,
            attribute_record.attname,
            constraint_record.contype
       )) = 7
       AND COALESCE(bool_and(
            constraint_record.oid IS NOT NULL
            AND constraint_record.convalidated
            AND NOT constraint_record.condeferrable
            AND NOT constraint_record.condeferred
            AND cardinality(constraint_record.conkey) = 1
            AND constraint_record.conkey[1] = attribute_record.attnum
            AND index_relation.relkind = 'i'
            AND index_relation.relpersistence = 'p'
            AND index_record.indisunique
            AND index_record.indisvalid
            AND index_record.indisready
            AND index_record.indimmediate
            AND index_record.indisprimary = (constraint_record.contype = 'p')
            AND index_record.indnkeyatts = 1
            AND index_record.indnatts = 1
            AND index_record.indpred IS NULL
            AND index_record.indexprs IS NULL
            AND index_record.indcollation[0] = attribute_record.attcollation
            AND access_method.amname = 'btree'
            AND opclass_record.opcdefault
            AND (
                (
                    constraint_record.contype = 'p'
                    AND table_record.relname IN ('pages', 'themes', 'block_types', 'edit_sessions')
                    AND attribute_record.attname = 'id'
                    AND type_record.typname = 'uuid'
                    AND attribute_record.attcollation = 0
                    AND opclass_record.opcname = 'uuid_ops'
                )
                OR (
                    constraint_record.contype = 'u'
                    AND type_record.typname = 'varchar'
                    AND attribute_record.attcollation <> 0
                    AND opclass_record.opcname = 'text_ops'
                    AND (
                        (table_record.relname = 'pages' AND attribute_record.attname = 'slug')
                        OR (table_record.relname = 'themes' AND attribute_record.attname = 'name')
                        OR (table_record.relname = 'block_types' AND attribute_record.attname = 'block_type')
                    )
                )
            )
       ), false) AS compatible
    FROM pg_catalog.pg_index AS index_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = index_record.indrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    JOIN pg_catalog.pg_class AS index_relation
      ON index_relation.oid = index_record.indexrelid
    LEFT JOIN pg_catalog.pg_constraint AS constraint_record
      ON constraint_record.conrelid = table_record.oid
     AND constraint_record.conindid = index_record.indexrelid
     AND constraint_record.contype IN ('p', 'u')
    LEFT JOIN pg_catalog.pg_attribute AS attribute_record
      ON attribute_record.attrelid = table_record.oid
     AND attribute_record.attnum = index_record.indkey[0]
     AND NOT attribute_record.attisdropped
    LEFT JOIN pg_catalog.pg_type AS type_record
      ON type_record.oid = attribute_record.atttypid
    LEFT JOIN pg_catalog.pg_opclass AS opclass_record
      ON opclass_record.oid = index_record.indclass[0]
    LEFT JOIN pg_catalog.pg_am AS access_method
      ON access_method.oid = index_relation.relam
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('pages', 'themes', 'block_types', 'edit_sessions')
      AND index_record.indisunique
)
SELECT
    to_regclass('public.pages') IS NOT NULL
    AND to_regclass('public.themes') IS NOT NULL
    AND to_regclass('public.block_types') IS NOT NULL
    AND to_regclass('public.edit_sessions') IS NOT NULL
    AND COALESCE((SELECT compatible FROM column_compatibility), false)
    AND (SELECT compatible FROM column_inventory_compatibility)
    AND (SELECT compatible FROM relation_compatibility)
    AND (SELECT compatible FROM primary_key_compatibility)
    AND (SELECT compatible FROM unique_key_compatibility)
    AND (SELECT compatible FROM foreign_key_compatibility)
    AND (SELECT compatible FROM unique_index_inventory_compatibility)
"#;

#[derive(Debug, Error)]
pub enum ContentSchemaError {
    #[error("content schema compatibility query failed")]
    Query(#[source] sqlx::Error),
    #[error("content schema is incompatible; run the reviewed content migration before startup")]
    Incompatible,
}

pub async fn verify_schema_compatibility(db: &sqlx::PgPool) -> Result<(), ContentSchemaError> {
    let compatible = sqlx::query_scalar::<_, bool>(CONTENT_SCHEMA_COMPATIBILITY_QUERY)
        .fetch_one(db)
        .await
        .map_err(ContentSchemaError::Query)?;
    if !compatible {
        return Err(ContentSchemaError::Incompatible);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum ContentConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, ContentConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-content/1")
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
    ContentAdmin,
    EditorIdentityRequired,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if path.contains('%') || path.contains('\\') {
        return AccessPolicy::Blocked;
    }
    let Some(tail) = path.strip_prefix("/api/v1/content/") else {
        return AccessPolicy::Blocked;
    };
    let segments: Vec<_> = tail.split('/').collect();
    if segments.iter().any(|segment| segment.is_empty()) {
        return AccessPolicy::Blocked;
    }

    match (method, segments.as_slice()) {
        (&Method::GET, ["pages", slug, "render"]) if safe_dynamic_segment(slug) => {
            AccessPolicy::Public
        }
        (
            &Method::GET,
            ["themes" | "blocks" | "navigation" | "site" | "news" | "plans" | "rankings"],
        ) => AccessPolicy::Public,
        (&Method::GET, ["themes" | "blocks" | "news" | "portfolio", value])
            if safe_dynamic_segment(value) =>
        {
            AccessPolicy::Public
        }
        (&Method::GET, ["pages"]) | (&Method::POST, ["pages" | "themes"]) => {
            AccessPolicy::ContentAdmin
        }
        (&Method::GET | &Method::PUT, ["pages", slug]) if safe_dynamic_segment(slug) => {
            AccessPolicy::ContentAdmin
        }
        (&Method::POST, ["pages", id, "publish"]) if safe_dynamic_segment(id) => {
            AccessPolicy::ContentAdmin
        }
        (&Method::PUT, ["themes", id]) if safe_dynamic_segment(id) => AccessPolicy::ContentAdmin,
        (&Method::POST, ["edit", "start" | "commit"]) | (&Method::GET, ["edit", "sessions"]) => {
            AccessPolicy::EditorIdentityRequired
        }
        _ => AccessPolicy::Blocked,
    }
}

fn safe_dynamic_segment(segment: &str) -> bool {
    !segment.is_empty() && !matches!(segment, "." | "..")
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
        AccessPolicy::ContentAdmin => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(CONTENT_MANAGE_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::EditorIdentityRequired | AccessPolicy::Blocked => {
            return StatusCode::NOT_FOUND.into_response();
        }
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
    use axum::body::Body;
    use epsx_service_auth::{VerifiedPrincipal, VerifyError, FRONTEND_AUDIENCE};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    struct FakeVerifier;

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            let (audience, permissions) = match token {
                "frontend-content" => (FRONTEND_AUDIENCE, vec![CONTENT_MANAGE_PERMISSION.into()]),
                "admin" => (ADMIN_AUDIENCE, vec![]),
                "admin-content" => (ADMIN_AUDIENCE, vec![CONTENT_MANAGE_PERMISSION.into()]),
                "admin-wildcard" => (ADMIN_AUDIENCE, vec!["admin:content:*".into()]),
                "admin-invalid-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:manage".into()]),
                "other-audience" => ("epsx-other", vec![CONTENT_MANAGE_PERMISSION.into()]),
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

    fn app() -> (Router, Arc<Downstream>) {
        let downstream = Arc::new(Downstream::default());
        let observed = downstream.clone();
        let router = Router::new().fallback(move |request: Request| {
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
                StatusCode::OK
            }
        });
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
    async fn exact_public_allowlist_is_anonymous() {
        let (app, downstream) = app();
        for (method, path) in [
            (Method::GET, "/health"),
            (Method::HEAD, "/health"),
            (Method::GET, "/api/v1/content/pages/welcome/render"),
            (Method::GET, "/api/v1/content/themes"),
            (Method::GET, "/api/v1/content/themes/theme-id"),
            (Method::GET, "/api/v1/content/blocks"),
            (Method::GET, "/api/v1/content/blocks/hero"),
            (Method::GET, "/api/v1/content/navigation"),
            (Method::GET, "/api/v1/content/site"),
            (Method::GET, "/api/v1/content/news"),
            (Method::GET, "/api/v1/content/news/launch"),
            (Method::GET, "/api/v1/content/plans"),
            (Method::GET, "/api/v1/content/rankings"),
            (Method::GET, "/api/v1/content/portfolio/0xabc"),
        ] {
            assert_eq!(
                status(&app, request(method, path, None)).await,
                StatusCode::OK
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 14);
    }

    #[tokio::test]
    async fn public_requests_strip_bearer_and_spoofable_identity_headers() {
        let (app, downstream) = app();
        let mut public = request(
            Method::GET,
            "/api/v1/content/navigation",
            Some("admin-content"),
        );
        public
            .headers_mut()
            .insert("x-user-id", "attacker".parse().unwrap());
        public
            .headers_mut()
            .insert("x-permissions", CONTENT_MANAGE_PERMISSION.parse().unwrap());
        assert_eq!(status(&app, public).await, StatusCode::OK);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn content_admin_routes_follow_canonical_backend_permission_grammar() {
        let (app, downstream) = app();
        let routes = [
            (Method::GET, "/api/v1/content/pages/article"),
            (Method::PUT, "/api/v1/content/pages/article"),
            (Method::POST, "/api/v1/content/pages"),
            (Method::GET, "/api/v1/content/pages"),
            (Method::POST, "/api/v1/content/pages/page-id/publish"),
            (Method::POST, "/api/v1/content/themes"),
            (Method::PUT, "/api/v1/content/themes/theme-id"),
        ];
        for bearer in ["admin-content", "admin-wildcard"] {
            for (method, path) in &routes {
                assert_eq!(
                    status(&app, request(method.clone(), path, Some(bearer))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 14);
    }

    #[tokio::test]
    async fn wrong_audience_missing_permission_and_spoof_headers_are_denied() {
        let (app, downstream) = app();
        for (bearer, expected) in [
            (None, StatusCode::UNAUTHORIZED),
            (Some("invalid"), StatusCode::UNAUTHORIZED),
            (Some("admin"), StatusCode::FORBIDDEN),
            (Some("admin-invalid-wildcard"), StatusCode::FORBIDDEN),
            (Some("frontend-content"), StatusCode::FORBIDDEN),
            (Some("other-audience"), StatusCode::FORBIDDEN),
        ] {
            assert_eq!(
                status(&app, request(Method::GET, "/api/v1/content/pages", bearer),).await,
                expected
            );
        }

        let mut spoofed = request(Method::GET, "/api/v1/content/pages", Some("admin"));
        spoofed
            .headers_mut()
            .insert("x-permissions", CONTENT_MANAGE_PERMISSION.parse().unwrap());
        assert_eq!(status(&app, spoofed).await, StatusCode::FORBIDDEN);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn editor_identity_routes_remain_fail_closed() {
        let (app, downstream) = app();
        for (method, path) in [
            (Method::POST, "/api/v1/content/edit/start"),
            (Method::POST, "/api/v1/content/edit/commit"),
            (Method::GET, "/api/v1/content/edit/sessions"),
        ] {
            assert_eq!(
                status(&app, request(method, path, Some("admin-content"))).await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn strict_arity_unknown_and_unapproved_methods_fail_before_downstream() {
        let (app, downstream) = app();
        for (method, path) in [
            (Method::GET, "/api/v1/content/pages/a/b/render"),
            (Method::GET, "/api/v1/content/themes/a/b"),
            (Method::GET, "/api/v1/content/blocks/a/b"),
            (Method::GET, "/api/v1/content/news/a/b"),
            (Method::GET, "/api/v1/content/portfolio/a/b"),
            (Method::GET, "/api/v1/content/pages/%2e%2e/render"),
            (Method::POST, "/api/v1/content/navigation"),
            (Method::DELETE, "/api/v1/content/pages/article"),
            (Method::GET, "/api/v1/content/unknown"),
        ] {
            assert_eq!(
                status(&app, request(method, path, Some("admin-content"))).await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn production_auth_requires_https_non_local_identity_urls() {
        assert!(build_auth_verifier(
            "http://issuer.example",
            "https://issuer.example/.well-known/jwks.json",
            true,
        )
        .is_err());
        assert!(build_auth_verifier(
            "https://issuer.localhost",
            "https://issuer.example/.well-known/jwks.json",
            true,
        )
        .is_err());
        assert!(build_auth_verifier(
            "https://issuer.example",
            "https://127.0.0.1/.well-known/jwks.json",
            true,
        )
        .is_err());
    }

    #[test]
    fn schema_probe_is_read_only_null_safe_and_public_qualified() {
        let query = CONTENT_SCHEMA_COMPATIBILITY_QUERY;
        assert!(query.trim_start().starts_with("WITH expected_columns ("));
        for relation in ["pages", "themes", "block_types", "edit_sessions"] {
            assert!(query.contains(&format!("to_regclass('public.{relation}')")));
        }
        for mutation in [
            "INSERT", "UPDATE", "DELETE", "CREATE", "ALTER", "DROP", "TRUNCATE", "CALL", "DO",
        ] {
            assert!(
                !query
                    .split(|character: char| !character.is_ascii_alphabetic())
                    .any(|token| token.eq_ignore_ascii_case(mutation)),
                "schema compatibility query contains mutation token {mutation}"
            );
        }
        assert!(query.contains("SELECT COUNT(*) = 34 AS compatible"));
        assert!(query.contains("AND COALESCE(\n            CASE expected.default_kind"));
        assert!(
            query.contains("AND COALESCE((SELECT compatible FROM column_compatibility), false)")
        );
        assert!(query.contains("index_record.indisvalid"));
        assert!(query.contains("index_record.indisready"));
        assert!(
            query.contains("COUNT(DISTINCT (table_record.relname, attribute_record.attname)) = 3")
        );
        assert!(query.contains("NOT constraint_record.condeferrable"));
        assert!(query.contains("NOT constraint_record.condeferred"));
        assert!(query.contains("index_record.indimmediate"));
        assert!(query.contains("unique_index_inventory_compatibility AS ("));
        assert!(query.contains("FROM pg_catalog.pg_index AS index_record"));
        assert!(query.contains("LEFT JOIN pg_catalog.pg_constraint AS constraint_record"));
        assert!(query.contains("constraint_record.oid IS NOT NULL"));
        assert!(query.contains("index_record.indcollation[0] = attribute_record.attcollation"));
        assert!(query.contains("opclass_record.opcname = 'uuid_ops'"));
        assert!(query.contains("opclass_record.opcname = 'text_ops'"));
        assert!(query.contains("index_record.indisunique\n"));
        assert!(query.contains("constraint_record.confdeltype = 'c'"));
        assert!(query.contains("OR (\n              target_namespace.nspname = 'public'"));
    }
}
