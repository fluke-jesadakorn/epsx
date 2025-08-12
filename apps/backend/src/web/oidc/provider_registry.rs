// Dynamic Multi-Provider OIDC Registry
// Manages multiple OIDC providers with tenant-aware configuration

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::errors::AppError;

/// OIDC Provider Types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OIDCProviderType {
    /// Google Workspace / Google OAuth2
    Google,
    /// Microsoft Azure AD / Entra ID
    Microsoft,
    /// Auth0
    Auth0,
    /// Generic OpenID Connect
    Generic,
    /// Custom Enterprise OIDC
    Enterprise(String),
}

impl std::fmt::Display for OIDCProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OIDCProviderType::Google => write!(f, "google"),
            OIDCProviderType::Microsoft => write!(f, "microsoft"),
            OIDCProviderType::Auth0 => write!(f, "auth0"),
            OIDCProviderType::Generic => write!(f, "generic"),
            OIDCProviderType::Enterprise(name) => write!(f, "enterprise_{}", name),
        }
    }
}

/// OIDC Provider Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OIDCProviderConfig {
    /// Unique provider ID
    pub provider_id: String,
    /// Provider type
    pub provider_type: OIDCProviderType,
    /// Display name for UI
    pub display_name: String,
    /// Tenant/organization identifier
    pub tenant_id: String,
    /// Email domains associated with this provider
    pub email_domains: Vec<String>,
    
    /// OIDC Configuration
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub client_id: String,
    pub client_secret: String,
    
    /// Supported features
    pub supported_scopes: Vec<String>,
    pub supported_response_types: Vec<String>,
    pub supports_pkce: bool,
    pub supports_refresh_token: bool,
    
    /// Timeouts and limits
    pub discovery_cache_ttl_seconds: u64,
    pub token_ttl_seconds: u64,
    pub refresh_token_ttl_seconds: u64,
    
    /// Provider-specific settings
    pub extra_params: HashMap<String, String>,
    
    /// Health and monitoring
    pub is_active: bool,
    pub priority: u8, // Higher number = higher priority
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OIDCProviderConfig {
    pub fn new(
        provider_type: OIDCProviderType,
        tenant_id: String,
        display_name: String,
        issuer: String,
        client_id: String,
        client_secret: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            provider_id: Uuid::new_v4().to_string(),
            provider_type,
            display_name,
            tenant_id,
            email_domains: Vec::new(),
            issuer,
            authorization_endpoint: String::new(),
            token_endpoint: String::new(),
            userinfo_endpoint: String::new(),
            jwks_uri: String::new(),
            client_id,
            client_secret,
            supported_scopes: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
            ],
            supported_response_types: vec!["code".to_string()],
            supports_pkce: true,
            supports_refresh_token: true,
            discovery_cache_ttl_seconds: 3600, // 1 hour
            token_ttl_seconds: 900,             // 15 minutes
            refresh_token_ttl_seconds: 2592000, // 30 days
            extra_params: HashMap::new(),
            is_active: true,
            priority: 50, // Default priority
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Update provider from OIDC discovery document
    pub fn update_from_discovery(&mut self, discovery: &OIDCDiscoveryDocument) {
        self.authorization_endpoint = discovery.authorization_endpoint.clone();
        self.token_endpoint = discovery.token_endpoint.clone();
        self.userinfo_endpoint = discovery.userinfo_endpoint.clone();
        self.jwks_uri = discovery.jwks_uri.clone();
        self.supported_scopes = discovery.scopes_supported.clone();
        self.supported_response_types = discovery.response_types_supported.clone();
        self.updated_at = Utc::now();
        
        // Check PKCE support
        self.supports_pkce = discovery.code_challenge_methods_supported
            .iter()
            .any(|method| method == "S256");
    }
}

/// OIDC Discovery Document (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OIDCDiscoveryDocument {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub claims_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
}

/// Tenant Resolution Result
#[derive(Debug, Clone)]
pub struct TenantResolution {
    pub tenant_id: String,
    pub provider_config: OIDCProviderConfig,
    pub fallback_providers: Vec<OIDCProviderConfig>,
}

/// Provider Registry Trait
#[async_trait]
pub trait ProviderRegistryTrait: Send + Sync {
    async fn register_provider(&self, config: OIDCProviderConfig) -> Result<(), AppError>;
    async fn get_provider(&self, provider_id: &str) -> Result<Option<OIDCProviderConfig>, AppError>;
    async fn get_providers_by_tenant(&self, tenant_id: &str) -> Result<Vec<OIDCProviderConfig>, AppError>;
    async fn resolve_provider_by_email(&self, email: &str) -> Result<Option<TenantResolution>, AppError>;
    async fn update_provider(&self, config: OIDCProviderConfig) -> Result<(), AppError>;
    async fn deactivate_provider(&self, provider_id: &str) -> Result<(), AppError>;
    async fn list_active_providers(&self) -> Result<Vec<OIDCProviderConfig>, AppError>;
}

