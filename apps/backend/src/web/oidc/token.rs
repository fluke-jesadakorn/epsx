// OIDC Token Endpoint implementation

use axum::{
    extract::{State, Form, FromRequest},
    response::Json,
    http::{StatusCode, HeaderMap},
    async_trait,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use std::collections::HashMap;
use base64::Engine;

use crate::web::auth::AppState;
use crate::web::oidc::authorization::AuthorizationCodeData;
use crate::infra::firebase_admin::FirebaseUser;

/// OIDC Error Response
#[derive(Debug, Serialize)]
pub struct TokenErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

/// Custom form extractor that always returns JSON errors
pub struct JsonForm<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for JsonForm<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<TokenErrorResponse>);

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match Form::<T>::from_request(req, state).await {
            Ok(Form(value)) => Ok(JsonForm(value)),
            Err(_) => Err((
                StatusCode::BAD_REQUEST,
                Json(TokenErrorResponse {
                    error: "invalid_request".to_string(),
                    error_description: Some("Invalid or malformed request body".to_string()),
                    error_uri: None,
                }),
            )),
        }
    }
}

/// Token request parameters (POST /oauth/token)
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: Option<String>,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
}

/// Token response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub id_token: String,
    pub refresh_token: Option<String>,
    pub scope: String,
}

/// JWT Claims for ID token
#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub jti: String, // JWT ID for token revocation support
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64, // Not before
    pub auth_time: i64,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub role: String,
    pub admin: Option<bool>,
    pub access_level: Option<String>,
}

/// JWT Claims for Access token
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub jti: String, // JWT ID for token revocation support
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64, // Not before
    pub scope: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
}

/// Refresh token data stored in Redis
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenData {
    pub firebase_user: FirebaseUser,
    pub client_id: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
}

/// POST /oauth/token - Token endpoint
pub async fn oidc_token(
    State(app_state): State<AppState>,
    JsonForm(token_request): JsonForm<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenErrorResponse>)> {
    tracing::error!("🔍 AUTH.JS DEBUG: Token endpoint request");
    tracing::error!("🔍 Full request: {:?}", token_request);
    
    // Validate required fields and return proper JSON errors
    let grant_type = token_request.grant_type.as_deref().unwrap_or("");
    let client_id = match token_request.client_id.as_deref() {
        Some(id) => id,
        None => {
            tracing::error!("🔍 AUTH.JS ERROR: Missing client_id parameter");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(TokenErrorResponse {
                    error: "invalid_request".to_string(),
                    error_description: Some("Missing 'client_id' parameter".to_string()),
                    error_uri: None,
                }),
            ))
        }
    };
    
    tracing::error!("🔍 AUTH.JS DEBUG: grant_type={}, client_id={}, code={:?}, redirect_uri={:?}, code_verifier={:?}", 
                   grant_type, client_id, token_request.code, token_request.redirect_uri, token_request.code_verifier);

    match grant_type {
        "authorization_code" => handle_authorization_code_grant(app_state, token_request).await,
        "refresh_token" => handle_refresh_token_grant(app_state, token_request).await,
        "" => Err((
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing 'grant_type' parameter".to_string()),
                error_uri: None,
            }),
        )),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "unsupported_grant_type".to_string(),
                error_description: Some("Only authorization_code and refresh_token grant types are supported".to_string()),
                error_uri: None,
            }),
        )),
    }
}

