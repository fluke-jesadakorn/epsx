use axum::{
    extract::{Query, State, Form},
    http::{StatusCode, HeaderMap},
    response::{Json, Redirect},
};
use chrono::Utc;

use crate::web::auth::routes::AppState;
use crate::web::oidc::types::*;
use crate::dom::services::{FirebaseUserService, FirebaseSessionService};
use crate::infra::firebase_admin::FirebaseUser;

/// OIDC Authorization Endpoint
/// GET/POST /oauth/authorize
pub async fn oidc_authorize(
    State(_state): State<AppState>,
    Query(params): Query<AuthorizationRequest>,
) -> Result<Redirect, (StatusCode, Json<OidcErrorResponse>)> {
    tracing::info!("OIDC authorization request from client: {}", params.client_id);
    
    // Validate required parameters
    if params.response_type != "code" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OidcErrorResponse {
                error: "unsupported_response_type".to_string(),
                error_description: Some("Only 'code' response type is supported".to_string()),
                error_uri: None,
            }),
        ));
    }
    
    // Validate client_id (for admin frontend)
    if params.client_id != "epsx-admin" && params.client_id != "epsx-frontend" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OidcErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Unknown client_id".to_string()),
                error_uri: None,
            }),
        ));
    }
    
    // Validate redirect_uri
    let allowed_redirects = match params.client_id.as_str() {
        "epsx-admin" => vec![
            "http://localhost:3001/auth/callback".to_string(),
            "https://admin.epsx.com/auth/callback".to_string(),
        ],
        "epsx-frontend" => vec![
            "http://localhost:3000/auth/callback".to_string(),
            "https://app.epsx.com/auth/callback".to_string(),
        ],
        _ => vec![],
    };
    
    if !allowed_redirects.contains(&params.redirect_uri) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OidcErrorResponse {
                error: "invalid_redirect_uri".to_string(),
                error_description: Some("Redirect URI not allowed for this client".to_string()),
                error_uri: None,
            }),
        ));
    }
    
    // For Firebase-native OIDC, redirect to Firebase Auth UI
    // The frontend will handle Firebase authentication and then call back to /oauth/callback
    let firebase_auth_url = format!(
        "{}?client_id={}&redirect_uri={}&state={}&scope={}",
        "/firebase-auth", // This would be handled by frontend
        params.client_id,
        urlencoding::encode(&params.redirect_uri),
        params.state.as_deref().unwrap_or(""),
        urlencoding::encode(&params.scope)
    );
    
    tracing::info!("Redirecting to Firebase auth for client: {}", params.client_id);
    Ok(Redirect::to(&firebase_auth_url))
}

/// OIDC Token Endpoint  
/// POST /oauth/token
pub async fn oidc_token(
    State(_state): State<AppState>,
    Form(request): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OidcErrorResponse>)> {
    tracing::info!("OIDC token request for grant_type: {}", request.grant_type);
    
    match request.grant_type.as_str() {
        "authorization_code" => handle_authorization_code_grant(_state, request).await,
        "refresh_token" => handle_refresh_token_grant(_state, request).await,
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(OidcErrorResponse {
                error: "unsupported_grant_type".to_string(),
                error_description: Some("Only authorization_code and refresh_token grants are supported".to_string()),
                error_uri: None,
            }),
        )),
    }
}

/// Handle authorization code grant (exchange code for tokens)
async fn handle_authorization_code_grant(
    _state: AppState,
    request: TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OidcErrorResponse>)> {
    let code = request.code.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(OidcErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing authorization code".to_string()),
                error_uri: None,
            }),
        )
    })?;
    
    // The "code" in our Firebase-native approach is actually the Firebase ID token
    // This is a simplified implementation - in production you'd store authorization codes
    let firebase_id_token = code;
    
    // Create Firebase user service
    let firebase_user_service = FirebaseUserService::new()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create Firebase user service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OidcErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Internal server error".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    // Verify Firebase ID token and get user data
    let firebase_user = firebase_user_service
        .verify_and_get_user(&firebase_id_token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to verify Firebase token: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(OidcErrorResponse {
                    error: "invalid_grant".to_string(),
                    error_description: Some("Invalid authorization code".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    // Check if user has required permissions for admin client
    if request.client_id == "epsx-admin" {
        let admin_access = firebase_user_service
            .validate_admin_access(&firebase_user.uid)
            .await
            .map_err(|e| {
                tracing::error!("Failed to validate admin access: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(OidcErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some("Failed to validate permissions".to_string()),
                        error_uri: None,
                    }),
                )
            })?;
            
        if !admin_access {
            return Err((
                StatusCode::FORBIDDEN,
                Json(OidcErrorResponse {
                    error: "insufficient_scope".to_string(),
                    error_description: Some("User does not have admin access".to_string()),
                    error_uri: None,
                }),
            ));
        }
    }
    
    // Create session using Firebase session service
    let firebase_session_service = create_firebase_session_service(&_state)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create session service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OidcErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Internal server error".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    let session_request = crate::dom::services::firebase_session_service::CreateSessionRequest {
        firebase_id_token: firebase_id_token.clone(),
        user_agent: None, // TODO: Extract from request headers
        ip_address: None, // TODO: Extract from request
        session_duration_hours: Some(8),
    };
    
    let session_info = firebase_session_service
        .create_session(session_request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OidcErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Failed to create session".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    // Generate ID token with Firebase user data
    let id_token = generate_id_token(&firebase_user, &request.client_id, None)?;
    
    // Return OIDC token response
    let token_response = TokenResponse {
        access_token: session_info.session_token.clone(),
        token_type: "Bearer".to_string(),
        expires_in: 28800, // 8 hours
        refresh_token: Some(session_info.session_token.clone()), // Simplified: use same token
        id_token,
        scope: "openid profile email".to_string(),
    };
    
    tracing::info!("Successfully issued tokens for Firebase user: {}", firebase_user.uid);
    Ok(Json(token_response))
}

/// Handle refresh token grant
async fn handle_refresh_token_grant(
    _state: AppState,
    request: TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OidcErrorResponse>)> {
    let refresh_token = request.refresh_token.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(OidcErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing refresh token".to_string()),
                error_uri: None,
            }),
        )
    })?;
    
    // Create session service
    let firebase_session_service = create_firebase_session_service(&_state)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create session service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OidcErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Internal server error".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    // Validate and refresh session
    let validation_result = firebase_session_service
        .validate_session(&refresh_token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to validate refresh token: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(OidcErrorResponse {
                    error: "invalid_grant".to_string(),
                    error_description: Some("Invalid refresh token".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    if !validation_result.is_valid {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OidcErrorResponse {
                error: "invalid_grant".to_string(),
                error_description: Some("Invalid or expired refresh token".to_string()),
                error_uri: None,
            }),
        ));
    }
    
    let firebase_user = validation_result.firebase_user.ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OidcErrorResponse {
                error: "server_error".to_string(),
                error_description: Some("Failed to get user data".to_string()),
                error_uri: None,
            }),
        )
    })?;
    
    // Refresh the session
    let refreshed_session = firebase_session_service
        .refresh_session(&refresh_token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to refresh session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OidcErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Failed to refresh session".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    // Generate new ID token
    let id_token = generate_id_token(&firebase_user, &request.client_id, None)?;
    
    let token_response = TokenResponse {
        access_token: refreshed_session.session_token.clone(),
        token_type: "Bearer".to_string(),
        expires_in: 28800, // 8 hours
        refresh_token: Some(refreshed_session.session_token.clone()),
        id_token,
        scope: "openid profile email".to_string(),
    };
    
    tracing::info!("Successfully refreshed tokens for Firebase user: {}", firebase_user.uid);
    Ok(Json(token_response))
}

