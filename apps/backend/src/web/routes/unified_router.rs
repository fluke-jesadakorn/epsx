// Unified Router Architecture
// Single source of truth for all routing - replaces 3 competing router systems
// Supports both stateful (DomainContainer) and stateless (StatelessServiceFactory) modes

use axum::{
    routing::{get, post, delete, put},
    Router,
    middleware as axum_middleware,
    Extension,
};
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

        // Combine all routes - SIMPLIFIED STANDARD
        let router = Router::new()
            // Core health endpoints (public, no auth)
            .merge(health_routes)

            // API documentation (public, no auth)
            .merge(docs_routes)

            // All routes under /api prefix
            .nest("/api", Router::new()
                // Permission Authority (centralized permission validation)
                .merge(permission_authority_routes)

                // Authentication routes
                .nest("/auth", auth_routes)

                // Public API endpoints (no authentication)
                .nest("/public", public_routes)

                // User routes (authenticated)
                .nest("/user", user_routes)

                // Admin routes (authenticated + permission validation)
                .nest("/admin", admin_routes)

                // Analytics routes (authenticated)
                .nest("/analytics", analytics_routes.clone())

                // Notifications routes (authenticated)
                .nest("/notifications", notification_routes)
            )

            // Legacy analytics route (backward compatibility only)
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
        let cache = self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            });

        let health_state = crate::web::health::HealthState {
            pool: self.container.db_pool(),
            cache,
        };

        Router::new()
            .route("/health", get(crate::web::health::health_check_handler))
            .with_state(health_state)
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
            .merge(admin_routes)
            .merge(permission_authority_routes)
    }

    // ============================================================================
    // ANALYTICS ROUTES
    // ============================================================================

    fn create_analytics_routes(&self) -> Router {
        let cache = self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            });

        // Create TradingView service and EPS ranking service
        let config = Arc::new(crate::config::get_fallback_config());
        let tradingview_service = Arc::new(crate::infrastructure::adapters::services::tradingview::TradingViewApiService::new(config));
        let eps_repository = Arc::new(crate::web::analytics::TradingViewEPSRepository::new(tradingview_service));
        let eps_ranking_service = Arc::new(crate::domain::shared_kernel::services::eps_ranking_service::EPSRankingService::new(eps_repository));

        Router::new()
            .route("/rankings", get(crate::web::analytics::eps_handlers::get_unified_analytics_rankings_cached))
            .route("/filters", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
            .route("/countries", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
            .route("/available-countries", get(crate::web::analytics::eps_handlers::get_available_countries))
            .route("/sectors", get(crate::web::analytics::eps_handlers::get_sectors_by_country))
            .layer(Extension(cache))
            .layer(Extension(eps_ranking_service))
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

        // Create TradingView service and EPS ranking service for public analytics
        let config = Arc::new(crate::config::get_fallback_config());
        let tradingview_service = Arc::new(crate::infrastructure::adapters::services::tradingview::TradingViewApiService::new(config));
        let eps_repository = Arc::new(crate::web::analytics::TradingViewEPSRepository::new(tradingview_service));
        let eps_ranking_service = Arc::new(crate::domain::shared_kernel::services::eps_ranking_service::EPSRankingService::new(eps_repository));

        let cache_clone = self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            });

        Router::new()
            .nest("/analytics", Router::new()
                .route("/rankings", get(crate::web::analytics::eps_handlers::get_unified_analytics_rankings_cached))
                .route("/filters", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
                .route("/countries", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
                .layer(Extension(cache_clone))
                .layer(Extension(eps_ranking_service))
            )
            .route("/plans", get(crate::web::public::plans_handlers::get_public_plans))
            .route("/plans/seed", post(crate::web::public::seed_plans_handler::seed_subscription_plans))
            .with_state(app_state)
    }

    // ============================================================================
    // USER ROUTES (authenticated users)
    // ============================================================================

    fn create_user_routes(&self) -> Router {
        Router::new()
            // TODO: User profile routes need proper middleware setup
            // User handlers exist in src/web/user/unified_user_handlers.rs but require:
            // 1. Web3 auth middleware to extract user context from Bearer token
            // 2. Permission validation middleware
            // Available handlers:
            //   - get_current_user_profile -> /api/v1/wallet/profile
            //   - get_user_permissions -> /api/v1/wallet/permissions
            //   - update_user_preferences -> /api/v1/wallet/preferences
            //
            // TODO: Implement watchlist, alerts, and push subscription handlers
            // Missing handlers that frontend expects:
            //   - /api/v1/user/watchlist (GET, POST, DELETE)
            //   - /api/v1/user/alerts (GET, POST, DELETE)
            //   - /api/v1/user/push-subscription (POST, DELETE)
            //
            // For now, these routes return 404 until proper implementation
    }

    // ============================================================================
    // NOTIFICATION ROUTES
    // ============================================================================

    fn create_notification_routes(&self) -> Router {
        use crate::web::admin::notification_handlers::{
            get_user_notifications_handler,
            mark_notification_read_handler,
            delete_notification_handler,
            get_unread_count_handler,
        };

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
            // SSE real-time stream
            .route("/stream", get(crate::web::notifications::sse_notifications_handler))

            // User notification management (authenticated)
            .route("/", get(get_user_notifications_handler))
            .route("/unread-count", get(get_unread_count_handler))
            .route("/:id/read", put(mark_notification_read_handler))
            .route("/:id", delete(delete_notification_handler))

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
        use axum::http::{HeaderName, HeaderValue};
        use std::time::Duration;
        use crate::config::env::is_production;

        if is_production() {
            // Production: Use Any origin without credentials
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
                    HeaderName::from_static("accept"),
                    HeaderName::from_static("authorization"),
                    HeaderName::from_static("content-type"),
                    HeaderName::from_static("origin"),
                    HeaderName::from_static("referer"),
                    HeaderName::from_static("x-api-version"),
                    HeaderName::from_static("x-request-id"),
                    HeaderName::from_static("x-client-version"),
                    HeaderName::from_static("x-admin-session"),
                    HeaderName::from_static("rsc"),
                    HeaderName::from_static("next-router-prefetch"),
                    HeaderName::from_static("next-router-state-tree"),
                    HeaderName::from_static("next-url"),
                    HeaderName::from_static("purpose"),
                    HeaderName::from_static("x-middleware-prefetch"),
                    HeaderName::from_static("x-nextjs-data"),
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
                .allow_credentials(false)
                .max_age(Duration::from_secs(86400))
        } else {
            // Development: Use specific origins with credentials
            let origins: Vec<HeaderValue> = vec![
                "http://localhost:3000".parse().unwrap(),
                "http://localhost:3001".parse().unwrap(),
                "http://127.0.0.1:3000".parse().unwrap(),
                "http://127.0.0.1:3001".parse().unwrap(),
            ];

            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    HeaderName::from_static("accept"),
                    HeaderName::from_static("authorization"),
                    HeaderName::from_static("content-type"),
                    HeaderName::from_static("origin"),
                    HeaderName::from_static("referer"),
                    HeaderName::from_static("x-api-version"),
                    HeaderName::from_static("x-request-id"),
                    HeaderName::from_static("x-client-version"),
                    HeaderName::from_static("x-admin-session"),
                    HeaderName::from_static("rsc"),
                    HeaderName::from_static("next-router-prefetch"),
                    HeaderName::from_static("next-router-state-tree"),
                    HeaderName::from_static("next-url"),
                    HeaderName::from_static("purpose"),
                    HeaderName::from_static("x-middleware-prefetch"),
                    HeaderName::from_static("x-nextjs-data"),
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
                .allow_credentials(true) // Can allow credentials with specific origins
                .max_age(Duration::from_secs(3600))
        }
    }
}
