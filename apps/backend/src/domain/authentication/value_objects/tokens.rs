// Authentication Tokens Value Objects
// JWT-based tokens for OIDC compliance

use crate::domain::authentication::{AuthenticatedUserId, Scope};
use crate::domain::authentication::value_objects::ClientInformation;
use crate::domain::shared_kernel::value_objects::SessionId;
use chrono::{DateTime, Utc};

use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use uuid::Uuid;

use super::scopes::ScopeError;

/// Access Token for API authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    token: String,
    claims: AccessTokenClaims,
    expires_at: DateTime<Utc>,
}

impl AccessToken {
    /// Generate new access token
    pub fn generate(
        user_id: &AuthenticatedUserId,
        scopes: &[Scope], 
        expires_at: DateTime<Utc>,
    ) -> Result<Self, TokenError> {
        let now = Utc::now();
        let jti = Uuid::new_v4().to_string();
        
        let claims = AccessTokenClaims {
            sub: user_id.to_string(),
            iss: "https://api.epsx.io".to_string(),
            aud: "epsx-api".to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            jti,
            scope: scopes.iter().map(|s| s.as_str().to_string()).collect::<Vec<_>>().join(" "),
            token_type: "Bearer".to_string(),
        };
        
        // Sign the token (in real implementation, use proper secret management)
        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret("your-secret-key".as_ref());
        
        let token = encode(&header, &claims, &encoding_key)
            .map_err(|e| TokenError::GenerationFailed(e.to_string()))?;
        
        Ok(Self {
            token,
            claims,
            expires_at,
        })
    }
    
    /// Validate and parse access token
    pub fn from_jwt(token: String) -> Result<Self, TokenError> {
        let decoding_key = DecodingKey::from_secret("your-secret-key".as_ref());
        let validation = Validation::new(Algorithm::HS256);
        
        let token_data = decode::<AccessTokenClaims>(&token, &decoding_key, &validation)
            .map_err(|e| TokenError::InvalidToken(e.to_string()))?;
        
        let expires_at = DateTime::from_timestamp(token_data.claims.exp, 0)
            .ok_or_else(|| TokenError::InvalidToken("Invalid expiry timestamp".to_string()))?;
        
        Ok(Self {
            token,
            claims: token_data.claims,
            expires_at,
        })
    }
    
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Get scopes from token
    pub fn scopes(&self) -> Vec<Scope> {
        self.claims.scope
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect()
    }
    
    /// Get user ID from token
    pub fn user_id(&self) -> Result<AuthenticatedUserId, TokenError> {
        // Parse user ID from subject claim
        let user_id_str = self.claims.sub
            .strip_prefix("auth:")
            .unwrap_or(&self.claims.sub);
        
        // Create UserId from string - assuming it's stored as string in JWT
        let user_id = crate::domain::shared_kernel::value_objects::UserId::from_string_unchecked(user_id_str.to_string());
        Ok(AuthenticatedUserId::from_verified_user(user_id))
    }
    
    // Getters
    pub fn token(&self) -> &str { &self.token }
    pub fn expires_at(&self) -> DateTime<Utc> { self.expires_at }
    pub fn issuer(&self) -> &str { &self.claims.iss }
    pub fn audience(&self) -> &str { &self.claims.aud }
    pub fn jti(&self) -> &str { &self.claims.jti }
}

/// Refresh Token for token renewal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    token: String,
    session_id: SessionId,
    expires_at: DateTime<Utc>,
    jti: String,
}

