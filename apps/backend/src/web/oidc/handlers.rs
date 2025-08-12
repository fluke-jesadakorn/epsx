use axum::{
    extract::{Query, State, Form},
    http::{StatusCode, HeaderMap},
    response::{Json, Redirect},
};
use chrono::Utc;
use serde_json;

use crate::web::auth::routes::AppState;
use crate::web::oidc::types::*;
use crate::dom::services::FirebaseSessionService;
use crate::infra::firebase_admin::FirebaseUser;

/// OIDC Authorization Endpoint
/// GET/POST /oauth/authorize
pub async fn oidc_authorize(
    State(state): State<AppState>,
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
    State(state): State<AppState>,
    Form(request): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OidcErrorResponse>)> {
    tracing::info!("OIDC token request for grant_type: {}", request.grant_type);
    
    match request.grant_type.as_str() {
        "authorization_code" => handle_authorization_code_grant(state, request).await,
        "refresh_token" => handle_refresh_token_grant(state, request).await,
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
    state: AppState,
    request: TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OidcErrorResponse>)> {
    let code = request.code.clone().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(OidcErrorResponse {
                error: "invalid_request".to_string(),
                error_description: Some("Missing authorization code".to_string()),
                error_uri: None,
            }),
        )
    })?;
    
    // For development/testing, allow the code to be either:
    // 1. An actual authorization code (starts with "ac_")
    // 2. A Firebase ID token (for backward compatibility)
    
    tracing::info!("Received code in token exchange: {}", &code[..10.min(code.len())]);
    tracing::info!("Code starts with 'ac_': {}", code.starts_with("ac_"));
    
    if code.starts_with("ac_") {
        // This is an authorization code - retrieve the stored data
        tracing::info!("Processing authorization code: {}", &code[..10.min(code.len())]);
        
        // Retrieve authorization code data from session repository
        let session_id = crate::dom::values::SessId::from_string(format!("auth_code:{}", code));
        tracing::info!("Looking up session_id: {}", session_id.to_string());
        let auth_data_result = state.session_repo.find_by_id(&session_id).await;
        tracing::info!("Session lookup result: {:?}", auth_data_result.as_ref().map(|_| "found").map_err(|e| e.to_string()));
        
        let auth_data = match auth_data_result {
            Ok(session) => {
                // Parse the stored authorization data from the access_token field
                let auth_data: crate::web::oidc::authorization::AuthorizationCodeData = 
                    serde_json::from_str(&session.access_token)
                    .map_err(|e| {
                        tracing::error!("Failed to parse authorization code data: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(OidcErrorResponse {
                                error: "server_error".to_string(),
                                error_description: Some("Internal server error".to_string()),
                                error_uri: None,
                            }),
                        )
                    })?;
                    
                // Check if authorization code is expired (10 minutes)
                if chrono::Utc::now() > session.expires_at {
                    tracing::warn!("Authorization code expired: {}", code);
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(OidcErrorResponse {
                            error: "invalid_grant".to_string(),
                            error_description: Some("Authorization code expired".to_string()),
                            error_uri: None,
                        }),
                    ));
                }
                
                auth_data
            },
            Err(e) => {
                tracing::error!("Failed to retrieve authorization code: {}", e);
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(OidcErrorResponse {
                        error: "invalid_grant".to_string(),
                        error_description: Some("Invalid authorization code".to_string()),
                        error_uri: None,
                    }),
                ));
            }
        };
        
        // Validate client_id matches
        if auth_data.client_id != request.client_id {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OidcErrorResponse {
                    error: "invalid_grant".to_string(),
                    error_description: Some("Client ID mismatch".to_string()),
                    error_uri: None,
                }),
            ));
        }
        
        // Clean up the used authorization code
        let _ = state.session_repo.delete(&session_id).await;
        
        // Create tokens using the authenticated Firebase user
        return create_session_and_tokens(state, auth_data.firebase_user, &request).await;
    }
    
    // Legacy path: treat as Firebase ID token (for backward compatibility)
    tracing::info!("Taking legacy Firebase ID token path for code: {}", &code[..10.min(code.len())]);
    let firebase_id_token = code;
    
    // Create Firebase user service with admin module service
    let firebase_admin = crate::infra::firebase_admin::FirebaseAdmin::new()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create Firebase admin: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OidcErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Internal server error".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    let firebase_user_service = crate::dom::services::FirebaseUserService::with_admin_module_service(
        firebase_admin,
        state.admin_module_service.clone()
    );
    
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
    let firebase_session_service = create_firebase_session_service(&state)
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
    let id_token = generate_id_token(&firebase_user, &request.client_id, None, &state.admin_module_service).await?;
    
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
    state: AppState,
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
    let firebase_session_service = create_firebase_session_service(&state)
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
    let id_token = generate_id_token(&firebase_user, &request.client_id, None, &state.admin_module_service).await?;
    
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
    State(state): State<AppState>,
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
    let firebase_session_service = create_firebase_session_service(&state)
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
async fn create_firebase_session_service(state: &AppState) -> Result<FirebaseSessionService, String> {
    // For development mode, we need to work around the fact that FirebaseSessionService
    // expects a PgPool directly, but we only have repositories in AppState
    
    // This is a workaround for development - in production this should be properly injected
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL not set".to_string())?;
    
    let db_pool = sqlx::PgPool::connect(&database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
        
    // Note: We need to dereference the Arc to get the FirebaseAdmin
    let firebase_admin = (*state.firebase_admin).clone();
    
    let firebase_user_service = crate::dom::services::FirebaseUserService::with_admin_module_service(
        firebase_admin.clone(),
        state.admin_module_service.clone()
    );
    
    let session_service = FirebaseSessionService::new(
        db_pool,
        firebase_admin,
        firebase_user_service,
    );
    
    Ok(session_service)
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

/// Generate ID token for Firebase user with enhanced IAM claims
pub async fn generate_id_token(
    firebase_user: &FirebaseUser,
    client_id: &str,
    nonce: Option<String>,
    admin_module_service: &std::sync::Arc<crate::dom::services::AdminModuleService>,
) -> Result<String, (StatusCode, Json<OidcErrorResponse>)> {
    let now = Utc::now().timestamp() as u64;
    let issuer = std::env::var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    // Get user's admin modules and compute permissions
    let admin_modules = admin_module_service
        .get_user_admin_modules(&firebase_user.uid)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to get admin modules for user {}: {}", firebase_user.uid, e);
            vec![]
        });
    
    let permissions = crate::core::permission_constants::get_permissions_for_modules(&admin_modules);
    let access_level = crate::core::permission_constants::AdminModuleValidator::get_effective_access_level(&admin_modules);
    let is_admin = !admin_modules.is_empty();
    
    let claims = IdTokenClaims {
        // Standard OIDC claims
        iss: issuer,
        sub: firebase_user.uid.clone(),
        aud: client_id.to_string(),
        exp: now + 3600, // 1 hour
        iat: now,
        auth_time: Some(now),
        nonce,
        
        // Standard profile claims
        email: firebase_user.email.clone(),
        email_verified: Some(firebase_user.email_verified),
        name: firebase_user.display_name.clone(),
        picture: firebase_user.photo_url.clone(),
        given_name: None, // TODO: Parse from display_name
        family_name: None, // TODO: Parse from display_name
        phone_number: firebase_user.phone_number.clone(),
        phone_number_verified: None, // TODO: Get from Firebase
        
        // Modern IAM Claims - Admin Module System Only
        admin: is_admin,
        access_level,
        admin_modules,
        permissions,
        
        // Subscription data (TODO: Get from subscription service)
        subscription_tier: None, // Will be populated from subscription service
        subscription_status: None, // Will be populated from subscription service
        
        // Firebase custom claims
        custom_claims: firebase_user.custom_claims.clone(),
    };
    
    // For development/testing, create a simple JWT-like token that can be decoded
    // In production, this should use proper RS256 signing with private keys
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    
    // Create a simple header
    let header = serde_json::json!({
        "typ": "JWT",
        "alg": "HS256"
    });
    
    let header_b64 = URL_SAFE_NO_PAD.encode(header.to_string());
    let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&claims).unwrap());
    
    // Use proper JWT signing with environment-based key
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-jwt-secret-key-change-in-production".to_string());
    
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    let signature_input = format!("{}.{}", header_b64, payload_b64);
    let mut mac = Hmac::<Sha256>::new_from_slice(jwt_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(signature_input.as_bytes());
    let signature = mac.finalize().into_bytes();
    let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);
    
    Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
}

