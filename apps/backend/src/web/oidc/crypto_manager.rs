/// Cryptographic Operations Manager
/// 
/// Handles cryptographic operations for OIDC tokens including key management,
/// PKCE validation, hashing, and signature verification.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use base64::Engine;
use uuid::Uuid;

use crate::config::env::get_env_var;

/// PKCE code challenge methods
#[derive(Debug, Clone, PartialEq)]
pub enum PkceMethod {
    Plain,
    S256,
}

impl PkceMethod {
    pub fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match s {
            "plain" => Ok(PkceMethod::Plain),
            "S256" => Ok(PkceMethod::S256),
            _ => Err(format!("Unsupported PKCE method: {}", s).into()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PkceMethod::Plain => "plain",
            PkceMethod::S256 => "S256",
        }
    }
}

/// JWT key information
#[derive(Debug, Clone)]
pub struct JwtKeyInfo {
    pub algorithm: String,
    pub key_id: String,
    pub public_key: Option<String>,
    pub private_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Cryptographic manager service
pub struct CryptoManager {
    jwt_secret: String,
    key_rotation_enabled: bool,
    current_key_id: String,
}

impl CryptoManager {
    pub fn new() -> Self {
        Self {
            jwt_secret: get_jwt_secret(),
            key_rotation_enabled: false, // TODO: Implement key rotation
            current_key_id: "default-key-1".to_string(),
        }
    }

    /// Validate PKCE code challenge and verifier
    pub fn validate_pkce(
        &self,
        challenge: &str,
        method: &PkceMethod,
        verifier: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Validating PKCE: method={:?}, challenge={}, verifier={}", 
                       method, challenge, verifier);

        // Validate code_verifier format (RFC 7636)
        self.validate_code_verifier(verifier)?;

        match method {
            PkceMethod::S256 => {
                let computed_challenge = self.generate_s256_challenge(verifier)?;
                
                if computed_challenge == challenge {
                    tracing::debug!("PKCE S256 validation successful");
                    Ok(())
                } else {
                    tracing::error!("PKCE challenge mismatch: expected={}, computed={}", 
                                   challenge, computed_challenge);
                    Err("PKCE challenge verification failed".into())
                }
            },
            PkceMethod::Plain => {
                if verifier == challenge {
                    tracing::debug!("PKCE plain validation successful");
                    Ok(())
                } else {
                    tracing::error!("PKCE plain challenge mismatch");
                    Err("PKCE plain challenge verification failed".into())
                }
            },
        }
    }

    /// Generate S256 challenge from verifier
    pub fn generate_s256_challenge(&self, verifier: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.validate_code_verifier(verifier)?;
        
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let digest = hasher.finalize();
        
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(&digest);
            
        Ok(challenge)
    }

