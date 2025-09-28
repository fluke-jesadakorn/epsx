// Stateless Router for Serverless Architecture
// Creates routes with per-request service instantiation instead of shared state

use axum::{
    routing::get,
    Router,
    response::{Json, IntoResponse},
    extract::State,
    http::Method,
};
use serde_json::json;
use tower_http::cors::CorsLayer;
use axum::middleware as axum_middleware;
use std::sync::Arc;

use crate::infrastructure::container::{StatelessServiceFactory, RequestServices};

/// Stateless Router Builder - Creates services per request instead of sharing state
#[derive(Clone)]
pub struct StatelessRouterBuilder {
    service_factory: StatelessServiceFactory,
}

impl StatelessRouterBuilder {
    pub fn new(service_factory: StatelessServiceFactory) -> Self {
        Self { service_factory }
    }

    /// Build complete router with stateless per-request service creation
    pub async fn build(self) -> Router {
        // Create core health routes (no services needed)
        let health_routes = self.create_health_routes();
        
        // Create API routes with per-request service factory
        let api_routes = self.create_api_routes().await;
        
        // Create admin routes
        let admin_routes = self.create_admin_routes();

        // Create real analytics routes with TradingView integration
        let analytics_routes = self.create_analytics_routes().await;

        // Configure CORS
        let cors = self.configure_cors();

        // Combine all routes
        Router::new()
            // Core health routes (public, no auth)
            .merge(health_routes)
            // Real analytics routes (includes full paths like /api/v1/analytics/*)
            .merge(analytics_routes)
            // API routes (with per-request services)
            .nest("/api", api_routes)
            // Admin routes
            .nest("/admin", admin_routes)
            // Apply middleware
            .layer(axum_middleware::from_fn(
                crate::web::middleware::security_headers_middleware
            ))
            .layer(axum_middleware::from_fn(
                crate::web::middleware::request_id_middleware
            ))
            .layer(cors)
    }

    /// Create health routes that don't require full services
    fn create_health_routes(&self) -> Router {
        Router::new()
            .route("/health", get(simple_health_handler))
            .route("/readiness", get(readiness_handler))
            .route("/liveness", get(liveness_handler))
    }

    /// Create API routes with per-request service creation
    async fn create_api_routes(&self) -> Router {
        Router::new()
            // Auth routes
            .nest("/auth", self.create_auth_routes())
            // V1 API routes
            .nest("/v1", self.create_v1_routes())
            // Public routes
            .nest("/public", self.create_public_routes())
    }

    /// Create real analytics routes with TradingView integration
    async fn create_analytics_routes(&self) -> Router {
        use crate::web::analytics;
        
        // Create the real analytics router with TradingView WebSocket and Scanner
        analytics::create_analytics_router().await
    }

    /// Create auth routes with per-request service instantiation
    fn create_auth_routes(&self) -> Router {
        // Clone factory for each handler
        let factory_challenge = self.service_factory.clone();
        let factory_verify = self.service_factory.clone();
        let factory_state = self.service_factory.clone();
        
        Router::new()
            .route("/web3/challenge", axum::routing::post(
                |Json(payload): Json<serde_json::Value>| async move {
                    use rand::Rng;
                    use std::time::{SystemTime, UNIX_EPOCH};
                    
                    // Parse wallet address from request
                    let wallet_address = match payload.get("wallet_address").and_then(|v| v.as_str()) {
                        Some(addr) => addr.to_string(),
                        None => {
                            return Json(json!({
                                "success": false,
                                "error": "invalid_request",
                                "message": "wallet_address is required"
                            }));
                        }
                    };
                    
                    // Create services for this request
                    match factory_challenge.create_request_services().await {
                        Ok(_services) => {
                            // Generate challenge components
                            let nonce = format!("{:x}", rand::thread_rng().gen::<u64>());
                            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                            let expires_at = current_time + 300; // 5 minutes
                            
                            // Create SIWE-compatible message
                            let message = format!(
                                "admin.epsx.io wants you to sign in with your Ethereum account:\n{}\n\n\
                                Sign in to EPSX Admin Dashboard\n\n\
                                URI: https://admin.epsx.io\n\
                                Version: 1\n\
                                Chain ID: 97\n\
                                Nonce: {}\n\
                                Issued At: {}\n\
                                Expiration Time: {}",
                                wallet_address,
                                nonce,
                                chrono::DateTime::from_timestamp(current_time as i64, 0).unwrap().to_rfc3339(),
                                chrono::DateTime::from_timestamp(expires_at as i64, 0).unwrap().to_rfc3339()
                            );
                            
                            // Return expected response format
                            Json(json!({
                                "success": true,
                                "nonce": nonce,
                                "message": message,
                                "expires_at": expires_at,
                                "wallet_address": wallet_address
                            }))
                        }
                        Err(e) => {
                            Json(json!({
                                "success": false,
                                "error": "service_creation_failed",
                                "message": e.to_string()
                            }))
                        }
                    }
                }
            ))
            .route("/web3/verify", axum::routing::post(
                |_body: String| async move {
                    // Create services for this request
                    match factory_verify.create_request_services().await {
                        Ok(_services) => {
                            // Use services for verification
                            // TODO: Implement actual verification handler
                            Json(json!({
                                "status": "verification_processed",
                                "mode": "stateless",
                                "message": "Signature verification with per-request services"
                            }))
                        }
                        Err(e) => {
                            Json(json!({
                                "error": "service_creation_failed",
                                "message": e.to_string()
                            }))
                        }
                    }
                }
            ))
            .route("/health", get(|| async {
                Json(json!({
                    "auth": "healthy",
                    "mode": "stateless"
                }))
            }))
            .with_state(factory_state)
    }

