use axum::{
    http::StatusCode,
    response::Json,
    extract::State,
};

use crate::web::oidc::types::OidcDiscoveryDocument;

/// OIDC Well-Known Configuration Endpoint
/// GET /.well-known/openid-configuration
pub async fn oidc_discovery(
    State(_state): State<crate::web::auth::routes::AppState>,
) -> Result<Json<OidcDiscoveryDocument>, StatusCode> {
    tracing::info!("OIDC discovery document requested");
    
    // Get base URL from environment or default
    let base_url = std::env::var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let discovery_doc = OidcDiscoveryDocument {
        issuer: base_url.clone(),
        authorization_endpoint: format!("{}/oauth/authorize", base_url),
        token_endpoint: format!("{}/oauth/token", base_url),
        userinfo_endpoint: format!("{}/oauth/userinfo", base_url),
        jwks_uri: format!("{}/oauth/jwks", base_url),
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
            "phone".to_string(),
            "admin".to_string(),
        ],
        response_types_supported: vec![
            "code".to_string(),
            "id_token".to_string(),
            "code id_token".to_string(),
        ],
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ],
        subject_types_supported: vec![
            "public".to_string(),
        ],
        id_token_signing_alg_values_supported: vec![
            "RS256".to_string(),
        ],
        claims_supported: vec![
            "sub".to_string(),
            "iss".to_string(),
            "aud".to_string(),
            "exp".to_string(),
            "iat".to_string(),
            "auth_time".to_string(),
            "nonce".to_string(),
            "email".to_string(),
            "email_verified".to_string(),
            "name".to_string(),
            "picture".to_string(),
            "given_name".to_string(),
            "family_name".to_string(),
            "phone_number".to_string(),
            "phone_number_verified".to_string(),
            // Firebase custom claims
            "role".to_string(),
            "admin".to_string(),
            "access_level".to_string(),
            "premium".to_string(),
        ],
        code_challenge_methods_supported: vec![
            "plain".to_string(),
            "S256".to_string(),
        ],
    };
    
    Ok(Json(discovery_doc))
}

/// JWKS (JSON Web Key Set) Endpoint  
/// GET /oauth/jwks
pub async fn jwks_endpoint(
    State(_state): State<crate::web::auth::routes::AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("JWKS endpoint requested");
    
    // TODO: Implement proper JWKS with RSA public keys
    // For now, return empty JWKS (Firebase tokens will be validated directly)
    let jwks = serde_json::json!({
        "keys": []
    });
    
    Ok(Json(jwks))
}