/// Handle authorization code grant
async fn handle_authorization_code_grant(
    app_state: AppState,
    token_request: TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenErrorResponse>)> {
    tracing::error!("🔍 AUTH.JS DEBUG: Handling authorization code grant");
    
    // Validate required parameters
    let code = token_request.code.ok_or_else(|| {
        tracing::error!("🔍 AUTH.JS ERROR: Missing code parameter");
        (
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing 'code' parameter".to_string()),
                error_uri: None,
            }),
        )
    })?;

    let redirect_uri = token_request.redirect_uri.ok_or_else(|| {
        tracing::error!("🔍 AUTH.JS ERROR: Missing redirect_uri parameter");
        (
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing 'redirect_uri' parameter".to_string()),
                error_uri: None,
            }),
        )
    })?;

    tracing::error!("🔍 AUTH.JS DEBUG: Validating authorization code: {}", code);
    
    // Validate and consume authorization code
    let auth_data = validate_and_consume_authorization_code(&app_state, &code).await
        .map_err(|e| {
            tracing::error!("🔍 AUTH.JS ERROR: Authorization code validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(TokenErrorResponse {
                    error: "invalid_grant".to_string(),
                    error_description: Some("Invalid or expired authorization code".to_string()),
                    error_uri: None,
                }),
            )
        })?;

    // Validate client_id and redirect_uri match
    let client_id = token_request.client_id.as_deref().unwrap_or("");
    if auth_data.client_id != client_id {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(TokenErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Client ID mismatch".to_string()),
                error_uri: None,
            }),
        ));
    }

    if auth_data.redirect_uri != redirect_uri {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Redirect URI mismatch".to_string()),
                error_uri: None,
            }),
        ));
    }

    // Validate PKCE if code_challenge was used
    if let (Some(challenge), Some(method), Some(verifier)) = 
        (&auth_data.code_challenge, &auth_data.code_challenge_method, &token_request.code_verifier) {
        validate_pkce(challenge, method, verifier).map_err(|e| {
            tracing::error!("PKCE validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(TokenErrorResponse {
                    error: "invalid_grant".to_string(),
                    error_description: Some("PKCE validation failed".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    } else if auth_data.code_challenge.is_some() && token_request.code_verifier.is_none() {
        // PKCE was used but no verifier provided
        tracing::error!("PKCE code_challenge was used but no code_verifier provided");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("code_verifier is required when PKCE was used".to_string()),
                error_uri: None,
            }),
        ));
    }

    // Generate tokens
    let now = Utc::now();
    let expires_in = 7200; // 2 hours (frontend client policy)

    // Generate access token
    let access_token = generate_access_token(&auth_data.firebase_user, &auth_data.scope, now, expires_in)?;

    // Generate ID token  
    let id_token = generate_id_token(&auth_data.firebase_user, client_id, now, expires_in)?;

    // Generate refresh token using new rotation service
    let refresh_token = generate_refresh_token_v2(&auth_data, client_id).await?;

    // Create session
    create_session(&app_state, &auth_data.firebase_user, &access_token).await
        .map_err(|e| {
            tracing::error!("Failed to create session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Failed to create session".to_string()),
                    error_uri: None,
                }),
            )
        })?;

    tracing::info!("Token exchange successful for user: {}", auth_data.firebase_user.email.as_ref().unwrap_or(&"unknown".to_string()));

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
        id_token,
        refresh_token: Some(refresh_token),
        scope: auth_data.scope,
    }))
}

