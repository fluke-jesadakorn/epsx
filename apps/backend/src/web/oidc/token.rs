/// OIDC Token Endpoint Implementation
/// 
/// Main token endpoint handlers using modular architecture.
/// This module maintains the existing API while delegating to specialized services.

use chrono::{DateTime, Utc, Duration};

use axum::{
    extract::{State, Form, FromRequest},
    response::Json,
    http::{StatusCode, HeaderMap},
    async_trait,
};
use serde::{Deserialize, Serialize};

use crate::domain::shared_kernel::value_objects::SessionId;
use crate::domain::shared_kernel::AggregateRoot;
use crate::web::auth::AppState;
use crate::infrastructure::adapters::services::firebase::FirebaseUser;

// Import refactored modules
use super::{
    token_generator::TokenGenerator,
    token_validator::TokenValidator,
    refresh_handler::RefreshHandler,
    crypto_manager::{CryptoManager, PkceMethod},
};


/// OIDC Error Response
#[derive(Debug, Serialize)]
pub struct TokenErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

impl TokenErrorResponse {
    pub fn server_error() -> Self {
        Self {
            error: "server_error".to_string(),
            error_description: Some("Internal server error occurred".to_string()),
            error_uri: None,
        }
    }
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

// Re-export types from modules for backward compatibility
pub use super::token_generator::{IdTokenClaims, AccessTokenClaims};
pub use super::refresh_handler::RefreshTokenData;

// Legacy RefreshTokenData is kept for backward compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct LegacyRefreshTokenData {
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
    
    // Initialize services
    let token_validator = TokenValidator::new();
    let token_generator = TokenGenerator::new();
    let crypto_manager = CryptoManager::new();
    
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
    
    // Validate and consume authorization code using TokenValidator
    let auth_data = token_validator.validate_and_consume_authorization_code(&code)
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
        
