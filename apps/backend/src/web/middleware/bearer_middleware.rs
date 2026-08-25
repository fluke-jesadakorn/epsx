// ============================================================================
// OPENID BEARER TOKEN AUTHENTICATION MIDDLEWARE
// Standard OpenID Connect Bearer token validation for all API requests
// ============================================================================

//! CORE PRINCIPLES:
//! - Uses OpenIDTokenService::validate_access_token() as SINGLE SOURCE OF TRUTH
//! - User context extraction from validated tokens
//! - Unified error responses for authentication failures

use axum::{
    extract::{Request, State},
    http::{
        header::{AUTHORIZATION, WWW_AUTHENTICATE},
        HeaderMap, HeaderValue, StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use crate::{
    auth::OpenIDTokenError,
    domain::developer_portal::{DeveloperEntitlementService, EffectiveApiRateLimits},
    infrastructure::adapters::repositories::developer_portal::ApiKeyRepository,
    infrastructure::cache::redis_cache::get_perm_invalidated,
    web::auth::AppState,
};

#[derive(Debug, Clone)]
pub struct ApiKeyIdentity {
    pub id: Uuid,
    pub effective_scopes: Vec<String>,
    pub rate_limits: EffectiveApiRateLimits,
}

/// OpenID Bearer Token User Context
/// Extracted from validated JWT access tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenIDUserContext {
    /// Subject (wallet address)
    pub sub: String,
    /// Wallet address (primary identifier)
    pub wallet_address: String,
    /// User permissions from token scope claim
    pub permissions: Vec<String>,
    /// Audiences retained from a server-validated JWT.
    ///
    /// API-key authentication sets this to `None`, allowing route-scoped
    /// middleware to require a JWT for sensitive reads without changing the
    /// authentication policy of existing routes.
    #[serde(default, skip_serializing, skip_deserializing)]
    pub token_audiences: Option<Vec<String>>,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub api_key: Option<ApiKeyIdentity>,
    /// Authentication method
    pub auth_method: String,
    /// JWT ID (unique token identifier)
    pub jti: String,
    /// Token expiration timestamp
    pub exp: i64,
    /// Issued at timestamp
    pub iat: i64,
    /// Authentication time
    pub auth_time: i64,
}

/// Unified API Error Response
#[derive(Debug, Serialize)]
pub struct UnifiedErrorResponse {
    pub success: bool,
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: u16,
    pub message: String,
    pub reason: String,
}

impl UnifiedErrorResponse {
    /// Create a new error response
    pub fn new(code: u16, message: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            success: false,
            error: ErrorDetails {
                code,
                message: message.into(),
                reason: reason.into(),
            },
        }
    }

    /// Wrap in Json for Axum handler returns
    pub fn json(code: u16, message: impl Into<String>, reason: impl Into<String>) -> Json<Self> {
        Json(Self::new(code, message, reason))
    }
}

impl IntoResponse for UnifiedErrorResponse {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.error.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

/// OpenID Bearer Token Authentication Middleware
/// Validates JWT Bearer tokens and extracts user context
pub async fn bearer_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<UnifiedErrorResponse>)> {
    if request.extensions().get::<OpenIDUserContext>().is_some() {
        return Ok(next.run(request).await);
    }
    let token = match extract_bearer_token_from_headers(request.headers()) {
        Some(token) => token,
        None => {
            debug!("No Bearer token found in Authorization header or auth cookies");
            return Err(create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Bearer token required",
                "Authorization Bearer token or HttpOnly auth cookie required",
            ));
        }
    };

    if token.is_empty() {
        debug!("Empty Bearer token");
        return Err(create_auth_error(
            StatusCode::UNAUTHORIZED,
            "Invalid token format",
            "Bearer token cannot be empty",
        ));
    }

    // Try JWT first (fast, no DB), then fall back to API key validation
    let user_context = match validate_bearer_token(&token, &app_state).await {
        Ok(context) => context,
        Err(_) => {
            // JWT failed — try API key fallback (SHA-256 hash + DB lookup)
            match validate_api_key(&token, &app_state).await {
                Ok(context) => context,
                Err((status, err)) => return Err((status, err)),
            }
        }
    };

    debug!(
        "Bearer token validated successfully for user: {} (method: {})",
        user_context.wallet_address, user_context.auth_method
    );

    // Add user context to request extensions
    request.extensions_mut().insert(user_context);

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Require a server-validated JWT whose audience is exactly `epsx-admin`.
///
/// This guard is deliberately route-scoped. The shared bearer middleware
/// continues to support API keys and other configured JWT audiences for
/// existing routes, while selected admin reads can require the stricter
/// browser-admin credential boundary.
pub async fn require_exact_admin_audience(request: Request, next: Next) -> Response {
    let is_exact_admin = request
        .extensions()
        .get::<OpenIDUserContext>()
        .and_then(|context| context.token_audiences.as_deref())
        .is_some_and(|audiences| matches!(audiences, [audience] if audience == "epsx-admin"));

    if is_exact_admin {
        return next.run(request).await;
    }

    let mut response = create_auth_error(
        StatusCode::UNAUTHORIZED,
        "Admin bearer token required",
        "A valid admin access token is required for this endpoint",
    )
    .into_response();
    response.headers_mut().insert(
        WWW_AUTHENTICATE,
        HeaderValue::from_static("Bearer realm=\"admin\", error=\"invalid_token\""),
    );
    response
}