/// Handle refresh token grant with secure token rotation
async fn handle_refresh_token_grant(
    _app_state: AppState,
    token_request: TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenErrorResponse>)> {
    use crate::auth::REFRESH_TOKEN_SERVICE;
    
    let refresh_token = token_request.refresh_token.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing 'refresh_token' parameter".to_string()),
                error_uri: None,
            }),
        )
    })?;

    // Validate client_id
    let client_id = token_request.client_id.as_deref().unwrap_or("");

    // Validate and rotate the refresh token using our new service
    let rotation = REFRESH_TOKEN_SERVICE.rotate_refresh_token(
        &refresh_token,
        None, // TODO: Add device info from request headers
    ).await.map_err(|e| {
        tracing::error!("Refresh token rotation failed: {}", e);
        
        let (error_code, error_description) = match e {
            crate::auth::RefreshTokenError::TokenNotFound { .. } => {
                ("invalid_grant", "Refresh token not found")
            }
            crate::auth::RefreshTokenError::TokenExpired { .. } => {
                ("invalid_grant", "Refresh token has expired")
            }
            crate::auth::RefreshTokenError::TokenRevoked { .. } => {
                ("invalid_grant", "Refresh token has been revoked")
            }
            crate::auth::RefreshTokenError::RotationLimitExceeded { .. } => {
                ("invalid_grant", "Token rotation limit exceeded")
            }
            _ => ("server_error", "Internal server error during token refresh")
        };
        
        (
            StatusCode::UNAUTHORIZED,
            Json(TokenErrorResponse {
                error: error_code.to_string(),
                error_description: Some(error_description.to_string()),
                error_uri: None,
            }),
        )
    })?;

    // Validate client_id matches the stored data
    if rotation.new_token_data.client_id != client_id {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(TokenErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Client ID mismatch".to_string()),
                error_uri: None,
            }),
        ));
    }

    // Get user data for token generation - we need to fetch fresh user data
    // TODO: This should ideally use a user service to get current user data
    // For now, we'll create a minimal FirebaseUser from the stored data
    let firebase_user = create_firebase_user_from_token_data(&rotation.new_token_data);

    // Generate new access and ID tokens
    let now = Utc::now();
    let expires_in = 7200; // 2 hours (frontend client policy)

    let access_token = generate_access_token(&firebase_user, &rotation.new_token_data.scope, now, expires_in)?;
    let id_token = generate_id_token(&firebase_user, client_id, now, expires_in)?;

    tracing::info!(
        old_token_id = %rotation.old_token_id,
        new_token_id = %rotation.new_token_data.token_id,
        user_id = %rotation.new_token_data.user_id,
        client_id = %client_id,
        rotation_count = %rotation.new_token_data.rotation_count,
        "Refresh token rotated successfully"
    );

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
        id_token,
        refresh_token: Some(rotation.new_token), // Return the NEW rotated refresh token
        scope: rotation.new_token_data.scope,
    }))
}

/// Helper function to create FirebaseUser from refresh token data
/// TODO: Replace with proper user service lookup
fn create_firebase_user_from_token_data(token_data: &crate::auth::RefreshTokenData) -> FirebaseUser {
    FirebaseUser {
        uid: token_data.user_id.clone(),
        email: Some(format!("user-{}@example.com", token_data.user_id)), // Placeholder
        email_verified: true,
        display_name: None,
        photo_url: None,
        phone_number: None,
        custom_claims: std::collections::HashMap::new(),
        provider_data: vec![],
        disabled: false,
        created_at: chrono::Utc::now(),
        last_login_at: Some(chrono::Utc::now()),
    }
}

/// Validate and consume authorization code
async fn validate_and_consume_authorization_code(
    app_state: &AppState,
    code: &str,
) -> Result<AuthorizationCodeData, Box<dyn std::error::Error>> {
    use crate::dom::values::SessId;
    
    // Get session for authorization code
    let session_id = SessId::from_string(format!("auth_code:{}", code));
    tracing::error!("🔍 AUTH.JS DEBUG: Looking for session with ID: {}", session_id);
    tracing::error!("🔍 AUTH.JS DEBUG: Session UUID: {:?}", session_id.value());
    
    let session_result = app_state.session_repo.get(&session_id).await;
    tracing::error!("🔍 AUTH.JS DEBUG: Session repo result: {:?}", session_result);
    
    let session = session_result?
        .ok_or("Authorization code not found")?;
    
    // Delete the session (single use)
    app_state.session_repo.delete(&session_id).await?;
    
    // Deserialize auth data from access_token field
    let auth_data: AuthorizationCodeData = serde_json::from_str(&session.access_token)?;
    
    // Check expiration (extra safety)
    if Utc::now() - auth_data.created_at > Duration::minutes(10) {
        return Err("Authorization code expired".into());
    }
    
    Ok(auth_data)
}

