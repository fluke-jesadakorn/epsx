use crate::core::errors::AppError;
use sha2::{Sha256, Digest};
use std::env;
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
    Public,       // Cannot store secrets securely (mobile apps, SPAs)
}

/// Client credential validation service
pub struct ClientCredentialService {
    clients: HashMap<String, ClientCredentials>,
}

/// Static registry of allowed clients loaded from environment
static CLIENT_REGISTRY: Lazy<HashMap<String, ClientCredentials>> = Lazy::new(|| {
    let mut clients = HashMap::new();
    
    // Frontend client
    if let (Ok(client_id), Ok(client_secret)) = (
        env::var("OIDC_FRONTEND_CLIENT_ID"),
        env::var("OIDC_FRONTEND_CLIENT_SECRET")
    ) {
        clients.insert(client_id.clone(), ClientCredentials {
            client_id: client_id.clone(),
            client_secret,
            redirect_uris: vec![
                "http://localhost:3000/auth/callback".to_string(),
                "https://app.epsx.com/auth/callback".to_string(),
            ],
            allowed_scopes: vec![
                "openid".to_string(),
                "profile".to_string(), 
                "email".to_string(),
            ],
            client_type: ClientType::Confidential,
        });
    }
    
    // Admin client  
    if let (Ok(client_id), Ok(client_secret)) = (
        env::var("OIDC_ADMIN_CLIENT_ID"),
        env::var("OIDC_ADMIN_CLIENT_SECRET")
    ) {
        clients.insert(client_id.clone(), ClientCredentials {
            client_id: client_id.clone(), 
            client_secret,
            redirect_uris: vec![
                "http://localhost:3001/auth/callback".to_string(),
                "https://admin.epsx.com/auth/callback".to_string(),
            ],
            allowed_scopes: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
                "admin".to_string(),
            ],
            client_type: ClientType::Confidential,
        });
    }
    
    clients
});

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
        client_secret: &str,
    ) -> Result<&ClientCredentials, AppError> {
        tracing::debug!("Validating client credentials for client_id: {}", client_id);
        
        let client = self.clients.get(client_id)
            .ok_or_else(|| AppError::security_error(
                format!("Unknown client_id: {}", client_id)
            ))?;
        
        // Use constant-time comparison to prevent timing attacks
        if !self.constant_time_compare(&client.client_secret, client_secret) {
            tracing::warn!("Invalid client secret for client_id: {}", client_id);
            return Err(AppError::security_error("Invalid client credentials".to_string()));
        }
        
        tracing::debug!("Client credentials validated successfully for: {}", client_id);
        Ok(client)
    }
    
    /// Validate client credentials from HTTP Basic Authentication
    pub fn validate_basic_auth(
        &self,
        auth_header: &str,
    ) -> Result<&ClientCredentials, AppError> {
        if !auth_header.starts_with("Basic ") {
            return Err(AppError::security_error("Invalid authorization header format".to_string()));
        }
        
        let encoded_credentials = &auth_header[6..];
        let decoded = BASE64_STANDARD.decode(encoded_credentials)
            .map_err(|_| AppError::security_error("Invalid base64 encoding".to_string()))?;
            
        let credentials_str = String::from_utf8(decoded)
            .map_err(|_| AppError::security_error("Invalid UTF-8 in credentials".to_string()))?;
            
        let parts: Vec<&str> = credentials_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AppError::security_error("Invalid credential format".to_string()));
        }
        
        let (client_id, client_secret) = (parts[0], parts[1]);
        self.validate_client_credentials(client_id, client_secret)
    }
    
    /// Check if redirect URI is allowed for client
    pub fn is_redirect_uri_allowed(
        &self,
        client_id: &str,
        redirect_uri: &str,
    ) -> Result<bool, AppError> {
        let client = self.clients.get(client_id)
            .ok_or_else(|| AppError::security_error(
                format!("Unknown client_id: {}", client_id)
            ))?;
            
        Ok(client.redirect_uris.iter().any(|allowed_uri| allowed_uri == redirect_uri))
    }
    
    /// Check if scope is allowed for client
    pub fn is_scope_allowed(
        &self,
        client_id: &str,
        requested_scope: &str,
    ) -> Result<bool, AppError> {
        let client = self.clients.get(client_id)
            .ok_or_else(|| AppError::security_error(
                format!("Unknown client_id: {}", client_id)
            ))?;
        
        let requested_scopes: Vec<&str> = requested_scope.split_whitespace().collect();
        
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
        
        let secret: String = rand::thread_rng()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    fn setup_test_env() {
        env::set_var("OIDC_FRONTEND_CLIENT_ID", "test-frontend");
        env::set_var("OIDC_FRONTEND_CLIENT_SECRET", "test-frontend-secret");
        env::set_var("OIDC_ADMIN_CLIENT_ID", "test-admin");
        env::set_var("OIDC_ADMIN_CLIENT_SECRET", "test-admin-secret");
    }
    
    #[test]
    fn test_client_credential_validation() {
        setup_test_env();
        let service = ClientCredentialService::new();
        
        // Valid credentials
        let result = service.validate_client_credentials("test-frontend", "test-frontend-secret");
        assert!(result.is_ok());
        
        // Invalid client_id
        let result = service.validate_client_credentials("unknown-client", "any-secret");
        assert!(result.is_err());
        
        // Invalid client_secret
        let result = service.validate_client_credentials("test-frontend", "wrong-secret");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_basic_auth_validation() {
        setup_test_env();
        let service = ClientCredentialService::new();
        
        // Valid Basic Auth header
        let credentials = BASE64_STANDARD.encode("test-frontend:test-frontend-secret");
        let auth_header = format!("Basic {}", credentials);
        let result = service.validate_basic_auth(&auth_header);
        assert!(result.is_ok());
        
        // Invalid format
        let result = service.validate_basic_auth("Bearer token123");
        assert!(result.is_err());
        
        // Invalid base64
        let result = service.validate_basic_auth("Basic invalid_base64!");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_redirect_uri_validation() {
        setup_test_env();
        let service = ClientCredentialService::new();
        
        // Allowed URI
        let result = service.is_redirect_uri_allowed(
            "test-frontend",
            "http://localhost:3000/auth/callback"
        );
        assert!(result.unwrap());
        
        // Not allowed URI
        let result = service.is_redirect_uri_allowed(
            "test-frontend", 
            "http://malicious.com/callback"
        );
        assert!(!result.unwrap());
    }
    
    #[test]
    fn test_scope_validation() {
        setup_test_env();
        let service = ClientCredentialService::new();
        
        // Allowed scopes
        let result = service.is_scope_allowed("test-frontend", "openid profile email");
        assert!(result.unwrap());
        
        // Admin scope only allowed for admin client
        let result = service.is_scope_allowed("test-frontend", "admin");
        assert!(!result.unwrap());
        
        let result = service.is_scope_allowed("test-admin", "admin");
        assert!(result.unwrap());
    }
    
    #[test]
    fn test_constant_time_compare() {
        let service = ClientCredentialService::new();
        
        assert!(service.constant_time_compare("test", "test"));
        assert!(!service.constant_time_compare("test", "fail"));
        assert!(!service.constant_time_compare("test", "testing")); // Different lengths
    }
    
    #[test] 
    fn test_secret_generation_and_hashing() {
        let secret = ClientCredentialService::generate_client_secret();
        assert!(secret.starts_with("sk-"));
        assert!(secret.len() > 10);
        
        let hash = ClientCredentialService::hash_client_secret(&secret);
        assert_eq!(hash.len(), 64); // SHA256 hex length
        
        let service = ClientCredentialService::new();
        assert!(service.verify_hashed_secret(&secret, &hash));
        assert!(!service.verify_hashed_secret("wrong", &hash));
    }
}