/// In-Memory Provider Registry (for development/testing)
pub struct InMemoryProviderRegistry {
    providers: Arc<RwLock<HashMap<String, OIDCProviderConfig>>>,
    tenant_providers: Arc<RwLock<HashMap<String, Vec<String>>>>, // tenant_id -> provider_ids
    domain_mapping: Arc<RwLock<HashMap<String, String>>>, // email_domain -> provider_id
}

impl InMemoryProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            tenant_providers: Arc::new(RwLock::new(HashMap::new())),
            domain_mapping: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize with default providers
    pub async fn with_defaults(self) -> Result<Self, AppError> {
        // Add Google provider
        let google_config = OIDCProviderConfig::new(
            OIDCProviderType::Google,
            "default".to_string(),
            "Google".to_string(),
            "https://accounts.google.com".to_string(),
            std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
        );
        
        // Add Microsoft provider
        let microsoft_config = OIDCProviderConfig::new(
            OIDCProviderType::Microsoft,
            "default".to_string(),
            "Microsoft".to_string(),
            "https://login.microsoftonline.com/common/v2.0".to_string(),
            std::env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
            std::env::var("MICROSOFT_CLIENT_SECRET").unwrap_or_default(),
        );
        
        self.register_provider(google_config).await?;
        self.register_provider(microsoft_config).await?;
        
        Ok(self)
    }
}

#[async_trait]
impl ProviderRegistryTrait for InMemoryProviderRegistry {
    async fn register_provider(&self, mut config: OIDCProviderConfig) -> Result<(), AppError> {
        config.updated_at = Utc::now();
        let provider_id = config.provider_id.clone();
        let tenant_id = config.tenant_id.clone();
        
        // Update providers map
        {
            let mut providers = self.providers.write().await;
            providers.insert(provider_id.clone(), config.clone());
        }
        
        // Update tenant mapping
        {
            let mut tenant_providers = self.tenant_providers.write().await;
            tenant_providers
                .entry(tenant_id)
                .or_insert_with(Vec::new)
                .push(provider_id.clone());
        }
        
        // Update domain mapping
        {
            let mut domain_mapping = self.domain_mapping.write().await;
            for domain in &config.email_domains {
                domain_mapping.insert(domain.clone(), provider_id.clone());
            }
        }
        
        tracing::info!(
            provider_id = %provider_id,
            provider_type = %config.provider_type,
            tenant_id = %config.tenant_id,
            "Registered OIDC provider"
        );
        
        Ok(())
    }
    
    async fn get_provider(&self, provider_id: &str) -> Result<Option<OIDCProviderConfig>, AppError> {
        let providers = self.providers.read().await;
        Ok(providers.get(provider_id).cloned())
    }
    