    /// Create V1 API routes
    fn create_v1_routes(&self) -> Router {
        let factory = self.service_factory.clone();
        let factory_token = self.service_factory.clone();
        let _factory_userinfo = self.service_factory.clone();
        
        Router::new()
            .route("/users/health", get(|| async {
                Json(json!({
                    "users": "healthy", 
                    "mode": "stateless"
                }))
            }))
            // OpenID + Web3 token endpoint
            .route("/auth/web3/token", axum::routing::post(
                |Json(payload): Json<serde_json::Value>| async move {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    
                    // Parse request
                    let wallet_address = payload.get("wallet_address").and_then(|v| v.as_str()).unwrap_or("");
                    let signature = payload.get("signature").and_then(|v| v.as_str()).unwrap_or("");
                    let message = payload.get("message").and_then(|v| v.as_str()).unwrap_or("");
                    let nonce = payload.get("nonce").and_then(|v| v.as_str()).unwrap_or("");
                    let _client_id = payload.get("client_id").and_then(|v| v.as_str()).unwrap_or("epsx-admin");
                    
                    if wallet_address.is_empty() || signature.is_empty() || message.is_empty() || nonce.is_empty() {
                        return Json(json!({
                            "error": "invalid_request",
                            "error_description": "Missing required fields: wallet_address, signature, message, nonce"
                        }));
                    }
                    
                    // Create services for this request
                    match factory_token.create_request_services().await {
                        Ok(_services) => {
                            // TODO: Add actual signature verification here
                            // For now, assume verification passes if all fields are present
                            
                            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                            let expires_in = 3600; // 1 hour
                            
                            // Generate mock tokens (in production, use proper JWT library)
                            let access_token = format!("epsx_access_{}_{}_{}", wallet_address, nonce, current_time);
                            let refresh_token = format!("epsx_refresh_{}_{}_{}", wallet_address, nonce, current_time);
                            let id_token = format!("epsx_id_{}_{}_{}", wallet_address, nonce, current_time);
                            
                            // Return OpenID token response
                            Json(json!({
                                "access_token": access_token,
                                "token_type": "Bearer",
                                "expires_in": expires_in,
                                "refresh_token": refresh_token,
                                "id_token": id_token,
                                "scope": "openid profile permissions"
                            }))
                        }
                        Err(e) => {
                            Json(json!({
                                "error": "server_error",
                                "error_description": e.to_string()
                            }))
                        }
                    }
                }
            ))
            // User info endpoint
            .route("/auth/userinfo", axum::routing::get(
                |headers: axum::http::HeaderMap| async move {
                    // Extract Bearer token from Authorization header
                    let auth_header = headers.get("authorization")
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or("");
                    
                    if !auth_header.starts_with("Bearer ") {
                        return Json(json!({
                            "error": "unauthorized",
                            "error_description": "Bearer token required"
                        }));
                    }
                    
                    let token = &auth_header[7..]; // Remove "Bearer " prefix
                    
                    // Extract wallet address from token (simple parsing for mock implementation)
                    if let Some(wallet_start) = token.find("epsx_access_0x") {
                        let wallet_part = &token[wallet_start + 12..]; // Skip "epsx_access_"
                        if let Some(wallet_end) = wallet_part.find('_') {
                            let wallet_address = &wallet_part[..wallet_end];
                            
                            // Return user info
                            return Json(json!({
                                "sub": wallet_address,
                                "wallet_address": wallet_address,
                                "tier_level": "admin",
                                "auth_method": "web3_siwe",
                                "permissions": ["admin:*:*", "epsx:admin:*"],
                                "email": format!("{}@wallet.epsx.io", wallet_address),
                                "packageTier": "admin"
                            }));
                        }
                    }
                    
                    Json(json!({
                        "error": "invalid_token",
                        "error_description": "Token format invalid"
                    }))
                }
            ))
            // Public plans endpoint in V1 (for frontend compatibility)
            .route("/public/plans", get(|| async {
                Json(json!({
                    "success": true,
                    "data": [
                        {
                            "id": 1,
                            "name": "Free Plan",
                            "plan_type": "personal",
                            "current_price": "0.00",
                            "currency": "USD",
                            "display_order": 1,
                            "is_active": true,
                            "is_highlighted": false,
                            "features": [
                                "View 3 rankings",
                                "Basic analytics",
                                "Community support",
                                "Public data access"
                            ],
                            "description": "Get started with basic analytics features"
                        },
                        {
                            "id": 2,
                            "name": "Professional Plan",
                            "plan_type": "personal",
                            "current_price": "49.99",
                            "currency": "USD",
                            "display_order": 2,
                            "is_active": true,
                            "is_highlighted": true,
                            "features": [
                                "View 50 rankings",
                                "Advanced analytics",
                                "Priority support",
                                "Custom reports",
                                "Real-time data",
                                "Export capabilities"
                            ],
                            "description": "Best value for individual traders and analysts"
                        },
                        {
                            "id": 3,
                            "name": "Enterprise Plan",
                            "plan_type": "personal",
                            "current_price": "199.99",
                            "currency": "USD",
                            "display_order": 3,
                            "is_active": true,
                            "is_highlighted": false,
                            "features": [
                                "Unlimited rankings",
                                "Full platform access",
                                "Dedicated support",
                                "Custom integrations",
                                "White-label options",
                                "SLA guarantee"
                            ],
                            "description": "Complete solution for enterprise clients"
                        },
                        {
                            "id": 4,
                            "name": "API Basic",
                            "plan_type": "api",
                            "current_price": "29.99",
                            "currency": "USD",
                            "display_order": 1,
                            "is_active": true,
                            "is_highlighted": false,
                            "features": [
                                "1,000 API calls/month",
                                "Basic endpoints",
                                "Documentation access",
                                "Email support"
                            ],
                            "description": "Perfect for small integrations"
                        },
                        {
                            "id": 5,
                            "name": "API Pro",
                            "plan_type": "api",
                            "current_price": "99.99",
                            "currency": "USD",
                            "display_order": 2,
                            "is_active": true,
                            "is_highlighted": true,
                            "features": [
                                "10,000 API calls/month",
                                "All endpoints",
                                "Webhook support",
                                "Priority support",
                                "Rate limit increase"
                            ],
                            "description": "Ideal for production applications"
                        },
                        {
                            "id": 6,
                            "name": "API Enterprise",
                            "plan_type": "api",
                            "current_price": "299.99",
                            "currency": "USD",
                            "display_order": 3,
                            "is_active": true,
                            "is_highlighted": false,
                            "features": [
                                "Unlimited API calls",
                                "Custom endpoints",
                                "SLA guarantee",
                                "Dedicated support",
                                "Custom rate limits"
                            ],
                            "description": "Enterprise-grade API access"
                        }
                    ],
                    "mode": "stateless_public_plans"
                }))
            }))
            .with_state(factory)
    }

