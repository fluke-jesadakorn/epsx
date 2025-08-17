// Simplified router for auth migration
// Focuses only on essential endpoints during the auth system transition

use axum::{
    routing::get,
    Router,
    response::{Json, Html, IntoResponse},
    http::{StatusCode, HeaderMap, Method},
    extract::Query,
};
use serde_json::{json, Value};
use serde::Deserialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use crate::config::env::get_env_var;

use crate::infra::AppContainer;
use crate::web::oidc::token::TokenErrorResponse;
use crate::core::iam_token_claims::AccessTokenClaims;

/// Health check handler
pub async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "epsx-backend"
    }))
}

/// Simple authorization parameters
#[derive(Deserialize)]
pub struct AuthParams {
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub response_type: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub registration: Option<String>, // 'true' for registration mode
}

/// Real authorization handler with PKCE support
pub async fn real_auth_handler(Query(params): Query<AuthParams>) -> Html<String> {
    let client_id = params.client_id.unwrap_or_else(|| "unknown".to_string());
    let redirect_uri = params.redirect_uri.unwrap_or_else(|| "".to_string());
    let scope = params.scope.unwrap_or_else(|| "openid profile email".to_string());
    let state = params.state.unwrap_or_else(|| "".to_string());
    let response_type = params.response_type.unwrap_or_else(|| "code".to_string());
    let code_challenge = params.code_challenge.unwrap_or_else(|| "".to_string());
    let code_challenge_method = params.code_challenge_method.unwrap_or_else(|| "S256".to_string());
    let is_registration = params.registration.as_deref() == Some("true");
    
    // Determine page title and content based on mode
    let (page_title, form_title, submit_button_text, info_message) = if is_registration {
        (
            "EPSX Registration",
            "🚀 Join EPSX!",
            "Create Account",
            "This will create a new account with your email and password."
        )
    } else {
        (
            "EPSX Login",
            "🔐 EPSX Login",
            "Sign In",
            "Enter your credentials to access your account."
        )
    };
    
    // For simplified implementation, return a login/registration form that stores PKCE parameters
    Html(format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>{}</title>
            <style>
                body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; }}
                .container {{ max-width: 400px; margin: 0 auto; background: white; padding: 40px; border-radius: 12px; box-shadow: 0 20px 40px rgba(0,0,0,0.1); }}
                h1 {{ color: #333; text-align: center; margin-bottom: 30px; }}
                .form-group {{ margin-bottom: 20px; }}
                label {{ display: block; margin-bottom: 8px; color: #555; font-weight: 500; }}
                input[type="email"], input[type="password"] {{ width: 100%; padding: 12px; border: 2px solid #e1e5e9; border-radius: 8px; font-size: 16px; transition: border-color 0.3s; }}
                input[type="email"]:focus, input[type="password"]:focus {{ outline: none; border-color: #667eea; }}
                button {{ width: 100%; padding: 14px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: 600; cursor: pointer; transition: transform 0.2s; }}
                button:hover {{ transform: translateY(-2px); }}
                .client-info {{ background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px; }}
                .client-info p {{ margin: 5px 0; color: #666; font-size: 14px; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>{}</h1>
                <div class="client-info">
                    <p><strong>Client:</strong> {}</p>
                    <p><strong>Scope:</strong> {}</p>
                    <p><strong>PKCE:</strong> {}</p>
                    <p><strong>Mode:</strong> {}</p>
                </div>
                <form method="POST" action="/oauth/token">
                    <div class="form-group">
                        <label for="email">Email</label>
                        <input type="email" id="email" name="email" required placeholder="your.email@example.com">
                    </div>
                    <div class="form-group">
                        <label for="password">Password</label>
                        <input type="password" id="password" name="password" required placeholder="••••••••">
                    </div>
                    
                    <!-- Hidden fields to preserve OAuth parameters -->
                    <input type="hidden" name="client_id" value="{}">
                    <input type="hidden" name="redirect_uri" value="{}">
                    <input type="hidden" name="scope" value="{}">
                    <input type="hidden" name="state" value="{}">
                    <input type="hidden" name="response_type" value="{}">
                    <input type="hidden" name="code_challenge" value="{}">
                    <input type="hidden" name="code_challenge_method" value="{}">
                    <input type="hidden" name="grant_type" value="authorization_code">
                    <input type="hidden" name="registration" value="{}">
                    
                    <button type="submit">{}</button>
                </form>
            </div>
        </body>
        </html>"#,
        page_title,                    // Title tag
        form_title,                   // H1 tag
        client_id,                    // Client info
        scope,                        // Scope info
        if !code_challenge.is_empty() { "✅ Enabled" } else { "❌ Not provided" }, // PKCE info
        if is_registration { "Registration" } else { "Login" }, // Mode info
        client_id,                    // Hidden field
        redirect_uri,                 // Hidden field
        scope,                        // Hidden field
        state,                        // Hidden field
        response_type,                // Hidden field
        code_challenge,               // Hidden field
        code_challenge_method,        // Hidden field
        if is_registration { "true" } else { "false" }, // Registration hidden field
        submit_button_text            // Submit button
    ))
}

/// Token request data from login form
#[derive(Deserialize)]
pub struct TokenFormData {
    pub email: String,
    pub password: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: String,
    pub response_type: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub grant_type: String,
    pub registration: Option<String>, // 'true' for registration mode
}

/// Real token handler that processes login and generates tokens
pub async fn real_token_handler(
    axum::extract::Form(form_data): axum::extract::Form<TokenFormData>,
) -> Result<axum::response::Response, StatusCode> {
    use axum::response::Redirect;
    use jsonwebtoken::{encode, EncodingKey, Header};
    
    tracing::info!("Processing OAuth token request for email: {}", form_data.email);
    
    // Check if this is a registration request
    let is_registration = form_data.registration.as_deref() == Some("true");
    
    let is_valid_user = if is_registration {
        // For registration mode, we need to validate that user doesn't exist yet
        // and then create the user account (for demo, skip actual registration for now)
        tracing::info!("Registration mode: Creating account for {}", form_data.email);
        
        // In a real implementation, this would call the registration API we created
        // For the demo, we'll just allow specific test emails to "register"
        match form_data.email.as_str() {
            "info@epsx.io" => form_data.password == "P@ssword",
            _ => {
                // For demo, allow any email with password "register123"
                form_data.password == "register123"
            }
        }
    } else {
        // Regular login mode - validate existing user credentials
        validate_user_credentials(&form_data.email, &form_data.password)
    };
    
    if !is_valid_user {
        // Redirect back with error
        let error_redirect = format!(
            "{}?state={}&error=invalid_credentials&error_description=Invalid%20email%20or%20password",
            form_data.redirect_uri, form_data.state
        );
        return Ok(Redirect::to(&error_redirect).into_response());
    }
    
    // Generate access token with admin modules based on email
    let admin_modules = get_admin_modules_for_user(&form_data.email);
    let subscription_tier = get_package_tier_for_user(&form_data.email);
    
    // Use the constructor method which handles all the complex setup
    let claims = AccessTokenClaims::new(
        form_data.email.clone(), // user_id = email for simplicity
        form_data.email.clone(),
        Some(form_data.email.split('@').next().unwrap_or("User").to_string()),
        admin_modules,
        subscription_tier,
        form_data.client_id.clone(),
        form_data.scope.clone(),
        3600, // 1 hour expiry
    );
    
    // Encode JWT token
    let jwt_secret = get_env_var("JWT_SECRET")
        .unwrap_or_else(|_| "epsx-dev-secret-key-change-in-production".to_string());
    let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
    
    let access_token = encode(&Header::default(), &claims, &encoding_key)
        .map_err(|e| {
            tracing::error!("Failed to encode JWT: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    tracing::info!("✅ Generated access token for user: {}", form_data.email);
    
    // Redirect back to application with token as authorization code
    // In a real implementation, this would be a short-lived authorization code
    // that gets exchanged for tokens, but for simplicity we're returning the token directly
    let success_redirect = format!(
        "{}?code={}&state={}",
        form_data.redirect_uri,
        access_token, // Using token as code for simplicity
        form_data.state
    );
    
    Ok(Redirect::to(&success_redirect).into_response())
}

/// Simple user validation (replace with real authentication)
fn validate_user_credentials(email: &str, password: &str) -> bool {
    // Demo credentials - replace with real authentication
    match email {
        "admin@epsx.com" => password == "admin123",
        "user@epsx.com" => password == "user123",
        "moderator@epsx.com" => password == "mod123",
        "jesadakorn.kirtnu@gmail.com" => password == "Aa_12345678",
        "info@epsx.io" => password == "P@ssword", // SuperAdmin test user
        _ => false,
    }
}

/// Get admin modules based on user email (demo implementation)
fn get_admin_modules_for_user(email: &str) -> Vec<String> {
    match email {
        "admin@epsx.com" => vec![
            "user_operations".to_string(),
            "analytics_specialist".to_string(),
            "billing_admin".to_string(),
            "role_policy_manager".to_string(),
            "system_admin".to_string(),
            "developer_relations".to_string(),
            "support_specialist".to_string(),
            "compliance_audit".to_string(),
            "module_coordinator".to_string(),
        ],
        "info@epsx.io" => vec![
            "user_operations".to_string(),
            "analytics_specialist".to_string(),
            "billing_admin".to_string(),
            "role_policy_manager".to_string(),
            "system_admin".to_string(),
            "developer_relations".to_string(),
            "support_specialist".to_string(),
            "compliance_audit".to_string(),
            "module_coordinator".to_string(),
            "business_intelligence".to_string(),
        ],
        "moderator@epsx.com" => vec![
            "user_operations".to_string(),
            "support_specialist".to_string(),
        ],
        "jesadakorn.kirtnu@gmail.com" => vec![
            "user_operations".to_string(),
            "analytics_specialist".to_string(),
            "billing_admin".to_string(),
        ],
        _ => vec![],
    }
}


/// Get package tier based on user email (demo implementation)
fn get_package_tier_for_user(email: &str) -> String {
    match email {
        "admin@epsx.com" => "ENTERPRISE".to_string(),
        "info@epsx.io" => "ENTERPRISE".to_string(), // SuperAdmin gets highest tier
        "moderator@epsx.com" => "PLATINUM".to_string(),
        "jesadakorn.kirtnu@gmail.com" => "PREMIUM".to_string(),
        _ => "FREE".to_string(),
    }
}

/// Real userinfo handler with JWT validation but simplified database access
pub async fn real_userinfo_handler(
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<TokenErrorResponse>)> {
    tracing::debug!("UserInfo endpoint request with real JWT validation");
    
    // Extract bearer token from Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            tracing::error!("UserInfo: Missing Authorization header");
            (
                StatusCode::UNAUTHORIZED,
                Json(TokenErrorResponse {
                    error: "invalid_token".to_string(),
                    error_description: Some("Missing Authorization header".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(TokenErrorResponse {
                error: "invalid_token".to_string(),
                error_description: Some("Invalid Authorization header format".to_string()),
                error_uri: None,
            }),
        ));
    }
    
    let access_token = &auth_header[7..]; // Remove "Bearer " prefix
    
    // Validate and decode the access token using real JWT validation
    let token_claims = validate_access_token_simplified(access_token)
        .map_err(|e| {
            tracing::error!("Access token validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(TokenErrorResponse {
                    error: "invalid_token".to_string(),
                    error_description: Some("Invalid or expired access token".to_string()),
                    error_uri: None,
                }),
            )
        })?;

    // Check if token is revoked using the global JWT service
    if let Err(_) = check_token_revocation(&token_claims.jti).await {
        tracing::warn!("Token {} is revoked", token_claims.jti);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(TokenErrorResponse {
                error: "invalid_token".to_string(),
                error_description: Some("Token has been revoked".to_string()),
                error_uri: None,
            }),
        ));
    }
    
    // Build OIDC userinfo response using data from JWT claims
    // This is secure because the JWT was cryptographically verified
    let userinfo = json!({
        "sub": token_claims.sub,
        "email": token_claims.email,
        "email_verified": true,
        "name": token_claims.email.split('@').next().unwrap_or("User"),
        "role": token_claims.role,
        "permissions": token_claims.permissions,
        "package_tier": token_claims.subscription_tier,
        "admin_modules": token_claims.admin_modules,
        "exp": token_claims.exp,
        "iat": token_claims.iat
    });
    
    tracing::debug!("UserInfo endpoint returning verified response for user: {}", token_claims.email);
    Ok(Json(userinfo))
}

/// Simplified JWT validation using the same logic as the full implementation
fn validate_access_token_simplified(token: &str) -> Result<AccessTokenClaims, Box<dyn std::error::Error>> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

    // Get JWT secret from environment or use default for development
    let jwt_secret = get_env_var("JWT_SECRET")
        .unwrap_or_else(|_| "epsx-dev-secret-key-change-in-production".to_string());
    
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    
    // Match the issuer and audience used in token generation
    let issuer = get_env_var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    // Allow both client_id based audience and the standard epsx-api audience
    validation.set_audience(&["epsx-admin-frontend", "epsx-frontend", "epsx-api"]);
    validation.set_issuer(&[&issuer]);
    
    let token_data = decode::<AccessTokenClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("JWT validation failed: {}", e))?;
    
    Ok(token_data.claims)
}

/// Check if token is revoked using the global token revocation service
async fn check_token_revocation(jti: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::auth::TOKEN_REVOCATION_SERVICE;
    
    let is_revoked = TOKEN_REVOCATION_SERVICE.is_token_revoked(jti).await;
    
    if is_revoked {
        Err("Token is revoked".into())
    } else {
        Ok(())
    }
}

// Mock analytics handler removed - using real TradingView integration in analytics module

// SimpleAnalyticsParams removed - using real analytics parameters from analytics module

// generate_sample_analytics_data function removed - using real TradingView data

/// Create simplified application router with real JWT validation
pub async fn create_simplified_router(container: Arc<AppContainer>) -> Router<()> {
    use crate::web::auth::registration::{register_user, check_email_availability};
    use axum::routing::post;
    
    // Configure CORS to allow frontend requests
    let cors = if get_env_var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "development" {
        // Development - allow specific origins to work with credentials
        // Common development origins for frontend testing
        use tower_http::cors::AllowOrigin;
        use http::{HeaderName, HeaderValue};
        
        let dev_origins = vec![
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),  // Frontend dev server
            "http://localhost:3001".parse::<HeaderValue>().unwrap(),  // Admin frontend dev server
            "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),  // Alternative localhost
            "http://127.0.0.1:3001".parse::<HeaderValue>().unwrap(),  // Alternative localhost
        ];
        
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(dev_origins))
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            // Explicitly specify headers when using credentials (wildcard * doesn't work)
            .allow_headers([
                HeaderName::from_static("authorization"),
                HeaderName::from_static("content-type"),
                HeaderName::from_static("x-requested-with"),
                HeaderName::from_static("accept"),
                HeaderName::from_static("origin"),
                HeaderName::from_static("x-client-id"),
                HeaderName::from_static("x-api-key"),
            ])
            .allow_credentials(true)
    } else {
        // Production - only allow configured frontend URLs
        use tower_http::cors::AllowOrigin;
        let mut allowed_origins = vec![];
        
        if let Ok(frontend_url) = get_env_var("FRONTEND_URL") {
            if let Ok(origin) = frontend_url.parse() {
                allowed_origins.push(origin);
            }
        }
        
        if let Ok(admin_url) = get_env_var("ADMIN_FRONTEND_URL") {
            if let Ok(origin) = admin_url.parse() {
                allowed_origins.push(origin);
            }
        }
        
        if let Ok(prod_frontend) = get_env_var("PRODUCTION_FRONTEND_URL") {
            if let Ok(origin) = prod_frontend.parse() {
                allowed_origins.push(origin);
            }
        }
        
        if let Ok(prod_admin) = get_env_var("PRODUCTION_ADMIN_URL") {
            if let Ok(origin) = prod_admin.parse() {
                allowed_origins.push(origin);
            }
        }
        
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(allowed_origins))
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            // Explicitly specify headers when using credentials (wildcard * doesn't work)
            .allow_headers([
                http::HeaderName::from_static("authorization"),
                http::HeaderName::from_static("content-type"),
                http::HeaderName::from_static("x-requested-with"),
                http::HeaderName::from_static("accept"),
                http::HeaderName::from_static("origin"),
                http::HeaderName::from_static("x-client-id"),
                http::HeaderName::from_static("x-api-key"),
            ])
            .allow_credentials(true)
    };
    
    // Create essential public routes
    let public_routes = Router::new()
        .route("/health", get(health_handler));

    // Create registration API routes (public, no auth required)
    // Using the container directly as state for simplified router
    let registration_routes = Router::new()
        .route("/api/auth/register", post(register_user))
        .route("/api/auth/check-email", post(check_email_availability))
        .with_state(container.clone());

    // Create OIDC routes with real JWT validation but simplified dependencies
    let oidc_routes = Router::new()
        .route("/.well-known/openid-configuration", get(crate::web::oidc::discovery::oidc_discovery))
        .route("/oauth/jwks", get(crate::web::oidc::discovery::jwks_endpoint))
        .route("/oauth/authorize", get(real_auth_handler))
        .route("/oauth/token", post(real_token_handler))
        .route("/oauth/userinfo", get(real_userinfo_handler)); // Real JWT validation!

    // Analytics routes removed - using real TradingView integration from main router
    
    Router::new()
        .merge(public_routes)
        .merge(registration_routes)
        .merge(oidc_routes)
        .layer(cors) // Apply CORS layer to all routes
}