impl RefreshToken {
    /// Generate new refresh token
    pub fn generate(
        session_id: &SessionId,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, TokenError> {
        let jti = Uuid::new_v4().to_string();
        
        // Generate cryptographically secure token
        let token = format!("rt_{}_{}", session_id.as_str(), jti);
        
        Ok(Self {
            token,
            session_id: session_id.clone(),
            expires_at,
            jti,
        })
    }
    
    /// Validate refresh token format
    pub fn from_string(token: String) -> Result<Self, TokenError> {
        // Parse refresh token format: rt_{session_id}_{jti}
        let parts: Vec<&str> = token.split('_').collect();
        if parts.len() < 3 || parts[0] != "rt" {
            return Err(TokenError::InvalidToken("Invalid refresh token format".to_string()));
        }
        
        let session_part = parts[1..parts.len()-1].join("_");
        let jti = parts.last().unwrap().to_string();
        
        let session_id = SessionId::from_string(format!("sess_{}", session_part));
        
        // For validation, we'd typically look up in database to get expiry
        // For now, assume 30 days from creation
        let expires_at = Utc::now() + chrono::Duration::days(30);
        
        Ok(Self {
            token,
            session_id,
            expires_at,
            jti,
        })
    }
    
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    // Getters
    pub fn token(&self) -> &str { &self.token }
    pub fn session_id(&self) -> &SessionId { &self.session_id }
    pub fn expires_at(&self) -> DateTime<Utc> { self.expires_at }
    pub fn jti(&self) -> &str { &self.jti }
}

/// ID Token for OpenID Connect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdToken {
    token: String,
    claims: IdTokenClaims,
    expires_at: DateTime<Utc>,
}

impl IdToken {
    /// Generate new ID token
    pub fn generate(
        user_id: &AuthenticatedUserId,
        client_info: &ClientInformation,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, TokenError> {
        let now = Utc::now();
        let jti = Uuid::new_v4().to_string();
        
        let claims = IdTokenClaims {
            sub: user_id.to_string(),
            iss: "https://api.epsx.io".to_string(),
            aud: client_info.client_id().to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            jti,
            // Additional OIDC standard claims would be populated from user profile
            email: None,
            email_verified: None,
            name: None,
            picture: None,
        };
        
        // Sign the token
        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret("your-secret-key".as_ref());
        
        let token = encode(&header, &claims, &encoding_key)
            .map_err(|e| TokenError::GenerationFailed(e.to_string()))?;
        
        Ok(Self {
            token,
            claims,
            expires_at,
        })
    }
    
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    // Getters
    pub fn token(&self) -> &str { &self.token }
    pub fn expires_at(&self) -> DateTime<Utc> { self.expires_at }
    pub fn subject(&self) -> &str { &self.claims.sub }
    pub fn audience(&self) -> &str { &self.claims.aud }
}

// Implement PartialEq for token comparison
impl PartialEq for RefreshToken {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

// JWT Claims structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccessTokenClaims {
    sub: String,          // Subject (user ID)
    iss: String,          // Issuer
    aud: String,          // Audience
    iat: i64,             // Issued at
    exp: i64,             // Expires at
    jti: String,          // JWT ID
    scope: String,        // OAuth2 scopes
    token_type: String,   // Token type
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdTokenClaims {
    sub: String,                    // Subject (user ID)
    iss: String,                    // Issuer
    aud: String,                    // Audience (client ID)
    iat: i64,                       // Issued at
    exp: i64,                       // Expires at
    jti: String,                    // JWT ID
    email: Option<String>,          // User email
    email_verified: Option<bool>,   // Email verification status
    name: Option<String>,           // User display name
    picture: Option<String>,        // User profile picture URL
}

/// Token-related errors
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Token generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Token has expired")]
    Expired,
    
    #[error("Token signature verification failed")]
    SignatureVerificationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_management::value_objects::UserId;
    
    #[test]
    fn access_token_generation_and_validation() {
        let user_id = AuthenticatedUserId::from_verified_user(
            UserId::new().unwrap()
        );
        let scopes = vec![Scope::OpenId, Scope::Profile];
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        
        let token = AccessToken::generate(&user_id, &scopes, expires_at).unwrap();
        
        assert!(!token.is_expired());
        assert_eq!(token.scopes().len(), 2);
        assert!(token.token().len() > 0);
    }
    
    #[test]
    fn refresh_token_generation() {
        let session_id = SessionId::generate();
        let expires_at = Utc::now() + chrono::Duration::days(30);
        
        let token = RefreshToken::generate(&session_id, expires_at).unwrap();
        
        assert!(!token.is_expired());
        assert_eq!(token.session_id(), &session_id);
        assert!(token.token().starts_with("rt_"));
    }
}