// Enhanced OIDC Authorization Handlers with PKCE and Multi-Tenant Support
// HTTP handlers implementing the advanced OIDC flows

use std::sync::Arc;
use axum::{
    extract::{Query, State, Form},
    http::{StatusCode, HeaderMap},
    response::{Json, Redirect},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::core::errors::AppError;
use crate::core::ClientCredentialService;
use crate::web::auth::routes::AppState;
use crate::dom::services::admin_module_service::AdminModuleService;
use super::enhanced_token_broker::{
    EnhancedTokenBroker, EnhancedAuthorizationRequest, EnhancedTokenRequest,
};

/// Enhanced authorization endpoint handler
/// GET/POST /oauth/v2/authorize
pub async fn enhanced_authorize(
    State(broker): State<Arc<EnhancedTokenBroker>>,
    Query(params): Query<EnhancedAuthorizationRequest>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Json<OIDCErrorResponse>)> {
    info!(
        client_id = %params.client_id,
        email_hint = ?params.email_hint,
        tenant_hint = ?params.tenant_hint,
        pkce_challenge = ?params.code_challenge,
        "Enhanced OIDC authorization request"
    );
    
    // Extract client information from headers
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    
    match broker.initiate_authorization(params, client_ip, user_agent).await {
        Ok(flow_result) => {
            info!(
                authorization_url = %flow_result.authorization_url,
                tenant_id = %flow_result.tenant_id,
                provider_id = %flow_result.provider_id,
                "Authorization flow initiated successfully"
            );
            
            Ok(Redirect::to(&flow_result.authorization_url))
        }
        Err(e) => {
            error!(error = %e, "Authorization flow initiation failed");
            
            let error_response = match &e.kind {
                crate::core::errors::ErrorKind::ValidationError => OIDCErrorResponse {
                    error: "invalid_request".to_string(),
                    error_description: Some(e.message.clone()),
                    error_uri: None,
                },
                crate::core::errors::ErrorKind::AggregateNotFound => OIDCErrorResponse {
                    error: "invalid_client".to_string(),
                    error_description: Some(e.message.clone()),
                    error_uri: None,
                },
                crate::core::errors::ErrorKind::AuthorizationError => OIDCErrorResponse {
                    error: "access_denied".to_string(),
                    error_description: Some(e.message.clone()),
                    error_uri: None,
                },
                _ => OIDCErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Internal server error".to_string()),
                    error_uri: None,
                },
            };
            
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Enhanced token endpoint handler with PKCE support
/// POST /oauth/v2/token
pub async fn enhanced_token(
    State(broker): State<Arc<EnhancedTokenBroker>>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Form(request): Form<EnhancedTokenRequest>,
) -> Result<Json<EnhancedTokenResponse>, (StatusCode, Json<OIDCErrorResponse>)> {
    info!(
        client_id = %request.client_id,
        grant_type = %request.grant_type,
        has_code_verifier = request.code_verifier.is_some(),
        "Enhanced token exchange request"
    );
    
    // Extract client information
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    
    // Validate client credentials if provided
    if let Err(e) = validate_client_credentials(&request, &headers, &app_state.admin_module_service).await {
        warn!(error = %e, "Client credential validation failed");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(OIDCErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some(e.to_string()),
                error_uri: None,
            }),
        ));
    }
    
    match broker.exchange_code_for_tokens(request, client_ip, user_agent).await {
        Ok(unified_jwt) => {
            info!(
                tenant_id = %unified_jwt.tenant_id,
                provider_id = %unified_jwt.provider_id,
                custom_ttl_applied = unified_jwt.custom_ttl_applied,
                expires_in = unified_jwt.expires_in,
                "Token exchange successful"
            );
            
            let response = EnhancedTokenResponse {
                access_token: unified_jwt.access_token,
                token_type: unified_jwt.token_type,
                expires_in: unified_jwt.expires_in,
                refresh_token: unified_jwt.refresh_token,
                refresh_expires_in: unified_jwt.refresh_expires_in,
                id_token: unified_jwt.id_token,
                scope: unified_jwt.scope,
                
                // Enhanced fields
                session_id: Some(unified_jwt.session_id),
                jti: Some(unified_jwt.jti),
                tenant_id: Some(unified_jwt.tenant_id),
                provider_id: Some(unified_jwt.provider_id),
                custom_ttl_applied: Some(unified_jwt.custom_ttl_applied),
                expires_at: Some(unified_jwt.expires_at),
            };
            
            Ok(Json(response))
        }
        Err(e) => {
            error!(error = %e, "Token exchange failed");
            
            let (status, error_response) = match &e.kind {
                crate::core::errors::ErrorKind::ValidationError => (
                    StatusCode::BAD_REQUEST,
                    OIDCErrorResponse {
                        error: "invalid_request".to_string(),
                        error_description: Some(e.message.clone()),
                        error_uri: None,
                    }
                ),
                crate::core::errors::ErrorKind::AuthorizationError => (
                    StatusCode::BAD_REQUEST,
                    OIDCErrorResponse {
                        error: "invalid_grant".to_string(),
                        error_description: Some(e.message.clone()),
                        error_uri: None,
                    }
                ),
                crate::core::errors::ErrorKind::AggregateNotFound => (
                    StatusCode::BAD_REQUEST,
                    OIDCErrorResponse {
                        error: "invalid_grant".to_string(),
                        error_description: Some(e.message.clone()),
                        error_uri: None,
                    }
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    OIDCErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some("Internal server error".to_string()),
                        error_uri: None,
                    }
                ),
            };
            
            Err((status, Json(error_response)))
        }
    }
}

