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
        let analytics_pool = self.container.get_analytics_pool();

        crate::web::auth::AppState::new(
            self.container.db_pool(),
            cache,
            Arc::clone(&self.container),
            redis_pool,
            redis_broadcaster,
            analytics_pool,
        )
    }

    /// Build complete router with all routes and middleware
    pub fn build(self) -> Router {
        // Create route plans
        let health_routes = self.create_health_routes();
        let auth_routes = self.create_auth_routes();
        let admin_routes = self.create_admin_routes();
        let analytics_routes = self.create_analytics_routes();
        let public_routes = self.create_public_routes();
        let user_routes = self.create_user_routes();
        let chat_routes = self.create_chat_routes();
        let notification_routes = self.create_notification_routes();
        let payment_routes = self.create_payment_routes();
        let developer_portal_routes = self.create_developer_portal_routes();
        let docs_routes = crate::web::docs::create_docs_routes();
        let permission_authority_routes = self.create_permission_authority_routes();

        // Combine all routes - MIXED /api/ and /api/ STRUCTURE
        let router = Router::new()
            // Core health endpoints (public, no auth)
            .merge(health_routes)

            // API documentation (public, no auth)
            .merge(docs_routes)

            // Authentication routes (Web3-first auth)
            .nest("/api/auth", auth_routes)

            // All routes under standardized /api/ prefix
            .nest("/api", Router::new()
                // Permission Authority (centralized permission validation - ALL apps use this)
                .nest("/permissions", permission_authority_routes)

                // Public API endpoints (no authentication)
                .nest("/public", public_routes)

                // User management routes (authenticated users)
                .nest("/users", user_routes)

                // Admin-only routes (authenticated + admin permissions)
                .nest("/admin", admin_routes)

                // Analytics routes (authenticated)
                .nest("/analytics", analytics_routes)

                // Notifications routes (authenticated)
                .nest("/notifications", notification_routes)

                // Payment validation and management routes (authenticated)
                .nest("/payments", payment_routes)

                // Plan and billing routes (authenticated)
                .nest("/plans", self.create_plan_routes())

                // Support Chat (authenticated users)
                .nest("/chat", chat_routes)

                // Developer Portal (user-facing API key management)
                .nest("/developer-portal", developer_portal_routes)
            );

        // Apply middleware stack
        router
            .layer(axum_middleware::from_fn(
                crate::web::middleware::security_headers_middleware
            ))
            .layer(axum_middleware::from_fn(
                crate::web::middleware::request_id_middleware
            ))
            .layer(axum_middleware::from_fn_with_state(
                self.container.clone(),
                crate::web::middleware::usage_tracking_middleware
            ))
            .layer(axum_middleware::from_fn_with_state(
                self.container.clone(),
                crate::web::middleware::unified_rate_limit_middleware
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
            get_user_permissions_handler,
            refresh_token_handler,
        };


        // Get Redis services - optional, log warning if not available
        let redis_pool = self.container.get_redis_pool();
        let redis_broadcaster = self.container.get_redis_broadcaster();

        if redis_pool.is_none() || redis_broadcaster.is_none() {
            tracing::warn!("Redis not configured - notifications and real-time features will not work");
        }

        let app_state = self.create_app_state();

        Router::new()
            // Web3 authentication endpoints
            .route("/web3/challenge", post(generate_challenge_handler))
            .route("/web3/verify", post(verify_signature_handler))
            .route("/web3/logout", delete(logout_handler))
            .route("/web3/session", get(get_session_handler))
            .route("/session/refresh", post(refresh_token_handler))

            // Permission management
            .route("/web3/permissions/check", post(check_permission_handler))
            .route("/web3/permissions/grant", post(grant_permission_handler))

            .route("/web3/permissions/revoke", delete(revoke_permission_handler))
            // Plan permissions route
            .route("/web3/plans/permissions/{wallet_address}", get(get_user_permissions_handler))

            .with_state(app_state)
    }

    // ============================================================================
    // ADMIN ROUTES
    // ============================================================================

    fn create_admin_routes(&self) -> Router {

        let redis_pool = self.container.redis_pool.clone();
        let redis_broadcaster = self.container.redis_broadcaster.clone();
        let cache = self.container.cache.clone().ok_or_else(|| {
            std::io::Error::other("Cache not configured")
        }).unwrap();

        let app_state = crate::web::auth::AppState::new(
            self.container.db_pool.clone(),
            cache.clone(),
            Arc::clone(&self.container),
            redis_pool.clone(),
            redis_broadcaster.clone(),
            self.container.get_analytics_pool(),
        );

        // Settings routes (protected - requires auth + admin permissions)
        let settings_routes = Router::new()
            .route("/settings", get(crate::web::admin::system_settings_handlers::get_all_settings_handler).put(crate::web::admin::system_settings_handlers::update_settings_handler))
            .route("/settings/reset", post(crate::web::admin::system_settings_handlers::reset_settings_handler))
            .route("/settings/{category}", get(crate::web::admin::system_settings_handlers::get_settings_by_category_handler))
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn(
                crate::web::middleware::permission_validation_middleware
            ))
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::bearer_middleware
            ));

        // Create admin routes with permission validation middleware
        let admin_routes = crate::web::admin::routes::create_admin_routes()
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn(
                crate::web::middleware::permission_validation_middleware
            ))
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::bearer_middleware
            ));

        Router::new()
            .merge(settings_routes)
            .merge(admin_routes)
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
        let permission_service = self.container.get_unified_permission_service()
            .unwrap_or_else(|| Arc::new(crate::auth::UnifiedPermissionService::new_without_cache(*self.container.db_pool())));

        let app_state = self.create_app_state();

        Router::new()
            .route("/rankings", get(crate::web::analytics::eps_handlers::get_unified_analytics_rankings_cached))
            .route("/filters", get(crate::web::analytics::eps_handlers::get_filter_options))
            .route("/countries", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
            .route("/available-countries", get(crate::web::analytics::eps_handlers::get_available_countries))
            .route("/sectors", get(crate::web::analytics::eps_handlers::get_sectors_by_country))
            .layer(Extension(self.get_or_default_cache()))
            .layer(Extension(eps_ranking_service))
            .layer(Extension(permission_service))
            .layer(axum_middleware::from_fn_with_state(
                "epsx:analytics:read",
                crate::web::middleware::perm_guard
            ))
            .layer(axum_middleware::from_fn_with_state(
                app_state,
                crate::web::middleware::optional_bearer_middleware
            ))
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

        let redis_pool = self.container.get_redis_pool();
        let redis_broadcaster = self.container.get_redis_broadcaster();

        let app_state = crate::web::auth::AppState::new(
            self.container.db_pool(),
            cache,
            Arc::clone(&self.container),
            redis_pool,
            redis_broadcaster,
            self.container.get_analytics_pool(),
        );

        // Create TradingView service and EPS ranking service for public analytics
        let config = Arc::new(crate::config::get_fallback_config());
        let tradingview_service = Arc::new(crate::infrastructure::adapters::services::tradingview::TradingViewApiService::new(config));
        let eps_repository = Arc::new(crate::web::analytics::TradingViewEPSRepository::new(tradingview_service));
        let eps_ranking_service = Arc::new(crate::domain::shared_kernel::services::eps_ranking_service::EPSRankingService::new(eps_repository));
        // Permission service is required by the rankings handler even for public access
        let permission_service = self.container.get_unified_permission_service()
            .unwrap_or_else(|| Arc::new(crate::auth::UnifiedPermissionService::new_without_cache(*self.container.db_pool())));

        Router::new()
            .nest("/analytics", Router::new()
                .route("/rankings", get(crate::web::analytics::eps_handlers::get_unified_analytics_rankings_cached))
                .route("/filters", get(crate::web::analytics::eps_handlers::get_filter_options))
                .route("/countries", get(crate::web::analytics::eps_handlers::get_all_valid_countries))
                .layer(Extension(self.get_or_default_cache()))
                .layer(Extension(eps_ranking_service))
                .layer(Extension(permission_service))
            )
            .route("/plans", get(crate::web::public::plans_handlers::get_public_plans))
            .route("/plans/{id}", get(crate::web::public::plans_handlers::get_public_plan_by_id))
            .route("/plans/seed", post(crate::web::public::seed_plans_handler::seed_subscription_plans))
            // V2 Dynamic Payment Links (public lookup by slug)
            .route("/payment-links/{slug}", get(crate::web::admin::payment_link_handlers::get_payment_link_by_slug_handler))
            .with_state(app_state)
    }

    // ============================================================================
    // USER ROUTES (authenticated users)
    // ============================================================================

    fn create_user_routes(&self) -> Router {
        let app_state = self.create_app_state();

        Router::new()
            // User profile and settings - using available handlers
            .route("/profile", get(crate::web::user::unified_user_handlers::get_current_user_profile))
            .route("/permissions", get(crate::web::user::unified_user_handlers::get_user_permissions))
            .route("/access-overview", get(crate::web::user::unified_user_handlers::get_user_access_overview))
            // User preferences update (POST with JSON body)
            .route("/preferences", post(crate::web::user::unified_user_handlers::update_user_preferences))

            // Watchlist management
            .route("/watchlist", get(crate::web::user::watchlist_handlers::get_watchlist))
            .route("/watchlist", post(crate::web::user::watchlist_handlers::add_to_watchlist))
            .route("/watchlist/{symbol}", delete(crate::web::user::watchlist_handlers::remove_from_watchlist))

            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state,
                crate::web::middleware::bearer_middleware
            ))
    }

    // ============================================================================
    // PLAN AND BILLING ROUTES (authenticated users)
    // ============================================================================

    fn create_plan_routes(&self) -> Router {
        let app_state = self.create_app_state();

        // NOTE: Plan management routes are deferred for future implementation.
        // These routes would include:
        // - /subscription (GET/POST) - subscription status and creation
        // - /subscription/status, /subscription/history
        // - /usage - API usage tracking
        // - /billing, /invoices - billing information
        //
        // Currently, plan/subscription management is handled via:
        // - /api/payments/plans - get user's subscription plans
        // - /api/payments/plans/expiry - check plan expiry status
        // - /api/payments/plans/cancel/{id} - cancel subscription
        // - /api/public/plans - public plan listing

        Router::new()
            .with_state(app_state)
    }

    // ============================================================================
    // DEVELOPER PORTAL ROUTES (user-facing API key management)
    // ============================================================================

    fn create_developer_portal_routes(&self) -> Router {
        use crate::web::user::developer_portal::{
            list_my_keys_handler,
            create_my_key_handler,
            get_my_key_handler,
            revoke_my_key_handler,
            list_available_plans_handler,
            get_my_plans_handler,
            get_usage_stats_handler,
            get_usage_history_handler,
            get_top_endpoints_handler,
        };

        let app_state = self.create_app_state();

        // Available permission plans (public info, no auth required)
        let public_routes = Router::new()
            .route("/available-plans", get(list_available_plans_handler))
            .with_state(app_state.clone());

        // User-facing routes (scoped to authenticated user's wallet)
        // Requires web3 auth middleware to inject wallet_address Extension
        let authenticated_routes = Router::new()
            .route("/my-keys", get(list_my_keys_handler))
            .route("/my-keys", post(create_my_key_handler))
            .route("/my-keys/{id}", get(get_my_key_handler))
            .route("/my-keys/{id}", delete(revoke_my_key_handler))
            .route("/my-keys/{id}/revoke", post(revoke_my_key_handler))
            .route("/my-plans", get(get_my_plans_handler))
            .route("/stats", get(get_usage_stats_handler))
            .route("/usage-history", get(get_usage_history_handler))
            .route("/top-endpoints", get(get_top_endpoints_handler))
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state,
                crate::web::middleware::web3_auth_middleware
            ));

        // Combine public and authenticated routes
        public_routes.merge(authenticated_routes)
    }

    // ============================================================================
    // CHAT ROUTES (authenticated users)
    // ============================================================================

    fn create_chat_routes(&self) -> Router {
        use crate::web::user::chat_handlers;

        let app_state = self.create_app_state();

        // SSE route (permissive CORS for EventSource)
        let sse_route = Router::new()
            .route("/stream", get(chat_handlers::chat_stream))
            .layer(Self::create_sse_cors_layer())
            .with_state(app_state.clone());

        // Admin SSE route
        let admin_sse_route = Router::new()
            .route("/admin/stream", get(crate::web::admin::chat_handlers::admin_chat_stream))
            .layer(Self::create_sse_cors_layer())
            .with_state(app_state.clone());

        // Topics (public, no auth needed)
        let public_routes = Router::new()
            .route("/topics", get(chat_handlers::list_topics))
            .with_state(app_state.clone());

        // Authenticated user chat routes
        let auth_routes = Router::new()
            .route("/conversations", get(chat_handlers::list_conversations).post(chat_handlers::create_conversation))
            .route("/conversations/{id}", get(chat_handlers::get_conversation))
            .route("/conversations/{id}/messages", get(chat_handlers::list_messages).post(chat_handlers::send_message))
            .route("/conversations/{id}/status", put(chat_handlers::update_status))
            .route("/conversations/{id}/read", put(chat_handlers::mark_read))
            .route("/unread", get(chat_handlers::get_unread))
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state,
                crate::web::middleware::bearer_middleware
            ));

        sse_route.merge(admin_sse_route).merge(public_routes).merge(auth_routes)
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

        let cache = self.container.cache.clone()
            .unwrap_or_else(|| {
                Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn crate::infrastructure::cache::Cache>
            });

        let redis_pool = self.container.get_redis_pool();
        let redis_broadcaster = self.container.get_redis_broadcaster();

        let app_state = crate::web::auth::AppState::new(
            self.container.db_pool(),
            cache,
            Arc::clone(&self.container),
            redis_pool,
            redis_broadcaster,
            self.container.get_analytics_pool(),
        );

        // Create SSE route with permissive CORS (EventSource cannot send credentials)
        let sse_route = Router::new()
            .route("/stream", get(crate::web::notifications::sse_notifications_handler))
            .layer(Self::create_sse_cors_layer())
            .with_state(app_state.clone());

        // Create authenticated user notification routes

        // Combine all notification routes
        let notification_routes = Router::new()
            .route("/", get(get_user_notifications_handler))
            .route("/preferences", get(crate::web::user::unified_user_handlers::get_user_notification_preferences))
            .route("/preferences", post(crate::web::user::unified_user_handlers::update_user_notification_preferences))
            .route("/unread-count", get(get_unread_count_handler))
            .route("/mark-all-read", put(mark_all_notifications_read_handler))
            .route("/clear-all", delete(clear_all_notifications_handler))
            .route("/{id}/read", put(mark_notification_read_handler))
            .route("/{id}/acknowledge", put(acknowledge_notification_handler))
            .route("/{id}", delete(delete_notification_handler))
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state,
                crate::web::middleware::bearer_middleware
            ));

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
    // PAYMENT ROUTES
    // ============================================================================

    fn create_payment_routes(&self) -> Router {
        use crate::web::payments::{
            validate_payment_handler,
            activate_subscription_handler,
            get_payment_details_handler,

            get_user_plans_handler,
            get_plan_expiry_status_handler,
            cancel_plan_handler,
            get_upgrade_preview_handler,
            get_user_payment_history,
            submit_transaction_handler,
            get_transaction_status_handler,
            // Admin payment handlers
            admin_list_payments_handler,
            admin_get_payment_details_handler,
            admin_update_payment_status_handler,
            admin_process_refund_handler,
            admin_list_subscriptions_handler,
            admin_get_payment_analytics_handler,
            // Credit handlers
            get_credit_balance,
            get_credit_history,
            admin_get_user_credits,
            admin_grant_credits,
            admin_revoke_credits,
            admin_get_credit_stats,
        };

        let app_state = self.create_app_state();

        // Core payment validation routes (authenticated users)
        let core_routes = Router::new()
            .route("/validate", post(validate_payment_handler))
            .route("/activate", post(activate_subscription_handler))

            .route("/submit", post(submit_transaction_handler))  // NEW: Submit tx for backend monitoring
            .route("/status/{tx_hash}", get(get_transaction_status_handler))  // NEW: Poll tx status
            .route("/details", get(get_payment_details_handler))
            .route("/history", get(get_user_payment_history))
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::bearer_middleware
            ));

        // Subscription management routes (authenticated users)
        let subscription_routes = Router::new()
            .route("/plans/my-plan-access", get(get_user_plans_handler)) // Add this for frontend compatibility
            .route("/plans", get(get_user_plans_handler)) // Changed from /subscriptions
            .route("/plans/expiry", get(get_plan_expiry_status_handler)) // Changed from /subscriptions/{id}
            .route("/plans/cancel/{id}", post(cancel_plan_handler)) // Changed from /subscriptions/{id}/cancel
            .route("/plans/upgrade_preview", get(get_upgrade_preview_handler)) // Changed from /subscriptions/upgrade-preview
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::bearer_middleware
            ));

        // Admin payment management routes (admin permissions required)
        let admin_routes = Router::new()
            .route("/admin/list", get(admin_list_payments_handler))
            .route("/admin/{id}", get(admin_get_payment_details_handler))
            .route("/admin/{id}/status", put(admin_update_payment_status_handler))
            .route("/admin/{id}/refund", post(admin_process_refund_handler))
            .route("/admin/subscriptions", get(admin_list_subscriptions_handler))
            .route("/admin/analytics", get(admin_get_payment_analytics_handler))
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn(
                crate::web::middleware::permission_validation_middleware
            ))
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::bearer_middleware
            ));

        // Credit wallet routes (authenticated users)
        let credit_routes = Router::new()
            .route("/credits/balance", get(get_credit_balance))
            .route("/credits/history", get(get_credit_history))
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::bearer_middleware
            ));

        // Admin credit routes (admin permissions required)
        let admin_credit_routes = Router::new()
            .route("/admin/credits/{wallet}", get(admin_get_user_credits))
            .route("/admin/credits/grant", post(admin_grant_credits))
            .route("/admin/credits/revoke", post(admin_revoke_credits))
            .route("/admin/credits/stats", get(admin_get_credit_stats))
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn(
                crate::web::middleware::permission_validation_middleware
            ))
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::bearer_middleware
            ));

        // Combine all payment routes
        core_routes
            .merge(subscription_routes)
            .merge(admin_routes)
            .merge(credit_routes)
            .merge(admin_credit_routes)
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

        let redis_pool = self.container.get_redis_pool();
        let redis_broadcaster = self.container.get_redis_broadcaster();

        let app_state = crate::web::auth::AppState::new(
            self.container.db_pool(),
            cache,
            Arc::clone(&self.container),
            redis_pool,
            redis_broadcaster,
            self.container.get_analytics_pool(),
        );

        crate::web::admin::routes::create_permission_authority_routes()
            .with_state(app_state.clone())
            .layer(axum_middleware::from_fn(
                crate::web::middleware::permission_validation_middleware
            ))
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                crate::web::middleware::bearer_middleware
            ))
    }

    // ============================================================================
    // CORS CONFIGURATION
    // ============================================================================

    // ============================================================================
    // CORS CONFIGURATION
    // ============================================================================

    fn configure_cors(&self) -> CorsLayer {
        // Use centralized CORS configuration from security module
        crate::web::security::cors::get_cors_layer()
    }
}