/// Developer-portal management is a browser-session surface. API keys and
/// tokens minted for another audience are never valid management credentials.
pub async fn require_exact_frontend_audience(request: Request, next: Next) -> Response {
    let is_exact_frontend = request
        .extensions()
        .get::<OpenIDUserContext>()
        .and_then(|context| context.token_audiences.as_deref())
        .is_some_and(|audiences| matches!(audiences, [audience] if audience == "epsx-frontend"));

    if is_exact_frontend {
        return next.run(request).await;
    }

    let mut response = create_auth_error(
        StatusCode::UNAUTHORIZED,
        "Frontend session token required",
        "A valid epsx-frontend JWT is required for developer portal management",
    )
    .into_response();
    response.headers_mut().insert(
        WWW_AUTHENTICATE,
        HeaderValue::from_static("Bearer realm=\"frontend\", error=\"invalid_token\""),
    );
    response
}

/// Extract an access token from an Authorization header or EPSX HttpOnly auth cookie.
///
/// Browser clients cannot set Authorization headers for EventSource and should not receive
/// access tokens during hydration. Supporting the HttpOnly cookie keeps those flows server-owned.
pub fn extract_bearer_token_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(auth_header) = headers
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            let token = token.trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }

    let cookie_header = headers
        .get("cookie")
        .and_then(|header| header.to_str().ok())?;
    for cookie in cookie_header.split(';') {
        let (name, value) = match cookie.trim().split_once('=') {
            Some(parts) => parts,
            None => continue,
        };
        if matches!(
            name.trim(),
            "epsx.access_token" | "__Host-epsx.access_token"
        ) {
            let token = value.trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }

    None
}

/// Validate Bearer JWT token and extract user context
///
/// Uses OpenIDTokenService::validate_access_token() as the SINGLE SOURCE OF TRUTH
/// for all JWT validation. This ensures consistent validation across the entire application.
pub async fn validate_bearer_token(
    token: &str,
    app_state: &AppState,
) -> Result<OpenIDUserContext, OpenIDTokenError> {
    // Get OpenID token service for validation
    let token_service = app_state
        .domain_container
        .get_token_service()
        .ok_or_else(|| {
            OpenIDTokenError::TokenGenerationFailed("Token service not available".to_string())
        })?;

    // Use the SINGLE SOURCE OF TRUTH for token validation
    let claims = token_service.validate_access_token(token).await?;

    // Parse permissions from OIDC standard scope claim
    // OIDC standard: scope is space-separated string like "openid profile epsx:analytics:read admin:users:manage"
    let permissions: Vec<String> = claims
        .scope
        .split_whitespace()
        .filter(|s| *s != "openid" && *s != "profile") // Remove standard OIDC scopes
        .map(|s| s.to_string())
        .collect();

    // Extract user context from claims
    let mut user_context = OpenIDUserContext {
        sub: claims.sub,
        wallet_address: claims.wallet_address,
        permissions, // Parsed from OIDC scope claim
        token_audiences: Some(claims.aud),
        api_key: None,
        auth_method: claims.auth_method,
        jti: claims.jti,
        exp: claims.exp,
        iat: claims.iat,
        auth_time: claims.auth_time,
    };

    // Check if permissions were invalidated after this token was issued.
    // If so, fetch live permissions from DB to reflect the change immediately.
    // Fail closed if the live permission reload fails; stale admin permissions must not survive revocation.
    if let Some(invalidated_at) =
        get_perm_invalidated(app_state.cache.as_ref(), &user_context.wallet_address)
    {
        if invalidated_at > user_context.iat {
            let fresh_perms = token_service
                .expand_plans(&user_context.wallet_address)
                .await
                .map_err(|e| {
                    OpenIDTokenError::DatabaseError(format!(
                        "Permission reload failed after invalidation: {}",
                        e
                    ))
                })?;
            debug!(
                "Live permissions loaded for {} ({} perms) due to invalidation flag",
                user_context.wallet_address,
                fresh_perms.len()
            );
            user_context.permissions = fresh_perms;
        }
    }

    debug!(
        "JWT token validated for user: {} (permissions: {})",
        user_context.wallet_address,
        user_context.permissions.len()
    );

    Ok(user_context)
}