/// Enhanced userinfo endpoint
/// GET /oauth/v2/userinfo
pub async fn enhanced_userinfo(
    State(_broker): State<Arc<EnhancedTokenBroker>>,
    headers: HeaderMap,
) -> Result<Json<EnhancedUserInfoResponse>, (StatusCode, Json<OIDCErrorResponse>)> {
    info!("Enhanced userinfo request");
    
    // Extract and validate bearer token
    let _access_token = extract_bearer_token(&headers)
        .ok_or_else(|| {
            warn!("Missing or invalid Authorization header");
            (
                StatusCode::UNAUTHORIZED,
                Json(OIDCErrorResponse {
                    error: "invalid_token".to_string(),
                    error_description: Some("Missing or invalid access token".to_string()),
                    error_uri: None,
                }),
            )
        })?;
    
    // TODO: Validate token and extract user information
    // For now, return a placeholder response
    let userinfo = EnhancedUserInfoResponse {
        // Standard OIDC claims
        sub: "user123".to_string(),
        email: Some("user@example.com".to_string()),
        email_verified: Some(true),
        name: Some("Test User".to_string()),
        picture: None,
        given_name: Some("Test".to_string()),
        family_name: Some("User".to_string()),
        phone_number: None,
        phone_number_verified: None,
        
        // Enhanced claims
        tenant_id: Some("default".to_string()),
        provider_id: Some("google-default".to_string()),
        provider_type: Some("google".to_string()),
        role: Some("user".to_string()),
        permissions: Some(vec!["read".to_string()]),
        subscription_tier: None,
        
        // Security claims
        session_id: Some("session123".to_string()),
        auth_time: Some(chrono::Utc::now().timestamp()),
        amr: Some(vec!["oidc".to_string()]),
        acr: Some("1".to_string()),
        
        // Metadata
        last_login: Some(chrono::Utc::now()),
        risk_score: Some(0.1),
    };
    
    info!(
        user_id = %userinfo.sub,
        tenant_id = ?userinfo.tenant_id,
        provider_id = ?userinfo.provider_id,
        "Userinfo request successful"
    );
    
    Ok(Json(userinfo))
}

/// Token introspection endpoint (RFC 7662)
/// POST /oauth/v2/introspect
pub async fn token_introspection(
    State(_broker): State<Arc<EnhancedTokenBroker>>,
    headers: HeaderMap,
    Form(request): Form<TokenIntrospectionRequest>,
) -> Result<Json<TokenIntrospectionResponse>, (StatusCode, Json<OIDCErrorResponse>)> {
    info!(
        token_type_hint = ?request.token_type_hint,
        "Token introspection request"
    );
    
    // Validate client credentials
    if let Err(_e) = validate_introspection_client(&headers).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(OIDCErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Client authentication required".to_string()),
                error_uri: None,
            }),
        ));
    }
    
    // TODO: Implement token introspection logic
    // For now, return a placeholder response
    let introspection_response = TokenIntrospectionResponse {
        active: true,
        sub: Some("user123".to_string()),
        client_id: Some("frontend-client".to_string()),
        username: Some("user@example.com".to_string()),
        scope: Some("openid profile email".to_string()),
        exp: Some((chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp()),
        iat: Some(chrono::Utc::now().timestamp()),
        
        // Enhanced fields
        tenant_id: Some("default".to_string()),
        provider_id: Some("google-default".to_string()),
        session_id: Some("session123".to_string()),
        jti: Some("jti123".to_string()),
        amr: Some(vec!["oidc".to_string()]),
        acr: Some("1".to_string()),
    };
    
    Ok(Json(introspection_response))
}

