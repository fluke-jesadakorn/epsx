use crate::core::errors::AppError;
use sha2::{ Sha256, Digest };
use crate::config::env::get_env_var;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;

/// Client credential information
#[derive(Debug, Clone)]
pub struct ClientCredentials {
  pub client_id: String,
  pub client_secret: String,
  pub redirect_uris: Vec<String>,
  pub allowed_scopes: Vec<String>,
  pub client_type: ClientType,
}

/// Type of OAuth2 client
#[derive(Debug, Clone, PartialEq)]
pub enum ClientType {
  Confidential, // Can securely store secrets (server-side apps)
  Public, // Cannot store secrets securely (mobile apps, SPAs)
}

/// Client credential validation service
pub struct ClientCredentialService {
  clients: HashMap<String, ClientCredentials>,
}

/// Static registry of allowed clients loaded from environment
static CLIENT_REGISTRY: Lazy<HashMap<String, ClientCredentials>> = Lazy::new(
  || {
    let mut clients = HashMap::new();

    // Check if we're in development mode
    let is_development =
      get_env_var("RUST_ENV").unwrap_or_else(|_| "production".to_string()) ==
      "development";

    // Frontend client
    if
      let (Ok(client_id), Ok(client_secret)) = (
        get_env_var("OIDC_FRONTEND_CLIENT_ID"),
        get_env_var("OIDC_FRONTEND_CLIENT_SECRET"),
      )
    {
      let frontend_url = get_env_var("FRONTEND_URL").unwrap_or_else(|_| "https://epsx.io".to_string());
      let mut redirect_uris = vec![
        format!("{}/api/auth/callback/epsx-backend", frontend_url),
      ];
      
      // Add additional production domains if configured
      if let Ok(app_url) = get_env_var("APP_URL") {
        redirect_uris.push(format!("{}/api/auth/callback/epsx-backend", app_url));
      }

      // Add localhost only in development mode
      if is_development {
        redirect_uris.push(
          "http://localhost:3000/api/auth/callback/epsx-backend".to_string()
        );
      }

      clients.insert(client_id.clone(), ClientCredentials {
        client_id: client_id.clone(),
        client_secret,
        redirect_uris,
        allowed_scopes: vec![
          "openid".to_string(),
          "profile".to_string(),
          "email".to_string()
        ],
        client_type: ClientType::Confidential,
      });
    }

    // Admin client
    if
      let (Ok(client_id), Ok(client_secret)) = (
        get_env_var("OIDC_ADMIN_CLIENT_ID"),
        get_env_var("OIDC_ADMIN_CLIENT_SECRET"),
      )
    {
      let admin_url = get_env_var("ADMIN_FRONTEND_URL").unwrap_or_else(|_| "https://admin.epsx.io".to_string());
      let mut redirect_uris = vec![
        format!("{}/api/auth/callback/epsx-backend", admin_url),
      ];
      
      // Add additional admin domains if configured
      if let Ok(admin_alt_url) = get_env_var("ADMIN_URL") {
        redirect_uris.push(format!("{}/api/auth/callback/epsx-backend", admin_alt_url));
      }

      // Add localhost only in development mode
      if is_development {
        redirect_uris.push(
          "http://localhost:3001/api/auth/callback/epsx-backend".to_string()
        );
      }

      clients.insert(client_id.clone(), ClientCredentials {
        client_id: client_id.clone(),
        client_secret,
        redirect_uris,
        allowed_scopes: vec![
          "openid".to_string(),
          "profile".to_string(),
          "email".to_string(),
          "admin_modules".to_string()
        ],
        client_type: ClientType::Confidential,
      });
    }

    clients
  }
);

impl ClientCredentialService {
  /// Create new client credential service
  pub fn new() -> Self {
    Self {
      clients: CLIENT_REGISTRY.clone(),
    }
  }