/// Create session and tokens for a Firebase user (helper function)
async fn create_session_and_tokens(
    state: AppState,
    firebase_user: FirebaseUser,
    request: &TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OidcErrorResponse>)> {
    // Check admin access for admin client
    if request.client_id == "epsx-admin" {
        // Use Firebase admin validation with admin module service
        let firebase_admin = crate::infra::firebase_admin::FirebaseAdmin::new()
            .await
            .map_err(|e| {
                tracing::error!("Failed to create Firebase admin: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(OidcErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some("Internal server error".to_string()),
                        error_uri: None,
                    }),
                )
            })?;
            
        let firebase_user_service = crate::dom::services::FirebaseUserService::with_admin_module_service(
            firebase_admin,
            state.admin_module_service.clone()
        );

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

    // For real users, use Firebase session service  
    let firebase_session_service = create_firebase_session_service(&state)
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
        firebase_id_token: "mock-token".to_string(), // For development
        user_agent: None,
        ip_address: None,
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

    // Generate ID token
    let id_token = generate_id_token(&firebase_user, &request.client_id, None, &state.admin_module_service).await?;

    // Return token response
    let token_response = TokenResponse {
        access_token: session_info.session_token.clone(),
        token_type: "Bearer".to_string(),
        expires_in: 28800, // 8 hours
        refresh_token: Some(session_info.session_token.clone()),
        id_token,
        scope: "openid profile email admin:read admin:write".to_string(),
    };

    tracing::info!("Successfully issued tokens for Firebase user: {}", firebase_user.uid);
    Ok(Json(token_response))
}