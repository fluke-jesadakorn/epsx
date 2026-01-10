//! User OpenAPI Specification
//!
//! Defines the OpenAPI 3.0 specification for user-facing EPSX API endpoints.
//! Excludes admin-only endpoints for cleaner public documentation.

use utoipa::{
    OpenApi,
    Modify,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

/// User-facing OpenAPI documentation
/// Includes public, analytics, user profile, and developer portal endpoints
#[derive(OpenApi)]
#[openapi(
    info(
        title = "EPSX Data Analytics Platform API",
        version = "1.0.0",
        description = "EPSX is a comprehensive data analytics platform providing real-time market data, EPS rankings, and advanced financial analysis tools.\n\n## Authentication\n\nThis API uses Web3 authentication (Sign-In with Ethereum). Most endpoints require a Bearer token obtained through the Web3 authentication flow.\n\n### Authentication Flow:\n1. **Generate Challenge**: `POST /api/auth/web3/challenge`\n2. **Sign Message**: Sign the SIWE message with your wallet\n3. **Verify Signature**: `POST /api/auth/web3/verify` to receive Bearer token\n4. **Use Token**: Include `Authorization: Bearer <token>` in subsequent requests",
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

        // ============================================================================
        // USER MANAGEMENT ENDPOINTS
        // ============================================================================
        crate::web::user::unified_user_handlers::get_current_user_profile,
        crate::web::user::unified_user_handlers::get_user_permissions,
        crate::web::user::unified_user_handlers::update_user_preferences,

        // ============================================================================
        // ANALYTICS ENDPOINTS
        // ============================================================================
        crate::web::analytics::eps::cache::get_unified_analytics_rankings_cached,
        crate::web::analytics::eps::cache::get_cache_stats,
        crate::web::analytics::eps::metadata::get_available_countries,
        crate::web::analytics::eps::metadata::get_all_valid_countries,
        crate::web::analytics::eps::metadata::get_sectors_by_country,
        crate::web::analytics::eps::metadata::get_filter_options,

        // ============================================================================
        // NOTIFICATIONS - SSE endpoints
        // ============================================================================
        crate::web::notifications::sse_handlers::sse_notifications_handler,

        // ============================================================================
        // PUBLIC ENDPOINTS
        // ============================================================================
        crate::web::public::plans_handlers::get_public_plans,
    ),
    components(
        schemas(
            // Web3 Authentication schemas
            crate::web::auth::handlers::ChallengeRequest,
            crate::web::auth::handlers::SignatureVerificationRequest,
            crate::web::auth::handlers::LogoutRequest,
            crate::web::auth::handlers::PermissionCheckQuery,

            // User Management schemas
            crate::web::user::unified_user_handlers::UserProfile,
            crate::web::user::unified_user_handlers::UserPermissionsQuery,
            crate::web::user::unified_user_handlers::UpdateUserPreferencesRequest,

            // Analytics schemas
            crate::web::analytics::eps::types::CountriesResponse,
            crate::web::analytics::eps::types::SectorsResponse,
            crate::web::analytics::eps::types::FiltersResponse,
            crate::web::analytics::eps::types::CountryData,
            crate::web::analytics::eps::types::EPSPaginationResponse,
            crate::web::analytics::eps::types::CacheStatsResponse,
            crate::web::analytics::eps::types::CardDashboardResponse,
            crate::web::analytics::eps::types::SymbolCardData,
            crate::web::analytics::eps::types::CardDashboardMetadata,
            crate::web::analytics::eps::types::QuarterlyPerformanceData,
            crate::web::analytics::eps::types::NextQuarterEstimate,
            crate::web::analytics::eps::types::EPSQuarterlyData,
            crate::domain::market_analytics::domain_services::eps_cache_service::CacheStats,

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
            crate::web::docs::schemas::Plan,
            crate::web::docs::schemas::Country,
            crate::web::docs::schemas::Sector,
            crate::web::docs::schemas::Notification,
            crate::web::docs::schemas::RankingItem,
            crate::web::docs::schemas::ChallengeResponse,
            crate::web::docs::schemas::VerificationResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check and status endpoints"),
        (name = "auth", description = "Web3 authentication and authorization (SIWE-based)"),
        (name = "user", description = "User profile and account management"),
        (name = "analytics", description = "Trading analytics: EPS rankings and market data"),
        (name = "notifications", description = "Server-Sent Events (SSE) notification system"),
        (name = "public", description = "Public API endpoints (no authentication required)")
    ),
    modifiers(&UserSecurityAddon)
)]
pub struct UserApiDoc;

/// Security scheme modifier for user documentation
struct UserSecurityAddon;

impl Modify for UserSecurityAddon {
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