  /// Validate client credentials using client_id and client_secret
  pub fn validate_client_credentials(
    &self,
    client_id: &str,
    client_secret: &str
  ) -> Result<&ClientCredentials, AppError> {
    tracing::debug!(
      "Validating client credentials for client_id: {}",
      client_id
    );

    let client = self.clients
      .get(client_id)
      .ok_or_else(||
        AppError::security_error(format!("Unknown client_id: {}", client_id))
      )?;

    // Use constant-time comparison to prevent timing attacks
    if !self.constant_time_compare(&client.client_secret, client_secret) {
      tracing::warn!("Invalid client secret for client_id: {}", client_id);
      return Err(
        AppError::security_error("Invalid client credentials".to_string())
      );
    }

    tracing::debug!(
      "Client credentials validated successfully for: {}",
      client_id
    );
    Ok(client)
  }

  /// Validate client credentials from HTTP Basic Authentication
  pub fn validate_basic_auth(
    &self,
    auth_header: &str
  ) -> Result<&ClientCredentials, AppError> {
    if !auth_header.starts_with("Basic ") {
      return Err(
        AppError::security_error(
          "Invalid authorization header format".to_string()
        )
      );
    }

    let encoded_credentials = &auth_header[6..];
    let decoded = BASE64_STANDARD.decode(encoded_credentials).map_err(|_|
      AppError::security_error("Invalid base64 encoding".to_string())
    )?;

    let credentials_str = String::from_utf8(decoded).map_err(|_|
      AppError::security_error("Invalid UTF-8 in credentials".to_string())
    )?;

    let parts: Vec<&str> = credentials_str.splitn(2, ':').collect();
    if parts.len() != 2 {
      return Err(
        AppError::security_error("Invalid credential format".to_string())
      );
    }

    let (client_id, client_secret) = (parts[0], parts[1]);
    self.validate_client_credentials(client_id, client_secret)
  }

  /// Check if redirect URI is allowed for client
  pub fn is_redirect_uri_allowed(
    &self,
    client_id: &str,
    redirect_uri: &str
  ) -> Result<bool, AppError> {
    let client = self.clients
      .get(client_id)
      .ok_or_else(||
        AppError::security_error(format!("Unknown client_id: {}", client_id))
      )?;

    Ok(
      client.redirect_uris.iter().any(|allowed_uri| allowed_uri == redirect_uri)
    )
  }

  /// Check if scope is allowed for client
  pub fn is_scope_allowed(
    &self,
    client_id: &str,
    requested_scope: &str
  ) -> Result<bool, AppError> {
    let client = self.clients
      .get(client_id)
      .ok_or_else(||
        AppError::security_error(format!("Unknown client_id: {}", client_id))
      )?;

    let requested_scopes: Vec<&str> = requested_scope
      .split_whitespace()
      .collect();

    for scope in requested_scopes {
      if !client.allowed_scopes.iter().any(|allowed| allowed == scope) {
        return Ok(false);
      }
    }

    Ok(true)
  }

  /// Get client information by ID
  pub fn get_client(&self, client_id: &str) -> Option<&ClientCredentials> {
    self.clients.get(client_id)
  }

  /// List all registered client IDs (for debugging)
  pub fn get_client_ids(&self) -> Vec<&String> {
    self.clients.keys().collect()
  }

  /// Constant-time string comparison to prevent timing attacks
  fn constant_time_compare(&self, a: &str, b: &str) -> bool {
    if a.len() != b.len() {
      return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
      result |= x ^ y;
    }

    result == 0
  }

  /// Generate secure client secret (for administrative use)
  pub fn generate_client_secret() -> String {
    use rand::Rng;
    use rand::distributions::Alphanumeric;

    let secret: String = rand
      ::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(64)
      .map(char::from)
      .collect();

    format!("sk-{}", secret)
  }

  /// Hash client secret for storage (for future use)
  pub fn hash_client_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    format!("{:x}", hasher.finalize())
  }

  /// Verify hashed client secret (for future use)
  pub fn verify_hashed_secret(&self, secret: &str, hash: &str) -> bool {
    let computed_hash = Self::hash_client_secret(secret);
    self.constant_time_compare(&computed_hash, hash)
  }
}

impl Default for ClientCredentialService {
  fn default() -> Self {
    Self::new()
  }
}
