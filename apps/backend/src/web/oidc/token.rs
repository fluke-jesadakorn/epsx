// OIDC Token Endpoint implementation
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use axum::{

    extract::{State, Form, FromRequest},
    response::Json,
    http::{StatusCode, HeaderMap},
    async_trait,
};
use serde::{Deserialize, Serialize};

use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};

use std::collections::HashMap;

use base64::Engine;

use crate::config::env::get_env_var;


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
    // Database-derived fields
    pub permissions: Vec<String>,  // Structured permissions: "platform:resource:action"
    pub package_tier: String,
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
    pub permissions: Vec<String>,  // Structured permissions: "platform:resource:action"
    pub package_tier: String,
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
    tracing::debug!("Token endpoint request: {:?}", token_request);
    
    // Validate required fields and return proper JSON errors
    let grant_type = token_request.grant_type.as_deref().unwrap_or("");
    let client_id = match token_request.client_id.as_deref() {
        Some(id) => id,
        None => {
            tracing::warn!("Missing client_id parameter");
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
    
    tracing::debug!("Token request - grant_type={}, client_id={}, has_code={}, has_redirect_uri={}, has_code_verifier={}", 
                   grant_type, client_id, token_request.code.is_some(), token_request.redirect_uri.is_some(), token_request.code_verifier.is_some());

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
    tracing::debug!("Handling authorization code grant");
    
    // Validate required parameters
    let code = token_request.code.ok_or_else(|| {
        tracing::warn!("Missing code parameter");
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
        tracing::warn!("Missing redirect_uri parameter");
        (
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing 'redirect_uri' parameter".to_string()),
                error_uri: None,
            }),
        )
    })?;

    tracing::debug!("Validating authorization code");
    
    // Validate and consume authorization code
    let auth_data = validate_and_consume_authorization_code(&app_state, &code).await
        .map_err(|e| {
            tracing::warn!("Authorization code validation failed: {}", e);
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

    // Generate access token with database user data
    let access_token = generate_access_token(&app_state, &auth_data.firebase_user, &auth_data.scope, now, expires_in).await?;

    // Generate ID token with database user data
    let id_token = generate_id_token(&app_state, &auth_data.firebase_user, client_id, now, expires_in).await?;

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

    // TODO: For refresh token flow, we need to pass app_state to get fresh database data
    // For now, generating tokens without database lookup (refresh tokens should be short-lived)
    let access_token = generate_access_token_simple(&firebase_user, &rotation.new_token_data.scope, now, expires_in)?;
    let id_token = generate_id_token_simple(&firebase_user, client_id, now, expires_in)?;

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
    tracing::debug!("Looking for session with ID: {}", session_id);
    
    let session_result = app_state.session_repo.get(&session_id).await;
    tracing::debug!("Session repo result: success={}", session_result.is_ok());
    
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

/// Generate access token with database user data and admin/user context detection
async fn generate_access_token(
    app_state: &AppState,
    firebase_user: &FirebaseUser,
    scope: &str,
    now: DateTime<Utc>,
    expires_in: i64,
) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
    // Get user data from database for accurate permissions
    let (_, package_tier, database_permissions) = 
        get_user_database_info(app_state, &firebase_user.uid, firebase_user.email.as_deref().unwrap_or("")).await;
    
    // Combine Firebase role with database permissions
    let firebase_role = get_role_from_custom_claims(&firebase_user.custom_claims);
    let effective_permissions = if !database_permissions.is_empty() {
        database_permissions // Use database permissions if available
    } else {
        get_user_permissions_from_role(&firebase_user.custom_claims) // Fallback to Firebase role-based permissions
    };
    
    // Detect admin context based on role and permissions
    let is_admin_context = is_admin_user(&firebase_role, &effective_permissions, scope);
    
    if is_admin_context {
        generate_admin_access_token(firebase_user, &firebase_role, &effective_permissions, now, expires_in)
    } else {
        generate_user_access_token(firebase_user, &firebase_role, &effective_permissions, &package_tier, now, expires_in).await
    }
}

/// Generate admin access token using AdminJWTService
fn generate_admin_access_token(
    firebase_user: &FirebaseUser,
    role: &str,
    permissions: &[String],
    now: DateTime<Utc>,
    _expires_in: i64,
) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
    use crate::auth::admin_jwt::{AdminJWTService, AdminSecurityContext, AdminPermissionMatrix};
    use std::collections::HashMap;
    
    let jwt_secret = get_jwt_secret();
    let admin_service = AdminJWTService::new(jwt_secret.as_bytes(), get_issuer_url());
    
    // Build admin security context
    let security_context = AdminSecurityContext {
        mfa_verified: true, // TODO: Get from actual MFA status
        mfa_timestamp: Some(now.timestamp() as u64),
        risk_score: 0.1, // Low risk for initial login
        risk_factors: vec![],
        device_binding: "web-session".to_string(), // TODO: Generate from request
        ip_restrictions: vec![],
        current_ip: "0.0.0.0".to_string(), // TODO: Extract from request
        location_hash: "unknown".to_string(),
        session_start: now.timestamp() as u64,
        last_activity: now.timestamp() as u64,
    };
    
    // Build admin permission matrix
    let admin_permissions = AdminPermissionMatrix {
        platforms: HashMap::new(), // TODO: Build from permissions
        system_access: crate::auth::admin_jwt::SystemAccessLevel {
            level: role.to_string(),
            capabilities: permissions.to_vec(),
            restrictions: vec![],
            monitoring_level: "standard".to_string(),
        },
        delegation_rights: vec![],
        emergency_access: None,
        version: 1,
        hash: "admin-permissions-v1".to_string(), // TODO: Proper hash generation
    };
    
    admin_service.generate_admin_token(
        firebase_user.uid.clone(),
        firebase_user.email.clone().unwrap_or_default(),
        firebase_user.display_name.clone().unwrap_or_else(|| "Admin".to_string()),
        role.to_string(),
        security_context,
        admin_permissions,
    ).map_err(|e| {
        tracing::error!("Failed to generate admin access token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TokenErrorResponse {
                error: "server_error".to_string(),
                error_description: Some("Failed to generate admin access token".to_string()),
                error_uri: None,
            }),
        )
    })
}