    /// Generate a cryptographically secure code verifier
    pub fn generate_code_verifier(&self) -> String {
        use rand::Rng;
        
        // Generate 32 random bytes
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        
        // Base64URL encode without padding
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes)
    }

    /// Generate authorization code challenge
    pub fn generate_code_challenge(&self, verifier: &str, method: &PkceMethod) -> Result<String, Box<dyn std::error::Error>> {
        match method {
            PkceMethod::S256 => self.generate_s256_challenge(verifier),
            PkceMethod::Plain => Ok(verifier.to_string()),
        }
    }

    /// Validate code verifier format
    pub fn validate_code_verifier(&self, verifier: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Validate length (RFC 7636)
        if verifier.len() < 43 || verifier.len() > 128 {
            return Err("code_verifier must be 43-128 characters long".into());
        }

        // Check for valid characters (unreserved characters from RFC 3986)
        let valid_chars = verifier.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_' || c == '~'
        });
        
        if !valid_chars {
            return Err("code_verifier contains invalid characters".into());
        }

        Ok(())
    }

    /// Generate a unique JWT ID (JTI) for token revocation support
    pub fn generate_jti(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate a secure random state parameter
    pub fn generate_state(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate a secure random nonce
    pub fn generate_nonce(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Hash a string using SHA256
    pub fn hash_sha256(&self, input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Generate a secure random token
    pub fn generate_random_token(&self, length: usize) -> String {
        use rand::Rng;
        
        if length == 0 {
            return String::new();
        }
        
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
        
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes)
    }

    /// Get current JWT signing key information
    pub fn get_current_key_info(&self) -> JwtKeyInfo {
        JwtKeyInfo {
            algorithm: "HS256".to_string(),
            key_id: self.current_key_id.clone(),
            public_key: None, // HMAC doesn't have public keys
            private_key: self.jwt_secret.clone(),
            created_at: chrono::Utc::now(), // TODO: Store actual creation time
            expires_at: None, // TODO: Implement key expiration
        }
    }

    /// Validate JWT signature (placeholder for future RSA support)
    pub fn validate_jwt_signature(
        &self,
        token: &str,
        key_id: Option<&str>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // For now, we only support HMAC256
        if let Some(kid) = key_id {
            if kid != self.current_key_id {
                return Err("Unknown key ID".into());
            }
        }

        // Basic JWT format validation
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".into());
        }

        // For HMAC, signature validation is done by jsonwebtoken crate
        // This is a placeholder for when we add RSA support
        Ok(true)
    }

    /// Generate key pair for RSA256 (placeholder for future implementation)
    pub fn generate_rsa_key_pair(&self) -> Result<(String, String), Box<dyn std::error::Error>> {
        // TODO: Implement RSA key pair generation
        Err("RSA key generation not yet implemented".into())
    }

    /// Rotate JWT signing keys (placeholder for future implementation)
    pub fn rotate_keys(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if !self.key_rotation_enabled {
            return Err("Key rotation is not enabled".into());
        }

        // TODO: Implement key rotation
        // 1. Generate new key pair
        // 2. Update current_key_id
        // 3. Keep old key for validation during transition period
        // 4. Update JWKS endpoint

        let new_key_id = format!("key-{}", Uuid::new_v4());
        self.current_key_id = new_key_id.clone();
        
        tracing::info!("JWT keys rotated, new key ID: {}", new_key_id);
        Ok(new_key_id)
    }

    /// Get JWKS (JSON Web Key Set) for public key distribution
    pub fn get_jwks(&self) -> Result<JwksResponse, Box<dyn std::error::Error>> {
        // For HMAC, we don't expose the secret key
        // This would be used for RSA public keys
        Ok(JwksResponse {
            keys: vec![], // Empty for HMAC
        })
    }

    /// Verify token timestamp claims
    pub fn verify_token_timing(
        &self,
        iat: i64,
        exp: i64,
        nbf: i64,
        clock_skew_seconds: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = chrono::Utc::now().timestamp();
        
        // Check if token is not yet valid (nbf - not before)
        if now < (nbf - clock_skew_seconds) {
            return Err("Token is not yet valid (nbf)".into());
        }
        
        // Check if token has expired
        if now > (exp + clock_skew_seconds) {
            return Err("Token has expired".into());
        }
        
        // Check if iat is in the future (with clock skew)
        if iat > (now + clock_skew_seconds) {
            return Err("Token issued in the future".into());
        }
        
        Ok(())
    }

    /// Generate secure session ID
    pub fn generate_session_id(&self) -> String {
        format!("sess_{}", Uuid::new_v4())
    }

    /// Generate device fingerprint
    pub fn generate_device_fingerprint(&self, user_agent: &str, ip: &str) -> String {
        let input = format!("{}:{}", user_agent, ip);
        self.hash_sha256(&input)
    }

    /// Constant-time string comparison to prevent timing attacks
    pub fn constant_time_compare(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
            result |= byte_a ^ byte_b;
        }
        
        result == 0
    }

    /// Secure random bytes generation
    pub fn generate_random_bytes(&self, length: usize) -> Vec<u8> {
        use rand::RngCore;
        
        let mut bytes = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// JWKS response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JwksResponse {
    pub keys: Vec<JwkKey>,
}

/// JSON Web Key structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JwkKey {
    pub kty: String, // Key type
    pub use_: String, // Key use
    pub alg: String, // Algorithm
    pub kid: String, // Key ID
    pub n: Option<String>, // RSA modulus
    pub e: Option<String>, // RSA exponent
}

