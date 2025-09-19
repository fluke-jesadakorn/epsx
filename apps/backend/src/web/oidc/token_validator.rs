/// Token Validation Service
/// 
/// Handles JWT token validation and verification for access tokens and ID tokens.
/// Supports both admin and user token validation with unified claims processing.

use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

use crate::config::env::get_env_var;
use super::token_generator::{AccessTokenClaims, IdTokenClaims};

/// Authorization code validation claims
#[derive(Debug, Serialize, Deserialize)]
struct AuthCodeClaims {
    iss: String,
    sub: String,
    aud: String,
    exp: i64,
    iat: i64,
    data: crate::web::oidc::authorization::AuthorizationCodeData,
}

/// Token validation service
pub struct TokenValidator {
    jwt_secret: String,
    issuer_url: String,
}

impl TokenValidator {
    pub fn new() -> Self {
        Self {
            jwt_secret: get_jwt_secret(),
            issuer_url: get_issuer_url(),
        }
    }

    /// Validate unified access token (supports both admin and user tokens)
    /// Returns (sub, email, permissions, package_tier)
    pub fn validate_unified_access_token(&self, token: &str) -> Result<(String, String, Vec<String>, String), Box<dyn std::error::Error>> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["epsx-api"]);
        validation.set_issuer(&[&self.issuer_url]);
        
        // Try to decode as admin token first by checking token_type
        if let Ok(admin_token) = decode::<crate::auth::admin_jwt::AdminJWTClaims>(token, &decoding_key, &validation) {
            if admin_token.claims.token_type == "admin_access" {
                tracing::debug!("Validated as admin token for user: {}", admin_token.claims.email);
                
                // Extract permissions from admin token
                let permissions: Vec<String> = admin_token.claims.permissions.system_access.capabilities;
                
                return Ok((
                    admin_token.claims.sub,
                    admin_token.claims.email,
                    permissions,
                    "ADMIN".to_string(),
                ));
            }
        }
        
        // Try to decode as user token by checking token_type
        if let Ok(user_token) = decode::<crate::auth::user_jwt::UserJWTClaims>(token, &decoding_key, &validation) {
            if user_token.claims.token_type == "user_access" {
                tracing::debug!("Validated as user token for user: {}", user_token.claims.email);
                
                // Extract permissions from user token
                let permissions = user_token.claims.permissions.permissions;
                let package_tier = user_token.claims.subscription
                    .as_ref()
                    .map(|s| s.tier.clone())
                    .unwrap_or_else(|| "FREE".to_string());
                
                return Ok((
                    user_token.claims.sub,
                    user_token.claims.email,
                    permissions,
                    package_tier,
                ));
            }
        }
        
        // Try legacy token format as fallback (no token_type field)
        if let Ok(legacy_token) = decode::<AccessTokenClaims>(token, &decoding_key, &validation) {
            tracing::debug!("Validated as legacy token for user: {}", legacy_token.claims.email);
            
            return Ok((
                legacy_token.claims.sub,
                legacy_token.claims.email,
                legacy_token.claims.permissions,
                legacy_token.claims.package_tier,
            ));
        }
        
        Err("Token validation failed for all supported formats".into())
    }

    /// Validate access token and extract claims (legacy function)
    pub fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims, Box<dyn std::error::Error>> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        
        // Validate standard claims
        validation.set_audience(&["epsx-api"]);
        validation.set_issuer(&[&self.issuer_url]);
        
        let token_data = decode::<AccessTokenClaims>(token, &decoding_key, &validation)
            .map_err(|e| format!("JWT validation failed: {}", e))?;
        
        Ok(token_data.claims)
    }

    /// Validate ID token and extract claims
    pub fn validate_id_token(&self, token: &str, expected_audience: &str) -> Result<IdTokenClaims, Box<dyn std::error::Error>> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        
        // Validate standard claims
        validation.set_audience(&[expected_audience]);
        validation.set_issuer(&[&self.issuer_url]);
        
        let token_data = decode::<IdTokenClaims>(token, &decoding_key, &validation)
            .map_err(|e| format!("JWT validation failed: {}", e))?;
        
        Ok(token_data.claims)
    }

    /// Validate and consume stateless authorization code
    pub fn validate_and_consume_authorization_code(
        &self,
        code: &str,
    ) -> Result<crate::web::oidc::authorization::AuthorizationCodeData, Box<dyn std::error::Error>> {
        use chrono::Utc;
        
        tracing::debug!("Validating authorization code: starts_with_ac_jwt={}", code.starts_with("ac_jwt_"));
        
        // Check if this is a stateless JWT authorization code
        if !code.starts_with("ac_jwt_") {
            tracing::error!("Authorization code does not start with 'ac_jwt_': {}", &code[0..20.min(code.len())]);
            return Err("Invalid authorization code format".into());
        }
        
        // Extract JWT token from authorization code
        let jwt_token = code.strip_prefix("ac_jwt_")
            .ok_or("Invalid authorization code format")?;
            
        tracing::debug!("Extracted JWT token length: {}", jwt_token.len());
        
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        
        // Disable audience validation since we validate client_id separately
        validation.validate_aud = false;
        
        // Decode and validate the JWT
        tracing::debug!("Attempting to decode JWT authorization code with secret length: {}", self.jwt_secret.len());
        let token_data = decode::<AuthCodeClaims>(jwt_token, &decoding_key, &validation)
            .map_err(|e| {
                tracing::error!("JWT decode failed: {:?}", e);
                format!("Invalid authorization code: {}", e)
            })?;
        
        let claims = token_data.claims;
        
        // Check if the authorization code has expired
        let now = Utc::now().timestamp();
        if now > claims.exp {
            return Err("Authorization code has expired".into());
        }
        
        tracing::info!("Successfully validated stateless authorization code for user: {}", claims.sub);
        
        // Return the embedded authorization data
        Ok(claims.data)
    }

    /// Check if a token has been revoked (placeholder for future implementation)
    pub fn is_token_revoked(&self, jti: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // TODO: Implement token revocation checking
        // This would typically check against a revoked tokens list in Redis or database
        tracing::debug!("Checking revocation status for token: {}", jti);
        Ok(false) // For now, no tokens are considered revoked
    }

    /// Validate token expiration
    pub fn is_token_expired(&self, exp: i64) -> bool {
        use chrono::Utc;
        let now = Utc::now().timestamp();
        now > exp
    }

    /// Extract JTI (JWT ID) from token without full validation
    pub fn extract_jti(&self, token: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Decode without verification to extract JTI
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.validate_aud = false;
        
        // Try different token types
        if let Ok(token_data) = decode::<AccessTokenClaims>(token, &decoding_key, &validation) {
            return Ok(token_data.claims.jti);
        }
        
        if let Ok(token_data) = decode::<IdTokenClaims>(token, &decoding_key, &validation) {
            return Ok(token_data.claims.jti);
        }
        
        Err("Could not extract JTI from token".into())
    }

    /// Validate token format and structure without claims validation
    pub fn validate_token_format(&self, token: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Basic JWT format validation (3 parts separated by dots)
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format: must have 3 parts".into());
        }
        
        // Check if parts are valid base64
        use base64::Engine;
        for (i, part) in parts.iter().enumerate() {
            if base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(part).is_err() {
                return Err(format!("Invalid base64 in JWT part {}", i + 1).into());
            }
        }
        
        Ok(())
    }

    /// Validate bearer token format from Authorization header
    pub fn extract_bearer_token<'a>(&self, auth_header: &'a str) -> Result<&'a str, Box<dyn std::error::Error>> {
        if !auth_header.starts_with("Bearer ") {
            return Err("Invalid Authorization header format".into());
        }
        
        let token = &auth_header[7..]; // Remove "Bearer " prefix
        
        // Validate basic token format
        self.validate_token_format(token)?;
        
        Ok(token)
    }
}

