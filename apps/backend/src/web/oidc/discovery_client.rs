// OIDC Discovery Client
// Fetches and caches OIDC discovery documents from providers at runtime

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
// use serde::Deserialize;
use tokio::sync::RwLock;
use reqwest::Client;
use url::Url;

use crate::core::errors::AppError;
use super::provider_registry::{OIDCDiscoveryDocument, OIDCProviderConfig, OIDCProviderType};

/// Cached discovery document
#[derive(Debug, Clone)]
struct CachedDiscoveryDocument {
    document: OIDCDiscoveryDocument,
    cached_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl CachedDiscoveryDocument {
    fn new(document: OIDCDiscoveryDocument, ttl_seconds: u64) -> Self {
        let now = Utc::now();
        Self {
            document,
            cached_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_seconds as i64),
        }
    }
    
    fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Discovery client configuration
#[derive(Debug, Clone)]
pub struct DiscoveryClientConfig {
    /// HTTP client timeout
    pub request_timeout_seconds: u64,
    /// Default cache TTL for discovery documents
    pub default_cache_ttl_seconds: u64,
    /// Maximum number of cached documents
    pub max_cache_size: usize,
    /// User agent for HTTP requests
    pub user_agent: String,
    /// Enable TLS verification
    pub verify_tls: bool,
}

impl Default for DiscoveryClientConfig {
    fn default() -> Self {
        Self {
            request_timeout_seconds: 10,
            default_cache_ttl_seconds: 3600, // 1 hour
            max_cache_size: 100,
            user_agent: "EPSX-OIDC-Client/1.0".to_string(),
            verify_tls: true,
        }
    }
}

/// Discovery client trait
#[async_trait]
pub trait DiscoveryClientTrait: Send + Sync {
    /// Discover OIDC configuration from issuer URL
    async fn discover(&self, issuer_url: &str) -> Result<OIDCDiscoveryDocument, AppError>;
    
    /// Get cached discovery document
    async fn get_cached(&self, issuer_url: &str) -> Option<OIDCDiscoveryDocument>;
    
    /// Force refresh a discovery document
    async fn refresh(&self, issuer_url: &str) -> Result<OIDCDiscoveryDocument, AppError>;
    
    /// Clear cache for a specific issuer
    async fn clear_cache(&self, issuer_url: &str);
    
    /// Clear all cached documents
    async fn clear_all_cache(&self);
    