/// Generate access token
fn generate_access_token(
    firebase_user: &FirebaseUser,
    scope: &str,
    now: DateTime<Utc>,
    expires_in: i64,
) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
    let permissions = get_user_permissions_from_role(&firebase_user.custom_claims);

    let claims = AccessTokenClaims {
        jti: generate_jti(),
        iss: get_issuer_url(),
        sub: firebase_user.uid.clone(),
        aud: "epsx-api".to_string(),
        exp: (now + Duration::seconds(expires_in)).timestamp(),
        iat: now.timestamp(),
        nbf: now.timestamp(),
        scope: scope.to_string(),
        email: firebase_user.email.clone().unwrap_or_default(),
        role: get_role_from_custom_claims(&firebase_user.custom_claims),
        permissions,
    };

    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(get_jwt_secret().as_ref());

    encode(&header, &claims, &encoding_key)
        .map_err(|e| {
            tracing::error!("Failed to generate access token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Failed to generate access token".to_string()),
                    error_uri: None,
                }),
            )
        })
}

/// Generate ID token
fn generate_id_token(
    firebase_user: &FirebaseUser,
    client_id: &str,
    now: DateTime<Utc>,
    expires_in: i64,
) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
    let claims = IdTokenClaims {
        jti: generate_jti(),
        iss: get_issuer_url(),
        sub: firebase_user.uid.clone(),
        aud: client_id.to_string(),
        exp: (now + Duration::seconds(expires_in)).timestamp(),
        iat: now.timestamp(),
        nbf: now.timestamp(),
        auth_time: now.timestamp(),
        email: firebase_user.email.clone().unwrap_or_default(),
        email_verified: firebase_user.email_verified,
        name: firebase_user.display_name.clone(),
        role: get_role_from_custom_claims(&firebase_user.custom_claims),
        admin: firebase_user.custom_claims.get("admin").and_then(|v| v.as_bool()),
        access_level: firebase_user.custom_claims.get("access_level").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };

    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(get_jwt_secret().as_ref());

    encode(&header, &claims, &encoding_key)
        .map_err(|e| {
            tracing::error!("Failed to generate ID token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Failed to generate ID token".to_string()),
                    error_uri: None,
                }),
            )
        })
}

/// Generate refresh token using new rotation service (DEPRECATED - use generate_refresh_token_v2)
async fn generate_refresh_token(
    _app_state: &AppState,
    auth_data: &AuthorizationCodeData,
) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
    // This function is deprecated - keeping for compatibility during migration
    generate_refresh_token_v2(auth_data, &auth_data.client_id).await
}

/// Generate refresh token using new rotation service
async fn generate_refresh_token_v2(
    auth_data: &AuthorizationCodeData,
    client_id: &str,
) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
    use crate::auth::REFRESH_TOKEN_SERVICE;
    
    let user_id = &auth_data.firebase_user.uid;
    let scope = &auth_data.scope;
    
    // TODO: Extract device info from request headers
    let device_info = Some("Web Browser".to_string()); // Placeholder
    
    REFRESH_TOKEN_SERVICE
        .create_refresh_token(user_id, client_id, scope, device_info)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create refresh token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Failed to generate refresh token".to_string()),
                    error_uri: None,
                }),
            )
        })
}

/// Validate and get refresh token data
async fn validate_and_get_refresh_token(
    app_state: &AppState,
    refresh_token: &str,
) -> Result<RefreshTokenData, Box<dyn std::error::Error>> {
    use crate::dom::values::SessId;
    
    // Get session for refresh token
    let session_id = SessId::from_string(format!("refresh_token:{}", refresh_token));
    let session = app_state.session_repo.get(&session_id).await?
        .ok_or("Refresh token not found")?;
    
    // Deserialize refresh data from access_token field
    let refresh_data: RefreshTokenData = serde_json::from_str(&session.access_token)?;

    // Check if token is too old (7 days for frontend clients)
    if Utc::now() - refresh_data.created_at > Duration::days(7) {
        return Err("Refresh token expired".into());
    }

    Ok(refresh_data)
}