/// OIDC UserInfo Endpoint
/// GET /oauth/userinfo
pub async fn oidc_userinfo(
    State(_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserInfoResponse>, (StatusCode, Json<OidcErrorResponse>)> {
    tracing::info!("OIDC userinfo request");
    
    // Extract access token from Authorization header
    let access_token = extract_bearer_token(&headers)
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(OidcErrorResponse {
                    error: "invalid_token".to_string(),
                    error_description: Some("Missing or invalid access token".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    // Create session service
    let firebase_session_service = create_firebase_session_service(&_state)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create session service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OidcErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Internal server error".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    // Validate access token (session token)
    let validation_result = firebase_session_service
        .validate_session(&access_token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to validate access token: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(OidcErrorResponse {
                    error: "invalid_token".to_string(),
                    error_description: Some("Invalid or expired access token".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    if !validation_result.is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(OidcErrorResponse {
                error: "invalid_token".to_string(),
                error_description: Some("Invalid or expired access token".to_string()),
                error_uri: None,
            }),
        ));
    }
    
    let firebase_user = validation_result.firebase_user.ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OidcErrorResponse {
                error: "server_error".to_string(),
                error_description: Some("Failed to get user data".to_string()),
                error_uri: None,
            }),
        )
    })?;
    
    // Build UserInfo response
    let firebase_uid = firebase_user.uid.clone();
    let userinfo = UserInfoResponse {
        sub: firebase_user.uid,
        email: firebase_user.email,
        email_verified: Some(firebase_user.email_verified),
        name: firebase_user.display_name,
        picture: firebase_user.photo_url,
        given_name: None, // TODO: Parse from display_name
        family_name: None, // TODO: Parse from display_name  
        phone_number: firebase_user.phone_number,
        phone_number_verified: None, // TODO: Get from Firebase
        custom_claims: firebase_user.custom_claims,
    };
    
    tracing::info!("Successfully returned userinfo for Firebase user: {}", firebase_uid);
    Ok(Json(userinfo))
}

// Helper functions

/// Create Firebase session service with dependencies
async fn create_firebase_session_service(_state: &AppState) -> Result<FirebaseSessionService, String> {
    let _firebase_user_service = FirebaseUserService::new()
        .await
        .map_err(|e| format!("Failed to create Firebase user service: {}", e))?;
    
    // TODO: Get database pool from state
    // For now, create a placeholder error
    Err("Database pool not available in AppState".to_string())
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|token| token.to_string())
}

/// Generate ID token for Firebase user
fn generate_id_token(
    firebase_user: &FirebaseUser,
    client_id: &str,
    nonce: Option<String>,
) -> Result<String, (StatusCode, Json<OidcErrorResponse>)> {
    let now = Utc::now().timestamp() as u64;
    let issuer = std::env::var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let claims = IdTokenClaims {
        iss: issuer,
        sub: firebase_user.uid.clone(),
        aud: client_id.to_string(),
        exp: now + 3600, // 1 hour
        iat: now,
        auth_time: Some(now),
        nonce,
        email: firebase_user.email.clone(),
        email_verified: Some(firebase_user.email_verified),
        name: firebase_user.display_name.clone(),
        picture: firebase_user.photo_url.clone(),
        given_name: None,
        family_name: None,
        phone_number: firebase_user.phone_number.clone(),
        phone_number_verified: None,
        custom_claims: firebase_user.custom_claims.clone(),
    };
    
    // TODO: Implement proper JWT signing with RS256
    // For now, return a placeholder token
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    
    let token = URL_SAFE_NO_PAD.encode(serde_json::to_string(&claims).unwrap());
    
    Ok(format!("header.{}.signature", token))
}