/// Generate user access token using UserJWTService
async fn generate_user_access_token(
    firebase_user: &FirebaseUser,
    _role: &str,
    permissions: &[String],
    package_tier: &str,
    now: DateTime<Utc>,
    _expires_in: i64,
) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
    use crate::auth::user_jwt::{UserJWTService, UserContext, UserPreferences, UserSubscription};
    use std::collections::HashMap;
    
    let jwt_secret = get_jwt_secret();
    let user_service = UserJWTService::new(jwt_secret.as_bytes(), get_issuer_url());
    
    // Build user context
    let user_context = UserContext {
        tier: package_tier.to_string(),
        verified: firebase_user.email_verified,
        created_at: firebase_user.created_at.timestamp() as u64,
        last_login: now.timestamp() as u64,
        preferences: UserPreferences {
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            currency: "USD".to_string(),
            theme: Some("light".to_string()),
        },
    };
    
    // Build subscription based on package tier
    let subscription = if package_tier != "FREE" {
        Some(UserSubscription {
            tier: package_tier.to_string(),
            status: "active".to_string(),
            expires_at: None, // TODO: Get from database
            features: determine_features_for_tier(package_tier),
            limits: determine_limits_for_tier(package_tier),
            usage: HashMap::new(),
        })
    } else {
        None
    };
    
    user_service.generate_user_token(
        firebase_user.uid.clone(),
        firebase_user.email.clone().unwrap_or_default(),
        firebase_user.display_name.clone(),
        user_context,
        permissions.to_vec(),
        subscription,
    ).map_err(|e| {
        tracing::error!("Failed to generate user access token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TokenErrorResponse {
                error: "server_error".to_string(),
                error_description: Some("Failed to generate user access token".to_string()),
                error_uri: None,
            }),
        )
    })
}

