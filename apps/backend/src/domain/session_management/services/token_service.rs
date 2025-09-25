// Token Domain Service for JWT operations
use crate::domain::shared_kernel::value_objects::UserId;
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn, error};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    sub: String,        // Subject (user ID)
    exp: usize,         // Expiration time
    iat: usize,         // Issued at
    nbf: usize,         // Not before
    iss: String,        // Issuer
    aud: String,        // Audience
    jti: String,        // JWT ID
}

pub struct TokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    audience: String,
    token_expiry_hours: i64,
}

impl TokenService {
    pub fn new(secret: &str, issuer: String, audience: String) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            issuer,
            audience,
            token_expiry_hours: 24, // 24 hour expiry
        }
    }
    
    pub fn with_expiry(secret: &str, issuer: String, audience: String, expiry_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            issuer,
            audience,
            token_expiry_hours: expiry_hours,
        }
    }

    /// Validates JWT token and returns user ID if valid
    pub fn validate_token(&self, token: &str) -> Result<UserId, String> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        
        match decode::<TokenClaims>(token, &self.decoding_key, &validation) {
            Ok(token_data) => {
                let user_id = UserId::parse(&token_data.claims.sub)
                    .map_err(|e| format!("Invalid user ID in token: {}", e))?;
                
                debug!("Token validation successful for user: {}", user_id);
                Ok(user_id)
            }
            Err(e) => {
                warn!("Token validation failed: {}", e);
                Err(format!("Invalid token: {}", e))
            }
        }
    }

    /// Generates JWT token for user
    pub fn generate_token(&self, user_id: &UserId) -> Result<String, String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.token_expiry_hours);
        
        let claims = TokenClaims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            nbf: now.timestamp() as usize,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            jti: Uuid::new_v4().to_string(), // Unique token ID
        };
        
        let header = Header::new(Algorithm::HS256);
        
        match encode(&header, &claims, &self.encoding_key) {
            Ok(token) => {
                debug!("Generated token for user: {}, expires: {}", user_id, exp);
                Ok(token)
            }
            Err(e) => {
                error!("Token generation failed for user {}: {}", user_id, e);
                Err(format!("Token generation failed: {}", e))
            }
        }
    }
    
    /// Extracts user ID from token without full validation (for middleware)
    pub fn extract_user_id(&self, token: &str) -> Result<UserId, String> {
        // Use lenient validation just to extract claims
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false; // Don't validate expiry for extraction
        validation.validate_aud = false;
        validation.validate_iss = false;
        
        match decode::<TokenClaims>(token, &self.decoding_key, &validation) {
            Ok(token_data) => {
                UserId::parse(&token_data.claims.sub)
                    .map_err(|e| format!("Invalid user ID in token: {}", e))
            }
            Err(e) => Err(format!("Cannot extract user ID: {}", e))
        }
    }
}