use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

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

// JWT decoding implementation

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    #[serde(default)]
    wallet_address: String,
    #[serde(default)]
    sub: String,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    jti: Option<String>,
    exp: i64,
}

fn decode_jwt_wallet_id(token: &str) -> Result<Uuid, String> {
    // Try to parse JTI as UUID, or generate a deterministic one from wallet address
    let claims = decode_token(token)?;

    if let Some(jti) = claims.jti {
        Uuid::parse_str(&jti)
            .map_err(|e| format!("Invalid UUID in JTI: {}", e))
    } else {
        // Generate deterministic UUID from wallet address
        let hash = md5::compute(claims.wallet_address.as_bytes());
        let uuid_str = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5],
            hash[6], hash[7],
            hash[8], hash[9],
            hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]
        );
        Uuid::parse_str(&uuid_str)
            .map_err(|e| format!("Failed to generate UUID: {}", e))
    }
}

fn decode_jwt_wallet_address(token: &str) -> Result<String, String> {
    let claims = decode_token(token)?;

    let wallet = if !claims.wallet_address.is_empty() {
        claims.wallet_address
    } else {
        claims.sub
    };

    if wallet.is_empty() || wallet == "anonymous" {
        Err("No wallet address in token".to_string())
    } else {
        Ok(wallet.to_lowercase())
    }
}

fn decode_jwt_permissions(token: &str) -> Result<Vec<String>, String> {
    let claims = decode_token(token)?;
    Ok(claims.permissions)
}

/// Decode and validate JWT token
fn decode_token(token: &str) -> Result<TokenClaims, String> {
    // First, try legacy format: "web3_token_{wallet_address}"
    if token.starts_with("web3_token_") {
        let wallet = token.strip_prefix("web3_token_").unwrap_or("");
        if !wallet.is_empty() && wallet.len() >= 20 {
            return Ok(TokenClaims {
                wallet_address: wallet.to_lowercase(),
                sub: wallet.to_lowercase(),
                permissions: vec!["user:*".to_string()],
                jti: None,
                exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp(),
            });
        }
    }

    // Fall back to JWT decoding
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "epsx-web3-bearer-token-secret-key".to_string());
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    decode::<TokenClaims>(token, &decoding_key, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|e| format!("JWT decode failed: {}", e))
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
