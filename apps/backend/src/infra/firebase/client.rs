// Firebase Client Management
// Focused module handling Firebase client initialization and connection management

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use reqwest::Client;
use chrono::{Utc, Duration};
use tracing::{info, warn};

use crate::config::env::get_env_var;
use super::types::{FirebaseAdmin, FirebasePublicKey, AccessTokenCache};

impl FirebaseAdmin {
    /// Create a new Firebase Admin SDK instance
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let project_id = get_env_var("FIREBASE_PROJECT_ID")
            .unwrap_or_else(|_| "default-project".to_string());
        
        let service_account_key = get_env_var("FIREBASE_SERVICE_ACCOUNT_KEY")
            .ok();

        info!("Initializing Firebase Admin with project_id: {}", project_id);

        Ok(Self {
            client: Client::new(),
            project_id,
            service_account_key,
            jwks_cache: HashMap::new(),
            jwks_cache_expiry: Utc::now() - Duration::hours(1), // Force initial fetch
            access_token_cache: Arc::new(Mutex::new(AccessTokenCache::default())),
        })
    }

    /// Get Firebase Admin SDK access token using service account with caching
    pub async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Check if we have a valid cached token
        {
            let cache = self.access_token_cache.lock().unwrap();
            if let Some(token) = &cache.token {
                if cache.expires_at > Utc::now() + Duration::minutes(5) { // 5 min buffer
                    info!("🔄 Using cached Firebase access token");
                    return Ok(token.clone());
                }
            }
            info!("♻️ Access token expired or not cached, refreshing...");
        } // Release lock
        
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Check if we have service account credentials
        if let (Ok(client_email), Ok(private_key_pem)) = (
            get_env_var("FIREBASE_CLIENT_EMAIL"),
            get_env_var("FIREBASE_PRIVATE_KEY"),
        ) {
            info!("Using Firebase service account: {}", client_email);
            
            // Create JWT claims
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let exp = now + 3600; // 1 hour expiration
            
            let claims = serde_json::json!({
                "iss": client_email,
                "scope": "https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/firebase.messaging",
                "aud": "https://oauth2.googleapis.com/token",
                "iat": now,
                "exp": exp
            });
            
            // Use jsonwebtoken library for proper JWT signing
            use jsonwebtoken::{encode, Header, Algorithm, EncodingKey};
            
            let header = Header::new(Algorithm::RS256);
            let key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
            let jwt = encode(&header, &claims, &key)?;
            
            info!("Generated JWT for service account authentication");
            
            // Exchange JWT for access token
            let token_url = "https://oauth2.googleapis.com/token";
            let params = [
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ];
            
            let response = self.client
                .post(token_url)
                .form(&params)
                .send()
                .await?;
                
            if response.status().is_success() {
                let token_response: serde_json::Value = response.json().await?;
                if let Some(access_token) = token_response.get("access_token") {
                    let token = access_token.as_str().unwrap().to_string();
                    let expires_in = token_response.get("expires_in")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(3600); // Default 1 hour
                    
                    // Cache the token
                    {
                        let mut cache = self.access_token_cache.lock().unwrap();
                        cache.token = Some(token.clone());
                        cache.expires_at = Utc::now() + Duration::seconds(expires_in as i64);
                    }
                    
                    info!("✅ Successfully obtained and cached Firebase access token (expires in {}s)", expires_in);
                    return Ok(token);
                } else {
                    warn!("No access token in response: {:?}", token_response);
                }
            } else {
                let error_text = response.text().await?;
                warn!("Failed to exchange JWT for access token: {}", error_text);
            }
            
            Err("Failed to exchange JWT for access token".into())
        } else {
            // Fallback to API key for development
            if let Ok(api_key) = get_env_var("FIREBASE_API_KEY") {
                info!("Using Firebase API key for development");
                Ok(api_key)
            } else {
                Err("No Firebase credentials configured - need FIREBASE_API_KEY or service account".into())
            }
        }
    }

    /// Get the project ID
    pub fn get_project_id(&self) -> &str {
        &self.project_id
    }

    /// Get the HTTP client
    pub fn get_client(&self) -> &Client {
        &self.client
    }

    /// Check if service account credentials are configured
    pub fn has_service_account(&self) -> bool {
        self.service_account_key.is_some() && get_env_var("FIREBASE_CLIENT_EMAIL").is_ok()
    }

    /// Check if API key is configured
    pub fn has_api_key(&self) -> bool {
        get_env_var("FIREBASE_API_KEY").is_ok()
    }

    /// Get authentication configuration summary
    pub fn get_auth_config_summary(&self) -> String {
        let has_service_account = self.has_service_account();
        let has_api_key = self.has_api_key();

        match (has_service_account, has_api_key) {
            (true, true) => "Service Account (preferred) + API Key (fallback)".to_string(),
            (true, false) => "Service Account only".to_string(),
            (false, true) => "API Key only (development mode)".to_string(),
            (false, false) => "No authentication configured".to_string(),
        }
    }

    /// Health check for Firebase client
    pub async fn health_check(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Try to get access token as a health check
        match self.get_access_token().await {
            Ok(_) => Ok(format!(
                "Firebase client healthy - Project: {}, Auth: {}",
                self.project_id,
                self.get_auth_config_summary()
            )),
            Err(e) => Err(format!("Firebase client unhealthy: {}", e).into()),
        }
    }

    /// Update JWKS cache
    pub fn update_jwks_cache(&mut self, keys: HashMap<String, FirebasePublicKey>) {
        self.jwks_cache = keys;
        self.jwks_cache_expiry = Utc::now() + Duration::hours(1);
        info!("Updated JWKS cache with {} keys", self.jwks_cache.len());
    }

    /// Check if JWKS cache is expired
    pub fn is_jwks_cache_expired(&self) -> bool {
        Utc::now() > self.jwks_cache_expiry
    }

    /// Get cached JWKS key by kid
    pub fn get_cached_jwks_key(&self, kid: &str) -> Option<&FirebasePublicKey> {
        if self.is_jwks_cache_expired() {
            warn!("JWKS cache expired but returning cached key anyway");
        }
        self.jwks_cache.get(kid)
    }

    /// Validate Firebase configuration
    pub fn validate_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check project ID
        if self.project_id.is_empty() || self.project_id == "default-project" {
            return Err("Invalid Firebase project ID - set FIREBASE_PROJECT_ID".into());
        }

        // Check authentication setup
        if !self.has_service_account() && !self.has_api_key() {
            return Err("No Firebase authentication configured - set FIREBASE_API_KEY or service account credentials".into());
        }

        info!("Firebase configuration valid: {}", self.get_auth_config_summary());
        Ok(())
    }
}

