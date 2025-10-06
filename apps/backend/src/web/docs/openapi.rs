//! Main OpenAPI Specification
//!
//! Defines the complete OpenAPI 3.0 specification for the EPSX trading platform API.

use utoipa::{
    OpenApi,
    Modify,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

/// Main OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "EPSX Trading Platform API",
        version = "1.0.0",
        description = "EPSX is a comprehensive trading analytics platform providing real-time market data, EPS rankings, and advanced financial analysis tools.",
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
        crate::web::health::readiness_check_handler,
        crate::web::health::liveness_check_handler,

        // ============================================================================
        // WEB3 AUTHENTICATION ENDPOINTS
        // ============================================================================
        crate::web::auth::web3_handlers::generate_challenge_handler,
        crate::web::auth::web3_handlers::verify_signature_handler,
        crate::web::auth::web3_handlers::logout_handler,
        crate::web::auth::web3_handlers::get_session_handler,
        crate::web::auth::web3_handlers::check_permission_handler,

        // ============================================================================
        // ADMIN - PLAN MANAGEMENT ENDPOINTS
        // ============================================================================
        crate::web::admin::plan_handlers::create_plan_handler,
        crate::web::admin::plan_handlers::list_plans_handler,

        // Note: Additional endpoints documented but not yet annotated with #[utoipa::path]:
        // - Admin: Security monitoring, performance, analytics, wallet management, Web3 admin
        // - Analytics: EPS rankings, metadata, health, cache management
        // - Notifications: SSE stream, send, broadcast
        // - User/Public: Profile, plans
        // - Permissions: Group CRUD, assignments, validation, bulk operations
        //
        // These endpoints are functional and accessible via the routes, but need individual
        // #[utoipa::path] annotations to appear in the interactive documentation.
        // The pattern demonstrated above can be applied to all remaining handlers.
    ),
    components(
        schemas(
            // Web3 Authentication schemas
            crate::web::auth::web3_handlers::ChallengeRequest,
            crate::web::auth::web3_handlers::SignatureVerificationRequest,
            crate::web::auth::web3_handlers::LogoutRequest,
            crate::web::auth::web3_handlers::PermissionCheckQuery,

            // Admin Plan Management schemas
            crate::web::admin::plan_handlers::CreatePlanRequest,
            crate::web::admin::plan_handlers::PlanResponse,
            crate::web::admin::plan_handlers::PermissionGroupRequest,

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