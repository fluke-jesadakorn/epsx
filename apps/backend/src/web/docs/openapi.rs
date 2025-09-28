//! Main OpenAPI Specification
//! 
//! Defines the complete OpenAPI 3.0 specification for the EPSX trading platform API.

use utoipa::OpenApi;

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
        // Health endpoints
        crate::web::health::health_check_handler,
        crate::web::health::readiness_check_handler,
        crate::web::health::liveness_check_handler,
        
        // Web3 Authentication endpoints
        crate::web::auth::web3_handlers::generate_challenge_handler,
        crate::web::auth::web3_handlers::verify_signature_handler,
        crate::web::auth::web3_handlers::logout_handler,
        crate::web::auth::web3_handlers::get_session_handler,
        crate::web::auth::web3_handlers::check_permission_handler,
    ),
    components(
        schemas(
            // Web3 Authentication schemas
            crate::web::auth::web3_handlers::ChallengeRequest,
            crate::web::auth::web3_handlers::SignatureVerificationRequest,
            crate::web::auth::web3_handlers::LogoutRequest,
            crate::web::auth::web3_handlers::PermissionCheckQuery,
            
            // Common response schemas
            crate::web::docs::schemas::ErrorResponse,
            crate::web::docs::schemas::HealthResponse,
            crate::web::docs::schemas::PaginationInfo,
        )
    ),
    tags(
        (name = "health", description = "Health check and status endpoints"),
        (name = "auth", description = "Web3 authentication and authorization"),
        (name = "analytics", description = "Trading analytics and market data"),
        (name = "admin", description = "Administrative operations"),
        (name = "public", description = "Public API endpoints (no authentication required)")
    )
)]
pub struct ApiDoc;