/// Firebase client factory methods
impl FirebaseAdmin {
    /// Create Firebase Admin client with custom project ID
    pub async fn with_project_id(project_id: String) -> Result<Self, Box<dyn std::error::Error>> {
        let service_account_key = get_env_var("FIREBASE_SERVICE_ACCOUNT_KEY")
            .ok();

        Ok(Self {
            client: Client::new(),
            project_id,
            service_account_key,
            jwks_cache: HashMap::new(),
            jwks_cache_expiry: Utc::now() - Duration::hours(1),
            access_token_cache: Arc::new(Mutex::new(AccessTokenCache::default())),
        })
    }

    /// Create Firebase Admin client with custom HTTP client
    pub async fn with_client(client: Client) -> Result<Self, Box<dyn std::error::Error>> {
        let project_id = get_env_var("FIREBASE_PROJECT_ID")
            .unwrap_or_else(|_| "default-project".to_string());
        
        let service_account_key = get_env_var("FIREBASE_SERVICE_ACCOUNT_KEY")
            .ok();

        Ok(Self {
            client,
            project_id,
            service_account_key,
            jwks_cache: HashMap::new(),
            jwks_cache_expiry: Utc::now() - Duration::hours(1),
            access_token_cache: Arc::new(Mutex::new(AccessTokenCache::default())),
        })
    }

    /// Create Firebase Admin client for testing
    pub fn create_test_client() -> Self {
        Self {
            client: Client::new(),
            project_id: "test-project".to_string(),
            service_account_key: Some("test-key".to_string()),
            jwks_cache: HashMap::new(),
            jwks_cache_expiry: Utc::now() + Duration::hours(1),
            access_token_cache: Arc::new(Mutex::new(AccessTokenCache::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firebase_admin_creation() {
        let admin = FirebaseAdmin::create_test_client();
        assert_eq!(admin.get_project_id(), "test-project");
        assert!(!admin.is_jwks_cache_expired());
    }

    #[test]
    fn test_auth_config_summary() {
        let admin = FirebaseAdmin::create_test_client();
        let summary = admin.get_auth_config_summary();
        // In test environment, this will depend on actual env vars
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_jwks_cache_management() {
        let mut admin = FirebaseAdmin::create_test_client();
        
        // Initially cache should not be expired (we set it to +1 hour in test client)
        assert!(!admin.is_jwks_cache_expired());
        
        // Add a key to cache
        let mut keys = HashMap::new();
        keys.insert("test-kid".to_string(), FirebasePublicKey {
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            r#use: "sig".to_string(),
            kid: "test-kid".to_string(),
            n: "test-n".to_string(),
            e: "AQAB".to_string(),
        });
        
        admin.update_jwks_cache(keys);
        
        // Should be able to retrieve the key
        let key = admin.get_cached_jwks_key("test-kid");
        assert!(key.is_some());
        assert_eq!(key.unwrap().kid, "test-kid");
    }

    #[tokio::test]
    async fn test_client_factory_methods() {
        let admin1 = FirebaseAdmin::with_project_id("custom-project".to_string()).await.unwrap();
        assert_eq!(admin1.get_project_id(), "custom-project");

        let custom_client = Client::new();
        let admin2 = FirebaseAdmin::with_client(custom_client).await.unwrap();
        assert!(!admin2.get_project_id().is_empty());
    }
}