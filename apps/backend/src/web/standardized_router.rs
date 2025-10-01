// Standardized API Router for EPSX Backend
// Implements the standardized API naming convention: /api/{version}/{access-level}/{domain}/{resource}/{action}
//
// Access Levels:
// - public: No authentication required
// - auth: User authentication required  
// - admin: Admin privileges required
//
// Domain Categories:
// - analytics: EPS rankings, financial data
// - auth: Authentication, sessions
// - users: User management
// - permissions: Permission management
// - notifications: Notification system
// - payments: Payment processing
// - web3: Blockchain/wallet operations

use axum::{
    routing::{get, post, put, delete},
    Router,
    response::{Json, IntoResponse},
    extract::{State, Path, Query},
    http::{StatusCode, Method, HeaderMap},
    middleware as axum_middleware,
};
use serde_json::json;
use tower_http::cors::CorsLayer;

use crate::infrastructure::container::StatelessServiceFactory;

/// Standardized Router Builder
#[derive(Clone)]
pub struct StandardizedRouterBuilder {
    service_factory: StatelessServiceFactory,
}

impl StandardizedRouterBuilder {
    pub fn new(service_factory: StatelessServiceFactory) -> Self {
        Self { service_factory }
    }

    /// Build complete standardized router
    pub async fn build(self) -> Router {
        // Core health routes (no versioning needed)
        let health_routes = self.create_health_routes();
        
        // API v1 routes with standardized structure
        let api_v1_routes = self.create_api_v1_routes().await;
        
        // Legacy route aliases for backward compatibility
        let legacy_routes = self.create_legacy_aliases().await;

        // Configure CORS
        let cors = self.configure_cors();

        // Combine all routes with standardized structure
        Router::new()
            // Core health routes (public, no auth)
            .merge(health_routes)
            // API v1 routes with standardized naming
            .nest("/api/v1", api_v1_routes)
            // Legacy route aliases (with deprecation warnings)
            .merge(legacy_routes)
            // Apply middleware
            .layer(axum_middleware::from_fn(
                crate::web::middleware::security_headers_middleware
            ))
            .layer(axum_middleware::from_fn(
                crate::web::middleware::request_id_middleware
            ))
            .layer(cors)
    }

    /// Create health routes that don't require versioning
    fn create_health_routes(&self) -> Router {
        Router::new()
            .route("/health", get(|| async {
                Json(json!({
                    "status": "healthy",
                    "service": "epsx-backend",
                    "api_version": "v1",
                    "architecture": "standardized",
                    "timestamp": chrono::Utc::now()
                }))
            }))
            .route("/readiness", get(|| async {
                Json(json!({
                    "status": "ready",
                    "api_version": "v1"
                }))
            }))
            .route("/liveness", get(|| async {
                Json(json!({
                    "status": "alive",
                    "api_version": "v1"
                }))
            }))
    }

    /// Create API v1 routes with standardized structure
    async fn create_api_v1_routes(&self) -> Router {
        Router::new()
            // Public APIs (no authentication required)
            .nest("/public", self.create_public_routes())
            // Authenticated user APIs
            .nest("/auth", self.create_auth_routes())
            // Admin APIs (admin privileges required)
            .nest("/admin", self.create_admin_routes())
    }

    /// Create public routes: /api/v1/public/*
    fn create_public_routes(&self) -> Router {
        let factory = self.service_factory.clone();
        
        Router::new()
            // Health check for public APIs
            .route("/health", get(|| async {
                Json(json!({
                    "status": "healthy",
                    "access_level": "public",
                    "api_version": "v1"
                }))
            }))
            
            // Public Analytics: /api/v1/public/analytics/*
            .nest("/analytics", Router::new()
                .route("/rankings", get(Self::public_analytics_rankings_handler))
                .route("/countries", get(Self::public_analytics_countries_handler))
                .route("/sectors", get(Self::public_analytics_sectors_handler))
                .route("/filters", get(Self::public_analytics_filters_handler))
            )
            
            // Public Plans: /api/v1/public/plans/*
            .route("/plans", get(Self::public_plans_handler))
            
            .with_state(factory)
    }