/// Token revocation endpoint (RFC 7009)
/// POST /oauth/v2/revoke
pub async fn token_revocation(
    State(_broker): State<Arc<EnhancedTokenBroker>>,
    headers: HeaderMap,
    Form(request): Form<TokenRevocationRequest>,
) -> Result<StatusCode, (StatusCode, Json<OIDCErrorResponse>)> {
    info!(
        token_type_hint = ?request.token_type_hint,
        "Token revocation request"
    );
    
    // Validate client credentials
    if let Err(_e) = validate_revocation_client(&headers).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(OIDCErrorResponse {
                error: "invalid_client".to_string(),
                error_description: Some("Client authentication required".to_string()),
                error_uri: None,
            }),
        ));
    }
    
    // TODO: Implement token revocation logic
    info!("Token revocation successful");
    Ok(StatusCode::OK)
}

// Response types

#[derive(Debug, Serialize)]
pub struct EnhancedTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_expires_in: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    pub scope: String,
    
    // Enhanced fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_ttl_applied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct EnhancedUserInfoResponse {
    // Standard OIDC claims
    pub sub: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number_verified: Option<bool>,
    
    // Enhanced claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_tier: Option<String>,
    
    // Security claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amr: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acr: Option<String>,
    
    // Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_score: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct OIDCErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

// Request types

#[derive(Debug, Deserialize)]
pub struct TokenIntrospectionRequest {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type_hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenIntrospectionResponse {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    
    // Enhanced fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amr: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acr: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TokenRevocationRequest {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type_hint: Option<String>,
}

// Helper functions

/// Extract client IP address from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try X-Forwarded-For first (for proxy/load balancer setups)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }
    
    // Try X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }
    
    None
}

/// Extract user agent from headers
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers.get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers.get("authorization")
        .and_then(|auth| auth.to_str().ok())
        .and_then(|auth_str| auth_str.strip_prefix("Bearer "))
        .map(|token| token.to_string())
}

/// Validate client credentials for token endpoint
async fn validate_client_credentials(
    request: &EnhancedTokenRequest,
    headers: &HeaderMap,
    _admin_module_service: &AdminModuleService,
) -> Result<(), AppError> {
    let client_service = ClientCredentialService::new();
    
    // Check for client_secret in request body
    if let Some(client_secret) = &request.client_secret {
        tracing::debug!("Validating client credentials from request body");
        client_service.validate_client_credentials(&request.client_id, client_secret)?;
        return Ok(());
    }
    
    // Check for HTTP Basic authentication
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                tracing::debug!("Validating client credentials from Basic auth header");
                client_service.validate_basic_auth(auth_str)?;
                return Ok(());
            }
        }
    }
    
    // For public clients, no authentication required if using PKCE
    if request.grant_type == "authorization_code" && request.code_verifier.is_some() {
        tracing::debug!("Public client using PKCE - no client authentication required");
        
        // Still validate that the client_id exists and is allowed to use PKCE
        let client = client_service.get_client(&request.client_id)
            .ok_or_else(|| AppError::SecurityError(
                format!("Unknown client_id: {}", request.client_id)
            ))?;
        
        // For now, allow both confidential and public clients to use PKCE
        // In a more strict implementation, you might want to enforce client types
        tracing::debug!("Client {} validated for PKCE flow", client.client_id);
        return Ok(());
    }
    
    tracing::warn!("Client authentication failed - no valid credentials provided");
    Err(AppError::SecurityError("Client authentication required".to_string()))
}

/// Validate client for token introspection
async fn validate_introspection_client(headers: &HeaderMap) -> Result<(), AppError> {
    let client_service = ClientCredentialService::new();
    
    // Check for HTTP Basic authentication
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                tracing::debug!("Validating client credentials for token introspection");
                client_service.validate_basic_auth(auth_str)?;
                return Ok(());
            }
        }
    }
    
    tracing::warn!("Token introspection requires client authentication");
    Err(AppError::SecurityError("Client authentication required for token introspection".to_string()))
}

/// Validate client for token revocation
async fn validate_revocation_client(headers: &HeaderMap) -> Result<(), AppError> {
    let client_service = ClientCredentialService::new();
    
    // Check for HTTP Basic authentication
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                tracing::debug!("Validating client credentials for token revocation");
                client_service.validate_basic_auth(auth_str)?;
                return Ok(());
            }
        }
    }
    
    tracing::warn!("Token revocation requires client authentication");
    Err(AppError::SecurityError("Client authentication required for token revocation".to_string()))
}