// Utility functions

fn get_jwt_secret() -> String {
    get_env_var("NEXTAUTH_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_s256_validation() {
        let crypto = CryptoManager::new();
        
        // Test case from RFC 7636
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected_challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        
        let computed_challenge = crypto.generate_s256_challenge(verifier).unwrap();
        assert_eq!(computed_challenge, expected_challenge);
        
        // Test validation
        let result = crypto.validate_pkce(expected_challenge, &PkceMethod::S256, verifier);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pkce_plain_validation() {
        let crypto = CryptoManager::new();
        
        let verifier = "test-verifier-1234567890-abcdefghijklmnop";
        let challenge = verifier; // Same for plain method
        
        let result = crypto.validate_pkce(challenge, &PkceMethod::Plain, verifier);
        assert!(result.is_ok());
    }

    #[test]
    fn test_code_verifier_validation() {
        let crypto = CryptoManager::new();
        
        // Valid verifier
        let valid_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert!(crypto.validate_code_verifier(valid_verifier).is_ok());
        
        // Too short
        let short_verifier = "short";
        assert!(crypto.validate_code_verifier(short_verifier).is_err());
        
        // Invalid characters
        let invalid_verifier = "invalid@characters#here$%^&*()+=[]{}|\\:;\"'<>?,./";
        assert!(crypto.validate_code_verifier(&invalid_verifier[0..43]).is_err());
    }

    #[test]
    fn test_generate_code_verifier() {
        let crypto = CryptoManager::new();
        
        let verifier = crypto.generate_code_verifier();
        
        // Should be valid format
        assert!(crypto.validate_code_verifier(&verifier).is_ok());
        
        // Should be proper length (base64 of 32 bytes = 43 chars without padding)
        assert_eq!(verifier.len(), 43);
        
        // Should be different each time
        let verifier2 = crypto.generate_code_verifier();
        assert_ne!(verifier, verifier2);
    }

    #[test]
    fn test_hash_sha256() {
        let crypto = CryptoManager::new();
        
        let input = "test string";
        let hash = crypto.hash_sha256(input);
        
        // SHA256 hex should be 64 characters
        assert_eq!(hash.len(), 64);
        
        // Should be deterministic
        let hash2 = crypto.hash_sha256(input);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_constant_time_compare() {
        let crypto = CryptoManager::new();
        
        assert!(crypto.constant_time_compare("hello", "hello"));
        assert!(!crypto.constant_time_compare("hello", "world"));
        assert!(!crypto.constant_time_compare("hello", "hello2"));
        assert!(!crypto.constant_time_compare("hello", "hell"));
    }

    #[test]
    fn test_verify_token_timing() {
        let crypto = CryptoManager::new();
        let now = chrono::Utc::now().timestamp();
        
        // Valid timing
        let iat = now - 60; // Issued 1 minute ago
        let exp = now + 3600; // Expires in 1 hour
        let nbf = now - 60; // Valid since 1 minute ago
        
        assert!(crypto.verify_token_timing(iat, exp, nbf, 30).is_ok());
        
        // Expired token
        let exp_past = now - 60; // Expired 1 minute ago
        assert!(crypto.verify_token_timing(iat, exp_past, nbf, 30).is_err());
        
        // Not yet valid
        let nbf_future = now + 60; // Valid in 1 minute
        assert!(crypto.verify_token_timing(iat, exp, nbf_future, 30).is_err());
        
        // Issued in future
        let iat_future = now + 60; // Issued in 1 minute
        assert!(crypto.verify_token_timing(iat_future, exp, nbf, 30).is_err());
    }

    #[test]
    fn test_generate_random_token() {
        let crypto = CryptoManager::new();
        
        let token1 = crypto.generate_random_token(32);
        let token2 = crypto.generate_random_token(32);
        
        // Should be different
        assert_ne!(token1, token2);
        
        // Should be proper base64
        assert!(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&token1).is_ok());
        
        // Empty length should return empty string
        let empty_token = crypto.generate_random_token(0);
        assert_eq!(empty_token, "");
    }
}