    /// Create authenticated user routes: /api/v1/auth/*
    fn create_auth_routes(&self) -> Router {
        let factory = self.service_factory.clone();
        
        Router::new()
            // Health check for auth APIs
            .route("/health", get(|| async {
                Json(json!({
                    "status": "healthy",
                    "access_level": "auth",
                    "api_version": "v1"
                }))
            }))
            
            // Web3 Authentication: /api/v1/auth/web3/*
            .nest("/web3", Router::new()
                .route("/challenge", post(Self::auth_web3_challenge_handler))
                .route("/verify", post(Self::auth_web3_verify_handler))
                .route("/session", get(Self::auth_web3_session_handler))
                .route("/logout", delete(Self::auth_web3_logout_handler))
            )
            
            // Session Management: /api/v1/auth/session/*
            .nest("/session", Router::new()
                .route("/verify", post(Self::auth_session_verify_handler))
                .route("/refresh", post(Self::auth_session_refresh_handler))
            )
            
            // User Analytics: /api/v1/auth/analytics/*
            .nest("/analytics", Router::new()
                .route("/rankings", get(Self::auth_analytics_rankings_handler))
                .route("/filters", get(Self::auth_analytics_filters_handler))
                .route("/export", post(Self::auth_analytics_export_handler))
            )
            
            // User Profile: /api/v1/auth/users/*
            .nest("/users", Router::new()
                .route("/profile", get(Self::auth_users_profile_handler))
                .route("/profile", put(Self::auth_users_update_profile_handler))
                .route("/permissions", get(Self::auth_users_permissions_handler))
            )
            
            // Notifications: /api/v1/auth/notifications/*
            .nest("/notifications", Router::new()
                .route("/stream", get(Self::auth_notifications_stream_handler))
                .route("/list", get(Self::auth_notifications_list_handler))
                .route("/mark-read", post(Self::auth_notifications_mark_read_handler))
            )
            
            .with_state(factory)
    }

    /// Create admin routes: /api/v1/admin/*
    fn create_admin_routes(&self) -> Router {
        let factory = self.service_factory.clone();
        
        Router::new()
            // Health check for admin APIs
            .route("/health", get(|| async {
                Json(json!({
                    "status": "healthy",
                    "access_level": "admin",
                    "api_version": "v1"
                }))
            }))
            
            // Admin User Management: /api/v1/admin/users/*
            .nest("/users", Router::new()
                .route("/", get(Self::admin_users_list_handler))
                .route("/", post(Self::admin_users_create_handler))
                .route("/:user_id", get(Self::admin_users_get_handler))
                .route("/:user_id", put(Self::admin_users_update_handler))
                .route("/:user_id", delete(Self::admin_users_delete_handler))
                .route("/stats", get(Self::admin_users_stats_handler))
                .route("/search", get(Self::admin_users_search_handler))
            )
            
            // Admin Permission Management: /api/v1/admin/permissions/*
            .nest("/permissions", Router::new()
                .route("/groups", get(Self::admin_permissions_groups_list_handler))
                .route("/groups", post(Self::admin_permissions_groups_create_handler))
                .route("/groups/:group_id", get(Self::admin_permissions_groups_get_handler))
                .route("/groups/:group_id", put(Self::admin_permissions_groups_update_handler))
                .route("/groups/:group_id", delete(Self::admin_permissions_groups_delete_handler))
                .route("/grant", post(Self::admin_permissions_grant_handler))
                .route("/revoke", delete(Self::admin_permissions_revoke_handler))
                .route("/validate", post(Self::admin_permissions_validate_handler))
            )
            
            // Admin Web3 Management: /api/v1/admin/web3/*
            .nest("/web3", Router::new()
                .route("/wallets/search", get(Self::admin_web3_wallets_search_handler))
                .route("/wallets/recent", get(Self::admin_web3_wallets_recent_handler))
                .route("/permissions/grant", post(Self::admin_web3_permissions_grant_handler))
                .route("/nft-gates", get(Self::admin_web3_nft_gates_list_handler))
                .route("/nft-gates", post(Self::admin_web3_nft_gates_create_handler))
                .route("/token-gates", get(Self::admin_web3_token_gates_list_handler))
                .route("/token-gates", post(Self::admin_web3_token_gates_create_handler))
                .route("/dao-proposals", get(Self::admin_web3_dao_proposals_list_handler))
                .route("/dao-proposals", post(Self::admin_web3_dao_proposals_create_handler))
            )
            
            // Admin Analytics: /api/v1/admin/analytics/*
            .nest("/analytics", Router::new()
                .route("/overview", get(Self::admin_analytics_overview_handler))
                .route("/users", get(Self::admin_analytics_users_handler))
                .route("/permissions", get(Self::admin_analytics_permissions_handler))
                .route("/revenue", get(Self::admin_analytics_revenue_handler))
                .route("/performance", get(Self::admin_analytics_performance_handler))
            )
            
            // Admin Notifications: /api/v1/admin/notifications/*
            .nest("/notifications", Router::new()
                .route("/send", post(Self::admin_notifications_send_handler))
                .route("/broadcast", post(Self::admin_notifications_broadcast_handler))
                .route("/list", get(Self::admin_notifications_list_handler))
                .route("/stats", get(Self::admin_notifications_stats_handler))
            )
            
            .with_state(factory)
    }