/// Generate access token without database lookup (for refresh token flow)
fn generate_access_token_simple(
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
        package_tier: "FREE".to_string(), // Default for refresh token flow
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

/// Generate ID token with database user data
async fn generate_id_token(
    app_state: &AppState,
    firebase_user: &FirebaseUser,
    client_id: &str,
    now: DateTime<Utc>,
    expires_in: i64,
) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
    // Get user data from database for accurate information
    let (_, package_tier, database_permissions) = 
        get_user_database_info(app_state, &firebase_user.uid, firebase_user.email.as_deref().unwrap_or("")).await;
    
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
        permissions: database_permissions,
        package_tier,
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

/// Generate ID token without database lookup (for refresh token flow)
fn generate_id_token_simple(
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
        permissions: vec![], // Empty for refresh token flow
        package_tier: "FREE".to_string(), // Default for refresh token flow
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


/// Create session for the authenticated user
async fn create_session(
    app_state: &AppState,
    firebase_user: &FirebaseUser,
    access_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::dom::entities::Session;
    

    // Look up user in database by firebase_uid to get the proper UUID
    let user = app_state.user_repo.find_by_firebase_uid(&firebase_user.uid).await?;
    let user_id = match user {
        Some(user) => user.id().clone(),
        None => {
            tracing::error!("User not found for firebase_uid: {}", firebase_user.uid);
            return Err("User not found in database".into());
        }
    };
    
    let expires_at = Utc::now() + Duration::hours(24);
    let session = Session::new(user_id, access_token.to_string(), expires_at);

    app_state.session_repo.save(&session).await?;
    Ok(())
}

/// Utility functions

fn get_issuer_url() -> String {
    get_env_var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
}

fn get_jwt_secret() -> String {
    get_env_var("NEXTAUTH_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string())
}

/// Generate a unique JWT ID (JTI) for token revocation support
fn generate_jti() -> String {
    Uuid::new_v4().to_string()
}

/// Validate unified access token (supports both admin and user tokens)
/// Returns (sub, email, permissions, package_tier)
fn validate_unified_access_token(token: &str) -> Result<(String, String, Vec<String>, String), Box<dyn std::error::Error>> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
    
    let decoding_key = DecodingKey::from_secret(get_jwt_secret().as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["epsx-api"]);
    validation.set_issuer(&[&get_issuer_url()]);
    
    // Try to decode as admin token first by checking token_type
    if let Ok(admin_token) = decode::<crate::auth::admin_jwt::AdminJWTClaims>(token, &decoding_key, &validation) {
        if admin_token.claims.token_type == "admin_access" {
            tracing::debug!("Validated as admin token for user: {}", admin_token.claims.email);
            
            // Extract permissions from admin token
            let permissions: Vec<String> = admin_token.claims.permissions.system_access.capabilities;
            
            return Ok((
                admin_token.claims.sub,
                admin_token.claims.email,
                permissions,
                "ADMIN".to_string(), // Admin users get special package tier
            ));
        }
    }
    
    // Try to decode as user token by checking token_type
    if let Ok(user_token) = decode::<crate::auth::user_jwt::UserJWTClaims>(token, &decoding_key, &validation) {
        if user_token.claims.token_type == "user_access" {
            tracing::debug!("Validated as user token for user: {}", user_token.claims.email);
            
            // Extract permissions from user token
            let permissions = user_token.claims.permissions.permissions;
            let package_tier = user_token.claims.subscription
                .as_ref()
                .map(|s| s.tier.clone())
                .unwrap_or_else(|| "FREE".to_string());
            
            return Ok((
                user_token.claims.sub,
                user_token.claims.email,
                permissions,
                package_tier,
            ));
        }
    }
    
    // Try legacy token format as fallback (no token_type field)
    if let Ok(legacy_token) = decode::<AccessTokenClaims>(token, &decoding_key, &validation) {
        tracing::debug!("Validated as legacy token for user: {}", legacy_token.claims.email);
        
        return Ok((
            legacy_token.claims.sub,
            legacy_token.claims.email,
            legacy_token.claims.permissions,
            legacy_token.claims.package_tier,
        ));
    }
    
    Err("Token validation failed for all supported formats".into())
}

/// Validate access token and extract claims (legacy function)
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

/// Get user permissions from the database (deprecated - use get_user_database_info instead)
async fn get_user_admin_modules(
    _app_state: &AppState,
    email: &str
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Simplified role system - no admin modules needed
    tracing::info!("Using structured permissions system for user: {}", email);
    Ok(vec![]) // Return empty - using structured permissions instead
}

/// Get user permissions from the database using Firebase UID (deprecated - integrated into get_user_database_info)
async fn get_user_admin_modules_from_db(
    _app_state: &AppState,
    firebase_uid: &str
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Legacy function - structured permissions are now handled by get_user_database_info
    tracing::info!("Legacy admin modules lookup for user {}: using structured permissions instead", firebase_uid);
    Ok(vec![]) // Return empty - use structured permissions from domain entity instead
}

/// Get comprehensive user database information for JWT token generation
/// Returns (legacy_placeholder, package_tier, permissions)
async fn get_user_database_info(
    app_state: &AppState,
    firebase_uid: &str,
    _email: &str
) -> (Vec<String>, String, Vec<String>) {
    // Legacy placeholder for compatibility - admin_modules no longer used
    let _admin_modules_placeholder = vec![];
    
    // Get user data from database using Firebase UID for package tier and permissions
    let (package_tier, permissions) = match app_state.user_repo.find_by_firebase_uid(firebase_uid).await {
        Ok(Some(user)) => {
            let tier = match user.subscription().tier {
                crate::dom::values::SubscriptionTier::Free => "FREE".to_string(),
                crate::dom::values::SubscriptionTier::Basic => "BASIC".to_string(),
                crate::dom::values::SubscriptionTier::Premium => "PREMIUM".to_string(),
                crate::dom::values::SubscriptionTier::Enterprise => "ENTERPRISE".to_string(),
            };
            
            // Get user permissions from separate table via PermissionApplicationService
            let user_permissions: Vec<String> = match app_state.permission_application_service.get_user_permissions(user.firebase_uid()).await {
                Ok(permissions) => permissions,
                Err(e) => {
                    tracing::error!("Failed to fetch permissions for JWT token {}: {:?}", user.id(), e);
                    vec![] // Default to empty permissions on error
                }
            };
            
            tracing::debug!("Retrieved user data for {}: tier={}, {} permissions", firebase_uid, tier, user_permissions.len());
            (tier, user_permissions)
        },
        Ok(None) => {
            tracing::warn!("User not found in database for Firebase UID: {}", firebase_uid);
            ("FREE".to_string(), vec![])
        },
        Err(e) => {
            tracing::error!("Database error getting user data for {}: {}", firebase_uid, e);
            ("FREE".to_string(), vec![])
        }
    };
    
    // Use permissions from domain entity (no legacy admin_modules enhancement needed)
    let final_permissions = permissions;
    
    tracing::info!("User database info for {}: {} tier, {} permissions", 
                  firebase_uid, package_tier, final_permissions.len());
    
    (_admin_modules_placeholder, package_tier, final_permissions)
}

/// Get user package tier from the database (legacy function - use get_user_database_info instead)
async fn get_user_package_tier(
    app_state: &AppState,
    email: &str
) -> Result<String, Box<dyn std::error::Error>> {
    use crate::dom::values::Email;
    
    // Try to get user from database
    let user_email = Email::new(email.to_string())?;
    
    match app_state.user_repo.find_by_email(&user_email).await {
        Ok(Some(user)) => {
            // Extract package tier from user data
            let tier = match user.subscription().tier {
                crate::dom::values::SubscriptionTier::Free => "FREE".to_string(),
                crate::dom::values::SubscriptionTier::Basic => "BASIC".to_string(),
                crate::dom::values::SubscriptionTier::Premium => "PREMIUM".to_string(),
                crate::dom::values::SubscriptionTier::Enterprise => "ENTERPRISE".to_string(),
            };
            Ok(tier)
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
        "admin" => vec![
            "api:admin:*".to_string(),
            "route:*".to_string(),
            "users:manage".to_string(),
            "system:configure".to_string(),
            "security:full".to_string(),
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
    State(_app_state): State<AppState>,
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
    
    // Validate and decode the access token (supports both admin and user tokens)
    let (sub, email, permissions, package_tier) = validate_unified_access_token(access_token)
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
    
    // Build OpenID Connect UserInfo response
    let userinfo = serde_json::json!({
        "sub": sub,
        "email": email,
        "email_verified": true, // This should come from the user data
        "name": email.split('@').next().unwrap_or("User"),
        "permissions": permissions,
        "package_tier": package_tier
    });
    
    tracing::debug!("UserInfo endpoint returning response for user: {}", email);
    Ok(Json(userinfo))
}

/// Detect if user should get admin-level JWT tokens
fn is_admin_user(role: &str, permissions: &[String], scope: &str) -> bool {
    // Check role-based admin detection
    let is_admin_role = matches!(role, "admin" | "super_admin" | "moderator");
    
    // Check permission-based admin detection
    let has_admin_permissions = permissions.iter().any(|p| {
        p.starts_with("admin:") || p.starts_with("system:") || p.contains("admin")
    });
    
    // Check scope-based admin detection (if admin scopes requested)
    let has_admin_scope = scope.contains("admin") || scope.contains("system");
    
    is_admin_role || has_admin_permissions || has_admin_scope
}

/// Determine features available for subscription tier
fn determine_features_for_tier(tier: &str) -> Vec<String> {
    match tier {
        "ENTERPRISE" => vec![
            "premium_analytics".to_string(),
            "api_access".to_string(),
            "custom_alerts".to_string(),
            "priority_support".to_string(),
            "white_label".to_string(),
        ],
        "PREMIUM" => vec![
            "premium_analytics".to_string(),
            "api_access".to_string(),
            "custom_alerts".to_string(),
            "priority_support".to_string(),
        ],
        "BASIC" => vec![
            "basic_analytics".to_string(),
            "standard_alerts".to_string(),
        ],
        _ => vec!["basic_access".to_string()],
    }
}

/// Determine usage limits for subscription tier
fn determine_limits_for_tier(tier: &str) -> std::collections::HashMap<String, u32> {
    use std::collections::HashMap;
    
    let mut limits = HashMap::new();
    match tier {
        "ENTERPRISE" => {
            limits.insert("api_calls_per_hour".to_string(), 10000);
            limits.insert("data_exports_per_day".to_string(), 100);
            limits.insert("custom_alerts".to_string(), 1000);
        }
        "PREMIUM" => {
            limits.insert("api_calls_per_hour".to_string(), 1000);
            limits.insert("data_exports_per_day".to_string(), 20);
            limits.insert("custom_alerts".to_string(), 100);
        }
        "BASIC" => {
            limits.insert("api_calls_per_hour".to_string(), 100);
            limits.insert("data_exports_per_day".to_string(), 5);
            limits.insert("custom_alerts".to_string(), 10);
        }
        _ => {
            limits.insert("api_calls_per_hour".to_string(), 20);
            limits.insert("data_exports_per_day".to_string(), 1);
            limits.insert("custom_alerts".to_string(), 1);
        }
    }
    limits
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