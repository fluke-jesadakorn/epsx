/// Unified Authentication Routes (Web3-First)
///
/// These routes provide a clean, unified API for authentication that supports both
/// Web3 wallet authentication and legacy Firebase authentication methods.
/// The API is designed to be Web3-first while maintaining backward compatibility.

use axum::{ extract::State, response::Json, routing::{ get, post }, Router };
use serde::{ Deserialize, Serialize };
use tracing::{ error, info, warn };

use crate::infrastructure::container::AppContainer;
use crate::core::errors::AppError;
use crate::auth::{
  AuthMethod,
  UnifiedVerifyRequest,
  VerificationData,
};

type ApiResult<T> = Result<T, AppError>;

/// Request to generate authentication challenge
#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
  pub method: String, // "web3"
  pub wallet_address: Option<String>, // Required for web3
  pub redirect_uri: Option<String>, // Legacy field (deprecated)
}

/// Unified challenge response
#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
  pub challenge_id: String,
  pub method: String,
  pub data: ChallengeData,
  pub expires_at: String,
}

/// Challenge data for Web3-first authentication
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ChallengeData {
  Web3 {
    nonce: String,
    message: String,
    wallet_address: String,
  },
}

/// Unified verification request
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
  pub challenge_id: String,
  pub method: String,
  pub data: VerifyData,
}

/// Verification data that varies by auth method
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VerifyData {
  Web3 {
    message: String,
    signature: String,
    wallet_address: String,
  },
}

/// Unified authentication response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
  pub success: bool,
  pub user_id: String,
  pub method: String,
  pub tokens: TokenSet,
  pub permissions: Vec<String>,
  pub wallet_address: Option<String>,
}