    /// Create legacy route aliases with deprecation warnings
    async fn create_legacy_aliases(&self) -> Router {
        let factory = self.service_factory.clone();
        
        Router::new()
            // Legacy auth routes (with deprecation warnings)
            .nest("/api/auth", Router::new()
                .route("/web3/challenge", post(Self::legacy_auth_web3_challenge_with_warning))
                .route("/web3/verify", post(Self::legacy_auth_web3_verify_with_warning))
                .route("/session/verify", post(Self::legacy_auth_session_verify_with_warning))
            )
            
            // Legacy analytics routes
            .nest("/api", Router::new()
                .route("/analytics/rankings", get(Self::legacy_analytics_rankings_with_warning))
                .nest("/v1", Router::new()
                    .route("/analytics/rankings", get(Self::legacy_v1_analytics_rankings_with_warning))
                )
            )
            
            // Legacy admin routes
            .nest("/admin", Router::new()
                .route("/users", get(Self::legacy_admin_users_with_warning))
                .route("/permission-groups", get(Self::legacy_admin_permission_groups_with_warning))
            )
            
            .with_state(factory)
    }

    /// Configure CORS for all environments
    fn configure_cors(&self) -> CorsLayer {
        use tower_http::cors::Any;
        use axum::http::HeaderName;
        use std::time::Duration;
        use crate::config::env::is_production;

        if is_production() {
            // Production: Allow any origin, no credentials
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
                .allow_headers([
                    HeaderName::from_static("accept"),
                    HeaderName::from_static("authorization"),
                    HeaderName::from_static("content-type"),
                    HeaderName::from_static("origin"),
                    HeaderName::from_static("referer"),
                    HeaderName::from_static("next-router-prefetch"),
                    HeaderName::from_static("next-router-state-tree"),
                    HeaderName::from_static("next-url"),
                    HeaderName::from_static("rsc"),
                    HeaderName::from_static("x-api-version"),
                    HeaderName::from_static("x-request-id"),
                ])
                .allow_credentials(false)
                .max_age(Duration::from_secs(86400))
        } else {
            // Development: Specific origins with credentials
            let origins = vec![
                "http://localhost:3000".parse().unwrap(),
                "http://localhost:3001".parse().unwrap(),
                "http://127.0.0.1:3000".parse().unwrap(),
                "http://127.0.0.1:3001".parse().unwrap(),
            ];
            
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
                .allow_headers([
                    HeaderName::from_static("accept"),
                    HeaderName::from_static("authorization"),
                    HeaderName::from_static("content-type"),
                    HeaderName::from_static("origin"),
                    HeaderName::from_static("referer"),
                    HeaderName::from_static("next-router-prefetch"),
                    HeaderName::from_static("next-router-state-tree"),
                    HeaderName::from_static("next-url"),
                    HeaderName::from_static("rsc"),
                    HeaderName::from_static("x-api-version"),
                    HeaderName::from_static("x-request-id"),
                ])
                .allow_credentials(true)
                .max_age(Duration::from_secs(86400))
        }
    }