    /// Create admin routes
    fn create_admin_routes(&self) -> Router {
        let factory = self.service_factory.clone();
        
        Router::new()
            .route("/health", get(|| async {
                Json(json!({
                    "admin": "healthy",
                    "mode": "stateless"
                }))
            }))
            .with_state(factory)
    }

    /// Create public routes (no auth required)
    fn create_public_routes(&self) -> Router {
        Router::new()
            .route("/health", get(|| async {
                Json(json!({
                    "public": "healthy",
                    "mode": "stateless"
                }))
            }))
            .route("/plans", get(|| async {
                Json(json!({
                    "success": true,
                    "data": [
                        {
                            "id": 1,
                            "name": "Free Plan",
                            "plan_type": "personal",
                            "current_price": "0.00",
                            "currency": "USD",
                            "display_order": 1,
                            "is_active": true,
                            "is_highlighted": false,
                            "features": [
                                "View 3 rankings",
                                "Basic analytics",
                                "Community support",
                                "Public data access"
                            ],
                            "description": "Get started with basic analytics features"
                        },
                        {
                            "id": 2,
                            "name": "Professional Plan",
                            "plan_type": "personal",
                            "current_price": "49.99",
                            "currency": "USD",
                            "display_order": 2,
                            "is_active": true,
                            "is_highlighted": true,
                            "features": [
                                "View 50 rankings",
                                "Advanced analytics",
                                "Priority support",
                                "Custom reports",
                                "Real-time data",
                                "Export capabilities"
                            ],
                            "description": "Best value for individual traders and analysts"
                        },
                        {
                            "id": 3,
                            "name": "Enterprise Plan",
                            "plan_type": "personal",
                            "current_price": "199.99",
                            "currency": "USD",
                            "display_order": 3,
                            "is_active": true,
                            "is_highlighted": false,
                            "features": [
                                "Unlimited rankings",
                                "Full platform access",
                                "Dedicated support",
                                "Custom integrations",
                                "White-label options",
                                "SLA guarantee"
                            ],
                            "description": "Complete solution for enterprise clients"
                        },
                        {
                            "id": 4,
                            "name": "API Basic",
                            "plan_type": "api",
                            "current_price": "29.99",
                            "currency": "USD",
                            "display_order": 1,
                            "is_active": true,
                            "is_highlighted": false,
                            "features": [
                                "1,000 API calls/month",
                                "Basic endpoints",
                                "Documentation access",
                                "Email support"
                            ],
                            "description": "Perfect for small integrations"
                        },
                        {
                            "id": 5,
                            "name": "API Pro",
                            "plan_type": "api",
                            "current_price": "99.99",
                            "currency": "USD",
                            "display_order": 2,
                            "is_active": true,
                            "is_highlighted": true,
                            "features": [
                                "10,000 API calls/month",
                                "All endpoints",
                                "Webhook support",
                                "Priority support",
                                "Rate limit increase"
                            ],
                            "description": "Ideal for production applications"
                        },
                        {
                            "id": 6,
                            "name": "API Enterprise",
                            "plan_type": "api",
                            "current_price": "299.99",
                            "currency": "USD",
                            "display_order": 3,
                            "is_active": true,
                            "is_highlighted": false,
                            "features": [
                                "Unlimited API calls",
                                "Custom endpoints",
                                "SLA guarantee",
                                "Dedicated support",
                                "Custom rate limits"
                            ],
                            "description": "Enterprise-grade API access"
                        }
                    ],
                    "mode": "stateless_public_plans"
                }))
            }))
    }