    /// Get cache statistics
    async fn get_cache_stats(&self) -> HashMap<String, serde_json::Value>;
}

/// HTTP-based OIDC discovery client
pub struct HttpDiscoveryClient {
    http_client: Client,
    config: DiscoveryClientConfig,
    cache: Arc<RwLock<HashMap<String, CachedDiscoveryDocument>>>,
}

impl HttpDiscoveryClient {
    pub fn new(config: DiscoveryClientConfig) -> Result<Self, AppError> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_seconds))
            .user_agent(&config.user_agent);
            
        if !config.verify_tls {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }
        
        let http_client = client_builder
            .build()
            .map_err(|e| AppError::internal_error(format!("Failed to create HTTP client: {}", e)))?;
            
        Ok(Self {
            http_client,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Construct discovery URL from issuer
    fn discovery_url(&self, issuer_url: &str) -> Result<String, AppError> {
        let base_url = Url::parse(issuer_url)
            .map_err(|e| AppError::validation_error(format!("Invalid issuer URL: {}", e)))?;
            
        let discovery_url = base_url.join("/.well-known/openid_configuration")
            .map_err(|e| AppError::validation_error(format!("Failed to construct discovery URL: {}", e)))?;
            
        Ok(discovery_url.to_string())
    }
    
    /// Fetch discovery document from remote endpoint
    async fn fetch_discovery_document(&self, issuer_url: &str) -> Result<OIDCDiscoveryDocument, AppError> {
        let discovery_url = self.discovery_url(issuer_url)?;
        
        tracing::info!(
            issuer = %issuer_url,
            discovery_url = %discovery_url,
            "Fetching OIDC discovery document"
        );
        
        let response = self.http_client
            .get(&discovery_url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| AppError::external_service_error(
                format!("Failed to fetch discovery document from {}: {}", discovery_url, e)
            ))?;
            
        if !response.status().is_success() {
            return Err(AppError::external_service_error(
                format!("Discovery endpoint returned {}: {}", response.status(), discovery_url)
            ));
        }
        
        let discovery_text = response.text().await
            .map_err(|e| AppError::external_service_error(
                format!("Failed to read discovery response: {}", e)
            ))?;
            
        let discovery_doc: OIDCDiscoveryDocument = serde_json::from_str(&discovery_text)
            .map_err(|e| AppError::external_service_error(
                format!("Failed to parse discovery document: {}", e)
            ))?;
            
        // Validate discovery document
        self.validate_discovery_document(&discovery_doc)?;
        
        tracing::info!(
            issuer = %issuer_url,
            auth_endpoint = %discovery_doc.authorization_endpoint,
            token_endpoint = %discovery_doc.token_endpoint,
            "Successfully fetched OIDC discovery document"
        );
        
        Ok(discovery_doc)
    }
    
    /// Validate discovery document has required fields
    fn validate_discovery_document(&self, doc: &OIDCDiscoveryDocument) -> Result<(), AppError> {
        if doc.issuer.is_empty() {
            return Err(AppError::validation_error("Discovery document missing issuer".to_string()));
        }
        
        if doc.authorization_endpoint.is_empty() {
            return Err(AppError::validation_error("Discovery document missing authorization_endpoint".to_string()));
        }
        
        if doc.token_endpoint.is_empty() {
            return Err(AppError::validation_error("Discovery document missing token_endpoint".to_string()));
        }
        
        if doc.jwks_uri.is_empty() {
            return Err(AppError::validation_error("Discovery document missing jwks_uri".to_string()));
        }
        
        // Validate URLs
        Url::parse(&doc.issuer)
            .map_err(|_| AppError::validation_error("Invalid issuer URL in discovery document".to_string()))?;
        Url::parse(&doc.authorization_endpoint)
            .map_err(|_| AppError::validation_error("Invalid authorization_endpoint URL in discovery document".to_string()))?;
        Url::parse(&doc.token_endpoint)
            .map_err(|_| AppError::validation_error("Invalid token_endpoint URL in discovery document".to_string()))?;
        Url::parse(&doc.jwks_uri)
            .map_err(|_| AppError::validation_error("Invalid jwks_uri URL in discovery document".to_string()))?;
        
        Ok(())
    }
    
    /// Cache discovery document
    async fn cache_document(&self, issuer_url: &str, document: OIDCDiscoveryDocument) {
        let cached_doc = CachedDiscoveryDocument::new(document, self.config.default_cache_ttl_seconds);
        
        let mut cache = self.cache.write().await;
        
        // Enforce cache size limit
        if cache.len() >= self.config.max_cache_size {
            // Remove oldest entry
            if let Some((oldest_key, _)) = cache.iter()
                .min_by_key(|(_, cached)| cached.cached_at)
                .map(|(k, v)| (k.clone(), v.cached_at)) {
                cache.remove(&oldest_key);
            }
        }
        
        cache.insert(issuer_url.to_string(), cached_doc);
        
        tracing::debug!(
            issuer = %issuer_url,
            cache_size = cache.len(),
            "Cached OIDC discovery document"
        );
    }
}

#[async_trait]
impl DiscoveryClientTrait for HttpDiscoveryClient {
    async fn discover(&self, issuer_url: &str) -> Result<OIDCDiscoveryDocument, AppError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(issuer_url) {
                if !cached.is_expired() {
                    tracing::debug!(issuer = %issuer_url, "Using cached discovery document");
                    return Ok(cached.document.clone());
                }
            }
        }
        
        // Fetch fresh document
        let document = self.fetch_discovery_document(issuer_url).await?;
        
        // Cache the document
        self.cache_document(issuer_url, document.clone()).await;
        
        Ok(document)
    }
    
    async fn get_cached(&self, issuer_url: &str) -> Option<OIDCDiscoveryDocument> {
        let cache = self.cache.read().await;
        cache.get(issuer_url)
            .filter(|cached| !cached.is_expired())
            .map(|cached| cached.document.clone())
    }
    
    async fn refresh(&self, issuer_url: &str) -> Result<OIDCDiscoveryDocument, AppError> {
        // Clear cache entry first
        {
            let mut cache = self.cache.write().await;
            cache.remove(issuer_url);
        }
        
        // Fetch fresh document
        self.discover(issuer_url).await
    }
    
    async fn clear_cache(&self, issuer_url: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(issuer_url);
        tracing::debug!(issuer = %issuer_url, "Cleared discovery cache entry");
    }
    
    async fn clear_all_cache(&self) {
        let mut cache = self.cache.write().await;
        let count = cache.len();
        cache.clear();
        tracing::info!(cleared_entries = count, "Cleared all discovery cache entries");
    }
    
    async fn get_cache_stats(&self) -> HashMap<String, serde_json::Value> {
        let cache = self.cache.read().await;
        let mut stats = HashMap::new();
        
        stats.insert("total_entries".to_string(), serde_json::Value::Number(cache.len().into()));
        stats.insert("max_cache_size".to_string(), serde_json::Value::Number(self.config.max_cache_size.into()));
        
        let expired_count = cache.values().filter(|cached| cached.is_expired()).count();
        stats.insert("expired_entries".to_string(), serde_json::Value::Number(expired_count.into()));
        
        let active_count = cache.len() - expired_count;
        stats.insert("active_entries".to_string(), serde_json::Value::Number(active_count.into()));
        
        // Add per-issuer stats
        let mut issuer_stats = HashMap::new();
        for (issuer, cached) in cache.iter() {
            issuer_stats.insert(issuer.clone(), serde_json::json!({
                "cached_at": cached.cached_at,
                "expires_at": cached.expires_at,
                "is_expired": cached.is_expired(),
                "issuer": cached.document.issuer
            }));
        }
        stats.insert("issuers".to_string(), serde_json::Value::Object(
            issuer_stats.into_iter()
                .map(|(k, v)| (k, v))
                .collect()
        ));
        
        stats
    }
}