/// Create standardized authentication error response
fn create_auth_error(
    status: StatusCode,
    message: &str,
    reason: &str,
) -> (StatusCode, Json<UnifiedErrorResponse>) {
    let error_response = UnifiedErrorResponse {
        success: false,
        error: ErrorDetails {
            code: status.as_u16(),
            message: message.to_string(),
            reason: reason.to_string(),
        },
    };

    (status, Json(error_response))
}

/// Validate a Bearer token as an API key (SHA-256 hash → DB lookup)
/// Returns OpenIDUserContext with auth_method = "api_key" on success
async fn validate_api_key(
    token: &str,
    app_state: &AppState,
) -> Result<OpenIDUserContext, (StatusCode, Json<UnifiedErrorResponse>)> {
    let repo = ApiKeyRepository::new(app_state.db_pool.clone());

    let api_key = match repo.validate_key(token).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            return Err(create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid token",
                "authentication_failed",
            ));
        }
        Err(_) => {
            return Err(create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid token",
                "authentication_failed",
            ));
        }
    };

    // Check expiration
    if let Some(exp) = api_key.expires_at {
        if exp < chrono::Utc::now() {
            return Err(create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Token expired",
                "token_expired",
            ));
        }
    }

    // Check status (validate_key filters active, but guard against race)
    if !matches!(
        api_key.status,
        crate::domain::developer_portal::ApiKeyStatus::Active
    ) {
        return Err(create_auth_error(
            StatusCode::UNAUTHORIZED,
            "Token revoked",
            "token_revoked",
        ));
    }

    // Re-evaluate delegated scopes against the owner's live normalized grants
    // on every request. Downgrade, expiry, or removal takes effect at once.
    let entitlement_service = DeveloperEntitlementService::new(*app_state.db_pool);
    let (permissions, entitlement) = entitlement_service
        .effective_key_scopes(&api_key.wallet_address, &api_key.selected_permissions)
        .await
        .map_err(|_| {
            create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid token",
                "authentication_failed",
            )
        })?;
    if permissions.is_empty() || !entitlement.has_active_api_entitlement {
        return Err(create_auth_error(
            StatusCode::UNAUTHORIZED,
            "API entitlement inactive",
            "api_entitlement_inactive",
        ));
    }

    let now = chrono::Utc::now();
    let ctx = OpenIDUserContext {
        sub: api_key.wallet_address.clone(),
        wallet_address: api_key.wallet_address,
        permissions: permissions.clone(),
        token_audiences: None,
        api_key: Some(ApiKeyIdentity {
            id: api_key.id,
            effective_scopes: permissions.clone(),
            rate_limits: entitlement.rate_limits,
        }),
        auth_method: "api_key".to_string(),
        jti: api_key.id.to_string(),
        exp: api_key
            .expires_at
            .map(|e| e.timestamp())
            .unwrap_or(i64::MAX),
        iat: api_key.created_at.timestamp(),
        auth_time: now.timestamp(),
    };

    debug!(
        "API key validated for user: {} (key: {}, permissions: {})",
        ctx.wallet_address,
        api_key.key_prefix,
        ctx.permissions.len()
    );

    Ok(ctx)
}