        let pkce_method = PkceMethod::from_str(method).map_err(|e| {
            tracing::error!("Invalid PKCE method: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(TokenErrorResponse {
                    error: "invalid_request".to_string(),
                    error_description: Some("Invalid code challenge method".to_string()),
                    error_uri: None,
                }),
            )
        })?;
        
        crypto_manager.validate_pkce(challenge, &pkce_method, verifier).map_err(|e| {
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

    // Generate tokens using TokenGenerator
    let now = Utc::now();
    let expires_in = 7200; // 2 hours (frontend client policy)

    let access_token = token_generator.generate_access_token(&app_state, &auth_data.firebase_user, &auth_data.scope, now, expires_in).await?;
    let id_token = token_generator.generate_id_token(&app_state, &auth_data.firebase_user, client_id, now, expires_in).await?;
    let refresh_token = token_generator.generate_refresh_token_v2(&auth_data, client_id).await?;

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
    app_state: AppState,
    token_request: TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenErrorResponse>)> {
    let refresh_handler = RefreshHandler::new();
    
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

    let client_id = token_request.client_id.as_deref().unwrap_or("").to_string();

    // Delegate to RefreshHandler
    refresh_handler.handle_refresh_token_grant(app_state, refresh_token, client_id).await
}

/// Create session for the authenticated user
async fn create_session(
    app_state: &AppState,
    firebase_user: &FirebaseUser,
    access_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    

    tracing::info!("🔄 Creating session for Firebase user: {} (email: {})", 
        firebase_user.uid, 
        firebase_user.email.as_deref().unwrap_or("no-email"));
    
    // Look up user in database by firebase_uid, create if doesn't exist
    let firebase_uid_vo = crate::domain::user_management::value_objects::FirebaseUid::new(&firebase_user.uid)
        .map_err(|e| format!("Invalid Firebase UID: {}", e))?;
    
    let user = app_state.user_repo.find_by_firebase_uid(&firebase_uid_vo).await?;
    let user_id = match user {
        Some(user) => {
            tracing::info!("✅ Found existing user in database: {} (ID: {})", 
                firebase_user.uid, user.id().as_str());
            user.id().clone()
        }
        None => {
            tracing::info!("👤 User not found in database, creating new user for Firebase UID: {}", firebase_user.uid);
            
            // Create new user using the same logic as token exchange
            let user_id = create_user_from_firebase_data(app_state, firebase_user).await
                .map_err(|e| {
                    tracing::error!("❌ Failed to create user from Firebase data: {}", e);
                    format!("Failed to create user: {}", e)
                })?;
            
            tracing::info!("✅ Successfully created new user: {} (ID: {})", firebase_user.uid, user_id.as_str());
            user_id
        }
    };
    
    let expires_at = Utc::now() + Duration::hours(24);
    let session_id = SessionId::new();
    let _session = crate::domain::user_management::aggregates::session::Session::create(
        session_id,
        user_id,
        access_token.to_string(),
        expires_at,
        None, // ip_address
        None, // user_agent
    ).map_err(|e| format!("Failed to create session: {}", e))?;

    // TODO: Remove session storage - using stateless Bearer tokens
    // app_state.session_repo.save(&session).await?;
    tracing::debug!("Session storage skipped - using stateless Bearer tokens");
    Ok(())
}

/// Create a new user from Firebase data
async fn create_user_from_firebase_data(
    app_state: &AppState,
    firebase_user: &FirebaseUser,
) -> Result<crate::domain::shared_kernel::value_objects::UserId, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("🔨 Creating new user from Firebase data: {} (email: {})", 
        firebase_user.uid, 
        firebase_user.email.as_deref().unwrap_or("no-email"));

    // Validate Firebase UID format
    if firebase_user.uid.is_empty() {
        return Err("Firebase UID cannot be empty".into());
    }
    
    if firebase_user.uid.len() < 10 || firebase_user.uid.len() > 128 {
        return Err(format!("Invalid Firebase UID length: {} (must be 10-128 characters)", firebase_user.uid.len()).into());
    }

    // Validate email is present
    let email_str = firebase_user.email.as_ref()
        .ok_or("Email is required for user registration")?;
    
    if email_str.is_empty() {
        return Err("Email cannot be empty".into());
    }

    // Create domain value objects
    let firebase_uid = crate::domain::user_management::value_objects::FirebaseUid::new(&firebase_user.uid)
        .map_err(|e| format!("Firebase UID validation failed: {}", e))?;
    
    let email = crate::domain::user_management::value_objects::Email::new(email_str.clone())
        .map_err(|e| format!("Invalid email format '{}': {}", email_str, e))?;

    // Check if user already exists by email 
    if let Ok(Some(existing_user)) = app_state.user_repo.find_by_email(&email).await {
        if existing_user.firebase_uid().to_string() != firebase_user.uid {
            tracing::warn!("⚠️ Email '{}' exists with different Firebase UID: {} (existing) vs {} (new)", 
                email_str, existing_user.firebase_uid(), firebase_user.uid);
            
            // Handle Firebase UID change - update the existing user instead of creating new one
            tracing::info!("🔄 Updating existing user's Firebase UID: {} -> {}", 
                existing_user.firebase_uid(), firebase_user.uid);
            
            return update_user_firebase_uid(app_state, existing_user, &firebase_uid).await
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, 
                    format!("Failed to update user Firebase UID: {}", e))) as Box<dyn std::error::Error + Send + Sync>);
        } else {
            tracing::info!("✅ User already exists with same email and Firebase UID: {}", email_str);
            return Ok(existing_user.id().clone());
        }
    }

    // Create new user domain object
    let user_id = crate::domain::shared_kernel::value_objects::UserId::new();
    let new_user = crate::domain::user_management::aggregates::user::User::create(
        user_id.clone(), 
        firebase_uid, 
        email
    ).map_err(|e| format!("User creation failed: {}", e))?;

    // Save user to database
    tracing::info!("💾 Saving new user to database: {} (ID: {})", firebase_user.uid, user_id.as_str());
    app_state.user_repo.save(&new_user).await
        .map_err(|e| format!("Failed to save user to database: {}", e))?;

    // Grant default permissions
    if let Err(e) = grant_default_permissions_to_new_user(app_state, &new_user).await {
        tracing::warn!("⚠️ Failed to grant default permissions to new user {}: {}", firebase_user.uid, e);
        // Don't fail user creation, just log the warning
    }

    tracing::info!("✅ User created successfully: {} (ID: {})", firebase_user.uid, user_id.as_str());
    Ok(user_id)
}

