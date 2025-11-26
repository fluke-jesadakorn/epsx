//! Main OpenAPI Specification
//!
//! Defines the complete OpenAPI 3.0 specification for the EPSX data analytics platform API.

use utoipa::{
    OpenApi,
    Modify,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

/// Main OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "EPSX Data Analytics Platform API",
        version = "1.0.0",
        description = "EPSX is a comprehensive data analytics platform providing real-time market data, EPS rankings, and advanced financial analysis tools.",
        contact(
            name = "EPSX Team",
            url = "https://epsx.io",
            email = "support@epsx.io"
        ),
        license(
            name = "MIT OR Apache-2.0",
            url = "https://github.com/your-org/epsx"
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

        // ============================================================================
        // ADMIN - PERMISSION GROUPS
        // ============================================================================
        crate::web::admin::permissions::groups::create_group,
        crate::web::admin::permissions::groups::get_group,
        crate::web::admin::permissions::groups::list_groups,
        crate::web::admin::permissions::groups::update_group,
        crate::web::admin::permissions::groups::delete_group,
        crate::web::admin::permissions::groups::get_group_members,

        // ============================================================================
        // ADMIN - PLAN MANAGEMENT
        // ============================================================================
        crate::web::admin::plan_handlers::create_plan_handler,
        crate::web::admin::plan_handlers::list_plans_handler,

        // ============================================================================
        // NOTIFICATIONS - SSE
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

            // Admin Plan Management schemas
            crate::web::admin::plan_handlers::CreatePlanRequest,
            crate::web::admin::plan_handlers::PlanResponse,
            crate::web::admin::plan_handlers::PermissionGroupRequest,

            // Admin Wallet Management schemas
            crate::web::admin::wallet_management_handlers::UpdateWalletRequest,

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
        )
    ),
    tags(
        (name = "health", description = "Health check and status endpoints"),
        (name = "auth", description = "Web3 authentication and authorization (SIWE-based)"),
        (name = "admin-plans", description = "Admin: Plan and subscription management"),
        (name = "admin-permissions", description = "Admin: Permission group management and validation"),
        (name = "admin-wallets", description = "Admin: Wallet user management"),
        (name = "admin-analytics", description = "Admin: Platform analytics and business intelligence"),
        (name = "admin-security", description = "Admin: Security monitoring and threat detection"),
        (name = "admin-performance", description = "Admin: Performance monitoring and cache management"),
        (name = "analytics", description = "Trading analytics: EPS rankings and market data"),
        (name = "notifications", description = "Server-Sent Events (SSE) notification system"),
        (name = "user", description = "User profile and account management"),
        (name = "public", description = "Public API endpoints (no authentication required)")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Security scheme modifier to add Bearer token authentication
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some(
                            "Web3 authentication token obtained from POST /api/auth/web3/verify. \
                            Use the following flow: \
                            1) Generate challenge via POST /api/auth/web3/challenge \
                            2) Sign the SIWE message with your wallet \
                            3) Verify signature via POST /api/auth/web3/verify to get token \
                            4) Use the token as Bearer authentication for protected endpoints"
                        ))
                        .build()
                ),
            );
        }
    }
}