/// Token set for successful authentication
#[derive(Debug, Serialize)]
pub struct TokenSet {
  pub access_token: String,
  pub id_token: String,
  pub refresh_token: String,
  pub token_type: String,
  pub expires_in: u64,
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct ProfileResponse {
  pub user_id: String,
  pub email: Option<String>,
  pub display_name: Option<String>,
  pub wallet_address: Option<String>,
  pub permissions: Vec<String>,
  pub auth_methods: Vec<String>,
  pub created_at: String,
  pub last_login_at: Option<String>,
}


/// Create unified authentication routes
pub fn create_routes() -> Router<AppContainer> {
  Router::new()
    .route("/challenge", post(generate_challenge))
    .route("/verify", post(verify_authentication))
    .route("/profile", get(get_profile))
    .route("/migrate", post(migrate_user))
    .route("/logout", post(logout))
    .route("/refresh", post(refresh_token))
}

/// Generate authentication challenge
/// POST /api/auth/unified/challenge
pub async fn generate_challenge(
  State(container): State<AppContainer>,
  Json(request): Json<ChallengeRequest>
) -> ApiResult<Json<ChallengeResponse>> {
  info!(
    "Generating unified authentication challenge for method: {}",
    request.method
  );

  let unified_auth = container.unified_auth_service();

  let challenge = match request.method.as_str() {
    "web3" => {
      let wallet_address = request.wallet_address.ok_or_else(||
        AppError::bad_request(
          "wallet_address is required for Web3 authentication"
        )
      )?;

      unified_auth.generate_web3_challenge(&wallet_address).await.map_err(|e| {
        error!("Failed to generate Web3 challenge: {}", e);
        match e.to_string().as_str() {
          s if s.contains("Invalid wallet address") =>
            AppError::bad_request("Invalid wallet address format"),
          _ => AppError::internal_server_error("Failed to generate challenge"),
        }
      })?
    }
    "firebase" => {
      // Firebase authentication deprecated - redirect to Web3
      warn!("Firebase challenge generation attempted. Use Web3 wallet authentication.");
      return Err(
        AppError::bad_request(
          "Firebase authentication is deprecated. Please use Web3 wallet authentication instead."
        )
      );
    }
    _ => {
      return Err(
        AppError::bad_request(
          "Invalid authentication method. Use 'web3' for wallet authentication."
        )
      );
    }
  };

  let response_data = match &challenge.data {
    crate::auth::AuthChallengeData::Web3 { nonce, message, wallet_address } => {
      ChallengeData::Web3 {
        nonce: nonce.clone(),
        message: message.clone(),
        wallet_address: wallet_address.clone(),
      }
    }
  };

  let response = ChallengeResponse {
    challenge_id: challenge.challenge_id,
    method: request.method,
    data: response_data,
    expires_at: challenge.expires_at.to_rfc3339(),
  };

  Ok(Json(response))
}

/// Verify authentication
/// POST /api/auth/unified/verify
pub async fn verify_authentication(
  State(container): State<AppContainer>,
  Json(request): Json<VerifyRequest>
) -> ApiResult<Json<AuthResponse>> {
  info!("Verifying unified authentication for method: {}", request.method);

  let unified_auth = container.unified_auth_service();

  // Convert request to internal format
  let verify_data = match request.data {
    VerifyData::Web3 { message, signature, wallet_address } => {
      VerificationData::Web3 { message, signature, wallet_address }
    }
  };

  let unified_request = UnifiedVerifyRequest {
    challenge_id: request.challenge_id,
    method: match request.method.as_str() {
      "web3" =>
        AuthMethod::Web3Wallet { wallet_address: "pending".to_string() },
      _ => {
        return Err(AppError::bad_request("Invalid authentication method. Only 'web3' is supported."));
      }
    },
    data: verify_data,
  };

  let auth_result = unified_auth
    .verify_authentication(unified_request).await
    .map_err(|e| {
      error!("Authentication verification failed: {}", e);
      match e.to_string().as_str() {
        s if s.contains("Invalid signature") =>
          AppError::unauthorized("Invalid signature"),
        s if s.contains("Invalid token") =>
          AppError::unauthorized("Invalid token"),
        _ => AppError::internal_server_error("Authentication failed"),
      }
    })?;

  let method_str = match auth_result.method {
    AuthMethod::Web3Wallet { .. } => "web3",
  };

  let response = AuthResponse {
    success: true,
    user_id: auth_result.user_id.to_string(),
    method: method_str.to_string(),
    tokens: TokenSet {
      access_token: auth_result.access_token,
      id_token: auth_result.id_token,
      refresh_token: auth_result.refresh_token,
      token_type: "Bearer".to_string(),
      expires_in: auth_result.expires_in,
    },
    permissions: auth_result.permissions,
    wallet_address: auth_result.wallet_address,
  };

  info!("Authentication successful for user: {}", response.user_id);
  Ok(Json(response))
}

/// Get user profile
/// GET /api/auth/unified/profile
pub async fn get_profile(
  State(container): State<AppContainer>,
  headers: axum::http::HeaderMap
) -> ApiResult<Json<ProfileResponse>> {
  info!("Getting user profile");

  let auth_header = headers
    .get("authorization")
    .and_then(|h| h.to_str().ok())
    .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?;

  if !auth_header.starts_with("Bearer ") {
    return Err(AppError::unauthorized("Invalid Authorization header format"));
  }

  let token = &auth_header[7..];

  let unified_auth = container.unified_auth_service();

  let profile = unified_auth.get_user_profile(token).await.map_err(|e| {
    error!("Failed to get user profile: {}", e);
    match e.to_string().as_str() {
      s if s.contains("Invalid token") =>
        AppError::unauthorized("Invalid or expired token"),
      _ => AppError::internal_server_error("Failed to get profile"),
    }
  })?;

  let auth_method_strings: Vec<String> = profile.auth_methods
    .iter()
    .map(|method| {
      match method {
        AuthMethod::Web3Wallet { .. } => "web3".to_string(),
      }
    })
    .collect();

  let response = ProfileResponse {
    user_id: profile.user_id.to_string(),
    email: profile.email,
    display_name: profile.display_name,
    wallet_address: profile.wallet_address,
    permissions: profile.permissions,
    auth_methods: auth_method_strings,
    created_at: profile.created_at.to_rfc3339(),
    last_login_at: profile.last_login_at.map(|dt| dt.to_rfc3339()),
  };

  Ok(Json(response))
}

/// Migrate Firebase user to Web3 - DEPRECATED
/// POST /api/auth/unified/migrate
pub async fn migrate_user(
  State(_container): State<AppContainer>,
  Json(_request): Json<serde_json::Value>
) -> ApiResult<Json<serde_json::Value>> {
  warn!("Migration endpoint accessed - Firebase migration is deprecated");

  // Firebase migration deprecated - users should register with Web3 directly
  Err(
    AppError::bad_request(
      "Firebase migration is deprecated. Please register using Web3 wallet authentication directly at /api/auth/unified/challenge with method 'web3'."
    )
  )
}

/// Logout user
/// POST /api/auth/unified/logout
pub async fn logout(
  State(container): State<AppContainer>,
  headers: axum::http::HeaderMap
) -> ApiResult<Json<serde_json::Value>> {
  info!("Logging out user");

  let auth_header = headers
    .get("authorization")
    .and_then(|h| h.to_str().ok())
    .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?;

  if !auth_header.starts_with("Bearer ") {
    return Err(AppError::unauthorized("Invalid Authorization header format"));
  }

  let token = &auth_header[7..];

  let unified_auth = container.unified_auth_service();

  // Validate token and get user context
  let auth_context = unified_auth
    .validate_bearer_token(token).await
    .map_err(|e| {
      error!("Token validation failed during logout: {}", e);
      AppError::unauthorized("Invalid or expired token")
    })?;

  // Revoke the user's authentication
  unified_auth.revoke_authentication(&auth_context.user_id).await.map_err(|e| {
    error!("Failed to revoke authentication: {}", e);
    AppError::internal_server_error("Failed to logout")
  })?;

  info!("User {} logged out successfully", auth_context.user_id);

  Ok(
    Json(
      serde_json::json!({
        "success": true,
        "message": "Successfully logged out"
    })
    )
  )
}

/// Refresh authentication token
/// POST /api/auth/unified/refresh
pub async fn refresh_token(
  State(_container): State<AppContainer>,
  Json(_request): Json<serde_json::Value>
) -> ApiResult<Json<serde_json::Value>> {
  // TODO: Implement token refresh functionality
  // This would involve:
  // 1. Validating the refresh token
  // 2. Generating new access and ID tokens
  // 3. Updating the refresh token (rotation)

  warn!("Token refresh not yet implemented");

  Err(AppError::internal_server_error("Token refresh not yet implemented"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_challenge_data_serialization() {
    let web3_data = ChallengeData::Web3 {
      nonce: "test_nonce".to_string(),
      message: "test_message".to_string(),
      wallet_address: "0x123".to_string(),
    };

    let json = serde_json::to_string(&web3_data).unwrap();
    assert!(json.contains("test_nonce"));
    assert!(json.contains("test_message"));
    assert!(json.contains("0x123"));
  }

  #[test]
  fn test_verify_data_deserialization() {
    let json =
      r#"{"message": "test", "signature": "sig", "wallet_address": "0x123"}"#;
    let data: VerifyData = serde_json::from_str(json).unwrap();

    match data {
      VerifyData::Web3 { message, signature, wallet_address } => {
        assert_eq!(message, "test");
        assert_eq!(signature, "sig");
        assert_eq!(wallet_address, "0x123");
      }
      _ => panic!("Expected Web3 variant"),
    }
  }
}