/// Optional Bearer Token Middleware (for public/optional auth endpoints)
/// Does not fail if no token is present, but validates if token exists
pub async fn optional_bearer_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    if request.extensions().get::<OpenIDUserContext>().is_some() {
        return next.run(request).await;
    }
    // Try to extract and validate token, but don't fail if missing
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(header) = auth_header {
        let Some(token) = header
            .strip_prefix("Bearer ")
            .filter(|token| !token.is_empty())
        else {
            return create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid token format",
                "Authorization must contain a non-empty Bearer credential",
            )
            .into_response();
        };
        // Try JWT first, then API key fallback
        match validate_bearer_token(token, &app_state).await {
            Ok(ctx) => {
                debug!(
                    "Optional auth: JWT validated for user: {}",
                    ctx.wallet_address
                );
                request.extensions_mut().insert(ctx);
            }
            Err(_) => {
                // A presented credential must never silently degrade to
                // anonymous access when validation fails.
                match validate_api_key(token, &app_state).await {
                    Ok(ctx) => {
                        debug!(
                            "Optional auth: API key validated for user: {}",
                            ctx.wallet_address
                        );
                        request.extensions_mut().insert(ctx);
                    }
                    Err(error) => return error.into_response(),
                }
            }
        }
    }

    next.run(request).await
}

/// Helper to extract user context from request
pub fn extract_user_context(request: &Request) -> Option<&OpenIDUserContext> {
    request.extensions().get::<OpenIDUserContext>()
}

/// Helper to require user context (for use in handlers)
pub fn require_user_context(
    request: &Request,
) -> Result<&OpenIDUserContext, (StatusCode, Json<UnifiedErrorResponse>)> {
    extract_user_context(request).ok_or_else(|| {
        create_auth_error(
            StatusCode::UNAUTHORIZED,
            "Authentication required",
            "Valid Bearer token required for this endpoint",
        )
    })
}

/// Helper to check if user has specific permission
/// Uses exact match + wildcard matching (platform:*:* and platform:resource:*)
pub fn check_user_permission(user_context: &OpenIDUserContext, required_permission: &str) -> bool {
    epsx_contracts::permissions::has_permission(&user_context.permissions, required_permission)
}

