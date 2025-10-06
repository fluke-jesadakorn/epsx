// Unified Router Architecture
// Single source of truth for all routing - replaces 3 competing router systems
// Supports both stateful (DomainContainer) and stateless (StatelessServiceFactory) modes

use axum::{
    routing::{get, post, delete},
    Router,
    response::Json,
    middleware as axum_middleware,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use axum::http::Method;

use crate::infrastructure::container::DomainContainer;

/// Unified Route Builder - Single source of truth for all application routes
/// Eliminates the need for multiple router implementations
pub struct UnifiedRouteBuilder {
    container: Arc<DomainContainer>,
}

impl UnifiedRouteBuilder {
    /// Create new unified router with domain container
    pub fn new(container: Arc<DomainContainer>) -> Self {
        Self { container }
    }

    /// Build complete router with all routes and middleware
    pub fn build(self) -> Router {
        // Create route groups
        let health_routes = self.create_health_routes();
        let auth_routes = self.create_auth_routes();
        let admin_routes = self.create_admin_routes();
        let analytics_routes = self.create_analytics_routes();
        let public_routes = self.create_public_routes();
        let user_routes = self.create_user_routes();
        let notification_routes = self.create_notification_routes();
        let docs_routes = crate::web::docs::create_docs_routes();
        let permission_authority_routes = self.create_permission_authority_routes();

        // Combine all routes
        let router = Router::new()
            // Core health endpoints (public, no auth)
            .merge(health_routes)

            // API documentation (public, no auth)
            .merge(docs_routes)

            // Permission Authority API (centralized permission validation)
            .merge(permission_authority_routes)
            .route("/api/permissions/health", get(|| async {
                Json(json!({"permission_authority": "ready", "integrated": true}))
            }))

            // Authentication routes
            .nest("/api/auth", auth_routes.clone())

            // API v1 routes (backward compatibility)
            .nest("/api/v1", Router::new()
                .nest("/analytics", analytics_routes.clone())
                .nest("/notifications", notification_routes.clone())
            )

            // Public API endpoints (no authentication)
            .nest("/api/v1/public", public_routes)

            // Admin routes (authenticated + permission validation)
            .nest("/admin", admin_routes.clone())
            .nest("/api/admin", admin_routes.clone()) // Alias for frontend compatibility
            .nest("/api/v1/admin", admin_routes) // Backward compatibility alias (legacy paths)

            // User routes (authenticated)
            .nest("/user", user_routes)

            // Analytics routes (legacy paths)
            .nest("/analytics", analytics_routes);

        // Apply middleware stack
        router
            .layer(axum_middleware::from_fn(
                crate::web::middleware::security_headers_middleware
            ))
            .layer(axum_middleware::from_fn(
                crate::web::middleware::request_id_middleware
            ))
            .layer(self.configure_cors())
    }

    // ============================================================================
    // HEALTH ROUTES
    // ============================================================================

    fn create_health_routes(&self) -> Router {
        Router::new()
            .route("/health", get(|| async {
                Json(json!({
                    "status": "healthy",
                    "service": "epsx-backend",
                    "timestamp": chrono::Utc::now(),
                    "router": "unified"
                }))
            }))
            .route("/readiness", get(|| async {
                Json(json!({"status": "ready"}))
            }))
            .route("/liveness", get(|| async {
                Json(json!({"status": "alive"}))
            }))
    }

    // ============================================================================
    // AUTHENTICATION ROUTES
    // ============================================================================

    fn create_auth_routes(&self) -> Router {
        use crate::web::auth::web3_handlers::{
            generate_challenge_handler,
            verify_signature_handler,
            logout_handler,
            get_session_handler,
            check_permission_handler,
            grant_permission_handler,
            revoke_permission_handler,
        };
        use crate::web::auth::AppState;

        let cache = self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            });

        let app_state = AppState::new(
            self.container.db_pool(),
            cache,
            self.container.clone(),
        );

        Router::new()
            // Web3 authentication endpoints
            .route("/web3/challenge", post(generate_challenge_handler))
            .route("/web3/verify", post(verify_signature_handler))
            .route("/web3/logout", delete(logout_handler))
            .route("/web3/session", get(get_session_handler))

            // Permission management
            .route("/web3/permissions/check", post(check_permission_handler))
            .route("/web3/permissions/grant", post(grant_permission_handler))
            .route("/web3/permissions/revoke", delete(revoke_permission_handler))

            // Health check
            .route("/health", get(|| async {
                Json(json!({"auth": "healthy", "type": "web3"}))
            }))

            .with_state(app_state)
    }

    // ============================================================================
    // ADMIN ROUTES
    // ============================================================================

    fn create_admin_routes(&self) -> Router {
        let cache = self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            });

        let app_state = crate::web::auth::AppState::new(
            self.container.db_pool(),
            cache,
            Arc::new((*self.container).clone()),
        );

        // Create admin routes with permission validation middleware
        let admin_routes = crate::web::admin::routes::create_admin_routes()
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::permission_validation_middleware
            ));

        let permission_authority_routes = crate::web::admin::routes::create_permission_authority_routes()
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::permission_validation_middleware
            ));

        Router::new()
            .route("/health", get(|| async {
                Json(json!({"admin": "healthy", "permission_authority": "active"}))
            }))
            .merge(admin_routes)
            .merge(permission_authority_routes)
    }

    // ============================================================================
    // ANALYTICS ROUTES
    // ============================================================================

    fn create_analytics_routes(&self) -> Router {
        Router::new()
            .route("/rankings", get(crate::web::analytics::eps_handlers::get_unified_analytics_rankings_cached))
            .route("/filters", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
            .route("/countries", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
            .route("/available-countries", get(crate::web::analytics::eps_handlers::get_available_countries))
            .route("/sectors", get(crate::web::analytics::eps_handlers::get_sectors_by_country))
            .route("/status", get(|| async { "analytics_ok" }))
    }

    // ============================================================================
    // PUBLIC ROUTES (no authentication required)
    // ============================================================================

    fn create_public_routes(&self) -> Router {
        let cache = self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            });

        let app_state = crate::web::auth::AppState::new(
            self.container.db_pool(),
            cache,
            Arc::new((*self.container).clone()),
        );

        Router::new()
            .nest("/analytics", Router::new()
                .route("/rankings", get(|| async {
                    Json(json!({
                        "success": true,
                        "data": [],
                        "pagination": {
                            "page": 1,
                            "limit": 5,
                            "total": 0,
                            "total_pages": 0
                        },
                        "message": "Public analytics data (limited)"
                    }))
                }))
                .route("/filters", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
                .route("/countries", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
            )
            .route("/plans", get(crate::web::public::plans_handlers::get_public_plans))
            .with_state(app_state)
    }

    // ============================================================================
    // USER ROUTES (authenticated users)
    // ============================================================================

    fn create_user_routes(&self) -> Router {
        Router::new()
            .route("/health", get(|| async {
                Json(json!({"user": "healthy"}))
            }))
    }

    // ============================================================================
    // NOTIFICATION ROUTES
    // ============================================================================

    fn create_notification_routes(&self) -> Router {
        let cache = self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            });

        let app_state = crate::web::auth::AppState::new(
            self.container.db_pool(),
            cache,
            Arc::new((*self.container).clone()),
        );

        Router::new()
            .route("/stream", get(crate::web::notifications::sse_notifications_handler))
            .route("/send", post(crate::web::notifications::send_sse_notification_handler))
            .route("/broadcast", post(crate::web::notifications::broadcast_sse_notification_handler))
            .route("/health", get(crate::web::notifications::sse_health_handler))
            .with_state(app_state)
    }

    // ============================================================================
    // PERMISSION AUTHORITY ROUTES
    // ============================================================================

    fn create_permission_authority_routes(&self) -> Router {
        let cache = self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            });

        let app_state = crate::web::auth::AppState::new(
            self.container.db_pool(),
            cache,
            Arc::new((*self.container).clone()),
        );

        crate::web::admin::routes::create_permission_authority_routes()
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::permission_validation_middleware
            ))
    }

    // ============================================================================
    // CORS CONFIGURATION
    // ============================================================================

    fn configure_cors(&self) -> CorsLayer {
        use tower_http::cors::Any;
        use axum::http::HeaderName;
        use std::time::Duration;

        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                // Standard HTTP headers
                HeaderName::from_static("accept"),
                HeaderName::from_static("authorization"),
                HeaderName::from_static("content-type"),
                HeaderName::from_static("origin"),
                HeaderName::from_static("referer"),
                // Custom API headers
                HeaderName::from_static("x-api-version"),
                HeaderName::from_static("x-request-id"),
                HeaderName::from_static("x-client-version"),
                HeaderName::from_static("x-admin-session"),
                // Next.js React Server Components
                HeaderName::from_static("rsc"),
                HeaderName::from_static("next-router-prefetch"),
                HeaderName::from_static("next-router-state-tree"),
                HeaderName::from_static("next-url"),
                HeaderName::from_static("purpose"),
                HeaderName::from_static("x-middleware-prefetch"),
                HeaderName::from_static("x-nextjs-data"),
                // Web3 authentication headers
                HeaderName::from_static("x-wallet-address"),
                HeaderName::from_static("x-chain-id"),
                HeaderName::from_static("x-web3-signature"),
                HeaderName::from_static("x-signed-message"),
                HeaderName::from_static("x-timestamp"),
                HeaderName::from_static("x-nonce"),
            ])
            .expose_headers([
                HeaderName::from_static("x-request-id"),
                HeaderName::from_static("x-rate-limit-remaining"),
                HeaderName::from_static("x-rate-limit-reset"),
            ])
            .allow_credentials(false) // Must be false when using Any origin
            .max_age(Duration::from_secs(86400))
    }
}
