use axum::{
    extract::{State, Form},
    response::Json,
    http::{StatusCode, HeaderMap},
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use crate::config::env::get_env_var;

use crate::web::auth::AppState;
use crate::infra::firebase_admin::FirebaseUser;
use super::{jwt, flow};

/// Token request
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

/// ID Token claims for OIDC
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
    pub nonce: Option<String>,
}

/// Refresh token data
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshData {
    pub firebase_user: FirebaseUser,
    pub client_id: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Invalid grant")]
    InvalidGrant,
    #[error("Invalid client")]
    InvalidClient,
    #[error("Unsupported grant type")]
    UnsupportedGrantType,
    #[error("Server error: {0}")]
    ServerError(String),
}

impl Error {
    pub fn to_response(&self) -> (StatusCode, Json<ErrorResponse>) {
        let (error_code, description) = match self {
            Error::InvalidRequest(msg) => ("invalid_request", Some(msg.clone())),
            Error::InvalidGrant => ("invalid_grant", Some("Invalid or expired authorization code".to_string())),
            Error::InvalidClient => ("invalid_client", Some("Client authentication failed".to_string())),
            Error::UnsupportedGrantType => ("unsupported_grant_type", Some("Only authorization_code and refresh_token are supported".to_string())),
            Error::ServerError(msg) => ("server_error", Some(msg.clone())),
        };

        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: error_code.to_string(),
                error_description: description,
                error_uri: None,
            })
        )
    }
}

/// POST /oauth/token - Token endpoint
pub async fn token(
    State(app_state): State<AppState>,
    Form(request): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("Token request: grant_type={}, client_id={}", request.grant_type, request.client_id);

    let result = match request.grant_type.as_str() {
        "authorization_code" => handle_auth_code(app_state, request).await,
        "refresh_token" => handle_refresh(app_state, request).await,
        _ => Err(Error::UnsupportedGrantType),
    };

    result.map(Json).map_err(|e| e.to_response())
}

/// Handle authorization code grant
async fn handle_auth_code(
    app_state: AppState,
    request: TokenRequest,
) -> Result<TokenResponse, Error> {
    // Validate required parameters
    let code = request.code
        .ok_or_else(|| Error::InvalidRequest("Missing 'code' parameter".to_string()))?;
    let redirect_uri = request.redirect_uri
        .ok_or_else(|| Error::InvalidRequest("Missing 'redirect_uri' parameter".to_string()))?;

    // Validate authorization code
    let auth_data = flow::validate_code(&app_state, &code).await
        .map_err(|_e| Error::InvalidGrant)?;

    // Validate client and redirect URI
    if auth_data.client_id != request.client_id {
        return Err(Error::InvalidClient);
    }
    if auth_data.redirect_uri != redirect_uri {
        return Err(Error::InvalidRequest("Redirect URI mismatch".to_string()));
    }

    // Validate PKCE if present
    if let (Some(challenge), Some(verifier)) = (&auth_data.code_challenge, &request.code_verifier) {
        let method = auth_data.code_challenge_method.as_deref().unwrap_or("plain");
        flow::validate_pkce(challenge, verifier, method)
            .map_err(|e| Error::InvalidRequest(e.to_string()))?;
    }

    // Generate tokens
    let expires_in = 7200; // 2 hours
    let access_token = create_access_token(&auth_data.firebase_user, &auth_data.scope, expires_in)?;
    let id_token = create_id_token(&auth_data.firebase_user, &request.client_id, expires_in)?;
    let refresh_token = create_refresh_token(&app_state, &auth_data).await?;

    // Create session
    create_session(&app_state, &auth_data.firebase_user, &access_token).await?;

    tracing::info!("Token exchange successful for user: {}", 
                   auth_data.firebase_user.email.as_ref().unwrap_or(&"unknown".to_string()));

    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
        id_token,
        refresh_token: Some(refresh_token),
        scope: auth_data.scope,
    })
}