/// Helper to create permission denied error
pub fn create_permission_denied_error(
    required_permission: &str,
) -> (StatusCode, Json<UnifiedErrorResponse>) {
    create_auth_error(
        StatusCode::FORBIDDEN,
        "Permission denied",
        &format!("Required permission: {}", required_permission),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_user_permission() {
        let user_context = OpenIDUserContext {
            sub: "0x123".to_string(),
            wallet_address: "0x123".to_string(),
            permissions: vec![
                "epsx:analytics:read".to_string(),
                "epsx:export:csv".to_string(),
            ],
            token_audiences: Some(vec!["epsx-frontend".to_string()]),
            api_key: None,
            auth_method: "web3_siwe".to_string(),
            jti: "test".to_string(),
            exp: 0,
            iat: 0,
            auth_time: 0,
        };

        assert!(check_user_permission(&user_context, "epsx:analytics:read"));
        assert!(!check_user_permission(&user_context, "admin:users:manage"));
    }

    #[test]
    fn test_admin_wildcard_permission() {
        let admin_context = OpenIDUserContext {
            sub: "0x456".to_string(),
            wallet_address: "0x456".to_string(),
            permissions: vec!["admin:*:*".to_string()],
            token_audiences: Some(vec!["epsx-admin".to_string()]),
            api_key: None,
            auth_method: "web3_siwe".to_string(),
            jti: "test".to_string(),
            exp: 0,
            iat: 0,
            auth_time: 0,
        };

        // admin:*:* grants all admin permissions
        assert!(check_user_permission(&admin_context, "admin:users:manage"));
        assert!(check_user_permission(
            &admin_context,
            "admin:permissions:read"
        ));
        // admin:*:* does NOT grant cross-platform permissions
        assert!(!check_user_permission(
            &admin_context,
            "epsx:analytics:read"
        ));
    }

    #[test]
    fn test_resource_wildcard_permission() {
        let ctx = OpenIDUserContext {
            sub: "0x789".to_string(),
            wallet_address: "0x789".to_string(),
            permissions: vec!["admin:users:*".to_string()],
            token_audiences: Some(vec!["epsx-admin".to_string()]),
            api_key: None,
            auth_method: "web3_siwe".to_string(),
            jti: "test".to_string(),
            exp: 0,
            iat: 0,
            auth_time: 0,
        };

        assert!(check_user_permission(&ctx, "admin:users:read"));
        assert!(check_user_permission(&ctx, "admin:users:manage"));
        // admin:users:* does NOT grant admin:permissions:read
        assert!(!check_user_permission(&ctx, "admin:permissions:read"));
    }

    fn guard_context(
        token_audiences: Option<Vec<&str>>,
        permissions: Vec<&str>,
    ) -> OpenIDUserContext {
        OpenIDUserContext {
            sub: "0x123".to_string(),
            wallet_address: "0x123".to_string(),
            permissions: permissions.into_iter().map(str::to_string).collect(),
            token_audiences: token_audiences
                .map(|audiences| audiences.into_iter().map(str::to_string).collect()),
            api_key: None,
            auth_method: "web3_siwe".to_string(),
            jti: "test".to_string(),
            exp: 9_999_999_999,
            iat: 1,
            auth_time: 1,
        }
    }

    async fn guarded_status(
        context: Option<OpenIDUserContext>,
    ) -> (StatusCode, Option<String>, String) {
        use axum::{
            body::Body,
            middleware::{from_fn, from_fn_with_state},
            routing::get,
            Router,
        };
        use tower::ServiceExt;

        async fn ok_handler() -> StatusCode {
            StatusCode::OK
        }

        let app = Router::new()
            .route("/test", get(ok_handler))
            .layer(from_fn_with_state(
                "admin:dashboard:view",
                crate::web::middleware::permission_validation_middleware::perm_guard,
            ))
            .layer(from_fn(require_exact_admin_audience));
        let mut request = axum::http::Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        if let Some(context) = context {
            request.extensions_mut().insert(context);
        }

        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let challenge = response
            .headers()
            .get(WWW_AUTHENTICATE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, challenge, String::from_utf8(body.to_vec()).unwrap())
    }

    async fn frontend_guard_status(context: OpenIDUserContext) -> StatusCode {
        use axum::{body::Body, middleware::from_fn, routing::get, Router};
        use tower::ServiceExt;

        let app = Router::new()
            .route("/test", get(|| async { StatusCode::OK }))
            .layer(from_fn(require_exact_frontend_audience));
        let mut request = axum::http::Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(context);
        app.oneshot(request).await.unwrap().status()
    }

    #[test]
    fn token_audiences_are_server_only_context() {
        let context = guard_context(Some(vec!["epsx-admin"]), vec!["admin:dashboard:view"]);
        let serialized = serde_json::to_value(context).unwrap();

        assert!(serialized.get("token_audiences").is_none());
        assert!(serialized.get("api_key").is_none());
    }

    #[tokio::test]
    async fn exact_admin_guard_rejects_missing_api_key_and_non_exact_audiences() {
        let denied = [
            None,
            Some(guard_context(None, vec!["admin:dashboard:view"])),
            Some(guard_context(Some(vec![]), vec!["admin:dashboard:view"])),
            Some(guard_context(
                Some(vec!["epsx-frontend"]),
                vec!["admin:dashboard:view"],
            )),
            Some(guard_context(
                Some(vec!["epsx-api"]),
                vec!["admin:dashboard:view"],
            )),
            Some(guard_context(
                Some(vec!["epsx-admin", "epsx-frontend"]),
                vec!["admin:dashboard:view"],
            )),
            Some(guard_context(
                Some(vec!["epsx-admin", "epsx-admin"]),
                vec!["admin:dashboard:view"],
            )),
        ];

        for context in denied {
            let (status, challenge, body) = guarded_status(context).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(
                challenge.as_deref(),
                Some("Bearer realm=\"admin\", error=\"invalid_token\"")
            );
            assert!(!body.contains("epsx-admin"));
            assert!(!body.contains("epsx-frontend"));
            assert!(!body.contains("epsx-api"));
        }
    }

    #[tokio::test]
    async fn exact_admin_guard_runs_before_permission_guard() {
        let wrong_audience_without_permission =
            guard_context(Some(vec!["epsx-frontend"]), vec!["epsx:analytics:read"]);
        let exact_audience_without_permission =
            guard_context(Some(vec!["epsx-admin"]), vec!["epsx:analytics:read"]);
        let exact_audience_with_permission =
            guard_context(Some(vec!["epsx-admin"]), vec!["admin:dashboard:view"]);

        assert_eq!(
            guarded_status(Some(wrong_audience_without_permission))
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            guarded_status(Some(exact_audience_without_permission))
                .await
                .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            guarded_status(Some(exact_audience_with_permission)).await.0,
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn developer_management_accepts_only_exact_frontend_jwt_audience() {
        assert_eq!(
            frontend_guard_status(guard_context(Some(vec!["epsx-frontend"]), vec![])).await,
            StatusCode::OK
        );
        for audiences in [
            None,
            Some(vec!["epsx-api"]),
            Some(vec!["epsx-frontend", "epsx-api"]),
        ] {
            assert_eq!(
                frontend_guard_status(guard_context(audiences, vec![])).await,
                StatusCode::UNAUTHORIZED
            );
        }
    }
}
