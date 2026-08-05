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

pub const PLANS_READ_PERMISSION: &str = "admin:plans:read";
pub const PLANS_MANAGE_PERMISSION: &str = "admin:plans:manage";
pub const ACCESS_READ_PERMISSION: &str = "admin:access:read";
pub const ACCESS_MANAGE_PERMISSION: &str = "admin:access:manage";

const SUBSCRIPTION_SCHEMA_COMPATIBILITY_QUERY: &str = r#"
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
        ('subscription_plans', 'id', 1, 'uuid', 'uuid', 'NO', NULL::bigint, 'uuid'),
        ('subscription_plans', 'merchant_id', 2, 'uuid', 'uuid', 'NO', NULL::bigint, 'none'),
        ('subscription_plans', 'name', 3, 'character varying', 'varchar', 'NO', 100::bigint, 'none'),
        ('subscription_plans', 'description', 4, 'text', 'text', 'YES', NULL::bigint, 'none'),
        ('subscription_plans', 'amount', 5, 'character varying', 'varchar', 'NO', 78::bigint, 'none'),
        ('subscription_plans', 'currency', 6, 'character varying', 'varchar', 'NO', 10::bigint, 'none'),
        ('subscription_plans', 'chain_id', 7, 'character varying', 'varchar', 'NO', 10::bigint, 'none'),
        ('subscription_plans', 'interval', 8, 'integer', 'int4', 'NO', NULL::bigint, 'none'),
        ('subscription_plans', 'active', 9, 'boolean', 'bool', 'YES', NULL::bigint, 'true'),
        ('subscription_plans', 'created_at', 10, 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 'now'),
        ('subscriptions', 'id', 1, 'uuid', 'uuid', 'NO', NULL::bigint, 'uuid'),
        ('subscriptions', 'user_id', 2, 'uuid', 'uuid', 'NO', NULL::bigint, 'none'),
        ('subscriptions', 'plan_id', 3, 'uuid', 'uuid', 'YES', NULL::bigint, 'none'),
        ('subscriptions', 'status', 4, 'character varying', 'varchar', 'YES', 20::bigint, 'active'),
        ('subscriptions', 'account_id', 5, 'character varying', 'varchar', 'YES', 42::bigint, 'none'),
        ('subscriptions', 'payment_token', 6, 'character varying', 'varchar', 'YES', 42::bigint, 'none'),
        ('subscriptions', 'vault_position_id', 7, 'character varying', 'varchar', 'YES', 100::bigint, 'none'),
        ('subscriptions', 'current_period_start', 8, 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 'none'),
        ('subscriptions', 'current_period_end', 9, 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 'none'),
        ('subscriptions', 'created_at', 10, 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 'now')
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
                WHEN 'true' THEN actual.column_default = 'true'
                WHEN 'now' THEN actual.column_default IN ('now()', 'CURRENT_TIMESTAMP')
                WHEN 'active' THEN actual.column_default IN (
                    '''active''::character varying',
                    '''active''::text',
                    '''active'''
                )
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
    SELECT COUNT(*) = 20 AS compatible
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name IN ('subscription_plans', 'subscriptions')
),
primary_key_compatibility AS (
    SELECT COUNT(*) = 2 AS compatible
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    JOIN pg_catalog.pg_attribute AS attribute_record
      ON attribute_record.attrelid = table_record.oid
     AND attribute_record.attnum = constraint_record.conkey[1]
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('subscription_plans', 'subscriptions')
      AND constraint_record.contype = 'p'
      AND constraint_record.convalidated
      AND cardinality(constraint_record.conkey) = 1
      AND attribute_record.attname = 'id'
),
primary_key_index_compatibility AS (
    SELECT COUNT(*) = 2 AS compatible
    FROM pg_catalog.pg_constraint AS constraint_record
    JOIN pg_catalog.pg_class AS table_record
      ON table_record.oid = constraint_record.conrelid
    JOIN pg_catalog.pg_namespace AS namespace_record
      ON namespace_record.oid = table_record.relnamespace
    JOIN pg_catalog.pg_index AS index_record
      ON index_record.indexrelid = constraint_record.conindid
    WHERE namespace_record.nspname = 'public'
      AND table_record.relname IN ('subscription_plans', 'subscriptions')
      AND constraint_record.contype = 'p'
      AND constraint_record.convalidated
      AND index_record.indisprimary
      AND index_record.indisunique
      AND index_record.indisvalid
      AND index_record.indisready
),
foreign_key_compatibility AS (
    SELECT COUNT(*) = 1
      AND (
          SELECT COUNT(*)
          FROM pg_catalog.pg_constraint AS all_foreign_keys
          JOIN pg_catalog.pg_class AS all_source_tables
            ON all_source_tables.oid = all_foreign_keys.conrelid
          JOIN pg_catalog.pg_namespace AS all_source_namespaces
            ON all_source_namespaces.oid = all_source_tables.relnamespace
          WHERE all_source_namespaces.nspname = 'public'
            AND all_source_tables.relname IN ('subscription_plans', 'subscriptions')
            AND all_foreign_keys.contype = 'f'
      ) = 1 AS compatible
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
    WHERE source_namespace.nspname = 'public'
      AND source_table.relname = 'subscriptions'
      AND source_column.attname = 'plan_id'
      AND target_namespace.nspname = 'public'
      AND target_table.relname = 'subscription_plans'
      AND target_column.attname = 'id'
      AND constraint_record.contype = 'f'
      AND constraint_record.convalidated
      AND cardinality(constraint_record.conkey) = 1
      AND cardinality(constraint_record.confkey) = 1
      AND constraint_record.confupdtype = 'a'
      AND constraint_record.confdeltype = 'a'
)
SELECT
    to_regclass('public.subscription_plans') IS NOT NULL
    AND to_regclass('public.subscriptions') IS NOT NULL
    AND COALESCE((SELECT compatible FROM column_compatibility), false)
    AND (SELECT compatible FROM column_inventory_compatibility)
    AND (SELECT compatible FROM primary_key_compatibility)
    AND (SELECT compatible FROM primary_key_index_compatibility)
    AND (SELECT compatible FROM foreign_key_compatibility)
    AND (
        SELECT COUNT(*) = 3
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'subscription_plan_state'
          AND column_name IN ('plan_id', 'version', 'updated_at')
    )