/// Create session for the authenticated user
async fn create_session(
    app_state: &AppState,
    firebase_user: &FirebaseUser,
    access_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::dom::entities::Session;
    use crate::dom::values::UserId;

    let user_id = UserId::new(firebase_user.uid.clone());
    let expires_at = Utc::now() + Duration::hours(24);
    let session = Session::new(user_id, access_token.to_string(), expires_at);

    app_state.session_repo.save(&session).await?;
    Ok(())
}

/// Utility functions

fn get_issuer_url() -> String {
    std::env::var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
}

fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string())
}

/// Generate a unique JWT ID (JTI) for token revocation support
fn generate_jti() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Validate access token and extract claims
fn validate_access_token(token: &str) -> Result<AccessTokenClaims, Box<dyn std::error::Error>> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

    let decoding_key = DecodingKey::from_secret(get_jwt_secret().as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    
    // Validate standard claims
    validation.set_audience(&["epsx-api"]);
    validation.set_issuer(&[&get_issuer_url()]);
    
    let token_data = decode::<AccessTokenClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("JWT validation failed: {}", e))?;
    
    Ok(token_data.claims)
}

/// Get user admin modules from the database
async fn get_user_admin_modules(
    app_state: &AppState,
    email: &str
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Use the admin module service to get user's admin modules
    match app_state.admin_module_service.get_user_admin_modules(email).await {
        Ok(modules) => Ok(modules),
        Err(e) => {
            tracing::warn!("Failed to get admin modules for user {}: {}", email, e);
            // Return default empty modules on error
            Ok(vec![])
        }
    }
}

/// Get user package tier from the database
async fn get_user_package_tier(
    app_state: &AppState,
    email: &str
) -> Result<String, Box<dyn std::error::Error>> {
    use crate::dom::values::Email;
    
    // Try to get user from database
    let user_email = Email::new(email.to_string())?;
    
    match app_state.user_repo.find_by_email(&user_email).await {
        Ok(Some(_user)) => {
            // Extract package tier from user data
            // This depends on your user model structure
            Ok("PREMIUM".to_string()) // Placeholder - replace with actual field
        },
        Ok(None) => {
            tracing::warn!("User not found for email: {}", email);
            Ok("FREE".to_string()) // Default for unknown users
        },
        Err(e) => {
            tracing::error!("Database error getting user package tier: {}", e);
            Ok("FREE".to_string()) // Default on error
        }
    }
}

fn get_role_from_custom_claims(custom_claims: &HashMap<String, serde_json::Value>) -> String {
    custom_claims.get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("user")
        .to_string()
}

fn get_user_permissions_from_role(custom_claims: &HashMap<String, serde_json::Value>) -> Vec<String> {
    let role = get_role_from_custom_claims(custom_claims);
    
    match role.as_str() {
        "super_admin" => vec![
            "api:admin:*".to_string(),
            "route:*".to_string(),
            "users:manage".to_string(),
            "system:configure".to_string(),
            "security:full".to_string(),
        ],
        "admin" => vec![
            "api:admin:*".to_string(),
            "route:*".to_string(),
            "users:manage".to_string(),
            "system:configure".to_string(),
        ],
        "moderator" => vec![
            "api:moderate:*".to_string(),
            "route:/moderate/*".to_string(),
            "content:moderate".to_string(),
            "users:view".to_string(),
        ],
        "premium" => vec![
            "api:premium:*".to_string(),
            "route:/premium/*".to_string(),
            "analytics:read".to_string(),
            "alerts:manage".to_string(),
        ],
        _ => vec![
            "api:basic:read".to_string(),
            "route:/dashboard".to_string(),
            "profile:manage:own".to_string(),
        ],
    }
}

