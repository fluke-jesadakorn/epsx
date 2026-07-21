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
