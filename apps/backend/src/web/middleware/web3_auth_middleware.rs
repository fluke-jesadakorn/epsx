use axum::{
  extract::{ Request, State },
  http::{ header::AUTHORIZATION, StatusCode },
  middleware::Next,
  response::Response,
};
use std::collections::HashMap;
use tracing::{ debug, error, warn };

use crate::infrastructure::container::AppContainer;
use crate::core::errors::AppError;

/// Web3 authentication middleware that validates wallet-based JWT tokens
pub async fn web3_auth_middleware(
  State(container): State<AppContainer>,
  mut request: Request,
  next: Next
) -> Result<Response, StatusCode> {
  // Extract Authorization header
  let auth_header = request
    .headers()
    .get(AUTHORIZATION)
    .and_then(|h| h.to_str().ok())
    .ok_or_else(|| {
      debug!("Missing Authorization header in Web3 middleware");
      StatusCode::UNAUTHORIZED
    })?;

  // Validate Bearer token format
  if !auth_header.starts_with("Bearer ") {
    debug!("Invalid authorization header format");
    return Err(StatusCode::UNAUTHORIZED);
  }

  let token = &auth_header[7..]; // Remove "Bearer " prefix

  // Validate JWT token
  let jwt_service = container.jwt_service().map_err(|e| {
    error!("Failed to get JWT service: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let claims = jwt_service.validate_access_token(token).map_err(|e| {
    warn!("Invalid JWT token: {}", e);
    StatusCode::UNAUTHORIZED
  })?;

  // Extract user ID from token claims
  let user_id = &claims.sub;

  // For Web3 middleware, use placeholder wallet address since standard JWT claims don't contain wallet info
  // In production, this would be retrieved from the user database using the sub claim
  let wallet_address = "placeholder_wallet_address";

  let _unused = user_id; // Avoid unused variable warning, keeping the pattern for when we implement proper lookup

  // Verify wallet still has access (real-time permission check)
  let web3_permissions = container.web3_permission_service();

  // Get current permissions for the wallet
  let permissions = web3_permissions
    .get_wallet_permissions(wallet_address).await
    .map_err(|e| {
      error!("Failed to get wallet permissions: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let active_permissions: Vec<String> = permissions
    .into_iter()
    .filter(|p| p.is_active)
    .map(|p| p.permission)
    .collect();

  // Add Web3 authentication context to request extensions
  let web3_context = Web3AuthContext {
    user_id: user_id.to_string(),
    wallet_address: wallet_address.to_string(),
    permissions: active_permissions,
    auth_method: "web3_wallet".to_string(),
    token_claims: {
      let mut map = std::collections::HashMap::new();
      map.insert(
        "sub".to_string(),
        serde_json::Value::String(claims.sub.clone())
      );
      map.insert(
        "iss".to_string(),
        serde_json::Value::String(claims.iss.clone())
      );
      map.insert(
        "aud".to_string(),
        serde_json::Value::String(claims.aud.clone())
      );
      map.insert(
        "exp".to_string(),
        serde_json::Value::Number(serde_json::Number::from(claims.exp))
      );
      map.insert(
        "iat".to_string(),
        serde_json::Value::Number(serde_json::Number::from(claims.iat))
      );
      map
    },
  };

  request.extensions_mut().insert(web3_context);

  Ok(next.run(request).await)
}

/// Web3 authorization middleware that checks specific permissions
pub fn require_web3_permission(
  required_permission: &'static str
) -> impl (Fn(
  Request,
  Next
) -> std::pin::Pin<
  Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>
>) +
  Clone {
  move |request: Request, next: Next| {
    let required_perm = required_permission;
    Box::pin(async move {
      // Get Web3 context from request extensions
      let web3_context = request
        .extensions()
        .get::<Web3AuthContext>()
        .ok_or_else(|| {
          warn!(
            "Web3 context not found - ensure web3_auth_middleware runs first"
          );
          StatusCode::UNAUTHORIZED
        })?;

      // Check if user has the required permission
      let has_permission = web3_context.permissions.iter().any(|p| {
        // Support wildcard permissions (e.g., "admin:*:*" grants "admin:users:view")
        if p.ends_with(":*:*") {
          let prefix = &p[..p.len() - 4]; // Remove ":*:*"
          required_perm.starts_with(prefix)
        } else if p.ends_with(":*") {
          let prefix = &p[..p.len() - 2]; // Remove ":*"
          required_perm.starts_with(prefix)
        } else {
          p == required_perm
        }
      });

      if !has_permission {
        warn!(
          "Web3 user {} with wallet {} lacks required permission: {}",
          web3_context.user_id,
          web3_context.wallet_address,
          required_perm
        );
        return Err(StatusCode::FORBIDDEN);
      }

      debug!(
        "Web3 permission granted for user {} with wallet {}: {}",
        web3_context.user_id,
        web3_context.wallet_address,
        required_perm
      );

      Ok(next.run(request).await)
    })
  }
}

/// Web3 authentication context stored in request extensions
#[derive(Debug, Clone)]
pub struct Web3AuthContext {
  pub user_id: String,
  pub wallet_address: String,
  pub permissions: Vec<String>,
  pub auth_method: String,
  pub token_claims: HashMap<String, serde_json::Value>,
}

impl Web3AuthContext {
  /// Check if user has a specific permission
  pub fn has_permission(&self, permission: &str) -> bool {
    self.permissions.iter().any(|p| {
      if p.ends_with(":*:*") {
        let prefix = &p[..p.len() - 4];
        permission.starts_with(prefix)
      } else if p.ends_with(":*") {
        let prefix = &p[..p.len() - 2];
        permission.starts_with(prefix)
      } else {
        p == permission
      }
    })
  }

  /// Get user ID as UUID
  pub fn user_uuid(&self) -> Result<uuid::Uuid, uuid::Error> {
    uuid::Uuid::parse_str(&self.user_id)
  }

  /// Check if user is Web3-only (no Firebase)
  pub fn is_web3_only(&self) -> bool {
    self.auth_method == "web3_wallet"
  }

  /// Check if user is hybrid (has both wallet and Firebase)
  pub fn is_hybrid(&self) -> bool {
    self.auth_method == "hybrid"
  }

  /// Get wallet address as ethers Address type
  pub fn wallet_address_typed(
    &self
  ) -> Result<ethers::types::Address, Box<dyn std::error::Error>> {
    self.wallet_address
      .parse()
      .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
  }
}

/// Extract Web3 auth context from request (for handlers)
pub trait Web3AuthExtractor {
  fn web3_context(&self) -> Option<&Web3AuthContext>;
  fn require_web3_context(&self) -> Result<&Web3AuthContext, AppError>;
}

impl Web3AuthExtractor for Request {
  fn web3_context(&self) -> Option<&Web3AuthContext> {
    self.extensions().get::<Web3AuthContext>()
  }

  fn require_web3_context(&self) -> Result<&Web3AuthContext, AppError> {
    self
      .web3_context()
      .ok_or_else(|| AppError::unauthorized("Web3 authentication required"))
  }
}

/// Utility function to create Web3 auth middleware stack
pub fn web3_auth_stack() -> impl Clone {
  axum::middleware::from_fn::<_, AppContainer>(web3_auth_middleware)
}

/// Utility function to create permission-required middleware
pub fn require_permission(
  permission: &'static str
) -> impl Clone +
  Fn(
    Request,
    Next
  ) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>
  > {
  require_web3_permission(permission)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_permission_matching() {
    let context = Web3AuthContext {
      user_id: "test-user".to_string(),
      wallet_address: "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
      permissions: vec![
        "admin:users:view".to_string(),
        "admin:permissions:*".to_string(),
        "epsx:*:*".to_string()
      ],
      auth_method: "web3_wallet".to_string(),
      token_claims: HashMap::new(),
    };

    // Exact match
    assert!(context.has_permission("admin:users:view"));

    // Wildcard resource match
    assert!(context.has_permission("admin:permissions:manage"));
    assert!(context.has_permission("admin:permissions:create"));

    // Wildcard platform match
    assert!(context.has_permission("epsx:trades:view"));
    assert!(context.has_permission("epsx:analytics:manage"));

    // No match
    assert!(!context.has_permission("admin:users:manage"));
    assert!(!context.has_permission("other:platform:view"));
  }

  #[test]
  fn test_wallet_address_parsing() {
    let context = Web3AuthContext {
      user_id: "test-user".to_string(),
      wallet_address: "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
      permissions: vec![],
      auth_method: "web3_wallet".to_string(),
      token_claims: HashMap::new(),
    };

    let addr = context.wallet_address_typed().unwrap();
    assert_eq!(
      addr.to_string().to_lowercase(),
      "0x742d35cc6634c0532925a3b8d369d7763f3c45c6"
    );
  }
}