/// Enhanced provider configurator that uses discovery client
pub struct EnhancedProviderConfigurator {
    discovery_client: Arc<dyn DiscoveryClientTrait>,
}

impl EnhancedProviderConfigurator {
    pub fn new(discovery_client: Arc<dyn DiscoveryClientTrait>) -> Self {
        Self { discovery_client }
    }
    
    /// Auto-configure provider from issuer URL
    pub async fn configure_provider_from_issuer(
        &self,
        issuer_url: &str,
        client_id: String,
        client_secret: String,
        tenant_id: String,
        display_name: String,
        email_domains: Vec<String>,
    ) -> Result<OIDCProviderConfig, AppError> {
        // Fetch discovery document
        let discovery = self.discovery_client.discover(issuer_url).await?;
        
        // Determine provider type from issuer
        let provider_type = self.detect_provider_type(issuer_url);
        
        // Create provider config
        let mut config = OIDCProviderConfig::new(
            provider_type,
            tenant_id,
            display_name,
            issuer_url.to_string(),
            client_id,
            client_secret,
        );
        
        // Update config from discovery
        config.update_from_discovery(&discovery);
        config.email_domains = email_domains;
        
        tracing::info!(
            provider_id = %config.provider_id,
            issuer = %issuer_url,
            provider_type = %config.provider_type,
            "Auto-configured OIDC provider from discovery"
        );
        
        Ok(config)
    }
    