    async fn get_providers_by_tenant(&self, tenant_id: &str) -> Result<Vec<OIDCProviderConfig>, AppError> {
        let tenant_providers = self.tenant_providers.read().await;
        let providers = self.providers.read().await;
        
        if let Some(provider_ids) = tenant_providers.get(tenant_id) {
            let mut configs = Vec::new();
            for provider_id in provider_ids {
                if let Some(config) = providers.get(provider_id) {
                    if config.is_active {
                        configs.push(config.clone());
                    }
                }
            }
            // Sort by priority (highest first)
            configs.sort_by(|a, b| b.priority.cmp(&a.priority));
            Ok(configs)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn resolve_provider_by_email(&self, email: &str) -> Result<Option<TenantResolution>, AppError> {
        let domain = email.split('@').nth(1).ok_or_else(|| {
            AppError::ValidationError("Invalid email format".to_string())
        })?;
        
        let domain_mapping = self.domain_mapping.read().await;
        let providers = self.providers.read().await;
        
        if let Some(provider_id) = domain_mapping.get(domain) {
            if let Some(config) = providers.get(provider_id) {
                if config.is_active {
                    let tenant_id = config.tenant_id.clone();
                    let provider_config = config.clone();
                    let provider_id_for_filter = config.provider_id.clone();
                    
                    // Get fallback providers for this tenant
                    drop(domain_mapping);
                    drop(providers);
                    
                    let fallback_providers = self.get_providers_by_tenant(&tenant_id).await?
                        .into_iter()
                        .filter(|p| p.provider_id != provider_id_for_filter)
                        .collect();
                    
                    return Ok(Some(TenantResolution {
                        tenant_id,
                        provider_config,
                        fallback_providers,
                    }));
                }
            }
        }
        
        Ok(None)
    }
    
    async fn update_provider(&self, config: OIDCProviderConfig) -> Result<(), AppError> {
        let provider_id = config.provider_id.clone();
        
        let mut providers = self.providers.write().await;
        if providers.contains_key(&provider_id) {
            providers.insert(provider_id.clone(), config);
            tracing::info!(provider_id = %provider_id, "Updated OIDC provider");
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Provider {} not found", provider_id)))
        }
    }
    
    async fn deactivate_provider(&self, provider_id: &str) -> Result<(), AppError> {
        let mut providers = self.providers.write().await;
        if let Some(config) = providers.get_mut(provider_id) {
            config.is_active = false;
            config.updated_at = Utc::now();
            tracing::info!(provider_id = %provider_id, "Deactivated OIDC provider");
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Provider {} not found", provider_id)))
        }
    }
    
    async fn list_active_providers(&self) -> Result<Vec<OIDCProviderConfig>, AppError> {
        let providers = self.providers.read().await;
        let mut active_providers: Vec<_> = providers
            .values()
            .filter(|config| config.is_active)
            .cloned()
            .collect();
        
        // Sort by priority (highest first)
        active_providers.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(active_providers)
    }
}

/// Database-backed Provider Registry (production implementation)
pub struct DatabaseProviderRegistry {
    // TODO: Add database connection pool
    // db: Arc<PgPool>,
}

#[async_trait]
impl ProviderRegistryTrait for DatabaseProviderRegistry {
    async fn register_provider(&self, _config: OIDCProviderConfig) -> Result<(), AppError> {
        // TODO: Implement database storage
        Err(AppError::InternalError("Database provider registry not implemented yet".to_string()))
    }
    
    async fn get_provider(&self, _provider_id: &str) -> Result<Option<OIDCProviderConfig>, AppError> {
        // TODO: Implement database retrieval
        Err(AppError::InternalError("Database provider registry not implemented yet".to_string()))
    }
    
    async fn get_providers_by_tenant(&self, _tenant_id: &str) -> Result<Vec<OIDCProviderConfig>, AppError> {
        // TODO: Implement database query
        Err(AppError::InternalError("Database provider registry not implemented yet".to_string()))
    }
    
    async fn resolve_provider_by_email(&self, _email: &str) -> Result<Option<TenantResolution>, AppError> {
        // TODO: Implement database lookup
        Err(AppError::InternalError("Database provider registry not implemented yet".to_string()))
    }
    
    async fn update_provider(&self, _config: OIDCProviderConfig) -> Result<(), AppError> {
        // TODO: Implement database update
        Err(AppError::InternalError("Database provider registry not implemented yet".to_string()))
    }
    
    async fn deactivate_provider(&self, _provider_id: &str) -> Result<(), AppError> {
        // TODO: Implement database deactivation
        Err(AppError::InternalError("Database provider registry not implemented yet".to_string()))
    }
    
    async fn list_active_providers(&self) -> Result<Vec<OIDCProviderConfig>, AppError> {
        // TODO: Implement database query
        Err(AppError::InternalError("Database provider registry not implemented yet".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_provider_registry_creation() {
        let registry = InMemoryProviderRegistry::new();
        let providers = registry.list_active_providers().await.unwrap();
        assert!(providers.is_empty());
    }
    
    #[tokio::test]
    async fn test_provider_registration() {
        let registry = InMemoryProviderRegistry::new();
        
        let config = OIDCProviderConfig::new(
            OIDCProviderType::Google,
            "test-tenant".to_string(),
            "Test Google".to_string(),
            "https://accounts.google.com".to_string(),
            "client_id".to_string(),
            "client_secret".to_string(),
        );
        
        registry.register_provider(config.clone()).await.unwrap();
        
        let retrieved = registry.get_provider(&config.provider_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().provider_id, config.provider_id);
    }
    
    #[tokio::test]
    async fn test_tenant_resolution() {
        let registry = InMemoryProviderRegistry::new();
        
        let mut config = OIDCProviderConfig::new(
            OIDCProviderType::Google,
            "test-tenant".to_string(),
            "Test Google".to_string(),
            "https://accounts.google.com".to_string(),
            "client_id".to_string(),
            "client_secret".to_string(),
        );
        config.email_domains = vec!["example.com".to_string()];
        
        registry.register_provider(config).await.unwrap();
        
        let resolution = registry.resolve_provider_by_email("user@example.com").await.unwrap();
        assert!(resolution.is_some());
        
        let resolution = resolution.unwrap();
        assert_eq!(resolution.tenant_id, "test-tenant");
        assert_eq!(resolution.provider_config.provider_type, OIDCProviderType::Google);
    }
}