"#;

#[derive(Debug, Error)]
pub enum SubscriptionSchemaError {
    #[error("subscription schema compatibility query failed")]
    Query(#[source] sqlx::Error),
    #[error(
        "subscription schema is incompatible; run the reviewed subscription migration before startup"
    )]
    Incompatible,
}

pub async fn verify_schema_compatibility(db: &sqlx::PgPool) -> Result<(), SubscriptionSchemaError> {
    let compatible = sqlx::query_scalar::<_, bool>(SUBSCRIPTION_SCHEMA_COMPATIBILITY_QUERY)
        .fetch_one(db)
        .await
        .map_err(SubscriptionSchemaError::Query)?;
    if !compatible {
        return Err(SubscriptionSchemaError::Incompatible);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum SubscriptionConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, SubscriptionConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-subscription/1")
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
    AdminPermission(&'static str),
    PlansRead,
    PlansManage,
    OwnerIdentityUnavailable,
    UnsafeVaultConfig,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if path.contains('%') || path.contains('\\') {
        return AccessPolicy::Blocked;
    }
    if path == "/api/v1/admin/subscription/plans" || path.starts_with("/api/v1/admin/subscription/")
    {
        let tail = path
            .strip_prefix("/api/v1/admin/subscription/")
            .unwrap_or_default();
        let segments: Vec<_> = tail.split('/').collect();
        if segments.iter().any(|segment| segment.is_empty()) {
            return AccessPolicy::Blocked;
        }
        return match (method, segments.as_slice()) {
            (&Method::GET, ["plans"]) => AccessPolicy::AdminPermission(PLANS_READ_PERMISSION),
            (&Method::GET, ["plans", id]) if safe_dynamic_segment(id) => {
                AccessPolicy::AdminPermission(PLANS_READ_PERMISSION)
            }
            (&Method::POST, ["plans"]) => AccessPolicy::AdminPermission(PLANS_MANAGE_PERMISSION),
            (&Method::PATCH, ["plans", id]) if safe_dynamic_segment(id) => {
                AccessPolicy::AdminPermission(PLANS_MANAGE_PERMISSION)
            }
            (&Method::GET, ["access"]) => AccessPolicy::AdminPermission(ACCESS_READ_PERMISSION),
            (&Method::POST, ["access", "assign" | "revoke"]) => {
                AccessPolicy::AdminPermission(ACCESS_MANAGE_PERMISSION)
            }
            _ => AccessPolicy::Blocked,
        };
    }
    let Some(tail) = path.strip_prefix("/api/v1/subscription/") else {
        return AccessPolicy::Blocked;
    };
    let segments: Vec<_> = tail.split('/').collect();
    if segments.iter().any(|segment| segment.is_empty()) {
        return AccessPolicy::Blocked;
    }

    match (method, segments.as_slice()) {
        (&Method::GET, ["plans"]) => AccessPolicy::PlansRead,
        (&Method::GET, ["plans", id]) if safe_dynamic_segment(id) => AccessPolicy::PlansRead,
        (&Method::POST, ["plans"]) => AccessPolicy::PlansManage,
        (&Method::GET | &Method::POST, ["subscriptions"])
        | (&Method::GET, ["subscriptions", _])
        | (&Method::POST, ["subscriptions", _, "cancel"]) => AccessPolicy::OwnerIdentityUnavailable,
        (&Method::GET, ["vault", chain_id]) if safe_dynamic_segment(chain_id) => {
            AccessPolicy::UnsafeVaultConfig
        }
        _ => AccessPolicy::Blocked,
    }
}

fn safe_dynamic_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        && !matches!(
            segment,
            "plans" | "subscriptions" | "vault" | "access" | "assign" | "revoke" | "cancel"
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
        AccessPolicy::AdminPermission(required) => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal
                    .permissions
                    .iter()
                    .any(|permission| permission == required)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::PlansRead => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(PLANS_READ_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::PlansManage => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(PLANS_MANAGE_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::OwnerIdentityUnavailable
        | AccessPolicy::UnsafeVaultConfig
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
    use epsx_service_auth::{VerifiedPrincipal, VerifyError, FRONTEND_AUDIENCE};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    struct FakeVerifier;

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            let (audience, permissions) = match token {
                "admin-none" => (ADMIN_AUDIENCE, vec![]),
                "admin-read" => (ADMIN_AUDIENCE, vec![PLANS_READ_PERMISSION.into()]),
                "admin-manage" => (ADMIN_AUDIENCE, vec![PLANS_MANAGE_PERMISSION.into()]),
                "admin-resource-wildcard" => (ADMIN_AUDIENCE, vec!["admin:plans:*".into()]),
                "admin-domain-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "admin-global-wildcard" => (ADMIN_AUDIENCE, vec!["*:*".into()]),
                "admin-invalid-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:manage".into()]),
                "frontend-read" => (FRONTEND_AUDIENCE, vec![PLANS_READ_PERMISSION.into()]),
                "frontend-manage" => (FRONTEND_AUDIENCE, vec![PLANS_MANAGE_PERMISSION.into()]),
                "other-audience" => ("epsx-other", vec![PLANS_READ_PERMISSION.into()]),
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

    #[test]
    fn schema_compatibility_query_is_read_only_and_complete() {
        let query = SUBSCRIPTION_SCHEMA_COMPATIBILITY_QUERY;
        assert!(query.trim_start().starts_with("WITH expected_columns ("));
        assert_eq!(
            query
                .lines()
                .filter(|line| line.trim_start().starts_with("('subscription_plans', '"))
                .count(),
            10
        );
        assert_eq!(
            query
                .lines()
                .filter(|line| line.trim_start().starts_with("('subscriptions', '"))
                .count(),
            10
        );
        for anchor in [
            "LEFT JOIN information_schema.columns AS actual",
            "AND COALESCE(\n            CASE expected.default_kind",
            "AND COALESCE((SELECT compatible FROM column_compatibility), false)",
            "to_regclass('public.subscription_plans') IS NOT NULL",
            "to_regclass('public.subscriptions') IS NOT NULL",
            "constraint_record.confupdtype = 'a'",
            "constraint_record.confdeltype = 'a'",
            "AND index_record.indisvalid",
            "AND index_record.indisready",
        ] {
            assert!(query.contains(anchor), "missing schema anchor: {anchor}");
        }

        let uppercase = query.to_ascii_uppercase();
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
            "CALL ",
            "DO ",
        ] {
            assert!(
                !uppercase.contains(forbidden),
                "schema query contains command token: {forbidden}"
            );
        }
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
        assert_eq!(
            status(&app, request(Method::POST, "/health", None)).await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status(&app, request(Method::GET, "/health/", None)).await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 2);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn admin_resource_prefix_is_a_strict_path_boundary() {
        assert!(matches!(
            classify(&Method::GET, "/api/v1/admin/subscription/plans"),
            AccessPolicy::AdminPermission(PLANS_READ_PERMISSION)
        ));
        assert!(matches!(
            classify(&Method::POST, "/api/v1/admin/subscription/plans"),
            AccessPolicy::AdminPermission(PLANS_MANAGE_PERMISSION)
        ));
        assert!(matches!(
            classify(&Method::GET, "/api/v1/admin/subscription/access"),
            AccessPolicy::AdminPermission(ACCESS_READ_PERMISSION)
        ));
        assert!(matches!(
            classify(&Method::POST, "/api/v1/admin/subscription/access/assign"),
            AccessPolicy::AdminPermission(ACCESS_MANAGE_PERMISSION)
        ));
        for path in [
            "/api/v1/admin/subscriptionfoo",
            "/api/v1/admin/subscription/plansfoo",
            "/api/v1/admin/subscription/plans/plan.id",
            "/api/v1/admin/subscription/plans/../",
            "/api/v1/admin/subscription/plans/%2e%2e",
            "/api/v1/admin/subscription/accessfoo",
        ] {
            assert_eq!(
                classify(&Method::GET, path),
                AccessPolicy::Blocked,
                "{path}"
            );
        }
    }

    #[tokio::test]
    async fn plan_reads_require_admin_audience_and_read_permission() {
        let (app, downstream) = app();
        for path in [
            "/api/v1/subscription/plans",
            "/api/v1/subscription/plans/plan-id",
        ] {
            assert_eq!(
                status(&app, request(Method::GET, path, None)).await,
                StatusCode::UNAUTHORIZED
            );
            assert_eq!(
                status(&app, request(Method::GET, path, Some("invalid"))).await,
                StatusCode::UNAUTHORIZED
            );
            for denied in [
                "admin-none",
                "admin-manage",
                "frontend-read",
                "other-audience",
                "admin-invalid-wildcard",
            ] {
                assert_eq!(
                    status(&app, request(Method::GET, path, Some(denied))).await,
                    StatusCode::FORBIDDEN
                );
            }
            for allowed in [
                "admin-read",
                "admin-resource-wildcard",
                "admin-domain-wildcard",
                "admin-global-wildcard",
            ] {
                assert_eq!(
                    status(&app, request(Method::GET, path, Some(allowed))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 8);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 8);
    }

    #[tokio::test]
    async fn plan_mutation_requires_admin_audience_and_manage_permission() {
        let (app, downstream) = app();
        let path = "/api/v1/subscription/plans";
        assert_eq!(
            status(&app, request(Method::POST, path, None)).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(&app, request(Method::POST, path, Some("invalid"))).await,
            StatusCode::UNAUTHORIZED
        );
        for denied in [
            "admin-none",
            "admin-read",
            "frontend-manage",
            "other-audience",
            "admin-invalid-wildcard",
        ] {
            assert_eq!(
                status(&app, request(Method::POST, path, Some(denied))).await,
                StatusCode::FORBIDDEN
            );
        }
        for allowed in [
            "admin-manage",
            "admin-resource-wildcard",
            "admin-domain-wildcard",
            "admin-global-wildcard",
        ] {
            assert_eq!(
                status(&app, request(Method::POST, path, Some(allowed))).await,
                StatusCode::OK
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 4);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn allowed_admin_requests_strip_spoofable_identity_headers() {
        let (app, downstream) = app();
        let mut req = request(
            Method::GET,
            "/api/v1/subscription/plans",
            Some("admin-read"),
        );
        req.headers_mut()
            .insert("x-user-id", "attacker".parse().unwrap());
        req.headers_mut()
            .insert("x-wallet-address", "0xattacker".parse().unwrap());
        req.headers_mut()
            .insert("x-permissions", "*:*".parse().unwrap());
        assert_eq!(status(&app, req).await, StatusCode::OK);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 1);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn owner_routes_fail_closed_before_verification_or_database_work() {
        let routes = [
            (Method::POST, "/api/v1/subscription/subscriptions"),
            (Method::GET, "/api/v1/subscription/subscriptions"),
            (Method::GET, "/api/v1/subscription/subscriptions/sub-id"),
            (
                Method::POST,
                "/api/v1/subscription/subscriptions/sub-id/cancel",
            ),
        ];
        let (app, downstream) = app();
        for bearer in [
            None,
            Some("invalid"),
            Some("frontend-read"),
            Some("admin-manage"),
        ] {
            for (method, path) in &routes {
                assert_eq!(
                    status(&app, request(method.clone(), path, bearer)).await,
                    StatusCode::NOT_FOUND
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn canned_zero_vault_is_not_exposed() {
        let (app, downstream) = app();
        for bearer in [None, Some("admin-read"), Some("admin-manage")] {
            assert_eq!(
                status(
                    &app,
                    request(Method::GET, "/api/v1/subscription/vault/56", bearer,),
                )
                .await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unknown_methods_paths_and_arity_fail_closed() {
        let cases = [
            (Method::HEAD, "/api/v1/subscription/plans"),
            (Method::PUT, "/api/v1/subscription/plans"),
            (Method::POST, "/api/v1/subscription/plans/plan-id"),
            (Method::GET, "/api/v1/subscription/plans/plan-id/extra"),
            (Method::GET, "/api/v1/subscription/plans/%2e%2e"),
            (Method::GET, "/api/v1/subscription//plans"),
            (
                Method::GET,
                "/api/v1/subscription/subscriptions/sub-id/cancel",
            ),
            (
                Method::POST,
                "/api/v1/subscription/subscriptions/sub-id/cancel/extra",
            ),
            (Method::GET, "/api/v1/subscription/vault/56/extra"),
            (Method::GET, "/api/v1/subscription/unknown"),
            (Method::GET, "/metrics"),
        ];
        let (app, downstream) = app();
        for (method, path) in cases {
            assert_eq!(
                status(&app, request(method, path, Some("admin-domain-wildcard"))).await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn production_requires_non_local_https_identity_endpoints() {
        assert!(build_auth_verifier(
            "http://127.0.0.1:8080",
            "http://127.0.0.1:8080/.well-known/jwks.json",
            true,
        )
        .is_err());
        assert!(build_auth_verifier(
            "http://127.0.0.1:8080",
            "http://127.0.0.1:8080/.well-known/jwks.json",
            false,
        )
        .is_ok());
    }
}