    /// Detect provider type from issuer URL
    fn detect_provider_type(&self, issuer_url: &str) -> OIDCProviderType {
        if issuer_url.contains("accounts.google.com") || issuer_url.contains("googleapis.com") {
            OIDCProviderType::Google
        } else if issuer_url.contains("login.microsoftonline.com") || issuer_url.contains("microsoft") {
            OIDCProviderType::Microsoft
        } else if issuer_url.contains("auth0.com") {
            OIDCProviderType::Auth0
        } else if issuer_url.contains("enterprise") || issuer_url.contains("corporate") {
            // Extract company name from URL for enterprise providers
            let domain = Url::parse(issuer_url)
                .ok()
                .and_then(|url| url.host_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown".to_string());
            OIDCProviderType::Enterprise(domain)
        } else {
            OIDCProviderType::Generic
        }
    }
    
    /// Validate and refresh provider configuration
    pub async fn refresh_provider_config(&self, config: &mut OIDCProviderConfig) -> Result<(), AppError> {
        let discovery = self.discovery_client.refresh(&config.issuer).await?;
        config.update_from_discovery(&discovery);
        
        tracing::info!(
            provider_id = %config.provider_id,
            issuer = %config.issuer,
            "Refreshed provider configuration"
        );
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_discovery_client_creation() {
        let config = DiscoveryClientConfig::default();
        let client = HttpDiscoveryClient::new(config);
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_discovery_url_construction() {
        let config = DiscoveryClientConfig::default();
        let client = HttpDiscoveryClient::new(config).unwrap();
        
        let url = client.discovery_url("https://accounts.google.com").unwrap();
        assert_eq!(url, "https://accounts.google.com/.well-known/openid_configuration");
        
        let url = client.discovery_url("https://login.microsoftonline.com/common/v2.0").unwrap();
        assert_eq!(url, "https://login.microsoftonline.com/common/v2.0/.well-known/openid_configuration");
    }
    
    #[tokio::test]
    async fn test_cache_operations() {
        let config = DiscoveryClientConfig::default();
        let client = HttpDiscoveryClient::new(config).unwrap();
        
        // Initially empty
        let cached = client.get_cached("https://example.com").await;
        assert!(cached.is_none());
        
        // Mock discovery document
        let discovery = OIDCDiscoveryDocument {
            issuer: "https://example.com".to_string(),
            authorization_endpoint: "https://example.com/auth".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            userinfo_endpoint: "https://example.com/userinfo".to_string(),
            jwks_uri: "https://example.com/jwks".to_string(),
            scopes_supported: vec!["openid".to_string()],
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec!["authorization_code".to_string()],
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec!["RS256".to_string()],
            claims_supported: vec!["sub".to_string(), "email".to_string()],
            code_challenge_methods_supported: vec!["S256".to_string()],
        };
        
        // Cache the document
        client.cache_document("https://example.com", discovery.clone()).await;
        
        // Should be cached now
        let cached = client.get_cached("https://example.com").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().issuer, "https://example.com");
        
        // Clear cache
        client.clear_cache("https://example.com").await;
        let cached = client.get_cached("https://example.com").await;
        assert!(cached.is_none());
    }
    
    #[test]
    fn test_provider_type_detection() {
        let discovery_client = Arc::new(HttpDiscoveryClient::new(DiscoveryClientConfig::default()).unwrap());
        let configurator = EnhancedProviderConfigurator::new(discovery_client);
        
        assert_eq!(configurator.detect_provider_type("https://accounts.google.com"), OIDCProviderType::Google);
        assert_eq!(configurator.detect_provider_type("https://login.microsoftonline.com/common"), OIDCProviderType::Microsoft);
        assert_eq!(configurator.detect_provider_type("https://dev-example.auth0.com"), OIDCProviderType::Auth0);
        assert_eq!(configurator.detect_provider_type("https://sso.generic.com"), OIDCProviderType::Generic);
    }
}