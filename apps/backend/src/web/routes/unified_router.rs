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

    // ============================================================================
    // HELPER METHODS (DRY - Don't Repeat Yourself)
    // ============================================================================

    /// Get cache or create default MemoryCache - eliminates 7 duplicate patterns
    fn get_or_default_cache(&self) -> Arc<dyn crate::infrastructure::cache::Cache> {
        self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            })
    }

    /// Create AppState with container dependencies - eliminates 4 duplicate patterns
    fn create_app_state(&self) -> crate::web::auth::AppState {
        let cache = self.get_or_default_cache();
        let redis_pool = self.container.get_redis_pool();
        let redis_broadcaster = self.container.get_redis_broadcaster();

        crate::web::auth::AppState::new(
            self.container.db_pool(),
            cache,
            Arc::new((*self.container).clone()),
            redis_pool,
            redis_broadcaster,
        )
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
        let health_state = crate::web::health::HealthState {
            pool: self.container.db_pool(),
            cache: self.get_or_default_cache(),
        };

        Router::new()
            .route("/health", get(crate::web::health::health_check_handler))
            .with_state(health_state)
    }

    // ============================================================================
    // AUTHENTICATION ROUTES
    // ============================================================================

    fn create_auth_routes(&self) -> Router {
        use crate::web::auth::handlers::{
            generate_challenge_handler,
            verify_signature_handler,
            logout_handler,
            get_session_handler,
            check_permission_handler,
            grant_permission_handler,
            revoke_permission_handler,
        };

        // Get Redis services - optional, log warning if not available
        let redis_pool = self.container.get_redis_pool();
        let redis_broadcaster = self.container.get_redis_broadcaster();

        if redis_pool.is_none() || redis_broadcaster.is_none() {
            tracing::warn!("⚠️ Redis not configured - notifications and real-time features will not work");
        }

        let app_state = self.create_app_state();

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
        let app_state = self.create_app_state();

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
            .layer(Extension(self.get_or_default_cache()))
            .layer(Extension(eps_ranking_service))
    }

    // ============================================================================
    // PUBLIC ROUTES (no authentication required)
    // ============================================================================

    fn create_public_routes(&self) -> Router {
        let app_state = self.create_app_state();

        // Create TradingView service and EPS ranking service for public analytics
        let config = Arc::new(crate::config::get_fallback_config());
        let tradingview_service = Arc::new(crate::infrastructure::adapters::services::tradingview::TradingViewApiService::new(config));
        let eps_repository = Arc::new(crate::web::analytics::TradingViewEPSRepository::new(tradingview_service));
        let eps_ranking_service = Arc::new(crate::domain::shared_kernel::services::eps_ranking_service::EPSRankingService::new(eps_repository));

        Router::new()
            .nest("/analytics", Router::new()
                .route("/rankings", get(crate::web::analytics::eps_handlers::get_unified_analytics_rankings_cached))
                .route("/filters", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
                .route("/countries", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
                .layer(Extension(self.get_or_default_cache()))
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
            mark_all_notifications_read_handler,
            delete_notification_handler,
            clear_all_notifications_handler,
            get_unread_count_handler,
            acknowledge_notification_handler,
        };

        let app_state = self.create_app_state();

        // Create SSE route with permissive CORS (EventSource cannot send credentials)
        let sse_route = Router::new()
            .route("/stream", get(crate::web::notifications::sse_notifications_handler))
            .layer(Self::create_sse_cors_layer())
            .with_state(app_state.clone());

        // Other notification routes with normal auth
        let notification_routes = Router::new()
            .route("/", get(get_user_notifications_handler))
            .route("/unread-count", get(get_unread_count_handler))
            .route("/mark-all-read", put(mark_all_notifications_read_handler))
            .route("/clear-all", delete(clear_all_notifications_handler))
            .route("/:id/read", put(mark_notification_read_handler))
            .route("/:id/acknowledge", put(acknowledge_notification_handler))
            .route("/:id", delete(delete_notification_handler))
            .with_state(app_state);

        // Merge SSE and notification routes
        sse_route.merge(notification_routes)
    }

    // Create permissive CORS for EventSource connections
    fn create_sse_cors_layer() -> CorsLayer {
        use tower_http::cors::Any;
        use axum::http::HeaderName;
        use std::time::Duration;

        // EventSource cannot send custom headers or credentials
        // Use permissive CORS without credentials
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::OPTIONS])
            .allow_headers([
                HeaderName::from_static("accept"),
                HeaderName::from_static("cache-control"),
                HeaderName::from_static("content-type"),
            ])
            .expose_headers([
                HeaderName::from_static("content-type"),
                HeaderName::from_static("cache-control"),
            ])
            .allow_credentials(false)
            .max_age(Duration::from_secs(3600))
    }

    // ============================================================================
    // PERMISSION AUTHORITY ROUTES
    // ============================================================================

    fn create_permission_authority_routes(&self) -> Router {
        let app_state = self.create_app_state();

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
                    HeaderName::from_static("x-access-level"),
                    HeaderName::from_static("x-admin-context"),
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
                "http://localhost:3000".parse()
                    .expect("Static localhost:3000 URL should always parse"),
                "http://localhost:3001".parse()
                    .expect("Static localhost:3001 URL should always parse"),
                "http://127.0.0.1:3000".parse()
                    .expect("Static 127.0.0.1:3000 URL should always parse"),
                "http://127.0.0.1:3001".parse()
                    .expect("Static 127.0.0.1:3001 URL should always parse"),
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
                    HeaderName::from_static("x-access-level"),
                    HeaderName::from_static("x-admin-context"),
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