    /// Configure CORS for frontend applications
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
                HeaderName::from_static("accept"),
                HeaderName::from_static("authorization"),
                HeaderName::from_static("content-type"),
                HeaderName::from_static("origin"),
                HeaderName::from_static("referer"),
                // Next.js headers
                HeaderName::from_static("next-router-prefetch"),
                HeaderName::from_static("next-router-state-tree"),
                HeaderName::from_static("next-url"),
                HeaderName::from_static("rsc"),
                // Web3 headers (standardized naming)
                HeaderName::from_static("x-wallet-address"), // Keep lowercase for CORS
                HeaderName::from_static("x-chain-id"),       // Keep lowercase for CORS  
                HeaderName::from_static("x-web3-signature"), // Standardized naming
                HeaderName::from_static("x-signed-message"), // Standardized naming
                HeaderName::from_static("x-nonce"),
            ])
            .allow_credentials(false)
            .max_age(Duration::from_secs(86400))
    }
}

/// Main function to create stateless router
pub async fn create_stateless_router(service_factory: StatelessServiceFactory) -> Router {
    StatelessRouterBuilder::new(service_factory).build().await
}

/// Stateless handler wrapper - creates services per request
pub async fn with_stateless_services<F, T>(
    State(factory): State<StatelessServiceFactory>,
    handler: F,
) -> Result<T, axum::response::Response>
where
    F: FnOnce(RequestServices) -> T,
{
    match factory.create_request_services().await {
        Ok(services) => Ok(handler(services)),
        Err(e) => {
            let error_response = Json(json!({
                "error": "service_creation_failed",
                "message": e.to_string()
            }));
            Err(error_response.into_response())
        }
    }
}

/// Middleware for injecting services into request
pub async fn service_injection_middleware(
    State(factory): State<StatelessServiceFactory>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::response::Response> {
    // Create services for this request
    match factory.create_request_services().await {
        Ok(services) => {
            // Add services to request extensions
            request.extensions_mut().insert(Arc::new(services));
            Ok(next.run(request).await)
        }
        Err(e) => {
            let error_response = Json(json!({
                "error": "service_creation_failed",
                "message": e.to_string()
            }));
            Err(error_response.into_response())
        }
    }
}
/// Simple health check handler with database connectivity test
async fn simple_health_handler() -> Json<serde_json::Value> {
    use crate::infrastructure::database::db_health_check;
    
    let db_healthy = db_health_check().await;
    
    Json(json!({
        "status": if db_healthy { "healthy" } else { "degraded" },
        "service": "epsx-backend",
        "mode": "stateless",
        "database": db_healthy,
        "timestamp": chrono::Utc::now()
    }))
}

/// Readiness check handler
async fn readiness_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ready",
        "mode": "stateless"
    }))
}

/// Liveness check handler  
async fn liveness_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "alive",
        "mode": "stateless"
    }))
}
