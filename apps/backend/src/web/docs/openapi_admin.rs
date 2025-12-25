//! Admin OpenAPI Specification
//!
//! Defines the OpenAPI 3.0 specification for admin-only EPSX API endpoints.
//! Includes all endpoints: public, user, and admin management.

use utoipa::{
    OpenApi,
    Modify,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

/// Admin OpenAPI documentation
/// Includes ALL endpoints: public, user, analytics, and admin management
#[derive(OpenApi)]
#[openapi(
    info(
        title = "EPSX Admin API Documentation",
        version = "1.0.0",
        description = "Complete EPSX API documentation including admin endpoints for platform management.\n\n## Authentication\n\nThis API uses Web3 authentication (Sign-In with Ethereum). Admin endpoints require specific permissions.\n\n### Authentication Flow:\n1. **Generate Challenge**: `POST /api/v1/auth/web3/challenge`\n2. **Sign Message**: Sign the SIWE message with your admin wallet\n3. **Verify Signature**: `POST /api/v1/auth/web3/verify` to receive Bearer token\n4. **Use Token**: Include `Authorization: Bearer <token>` in subsequent requests\n\n### Admin Permissions\n\nAdmin endpoints require the `admin:*:*` permission or specific permissions like `admin:users:manage`, `admin:plans:manage`, etc.",
        contact(
            name = "EPSX Team",
            url = "https://epsx.io",
            email = "support@epsx.io"
        ),
        license(
            name = "MIT OR Apache-2.0"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.epsx.io", description = "Production API server")
    ),
    paths(
        // ============================================================================
        // HEALTH & SYSTEM ENDPOINTS
        // ============================================================================
        crate::web::health::health_check_handler,

        // ============================================================================
        // WEB3 AUTHENTICATION ENDPOINTS
        // ============================================================================
        crate::web::auth::handlers::generate_challenge_handler,
        crate::web::auth::handlers::verify_signature_handler,
        crate::web::auth::handlers::logout_handler,
        crate::web::auth::handlers::get_session_handler,
        crate::web::auth::handlers::check_permission_handler,
        crate::web::auth::handlers::grant_permission_handler,
        crate::web::auth::handlers::revoke_permission_handler,

        // ============================================================================
        // USER MANAGEMENT ENDPOINTS
        // ============================================================================
        crate::web::user::unified_user_handlers::get_current_user_profile,
        crate::web::user::unified_user_handlers::get_user_permissions,
        crate::web::user::unified_user_handlers::update_user_preferences,
        crate::web::user::unified_user_handlers::get_user_by_wallet_address,

        // ============================================================================
        // ANALYTICS ENDPOINTS
        // ============================================================================
        crate::web::analytics::eps::cache::get_unified_analytics_rankings_cached,
        crate::web::analytics::eps::cache::get_cache_stats,
        crate::web::analytics::eps::cache::force_cache_refresh,
        crate::web::analytics::eps::metadata::get_available_countries,
        crate::web::analytics::eps::metadata::get_all_valid_countries,
        crate::web::analytics::eps::metadata::get_sectors_by_country,
        crate::web::analytics::eps::metadata::get_filter_options,
        crate::web::analytics::system_metrics_handler,
        crate::web::analytics::admin_time_series_handler,
        crate::web::analytics::admin_modules_handler,

        // ============================================================================
        // ADMIN - PERFORMANCE MONITORING
        // ============================================================================
        crate::web::admin::performance_handlers::get_auth_cache_performance,
        crate::web::admin::performance_handlers::get_cache_summary,
        crate::web::admin::performance_handlers::clear_auth_cache,

        // ============================================================================
        // ADMIN - WALLET MANAGEMENT
        // ============================================================================
        crate::web::admin::wallet_management_handlers::list_users_handler,
        crate::web::admin::wallet_management_handlers::get_user_handler,
        crate::web::admin::wallet_management_handlers::update_user_handler,
        crate::web::admin::wallet_management_handlers::get_user_stats_handler,
        crate::web::admin::wallet_management_handlers::validate_user_permissions_bulk,

        // ============================================================================
        // ADMIN - PERMISSION GROUPS (Diesel-migrated handlers)
        // ============================================================================
        crate::web::admin::permissions::groups::create_group,
        crate::web::admin::permissions::groups::get_group,
        crate::web::admin::permissions::groups::list_groups,
        crate::web::admin::permissions::groups::update_group,
        crate::web::admin::permissions::groups::delete_group,

        // ============================================================================
        // ADMIN - PLAN MANAGEMENT
        // ============================================================================
        crate::web::admin::plan_handlers::create_plan_handler,
        crate::web::admin::plan_handlers::list_plans_handler,

        // ============================================================================
        // NOTIFICATIONS - SSE endpoints
        // ============================================================================
        crate::web::notifications::sse_handlers::sse_notifications_handler,

        // ============================================================================
        // PUBLIC ENDPOINTS
        // ============================================================================
        crate::web::public::plans_handlers::get_public_plans,
        crate::web::public::seed_plans_handler::seed_subscription_plans,
    ),
    components(
        schemas(
            // Web3 Authentication schemas
            crate::web::auth::handlers::ChallengeRequest,
            crate::web::auth::handlers::SignatureVerificationRequest,
            crate::web::auth::handlers::LogoutRequest,
            crate::web::auth::handlers::PermissionCheckQuery,
            crate::web::auth::handlers::GrantPermissionRequest,
            crate::web::auth::handlers::RevokePermissionRequest,

            // User Management schemas
            crate::web::user::unified_user_handlers::UserProfile,
            crate::web::user::unified_user_handlers::UserPermissionsQuery,
            crate::web::user::unified_user_handlers::UpdateUserPreferencesRequest,

            // Admin Plan Management schemas
            crate::web::admin::plan_handlers::CreatePlanRequest,
            crate::web::admin::plan_handlers::PlanResponse,
            crate::web::admin::plan_handlers::PermissionGroupRequest,

            // Admin Wallet Management schemas
            crate::web::admin::wallet_management_handlers::UpdateWalletRequest,
            crate::web::admin::wallet_management_handlers::BulkPermissionValidationRequest,
            crate::web::admin::wallet_management_handlers::BulkPermissionValidationResponse,
            crate::web::admin::wallet_management_handlers::WalletListQuery,
            crate::web::admin::wallet_management_handlers::WalletSummaryResponse,
            crate::web::admin::wallet_management_handlers::WalletDetailResponse,
            crate::web::admin::wallet_management_handlers::WalletPermission,
            crate::web::admin::wallet_management_handlers::WalletGroup,
            crate::web::admin::wallet_management_handlers::WalletActivitySummary,
            crate::web::admin::wallet_management_handlers::WalletListResponse,
            crate::web::admin::wallet_management_handlers::WalletStatsResponse,

            // Admin Permission Group schemas
            crate::web::admin::permissions::groups::CreateGroupRequest,
            crate::web::admin::permissions::groups::UpdateGroupRequest,

            // Analytics schemas
            crate::web::analytics::eps::types::CountriesResponse,
            crate::web::analytics::eps::types::SectorsResponse,
            crate::web::analytics::eps::types::FiltersResponse,
            crate::web::analytics::eps::types::CountryData,
            crate::web::analytics::eps::types::EPSPaginationResponse,
            crate::web::analytics::eps::types::CacheStatsResponse,
            crate::web::analytics::eps::types::CacheRefreshResponse,
            crate::web::analytics::eps::types::CardDashboardResponse,
            crate::web::analytics::eps::types::SymbolCardData,
            crate::web::analytics::eps::types::CardDashboardMetadata,
            crate::web::analytics::eps::types::QuarterlyPerformanceData,
            crate::web::analytics::eps::types::NextQuarterEstimate,
            crate::web::analytics::eps::types::EPSQuarterlyData,
            crate::domain::shared_kernel::services::eps_cache_service::CacheStats,

            // Notification schemas
            crate::web::notifications::sse_handlers::SSENotification,
            crate::web::notifications::sse_handlers::NotificationType,
            crate::web::notifications::sse_handlers::NotificationPriority,
            crate::web::notifications::sse_handlers::ScalarQuery,
            crate::web::notifications::sse_handlers::ScalarListQuery,

            // Common response schemas
            crate::web::docs::schemas::ErrorResponse,
            crate::web::docs::schemas::HealthResponse,
            crate::web::docs::schemas::PaginationInfo,
            crate::web::docs::schemas::SessionInfo,
            crate::web::docs::schemas::PermissionGroup,
            crate::web::docs::schemas::PermissionCheckRequest,
            crate::web::docs::schemas::PermissionCheckResponse,
            crate::web::docs::schemas::Plan,
            crate::web::docs::schemas::WalletUser,
            crate::web::docs::schemas::Country,
            crate::web::docs::schemas::Sector,
            crate::web::docs::schemas::Notification,
            crate::web::docs::schemas::SecurityEvent,
            crate::web::docs::schemas::PlatformStats,
            crate::web::docs::schemas::RankingItem,
            crate::web::docs::schemas::ChallengeResponse,
            crate::web::docs::schemas::VerificationResponse,

            // Admin Response schemas
            crate::web::admin::responses::AdminApiResponse<serde_json::Value>,
            crate::web::admin::responses::AdminMetadata,
            crate::web::admin::responses::AdminPermissionContext,
        )
    ),
    tags(
        (name = "health", description = "Health check and status endpoints"),
        (name = "auth", description = "Web3 authentication and authorization (SIWE-based)"),
        (name = "user", description = "User profile and account management"),
        (name = "analytics", description = "Trading analytics: EPS rankings and market data"),
        (name = "notifications", description = "Server-Sent Events (SSE) notification system"),
        (name = "public", description = "Public API endpoints (no authentication required)"),
        (name = "admin-plans", description = "Admin: Plan and subscription management"),
        (name = "admin-permissions", description = "Admin: Permission group management and validation"),
        (name = "admin-wallets", description = "Admin: Wallet user management"),
        (name = "admin-analytics", description = "Admin: Platform analytics and business intelligence"),
        (name = "admin-security", description = "Admin: Security monitoring and threat detection"),
        (name = "admin-performance", description = "Admin: Performance monitoring and cache management")
    ),
    modifiers(&AdminSecurityAddon)
)]
pub struct AdminApiDoc;

/// Security scheme modifier for admin documentation
struct AdminSecurityAddon;

impl Modify for AdminSecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some(
                            "Web3 authentication token with admin permissions. \
                            Admin endpoints require specific permissions like `admin:*:*` or granular permissions. \
                            Use the following flow: \
                            1) Generate challenge via POST /api/auth/web3/challenge \
                            2) Sign the SIWE message with your admin wallet \
                            3) Verify signature via POST /api/auth/web3/verify to get token \
                            4) Use the token as Bearer authentication for admin endpoints"
                        ))
                        .build()
                ),
            );
        }
    }
}