/// Grant default permissions to newly created user
async fn grant_default_permissions_to_new_user(
    app_state: &AppState,
    user: &crate::domain::user_management::aggregates::user::User,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("🔐 Granting default permissions to new user: {}", user.firebase_uid());
    
    // Define default permissions for new users
    let default_permissions = vec![
        "epsx:analytics:view",
        "epsx:profile:manage",
        "epsx:rankings:view",
        "epsx:realtime:access",
        "epsx:data:export",
        "epsx:notifications:manage"
    ];
    
    let mut user_with_permissions = user.clone();
    let mut permissions_granted = 0;
    
    // Grant each default permission
    for permission_str in default_permissions {
        match crate::domain::user_management::value_objects::Permission::new(permission_str) {
            Ok(permission) => {
                match user_with_permissions.grant_permission(permission.clone(), None) {
                    Ok(_) => {
                        tracing::info!("✅ Granted permission '{}' to user: {}", permission_str, user.firebase_uid());
                        permissions_granted += 1;
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to grant permission '{}' to user {}: {}", 
                            permission_str, user.firebase_uid(), e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("❌ Invalid permission format '{}': {}", permission_str, e);
            }
        }
    }
    
    // Save user with granted permissions to database
    if permissions_granted > 0 {
        tracing::info!("💾 Saving user with {} granted permissions to database: {}", 
            permissions_granted, user.firebase_uid());
        
        app_state.user_repo.save(&user_with_permissions).await
            .map_err(|e| format!("Failed to save user permissions to database: {}", e))?;
        
        tracing::info!("✅ User permissions saved successfully: {}", user.firebase_uid());
    }
    
    Ok(())
}

/// Update an existing user's Firebase UID when it changes
async fn update_user_firebase_uid(
    app_state: &AppState,
    existing_user: crate::domain::user_management::aggregates::user::User,
    new_firebase_uid: &crate::domain::user_management::value_objects::FirebaseUid,
) -> Result<crate::domain::shared_kernel::value_objects::UserId, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("🔧 Updating Firebase UID for existing user: {} (email: {})", 
        existing_user.firebase_uid(), existing_user.email().as_str());
    
    let old_firebase_uid = existing_user.firebase_uid().clone();
    let user_id = existing_user.id().clone();
    let email = existing_user.email().clone();
    
    let permissions = existing_user.permissions().clone();
    let is_email_verified = existing_user.is_email_verified();
    let created_at = existing_user.created_at();
    
    tracing::info!("📋 Preserving {} permissions and verification status: {}", 
        permissions.len(), is_email_verified);
    
    let updated_user = crate::domain::user_management::aggregates::user::User::load(
        user_id.clone(),
        new_firebase_uid.clone(),
        email,
        existing_user.is_active(),
        is_email_verified,
        permissions,
        created_at,
        chrono::Utc::now(),
        existing_user.last_login_at(),
        existing_user.version(),
    );
    
    // Save the updated user
    tracing::info!("💾 Saving updated user with new Firebase UID: {} -> {}", 
        old_firebase_uid, new_firebase_uid);
    
    app_state.user_repo.save(&updated_user).await
        .map_err(|e| format!("Failed to save updated user: {}", e))?;
    
    tracing::info!("✅ Successfully updated user Firebase UID: {} (email: {})", 
        new_firebase_uid, updated_user.email().as_str());
    
    Ok(user_id)
}

/// GET /oauth/userinfo - UserInfo endpoint
pub async fn oidc_userinfo(
    State(_app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<TokenErrorResponse>)> {
    tracing::debug!("UserInfo endpoint request");
    
    let token_validator = TokenValidator::new();
    
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
    
    let access_token = token_validator.extract_bearer_token(auth_header)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(TokenErrorResponse {
                    error: "invalid_token".to_string(),
                    error_description: Some("Invalid Authorization header format".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    // Validate and decode the access token (supports both admin and user tokens)
    let (sub, email, permissions, package_tier) = token_validator.validate_unified_access_token(access_token)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Basic test to ensure module compiles
        assert!(true);
    }
}