/// Handle refresh token grant
async fn handle_refresh(
    app_state: AppState,
    request: TokenRequest,
) -> Result<TokenResponse, Error> {
    let refresh_token = request.refresh_token
        .ok_or_else(|| Error::InvalidRequest("Missing 'refresh_token' parameter".to_string()))?;

    // Validate refresh token
    let refresh_data = get_refresh_data(&app_state, &refresh_token).await
        .map_err(|_| Error::InvalidGrant)?;

    // Validate client
    if refresh_data.client_id != request.client_id {
        return Err(Error::InvalidClient);
    }

    // Generate new tokens
    let expires_in = 7200;
    let access_token = create_access_token(&refresh_data.firebase_user, &refresh_data.scope, expires_in)?;
    let id_token = create_id_token(&refresh_data.firebase_user, &request.client_id, expires_in)?;

    tracing::info!("Token refresh successful for user: {}", 
                   refresh_data.firebase_user.email.as_ref().unwrap_or(&"unknown".to_string()));

    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
        id_token,
        refresh_token: Some(refresh_token), // Reuse same refresh token
        scope: refresh_data.scope,
    })
}

/// Create access token
fn create_access_token(
    firebase_user: &FirebaseUser,
    _scope: &str,
    expires_in: i64,
) -> Result<String, Error> {
    let permissions = get_permissions(&firebase_user.custom_claims);
    
    let user_data = jwt::UserData {
        id: firebase_user.uid.clone(),
        email: firebase_user.email.clone().unwrap_or_default(),
        name: firebase_user.display_name.clone(),
        permissions: Some(permissions),
        admin_modules: Some(get_admin_modules(&firebase_user.custom_claims)),
        package_tier: Some(get_tier(&firebase_user.custom_claims)),
        firebase_uid: Some(firebase_user.uid.clone()),
        audience: Some("epsx-api".to_string()),
        ttl_seconds: Some(expires_in as usize),
    };

    jwt::JWT.create(user_data)
        .map_err(|e| Error::ServerError(format!("Failed to create access token: {}", e)))
}

/// Create ID token
fn create_id_token(
    firebase_user: &FirebaseUser,
    client_id: &str,
    expires_in: i64,
) -> Result<String, Error> {
    use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};
    
    let now = Utc::now().timestamp();
    let claims = IdTokenClaims {
        iss: get_issuer(),
        sub: firebase_user.uid.clone(),
        aud: client_id.to_string(),
        exp: now + expires_in,
        iat: now,
        auth_time: now,
        email: firebase_user.email.clone().unwrap_or_default(),
        email_verified: firebase_user.email_verified,
        name: firebase_user.display_name.clone(),
        role: get_role(&firebase_user.custom_claims),
        nonce: None, // TODO: Extract from original request
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(get_jwt_secret().as_ref());

    encode(&header, &claims, &key)
        .map_err(|e| Error::ServerError(format!("Failed to create ID token: {}", e)))
}

/// Create refresh token
async fn create_refresh_token(
    app_state: &AppState,
    auth_data: &flow::CodeData,
) -> Result<String, Error> {
    use rand::Rng;
    use base64::Engine;
    
    let random_bytes: [u8; 32] = rand::thread_rng().gen();
    let refresh_token = format!("rt_{}", 
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes));

    let refresh_data = RefreshData {
        firebase_user: auth_data.firebase_user.clone(),
        client_id: auth_data.client_id.clone(),
        scope: auth_data.scope.clone(),
        created_at: Utc::now(),
    };

    let data_json = serde_json::to_string(&refresh_data)
        .map_err(|e| Error::ServerError(format!("Failed to serialize refresh data: {}", e)))?;

    // Store refresh token
    store_refresh_token(app_state, &refresh_token, &data_json).await?;

    Ok(refresh_token)
}

/// Get refresh token data
async fn get_refresh_data(
    app_state: &AppState,
    refresh_token: &str,
) -> Result<RefreshData, Error> {
    use crate::dom::values::SessId;
    
    let session_id = SessId::from_string(format!("refresh_token:{}", refresh_token));
    let session = app_state.session_repo.get(&session_id).await
        .map_err(|e| Error::ServerError(e.to_string()))?
        .ok_or(Error::InvalidGrant)?;

    let refresh_data: RefreshData = serde_json::from_str(&session.access_token)
        .map_err(|e| Error::ServerError(format!("Failed to parse refresh data: {}", e)))?;

    // Check expiration
    if Utc::now() - refresh_data.created_at > Duration::days(7) {
        return Err(Error::InvalidGrant);
    }

    Ok(refresh_data)
}

/// Store refresh token
async fn store_refresh_token(
    app_state: &AppState,
    refresh_token: &str,
    data_json: &str,
) -> Result<(), Error> {
    use crate::dom::entities::auth::Session;
    use crate::dom::values::{SessId, UserId};
    
    let session_id = SessId::from_string(format!("refresh_token:{}", refresh_token));
    let user_id = UserId::new("system".to_string()); // Generic user ID for refresh tokens
    
    let session = Session {
        id: session_id,
        user_id,
        access_token: data_json.to_string(),
        refresh_token: None,
        expires_at: Utc::now() + Duration::days(7),
        created_at: Utc::now(),
        is_active: true,
    };

    app_state.session_repo.save(&session).await
        .map_err(|e| Error::ServerError(e.to_string()))?;

    Ok(())
}

