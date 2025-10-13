use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;

/// Authenticated wallet extracted from Bearer token
#[derive(Debug, Clone)]
pub struct AuthWallet {
    pub id: Uuid,
    pub address: String,
    pub permissions: Vec<String>,
}

/// Authenticated admin wallet with elevated permissions
#[derive(Debug, Clone)]
pub struct AuthAdmin {
    pub wallet: AuthWallet,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthWallet
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Bearer token from Authorization header
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid authorization format".to_string()));
        }

        let token = &auth_header[7..];

        // Decode JWT and extract wallet information
        let wallet_id = decode_jwt_wallet_id(token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        let wallet_address = decode_jwt_wallet_address(token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        let permissions = decode_jwt_permissions(token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        Ok(AuthWallet {
            id: wallet_id,
            address: wallet_address,
            permissions,
        })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthAdmin
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let wallet = AuthWallet::from_request_parts(parts, state).await?;

        // Check admin permissions
        if !wallet.permissions.iter().any(|p| p.starts_with("admin:")) {
            return Err((StatusCode::FORBIDDEN, "Admin access required".to_string()));
        }

        Ok(AuthAdmin { wallet })
    }
}

// TODO: Integration with existing JWT auth system
// These functions need to be implemented based on your JWT library and claims structure

fn decode_jwt_wallet_id(token: &str) -> Result<Uuid, String> {
    // Temporary implementation - integrate with your actual JWT validation
    // This should:
    // 1. Validate JWT signature
    // 2. Check expiry
    // 3. Extract wallet_id claim

    tracing::warn!("Using stub JWT decoder - implement real validation");

    // For now, accept any token and return a test UUID
    // REMOVE THIS IN PRODUCTION
    if token.len() > 0 {
        Ok(Uuid::new_v4())
    } else {
        Err("Empty token".to_string())
    }
}

fn decode_jwt_wallet_address(token: &str) -> Result<String, String> {
    // Temporary implementation - integrate with your actual JWT validation
    // This should extract the wallet_address claim from JWT

    tracing::warn!("Using stub JWT decoder - implement real validation");

    // For now, return a test address
    // REMOVE THIS IN PRODUCTION
    if token.len() > 0 {
        Ok("0x0000000000000000000000000000000000000000".to_string())
    } else {
        Err("Empty token".to_string())
    }
}

fn decode_jwt_permissions(token: &str) -> Result<Vec<String>, String> {
    // Temporary implementation - integrate with your actual JWT validation
    // This should extract the permissions array from JWT claims

    tracing::warn!("Using stub JWT decoder - implement real validation");

    // For now, return test permissions
    // REMOVE THIS IN PRODUCTION
    if token.len() > 0 {
        Ok(vec![
            "user:notifications:read".to_string(),
            "user:notifications:delete".to_string(),
        ])
    } else {
        Err("Empty token".to_string())
    }
}

// Helper function to check if wallet has specific permission
impl AuthWallet {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission || p.ends_with(":*"))
    }
}

impl AuthAdmin {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.wallet.has_permission(permission)
    }
}
