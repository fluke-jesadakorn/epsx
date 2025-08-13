// OIDC Token Endpoint implementation

use axum::{
    extract::{State, Form},
    response::Json,
    http::{StatusCode, HeaderMap},
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use std::collections::HashMap;
use base64::Engine;

use crate::web::auth::AppState;
use crate::web::oidc::authorization::AuthorizationCodeData;
use crate::infra::firebase_admin::FirebaseUser;

/// Token request parameters (POST /oauth/token)
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
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

/// OIDC Error Response
#[derive(Debug, Serialize)]
pub struct TokenErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

/// JWT Claims for ID token
#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
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
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
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
    Form(token_request): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenErrorResponse>)> {
    tracing::info!("Token endpoint request for grant_type: {}", token_request.grant_type);

    match token_request.grant_type.as_str() {
        "authorization_code" => handle_authorization_code_grant(app_state, token_request).await,
        "refresh_token" => handle_refresh_token_grant(app_state, token_request).await,
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
    // Validate required parameters
    let code = token_request.code.ok_or_else(|| {
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
        (
            StatusCode::BAD_REQUEST,
            Json(TokenErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing 'redirect_uri' parameter".to_string()),
                error_uri: None,
            }),
        )
    })?;

    // Validate and consume authorization code
    let auth_data = validate_and_consume_authorization_code(&app_state, &code).await
        .map_err(|e| {
            tracing::error!("Authorization code validation failed: {}", e);
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
    if auth_data.client_id != token_request.client_id {
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

    // TODO: Validate PKCE if code_challenge was used
    // if let (Some(challenge), Some(verifier)) = (&auth_data.code_challenge, &token_request.code_verifier) {
    //     validate_pkce(challenge, verifier)?;
    // }

    // Generate tokens
    let now = Utc::now();
    let expires_in = 7200; // 2 hours (frontend client policy)

    // Generate access token
    let access_token = generate_access_token(&auth_data.firebase_user, &auth_data.scope, now, expires_in)?;

    // Generate ID token
    let id_token = generate_id_token(&auth_data.firebase_user, &token_request.client_id, now, expires_in)?;

    // Generate refresh token
    let refresh_token = generate_refresh_token(&app_state, &auth_data).await?;

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

/// Handle refresh token grant
async fn handle_refresh_token_grant(
    app_state: AppState,
    token_request: TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenErrorResponse>)> {
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

    // Validate and get refresh token data
    let refresh_data = validate_and_get_refresh_token(&app_state, &refresh_token).await
        .map_err(|e| {
            tracing::error!("Refresh token validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(TokenErrorResponse {
                    error: "invalid_grant".to_string(),
                    error_description: Some("Invalid or expired refresh token".to_string()),
                    error_uri: None,
                }),
            )
        })?;

    // Validate client_id
    if refresh_data.client_id != token_request.client_id {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(TokenErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Client ID mismatch".to_string()),
                error_uri: None,
            }),
        ));
    }

    // Generate new tokens
    let now = Utc::now();
    let expires_in = 7200; // 2 hours (frontend client policy)

    let access_token = generate_access_token(&refresh_data.firebase_user, &refresh_data.scope, now, expires_in)?;
    let id_token = generate_id_token(&refresh_data.firebase_user, &token_request.client_id, now, expires_in)?;

    tracing::info!("Token refresh successful for user: {}", refresh_data.firebase_user.email.as_ref().unwrap_or(&"unknown".to_string()));

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
        id_token,
        refresh_token: Some(refresh_token), // Return the same refresh token
        scope: refresh_data.scope,
    }))
}

/// Validate and consume authorization code
async fn validate_and_consume_authorization_code(
    app_state: &AppState,
    code: &str,
) -> Result<AuthorizationCodeData, Box<dyn std::error::Error>> {
    use crate::dom::values::SessId;
    
    // Get session for authorization code
    let session_id = SessId::from_string(format!("auth_code:{}", code));
    let session = app_state.session_repo.get(&session_id).await?
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
        iss: get_issuer_url(),
        sub: firebase_user.uid.clone(),
        aud: "epsx-api".to_string(),
        exp: (now + Duration::seconds(expires_in)).timestamp(),
        iat: now.timestamp(),
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
        iss: get_issuer_url(),
        sub: firebase_user.uid.clone(),
        aud: client_id.to_string(),
        exp: (now + Duration::seconds(expires_in)).timestamp(),
        iat: now.timestamp(),
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

/// Generate refresh token
async fn generate_refresh_token(
    app_state: &AppState,
    auth_data: &AuthorizationCodeData,
) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
    // Generate random bytes first, before any async operations
    use rand::Rng;
    let random_bytes: [u8; 32] = {
        let mut rng = rand::thread_rng();
        rng.gen()
    };
    let refresh_token = format!("rt_{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes));

    let refresh_data = RefreshTokenData {
        firebase_user: auth_data.firebase_user.clone(),
        client_id: auth_data.client_id.clone(),
        scope: auth_data.scope.clone(),
        created_at: Utc::now(),
    };

    let serialized = serde_json::to_string(&refresh_data)
        .map_err(|e| {
            tracing::error!("Failed to serialize refresh token data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Failed to generate refresh token".to_string()),
                    error_uri: None,
                }),
            )
        })?;

    // Store with 30 day expiration using session repository
    {
        use crate::dom::entities::auth::Session;
        use crate::dom::values::{SessId, UserId};
        
        let session_id = SessId::from_string(format!("refresh_token:{}", refresh_token));
        let user_id = UserId::new(auth_data.firebase_user.uid.clone());
        let expires_at = Utc::now() + Duration::days(7); // 7 days for frontend clients
        
        let session = Session {
            id: session_id,
            user_id,
            access_token: serialized, // Store serialized refresh data here
            refresh_token: None,
            expires_at,
            created_at: Utc::now(),
            is_active: true,
        };
        
        app_state.session_repo.save(&session).await
            .map_err(|e| {
                tracing::error!("Failed to store refresh token: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(TokenErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some("Failed to store refresh token".to_string()),
                        error_uri: None,
                    }),
                )
            })?;
    }

    Ok(refresh_token)
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

/// GET /oauth/userinfo - UserInfo endpoint
pub async fn oidc_userinfo(
    State(_app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<TokenErrorResponse>)> {
    tracing::info!("UserInfo endpoint request");
    
    // Extract bearer token from Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
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
    
    let _access_token = &auth_header[7..]; // Remove "Bearer " prefix
    
    // TODO: Validate access token and extract user information
    // For now, return a mock response
    let userinfo = serde_json::json!({
        "sub": "user123",
        "email": "user@example.com",
        "email_verified": true,
        "name": "Demo User",
        "role": "user"
    });
    
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