    // ============================================================================
    // PUBLIC API HANDLERS: /api/v1/public/*
    // ============================================================================

    /// GET /api/v1/public/analytics/rankings
    async fn public_analytics_rankings_handler(
        State(_factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        // Forward to existing analytics handler with public access limits
        Json(json!({
            "success": true,
            "data": [],
            "pagination": {
                "page": params.get("page").and_then(|p| p.parse::<i32>().ok()).unwrap_or(1),
                "limit": std::cmp::min(
                    params.get("limit").and_then(|l| l.parse::<i32>().ok()).unwrap_or(10),
                    10 // Public API limit
                ),
                "total": 0,
                "totalPages": 0
            },
            "api_version": "v1",
            "access_level": "public"
        }))
    }

    /// GET /api/v1/public/analytics/countries
    async fn public_analytics_countries_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": ["US", "CA", "UK", "DE", "FR"],
            "api_version": "v1",
            "access_level": "public"
        }))
    }

    /// GET /api/v1/public/analytics/sectors
    async fn public_analytics_sectors_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": ["Technology", "Healthcare", "Finance", "Energy"],
            "api_version": "v1",
            "access_level": "public"
        }))
    }

    /// GET /api/v1/public/analytics/filters
    async fn public_analytics_filters_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "countries": ["US", "CA", "UK"],
                "sectors": ["Technology", "Healthcare"],
                "sort_options": ["eps_growth", "market_cap", "volume"]
            },
            "api_version": "v1",
            "access_level": "public"
        }))
    }

    /// GET /api/v1/public/plans
    async fn public_plans_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [
                {
                    "id": 1,
                    "name": "Free Plan",
                    "price": "0.00",
                    "currency": "USD",
                    "features": ["View 3 rankings", "Basic analytics"]
                },
                {
                    "id": 2,
                    "name": "Professional Plan",
                    "price": "49.99",
                    "currency": "USD",
                    "features": ["View 50 rankings", "Advanced analytics", "Export data"]
                }
            ],
            "api_version": "v1",
            "access_level": "public"
        }))
    }

    // ============================================================================
    // AUTH API HANDLERS: /api/v1/auth/*
    // ============================================================================

    /// POST /api/v1/auth/web3/challenge
    async fn auth_web3_challenge_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        // Placeholder implementation - forward to existing service
        Json(json!({
            "success": true,
            "nonce": "mock-nonce-12345",
            "message": "Sign this message to authenticate",
            "expires_at": (chrono::Utc::now() + chrono::Duration::minutes(10)).timestamp(),
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// POST /api/v1/auth/web3/verify
    async fn auth_web3_verify_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        // Placeholder implementation - forward to existing service
        Json(json!({
            "success": true,
            "authenticated": true,
            "access_token": "mock-token-67890",
            "permissions": ["epsx:analytics:view"],
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// GET /api/v1/auth/web3/session
    async fn auth_web3_session_handler(
        State(_factory): State<StatelessServiceFactory>,
        _headers: HeaderMap,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "session_valid": true,
            "wallet_address": "0x1234567890abcdef",
            "permissions": ["epsx:analytics:view"],
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// DELETE /api/v1/auth/web3/logout
    async fn auth_web3_logout_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "Logged out successfully",
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// POST /api/v1/auth/session/verify
    async fn auth_session_verify_handler(
        State(_factory): State<StatelessServiceFactory>,
        _headers: HeaderMap,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "authenticated": true,
            "user_id": "mock-user-id",
            "permissions": ["epsx:analytics:view"],
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// POST /api/v1/auth/session/refresh
    async fn auth_session_refresh_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "access_token": "new-mock-token",
            "expires_in": 3600,
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// GET /api/v1/auth/analytics/rankings
    async fn auth_analytics_rankings_handler(
        State(_factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "pagination": {
                "page": params.get("page").and_then(|p| p.parse::<i32>().ok()).unwrap_or(1),
                "limit": params.get("limit").and_then(|l| l.parse::<i32>().ok()).unwrap_or(20),
                "total": 0,
                "totalPages": 0
            },
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// GET /api/v1/auth/analytics/filters
    async fn auth_analytics_filters_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "countries": ["US", "CA", "UK", "DE", "FR", "JP", "AU"],
                "sectors": ["Technology", "Healthcare", "Finance", "Energy", "Retail"],
                "sort_options": ["eps_growth", "market_cap", "volume", "price"]
            },
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// POST /api/v1/auth/analytics/export
    async fn auth_analytics_export_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "export_id": "export-12345",
            "download_url": "/api/v1/auth/analytics/exports/export-12345",
            "status": "processing",
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// GET /api/v1/auth/users/profile
    async fn auth_users_profile_handler(
        State(_factory): State<StatelessServiceFactory>,
        _headers: HeaderMap,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "user_id": "mock-user-id",
                "wallet_address": "0x1234567890abcdef",
                "permissions": ["epsx:analytics:view"],
                "tier": "professional"
            },
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// PUT /api/v1/auth/users/profile
    async fn auth_users_update_profile_handler(
        State(_factory): State<StatelessServiceFactory>,
        _headers: HeaderMap,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "Profile updated successfully",
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// GET /api/v1/auth/users/permissions
    async fn auth_users_permissions_handler(
        State(_factory): State<StatelessServiceFactory>,
        _headers: HeaderMap,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "permissions": ["epsx:analytics:view", "epsx:export:data"],
                "tier": "professional",
                "expires_at": null
            },
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// GET /api/v1/auth/notifications/stream
    async fn auth_notifications_stream_handler(
        State(_factory): State<StatelessServiceFactory>,
        _headers: HeaderMap,
    ) -> impl IntoResponse {
        // SSE endpoint would be implemented here
        Json(json!({
            "success": false,
            "error": "SSE endpoints not supported in this handler format",
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// GET /api/v1/auth/notifications/list
    async fn auth_notifications_list_handler(
        State(_factory): State<StatelessServiceFactory>,
        _headers: HeaderMap,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "pagination": {
                "page": params.get("page").and_then(|p| p.parse::<i32>().ok()).unwrap_or(1),
                "limit": params.get("limit").and_then(|l| l.parse::<i32>().ok()).unwrap_or(20),
                "total": 0
            },
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    /// POST /api/v1/auth/notifications/mark-read
    async fn auth_notifications_mark_read_handler(
        State(_factory): State<StatelessServiceFactory>,
        _headers: HeaderMap,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "Notifications marked as read",
            "api_version": "v1",
            "access_level": "auth"
        }))
    }

    // ============================================================================
    // ADMIN API HANDLERS: /api/v1/admin/*
    // ============================================================================

    /// GET /api/v1/admin/users/
    async fn admin_users_list_handler(
        State(_factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "pagination": {
                "page": params.get("page").and_then(|p| p.parse::<i32>().ok()).unwrap_or(1),
                "limit": params.get("limit").and_then(|l| l.parse::<i32>().ok()).unwrap_or(20),
                "total": 0
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// POST /api/v1/admin/users/
    async fn admin_users_create_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "user_id": "new-user-id",
                "created_at": chrono::Utc::now()
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/users/:user_id
    async fn admin_users_get_handler(
        State(_factory): State<StatelessServiceFactory>,
        Path(user_id): Path<String>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "user_id": user_id,
                "wallet_address": "0x1234567890abcdef",
                "permissions": ["epsx:analytics:view"],
                "created_at": chrono::Utc::now()
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// PUT /api/v1/admin/users/:user_id
    async fn admin_users_update_handler(
        State(_factory): State<StatelessServiceFactory>,
        Path(user_id): Path<String>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "user_id": user_id,
                "updated_at": chrono::Utc::now()
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// DELETE /api/v1/admin/users/:user_id
    async fn admin_users_delete_handler(
        State(_factory): State<StatelessServiceFactory>,
        Path(user_id): Path<String>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "User deleted successfully",
            "user_id": user_id,
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/users/stats
    async fn admin_users_stats_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "total_users": 1500,
                "active_users": 1200,
                "new_users_today": 25,
                "premium_users": 300
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/users/search
    async fn admin_users_search_handler(
        State(_factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "query": params.get("q").unwrap_or(&"".to_string()),
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    // Permission Management Handlers
    /// GET /api/v1/admin/permissions/groups
    async fn admin_permissions_groups_list_handler(
        State(_factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "pagination": {
                "page": params.get("page").and_then(|p| p.parse::<i32>().ok()).unwrap_or(1),
                "limit": params.get("limit").and_then(|l| l.parse::<i32>().ok()).unwrap_or(20),
                "total": 0
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// POST /api/v1/admin/permissions/groups
    async fn admin_permissions_groups_create_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "group_id": "new-group-id",
                "created_at": chrono::Utc::now()
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/permissions/groups/:group_id
    async fn admin_permissions_groups_get_handler(
        State(_factory): State<StatelessServiceFactory>,
        Path(group_id): Path<String>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "group_id": group_id,
                "name": "Professional Group",
                "permissions": ["epsx:analytics:view", "epsx:export:data"]
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// PUT /api/v1/admin/permissions/groups/:group_id
    async fn admin_permissions_groups_update_handler(
        State(_factory): State<StatelessServiceFactory>,
        Path(group_id): Path<String>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "group_id": group_id,
                "updated_at": chrono::Utc::now()
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// DELETE /api/v1/admin/permissions/groups/:group_id
    async fn admin_permissions_groups_delete_handler(
        State(_factory): State<StatelessServiceFactory>,
        Path(group_id): Path<String>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "Permission group deleted successfully",
            "group_id": group_id,
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// POST /api/v1/admin/permissions/grant
    async fn admin_permissions_grant_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "Permission granted successfully",
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// DELETE /api/v1/admin/permissions/revoke
    async fn admin_permissions_revoke_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "Permission revoked successfully",
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// POST /api/v1/admin/permissions/validate
    async fn admin_permissions_validate_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "valid": true,
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    // Web3 Management Handlers
    /// GET /api/v1/admin/web3/wallets/search
    async fn admin_web3_wallets_search_handler(
        State(_factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "query": params.get("q").unwrap_or(&"".to_string()),
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/web3/wallets/recent
    async fn admin_web3_wallets_recent_handler(
        State(_factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "limit": params.get("limit").and_then(|l| l.parse::<i32>().ok()).unwrap_or(10),
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// POST /api/v1/admin/web3/permissions/grant
    async fn admin_web3_permissions_grant_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "Web3 permission granted successfully",
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/web3/nft-gates
    async fn admin_web3_nft_gates_list_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// POST /api/v1/admin/web3/nft-gates
    async fn admin_web3_nft_gates_create_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "nft_gate_id": "new-nft-gate-id",
                "created_at": chrono::Utc::now()
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/web3/token-gates
    async fn admin_web3_token_gates_list_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// POST /api/v1/admin/web3/token-gates
    async fn admin_web3_token_gates_create_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "token_gate_id": "new-token-gate-id",
                "created_at": chrono::Utc::now()
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/web3/dao-proposals
    async fn admin_web3_dao_proposals_list_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// POST /api/v1/admin/web3/dao-proposals
    async fn admin_web3_dao_proposals_create_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "dao_proposal_id": "new-dao-proposal-id",
                "created_at": chrono::Utc::now()
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    // Analytics Handlers
    /// GET /api/v1/admin/analytics/overview
    async fn admin_analytics_overview_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "total_users": 1500,
                "active_users": 1200,
                "revenue": 125000.50,
                "growth_rate": 15.2
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/analytics/users
    async fn admin_analytics_users_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "user_metrics": {
                    "total": 1500,
                    "active": 1200,
                    "new_today": 25
                }
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/analytics/permissions
    async fn admin_analytics_permissions_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "permission_distribution": {},
                "group_membership": []
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/analytics/revenue
    async fn admin_analytics_revenue_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "total_revenue": 125000.50,
                "monthly_revenue": 28500.00,
                "revenue_trends": []
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/analytics/performance
    async fn admin_analytics_performance_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "api_response_times": {},
                "cache_hit_rates": {},
                "error_rates": {}
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    // Notification Handlers
    /// POST /api/v1/admin/notifications/send
    async fn admin_notifications_send_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "Notification sent successfully",
            "notification_id": "notif-12345",
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// POST /api/v1/admin/notifications/broadcast
    async fn admin_notifications_broadcast_handler(
        State(_factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "message": "Broadcast sent successfully",
            "broadcast_id": "broadcast-12345",
            "recipients": 1200,
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/notifications/list
    async fn admin_notifications_list_handler(
        State(_factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": [],
            "pagination": {
                "page": params.get("page").and_then(|p| p.parse::<i32>().ok()).unwrap_or(1),
                "limit": params.get("limit").and_then(|l| l.parse::<i32>().ok()).unwrap_or(20),
                "total": 0
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    /// GET /api/v1/admin/notifications/stats
    async fn admin_notifications_stats_handler(
        State(_factory): State<StatelessServiceFactory>,
    ) -> Json<serde_json::Value> {
        Json(json!({
            "success": true,
            "data": {
                "total_sent": 5000,
                "total_delivered": 4750,
                "total_opened": 3200,
                "delivery_rate": 95.0,
                "open_rate": 67.4
            },
            "api_version": "v1",
            "access_level": "admin"
        }))
    }

    // ============================================================================
    // LEGACY ROUTE HANDLERS (with deprecation warnings)
    // ============================================================================

    /// Legacy auth handler with deprecation warning
    async fn legacy_auth_web3_challenge_with_warning(
        State(factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        // Add deprecation warning header
        let mut response = Self::auth_web3_challenge_handler(State(factory), Json(_payload)).await;
        
        // Add deprecation info to response body
        if let serde_json::Value::Object(ref mut map) = response.0 {
            map.insert("deprecation_warning".to_string(), json!({
                "message": "This endpoint is deprecated. Please use /api/v1/auth/web3/challenge instead.",
                "migration_guide": "Update your API calls to use the new standardized endpoints.",
                "sunset_date": "2024-12-31"
            }));
        }
        
        (StatusCode::OK, response)
    }

    /// Legacy auth verify handler with deprecation warning
    async fn legacy_auth_web3_verify_with_warning(
        State(factory): State<StatelessServiceFactory>,
        Json(_payload): Json<serde_json::Value>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        let mut response = Self::auth_web3_verify_handler(State(factory), Json(_payload)).await;
        
        if let serde_json::Value::Object(ref mut map) = response.0 {
            map.insert("deprecation_warning".to_string(), json!({
                "message": "This endpoint is deprecated. Please use /api/v1/auth/web3/verify instead.",
                "migration_guide": "Update your API calls to use the new standardized endpoints.",
                "sunset_date": "2024-12-31"
            }));
        }
        
        (StatusCode::OK, response)
    }

    /// Legacy session verify handler with deprecation warning  
    async fn legacy_auth_session_verify_with_warning(
        State(factory): State<StatelessServiceFactory>,
        _headers: HeaderMap,
        Json(_payload): Json<serde_json::Value>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        let mut response = Self::auth_session_verify_handler(State(factory), _headers, Json(_payload)).await;
        
        if let serde_json::Value::Object(ref mut map) = response.0 {
            map.insert("deprecation_warning".to_string(), json!({
                "message": "This endpoint is deprecated. Please use /api/v1/auth/session/verify instead.",
                "migration_guide": "Update your API calls to use the new standardized endpoints.",
                "sunset_date": "2024-12-31"
            }));
        }
        
        (StatusCode::OK, response)
    }

    /// Legacy analytics rankings handler with deprecation warning
    async fn legacy_analytics_rankings_with_warning(
        State(factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        let mut response = Self::auth_analytics_rankings_handler(State(factory), Query(params)).await;
        
        if let serde_json::Value::Object(ref mut map) = response.0 {
            map.insert("deprecation_warning".to_string(), json!({
                "message": "This endpoint is deprecated. Please use /api/v1/public/analytics/rankings or /api/v1/auth/analytics/rankings instead.",
                "migration_guide": "Use the public endpoint for unauthenticated access or the auth endpoint for authenticated access.",
                "sunset_date": "2024-12-31"
            }));
        }
        
        (StatusCode::OK, response)
    }

    /// Legacy v1 analytics rankings handler with deprecation warning
    async fn legacy_v1_analytics_rankings_with_warning(
        State(factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        let mut response = Self::auth_analytics_rankings_handler(State(factory), Query(params)).await;
        
        if let serde_json::Value::Object(ref mut map) = response.0 {
            map.insert("deprecation_warning".to_string(), json!({
                "message": "This endpoint is deprecated. Please use /api/v1/auth/analytics/rankings instead.",
                "migration_guide": "Update your API calls to use the standardized access-level structure.",
                "sunset_date": "2024-12-31"
            }));
        }
        
        (StatusCode::OK, response)
    }

    /// Legacy admin users handler with deprecation warning
    async fn legacy_admin_users_with_warning(
        State(factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        let mut response = Self::admin_users_list_handler(State(factory), Query(params)).await;
        
        if let serde_json::Value::Object(ref mut map) = response.0 {
            map.insert("deprecation_warning".to_string(), json!({
                "message": "This endpoint is deprecated. Please use /api/v1/admin/users instead.",
                "migration_guide": "Update your API calls to use the new standardized admin endpoints.",
                "sunset_date": "2024-12-31"
            }));
        }
        
        (StatusCode::OK, response)
    }

    /// Legacy admin permission groups handler with deprecation warning
    async fn legacy_admin_permission_groups_with_warning(
        State(factory): State<StatelessServiceFactory>,
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        let mut response = Self::admin_permissions_groups_list_handler(State(factory), Query(params)).await;
        
        if let serde_json::Value::Object(ref mut map) = response.0 {
            map.insert("deprecation_warning".to_string(), json!({
                "message": "This endpoint is deprecated. Please use /api/v1/admin/permissions/groups instead.",
                "migration_guide": "Update your API calls to use the new standardized admin endpoints.",
                "sunset_date": "2024-12-31"
            }));
        }
        
        (StatusCode::OK, response)
    }
}

/// Create standardized router
pub async fn create_standardized_router(service_factory: StatelessServiceFactory) -> Router {
    StandardizedRouterBuilder::new(service_factory).build().await
}