/// Validate PKCE code challenge and verifier
fn validate_pkce(
    challenge: &str, 
    method: &str, 
    verifier: &str
) -> Result<(), Box<dyn std::error::Error>> {
    use sha2::{Sha256, Digest};

    tracing::debug!("Validating PKCE: method={}, challenge={}, verifier={}", 
                   method, challenge, verifier);

    // Validate code_verifier format (RFC 7636)
    if verifier.len() < 43 || verifier.len() > 128 {
        return Err("code_verifier must be 43-128 characters long".into());
    }

    // Check for valid characters (unreserved characters from RFC 3986)
    let valid_chars = verifier.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_' || c == '~'
    });
    
    if !valid_chars {
        return Err("code_verifier contains invalid characters".into());
    }

    match method {
        "S256" => {
            // Create SHA256 hash of the verifier
            let mut hasher = Sha256::new();
            hasher.update(verifier.as_bytes());
            let digest = hasher.finalize();
            
            // Base64URL encode the hash
            let computed_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(&digest);
                
            if computed_challenge == challenge {
                tracing::debug!("PKCE validation successful");
                Ok(())
            } else {
                tracing::error!("PKCE challenge mismatch: expected={}, computed={}", 
                               challenge, computed_challenge);
                Err("PKCE challenge verification failed".into())
            }
        },
        "plain" => {
            // Plain text comparison (not recommended but supported)
            if verifier == challenge {
                tracing::debug!("PKCE plain validation successful");
                Ok(())
            } else {
                tracing::error!("PKCE plain challenge mismatch");
                Err("PKCE plain challenge verification failed".into())
            }
        },
        _ => {
            tracing::error!("Unsupported PKCE method: {}", method);
            Err("Unsupported code challenge method".into())
        }
    }
}

/// GET /oauth/userinfo - UserInfo endpoint
pub async fn oidc_userinfo(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<TokenErrorResponse>)> {
    tracing::debug!("UserInfo endpoint request");
    
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
    
    // Validate and decode the access token
    let token_claims = validate_access_token(access_token)
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

    // Check if token is revoked (if we implement token revocation)
    // TODO: Check JTI against revoked tokens list
    
    // Get additional user information based on the token claims
    let admin_modules = if token_claims.scope.contains("admin") {
        get_user_admin_modules(&app_state, &token_claims.email).await
            .unwrap_or_else(|_| vec!["system_admin".to_string(), "user_management".to_string()])
    } else {
        vec![]
    };

    let package_tier = get_user_package_tier(&app_state, &token_claims.email).await
        .unwrap_or_else(|_| "FREE".to_string());
    
    // Build OpenID Connect UserInfo response
    let userinfo = serde_json::json!({
        "sub": token_claims.sub,
        "email": token_claims.email,
        "email_verified": true, // This should come from the user data
        "name": token_claims.email.split('@').next().unwrap_or("User"),
        "role": token_claims.role,
        "permissions": token_claims.permissions,
        "package_tier": package_tier,
        "admin_modules": admin_modules
    });
    
    tracing::debug!("UserInfo endpoint returning response for user: {}", token_claims.email);
    Ok(Json(userinfo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_permissions_from_role() {
        let mut claims = HashMap::new();
        claims.insert(serde_json::Value::String("role".to_string()), serde_json::Value::String("admin".to_string()));
        
        let permissions = get_user_permissions_from_role(&claims);
        assert!(permissions.contains(&"api:admin:*".to_string()));
        assert!(permissions.contains(&"users:manage".to_string()));
    }

    #[test]
    fn test_get_role_from_custom_claims() {
        let mut claims = HashMap::new();
        claims.insert(serde_json::Value::String("role".to_string()), serde_json::Value::String("admin".to_string()));
        
        let role = get_role_from_custom_claims(&claims);
        assert_eq!(role, "admin");
    }
}