/// Create user session
async fn create_session(
    app_state: &AppState,
    firebase_user: &FirebaseUser,
    access_token: &str,
) -> Result<(), Error> {
    use crate::dom::entities::Session;
    use crate::dom::values::UserId;

    let user_id = UserId::new(firebase_user.uid.clone());
    let expires_at = Utc::now() + Duration::hours(24);
    let session = Session::new(user_id, access_token.to_string(), expires_at);

    app_state.session_repo.save(&session).await
        .map_err(|e| Error::ServerError(e.to_string()))?;

    Ok(())
}

/// GET /oauth/userinfo - UserInfo endpoint
pub async fn userinfo(
    State(_app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("UserInfo endpoint request");
    
    // Extract bearer token
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| Error::InvalidRequest("Missing Authorization header".to_string()));
    
    let auth_header = match auth_header {
        Ok(header) => header,
        Err(e) => return Err(e.to_response()),
    };
    
    if !auth_header.starts_with("Bearer ") {
        return Err(Error::InvalidRequest("Invalid Authorization header format".to_string()).to_response());
    }
    
    let token = &auth_header[7..];
    
    // Validate token
    let user = match jwt::JWT.decode(token) {
        Ok(user) => user,
        Err(_) => return Err(Error::InvalidRequest("Invalid or expired token".to_string()).to_response()),
    };
    
    let userinfo = serde_json::json!({
        "sub": user.id,
        "email": user.email,
        "email_verified": true,
        "name": user.name,
        "role": if !user.admin_modules.is_empty() { "admin" } else { "user" },
        "permissions": user.permissions,
        "package_tier": user.package_tier,
        "admin_modules": user.admin_modules
    });
    
    Ok(Json(userinfo))
}

// Helper functions

fn get_role(custom_claims: &HashMap<String, serde_json::Value>) -> String {
    custom_claims.get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("user")
        .to_string()
}

fn get_permissions(custom_claims: &HashMap<String, serde_json::Value>) -> Vec<String> {
    let role = get_role(custom_claims);
    
    match role.as_str() {
        "super_admin" => vec![
            "api:admin:*".to_string(),
            "route:*".to_string(),
            "users:manage".to_string(),
            "system:configure".to_string(),
        ],
        "admin" => vec![
            "api:admin:*".to_string(),
            "route:*".to_string(),
            "users:manage".to_string(),
        ],
        "moderator" => vec![
            "api:moderate:*".to_string(),
            "content:moderate".to_string(),
            "users:view".to_string(),
        ],
        "premium" => vec![
            "api:premium:*".to_string(),
            "analytics:read".to_string(),
            "alerts:manage".to_string(),
        ],
        _ => vec![
            "api:basic:read".to_string(),
            "profile:manage:own".to_string(),
        ],
    }
}

fn get_admin_modules(custom_claims: &HashMap<String, serde_json::Value>) -> Vec<String> {
    custom_claims.get("admin_modules")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

fn get_tier(custom_claims: &HashMap<String, serde_json::Value>) -> String {
    custom_claims.get("package_tier")
        .and_then(|v| v.as_str())
        .unwrap_or("FREE")
        .to_string()
}

fn get_issuer() -> String {
    get_env_var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
}

fn get_jwt_secret() -> String {
    get_env_var("NEXTAUTH_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_permissions_from_role() {
        let mut claims = HashMap::new();
        claims.insert("role".to_string(), serde_json::Value::String("admin".to_string()));
        
        let permissions = get_permissions(&claims);
        assert!(permissions.contains(&"api:admin:*".to_string()));
        assert!(permissions.contains(&"users:manage".to_string()));
    }

    #[test]
    fn test_get_role_from_custom_claims() {
        let mut claims = HashMap::new();
        claims.insert("role".to_string(), serde_json::Value::String("admin".to_string()));
        
        let role = get_role(&claims);
        assert_eq!(role, "admin");
    }

    #[test]
    fn test_error_to_response() {
        let error = Error::InvalidRequest("test error".to_string());
        let (status, response) = error.to_response();
        
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(response.error, "invalid_request");
    }
}