impl Default for TokenValidator {
    fn default() -> Self {
        Self::new()
    }
}

// Utility functions

fn get_issuer_url() -> String {
    get_env_var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
}

fn get_jwt_secret() -> String {
    get_env_var("NEXTAUTH_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_token_format() {
        let validator = TokenValidator::new();
        
        // Valid JWT format
        let valid_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert!(validator.validate_token_format(valid_token).is_ok());
        
        // Invalid format (2 parts)
        let invalid_token = "header.payload";
        assert!(validator.validate_token_format(invalid_token).is_err());
        
        // Invalid format (4 parts)
        let invalid_token = "a.b.c.d";
        assert!(validator.validate_token_format(invalid_token).is_err());
    }

    #[test]
    fn test_extract_bearer_token() {
        let validator = TokenValidator::new();
        
        let auth_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let result = validator.extract_bearer_token(auth_header);
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("eyJ"));
        
        // Invalid format
        let invalid_header = "Basic token123";
        assert!(validator.extract_bearer_token(invalid_header).is_err());
    }

    #[test]
    fn test_is_token_expired() {
        let validator = TokenValidator::new();
        
        // Token expired 1 hour ago
        let past_exp = chrono::Utc::now().timestamp() - 3600;
        assert!(validator.is_token_expired(past_exp));
        
        // Token expires in 1 hour
        let future_exp = chrono::Utc::now().timestamp() + 3600;
        assert!(!validator.is_token_expired(future_exp));
    }
}