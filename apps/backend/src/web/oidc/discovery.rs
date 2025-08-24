use axum::{ http::StatusCode, response::Json };
use crate::config::env::get_env_var;

use crate::web::oidc::types::OidcDiscoveryDocument;

/// Get the OIDC issuer URL based on environment and deployment context
fn get_oidc_issuer_url() -> String {
  // Priority order for determining issuer URL:
  // 1. OIDC_ISSUER environment variable (explicit override)
  // 2. PUBLIC_URL environment variable (for deployments)
  // 3. BACKEND_URL environment variable (for local development)
  // 4. Detect from common deployment environments
  // 5. Default localhost fallback

  if let Ok(issuer) = get_env_var("OIDC_ISSUER") {
    if !issuer.is_empty() {
      tracing::info!("Using OIDC issuer from OIDC_ISSUER: {}", issuer);
      return issuer;
    }
  }

  if let Ok(public_url) = get_env_var("PUBLIC_URL") {
    if !public_url.is_empty() {
      tracing::info!("Using OIDC issuer from PUBLIC_URL: {}", public_url);
      return public_url;
    }
  }

  if let Ok(backend_url) = get_env_var("BACKEND_URL") {
    if !backend_url.is_empty() {
      tracing::info!("Using OIDC issuer from BACKEND_URL: {}", backend_url);
      return backend_url;
    }
  }

  // Check common deployment environment variables
  if let Ok(render_external_url) = get_env_var("RENDER_EXTERNAL_URL") {
    tracing::info!(
      "Detected Render deployment, using: {}",
      render_external_url
    );
    return render_external_url;
  }

  if let Ok(railway_public_domain) = get_env_var("RAILWAY_PUBLIC_DOMAIN") {
    let url = format!("https://{}", railway_public_domain);
    tracing::info!("Detected Railway deployment, using: {}", url);
    return url;
  }

  if get_env_var("HEROKU_APP_NAME").is_ok() {
    if let Ok(app_name) = get_env_var("HEROKU_APP_NAME") {
      let url = format!("https://{}.herokuapp.com", app_name);
      tracing::info!("Detected Heroku deployment, using: {}", url);
      return url;
    }
  }

  // Check if we're in Docker
  if
    get_env_var("DOCKER_CONTAINER").is_ok() ||
    std::path::Path::new("/.dockerenv").exists()
  {
    if let Ok(port) = get_env_var("PORT") {
      let url = format!("http://localhost:{}", port);
      tracing::info!("Detected Docker environment, using: {}", url);
      return url;
    }
  }

  // Development fallback
  let default_url = "http://localhost:8080".to_string();
  tracing::warn!(
    "No explicit OIDC issuer configured, falling back to development default: {}",
    default_url
  );
  tracing::warn!(
    "For production, set OIDC_ISSUER or PUBLIC_URL environment variable"
  );

  default_url
}

/// OIDC Well-Known Configuration Endpoint
/// GET /.well-known/openid-configuration
pub async fn oidc_discovery() -> Result<
  Json<OidcDiscoveryDocument>,
  StatusCode
> {
  tracing::info!(
    "OIDC discovery document requested - DEBUGGING CONTENT-TYPE ERROR"
  );
  tracing::info!("This endpoint returns JSON content-type");

  // Determine base URL with proper environment detection
  let base_url = get_oidc_issuer_url();

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
      "admin".to_string()
    ],
    response_types_supported: vec![
      "code".to_string(),
      "id_token".to_string(),
      "code id_token".to_string()
    ],
    grant_types_supported: vec![
      "authorization_code".to_string(),
      "refresh_token".to_string()
    ],
    subject_types_supported: vec!["public".to_string()],
    id_token_signing_alg_values_supported: vec![
      "HS256".to_string(),
      "RS256".to_string()
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
      "premium".to_string()
    ],
    code_challenge_methods_supported: vec![
      "plain".to_string(),
      "S256".to_string()
    ],
  };

  Ok(Json(discovery_doc))
}

/// JWKS (JSON Web Key Set) Endpoint
/// GET /oauth/jwks
pub async fn jwks_endpoint() -> Result<Json<serde_json::Value>, StatusCode> {
  tracing::info!("JWKS endpoint requested - DEBUGGING CONTENT-TYPE ERROR");
  tracing::info!("This endpoint returns JSON content-type");

  // Get JWKS from the new simplified JWT service
  match crate::auth::JWT.keys().generate_jwks() {
    Ok(jwks) => {
      let jwks_json = serde_json::to_value(jwks).map_err(|e| {
        tracing::error!("Failed to serialize JWKS: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
      })?;

      tracing::info!(
        "Served JWKS with {} keys",
        jwks_json["keys"]
          .as_array()
          .map(|k| k.len())
          .unwrap_or(0)
      );
      Ok(Json(jwks_json))
    }
    Err(e) => {
      tracing::error!("Failed to generate JWKS: {}", e);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}
