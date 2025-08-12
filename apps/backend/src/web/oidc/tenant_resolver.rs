// Tenant Resolver
// Maps email domains to OIDC providers and handles complex tenant resolution

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use regex::Regex;

use crate::core::errors::AppError;
use super::provider_registry::{ProviderRegistryTrait, TenantResolution};

/// Domain matching strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DomainMatchStrategy {
    /// Exact domain match (user@company.com matches company.com)
    Exact,
    /// Subdomain wildcard match (*.company.com matches any subdomain)
    Subdomain,
    /// Regex pattern match
    Regex(String),
}

/// Tenant mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMapping {
    /// Unique mapping ID
    pub mapping_id: String,
    /// Tenant identifier
    pub tenant_id: String,
    /// Email domain pattern
    pub domain_pattern: String,
    /// Matching strategy
    pub match_strategy: DomainMatchStrategy,
    /// Primary provider ID for this domain
    pub provider_id: String,
    /// Fallback provider IDs (ordered by preference)
    pub fallback_provider_ids: Vec<String>,
    /// Priority (higher = more preferred)
    pub priority: u8,
    /// Whether this mapping is active
    pub is_active: bool,
    /// Auto-provision users from this domain
    pub auto_provision: bool,
    /// Default role for auto-provisioned users
    pub default_role: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Creation and update timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TenantMapping {
    pub fn new(
        tenant_id: String,
        domain_pattern: String,
        match_strategy: DomainMatchStrategy,
        provider_id: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            mapping_id: uuid::Uuid::new_v4().to_string(),
            tenant_id,
            domain_pattern,
            match_strategy,
            provider_id,
            fallback_provider_ids: Vec::new(),
            priority: 50, // Default priority
            is_active: true,
            auto_provision: true,
            default_role: "user".to_string(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if email domain matches this mapping
    pub fn matches_email(&self, email: &str) -> bool {
        if !self.is_active {
            return false;
        }
        
        let domain = match email.split('@').nth(1) {
            Some(d) => d.to_lowercase(),
            None => return false,
        };
        
        match &self.match_strategy {
            DomainMatchStrategy::Exact => {
                domain == self.domain_pattern.to_lowercase()
            }
            DomainMatchStrategy::Subdomain => {
                let pattern = self.domain_pattern.to_lowercase();
                if pattern.starts_with("*.") {
                    let base_domain = &pattern[2..]; // Remove "*."
                    domain == base_domain || domain.ends_with(&format!(".{}", base_domain))
                } else {
                    domain == pattern
                }
            }
            DomainMatchStrategy::Regex(pattern) => {
                match Regex::new(pattern) {
                    Ok(regex) => regex.is_match(&domain),
                    Err(_) => false, // Invalid regex, no match
                }
            }
        }
    }
}

/// Enhanced tenant resolution result
#[derive(Debug, Clone)]
pub struct EnhancedTenantResolution {
    /// Basic tenant resolution info
    pub tenant_resolution: TenantResolution,
    /// Matching tenant mapping
    pub mapping: TenantMapping,
    /// Whether user should be auto-provisioned
    pub auto_provision: bool,
    /// Default role for new user
    pub default_role: String,
    /// Additional resolution metadata
    pub metadata: HashMap<String, String>,
}

/// Tenant resolver trait
#[async_trait]
pub trait TenantResolverTrait: Send + Sync {
    /// Register a new tenant mapping
    async fn register_mapping(&self, mapping: TenantMapping) -> Result<(), AppError>;
    
    /// Resolve provider for email address
    async fn resolve_tenant_for_email(&self, email: &str) -> Result<Option<EnhancedTenantResolution>, AppError>;
    
    /// Get all mappings for a tenant
    async fn get_tenant_mappings(&self, tenant_id: &str) -> Result<Vec<TenantMapping>, AppError>;
    
    /// Update an existing mapping
    async fn update_mapping(&self, mapping: TenantMapping) -> Result<(), AppError>;
    
    /// Deactivate a mapping
    async fn deactivate_mapping(&self, mapping_id: &str) -> Result<(), AppError>;
    
    /// List all active mappings
    async fn list_active_mappings(&self) -> Result<Vec<TenantMapping>, AppError>;
    
    /// Get resolution statistics
    async fn get_resolution_stats(&self) -> Result<HashMap<String, serde_json::Value>, AppError>;
}

/// In-memory tenant resolver implementation
pub struct InMemoryTenantResolver {
    provider_registry: Arc<dyn ProviderRegistryTrait>,
    mappings: Arc<RwLock<HashMap<String, TenantMapping>>>, // mapping_id -> mapping
    domain_index: Arc<RwLock<HashMap<String, Vec<String>>>>, // domain -> mapping_ids
}

impl InMemoryTenantResolver {
    pub fn new(provider_registry: Arc<dyn ProviderRegistryTrait>) -> Self {
        Self {
            provider_registry,
            mappings: Arc::new(RwLock::new(HashMap::new())),
            domain_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize with default domain mappings
    pub async fn with_default_mappings(self) -> Result<Self, AppError> {
        // Add common domain mappings
        let google_mapping = TenantMapping::new(
            "default".to_string(),
            "gmail.com".to_string(),
            DomainMatchStrategy::Exact,
            "google-default".to_string(),
        );
        
        let microsoft_mapping = TenantMapping::new(
            "default".to_string(),
            "outlook.com".to_string(),
            DomainMatchStrategy::Exact,
            "microsoft-default".to_string(),
        );
        
        self.register_mapping(google_mapping).await?;
        self.register_mapping(microsoft_mapping).await?;
        
        Ok(self)
    }
    
    /// Update domain index when mapping changes
    async fn update_domain_index(&self, mapping: &TenantMapping) {
        let mut domain_index = self.domain_index.write().await;
        
        match &mapping.match_strategy {
            DomainMatchStrategy::Exact => {
                let domain = mapping.domain_pattern.to_lowercase();
                domain_index
                    .entry(domain)
                    .or_insert_with(Vec::new)
                    .push(mapping.mapping_id.clone());
            }
            DomainMatchStrategy::Subdomain => {
                let pattern = mapping.domain_pattern.to_lowercase();
                let base_domain = if pattern.starts_with("*.") {
                    pattern[2..].to_string()
                } else {
                    pattern
                };
                domain_index
                    .entry(base_domain)
                    .or_insert_with(Vec::new)
                    .push(mapping.mapping_id.clone());
            }
            DomainMatchStrategy::Regex(_) => {
                // For regex patterns, we'll need to check all mappings
                // Store under special key for regex patterns
                domain_index
                    .entry("__regex_patterns__".to_string())
                    .or_insert_with(Vec::new)
                    .push(mapping.mapping_id.clone());
            }
        }
    }
    
    /// Find matching mappings for an email domain
    async fn find_matching_mappings(&self, email: &str) -> Vec<TenantMapping> {
        let domain = match email.split('@').nth(1) {
            Some(d) => d.to_lowercase(),
            None => return Vec::new(),
        };
        
        let mut matching_mappings = Vec::new();
        let mappings = self.mappings.read().await;
        let domain_index = self.domain_index.read().await;
        
        // Check exact domain matches
        if let Some(mapping_ids) = domain_index.get(&domain) {
            for mapping_id in mapping_ids {
                if let Some(mapping) = mappings.get(mapping_id) {
                    if mapping.matches_email(email) {
                        matching_mappings.push(mapping.clone());
                    }
                }
            }
        }
        
        // Check wildcard subdomain matches
        let domain_parts: Vec<&str> = domain.split('.').collect();
        for i in 1..domain_parts.len() {
            let parent_domain = domain_parts[i..].join(".");
            if let Some(mapping_ids) = domain_index.get(&parent_domain) {
                for mapping_id in mapping_ids {
                    if let Some(mapping) = mappings.get(mapping_id) {
                        if mapping.matches_email(email) {
                            matching_mappings.push(mapping.clone());
                        }
                    }
                }
            }
        }
        
        // Check regex patterns
        if let Some(regex_mapping_ids) = domain_index.get("__regex_patterns__") {
            for mapping_id in regex_mapping_ids {
                if let Some(mapping) = mappings.get(mapping_id) {
                    if mapping.matches_email(email) {
                        matching_mappings.push(mapping.clone());
                    }
                }
            }
        }
        
        // Sort by priority (highest first)
        matching_mappings.sort_by(|a, b| b.priority.cmp(&a.priority));
        matching_mappings
    }
}

#[async_trait]
impl TenantResolverTrait for InMemoryTenantResolver {
    async fn register_mapping(&self, mapping: TenantMapping) -> Result<(), AppError> {
        let mapping_id = mapping.mapping_id.clone();
        
        // Update mappings
        {
            let mut mappings = self.mappings.write().await;
            mappings.insert(mapping_id.clone(), mapping.clone());
        }
        
        // Update domain index
        self.update_domain_index(&mapping).await;
        
        tracing::info!(
            mapping_id = %mapping_id,
            tenant_id = %mapping.tenant_id,
            domain_pattern = %mapping.domain_pattern,
            provider_id = %mapping.provider_id,
            "Registered tenant mapping"
        );
        
        Ok(())
    }
    
    async fn resolve_tenant_for_email(&self, email: &str) -> Result<Option<EnhancedTenantResolution>, AppError> {
        let matching_mappings = self.find_matching_mappings(email).await;
        
        if matching_mappings.is_empty() {
            tracing::debug!(email = %email, "No tenant mapping found for email");
            return Ok(None);
        }
        
        // Use the highest priority mapping
        let primary_mapping = &matching_mappings[0];
        
        // Get provider configuration
        let provider_config = match self.provider_registry.get_provider(&primary_mapping.provider_id).await? {
            Some(config) if config.is_active => config,
            _ => {
                tracing::warn!(
                    provider_id = %primary_mapping.provider_id,
                    email = %email,
                    "Primary provider not found or inactive"
                );
                
                // Try fallback providers
                for fallback_id in &primary_mapping.fallback_provider_ids {
                    if let Some(config) = self.provider_registry.get_provider(fallback_id).await? {
                        if config.is_active {
                            tracing::info!(
                                fallback_provider_id = %fallback_id,
                                email = %email,
                                "Using fallback provider"
                            );
                            break;
                        }
                    }
                }
                
                return Err(AppError::NotFound(format!(
                    "No active provider found for email domain: {}", 
                    email.split('@').nth(1).unwrap_or("unknown")
                )));
            }
        };
        
        // Get all providers for the tenant as fallbacks
        let tenant_providers = self.provider_registry
            .get_providers_by_tenant(&primary_mapping.tenant_id)
            .await?;
        
        let fallback_providers: Vec<_> = tenant_providers
            .into_iter()
            .filter(|p| p.provider_id != provider_config.provider_id)
            .collect();
        
        let tenant_resolution = TenantResolution {
            tenant_id: primary_mapping.tenant_id.clone(),
            provider_config,
            fallback_providers,
        };
        
        let enhanced_resolution = EnhancedTenantResolution {
            tenant_resolution,
            mapping: primary_mapping.clone(),
            auto_provision: primary_mapping.auto_provision,
            default_role: primary_mapping.default_role.clone(),
            metadata: primary_mapping.metadata.clone(),
        };
        
        tracing::info!(
            email = %email,
            tenant_id = %primary_mapping.tenant_id,
            provider_id = %primary_mapping.provider_id,
            auto_provision = primary_mapping.auto_provision,
            "Resolved tenant for email"
        );
        
        Ok(Some(enhanced_resolution))
    }
    
    async fn get_tenant_mappings(&self, tenant_id: &str) -> Result<Vec<TenantMapping>, AppError> {
        let mappings = self.mappings.read().await;
        let tenant_mappings: Vec<_> = mappings
            .values()
            .filter(|mapping| mapping.tenant_id == tenant_id && mapping.is_active)
            .cloned()
            .collect();
        
        Ok(tenant_mappings)
    }
    
    async fn update_mapping(&self, mapping: TenantMapping) -> Result<(), AppError> {
        let mapping_id = mapping.mapping_id.clone();
        
        // Check if mapping exists
        {
            let mappings = self.mappings.read().await;
            if !mappings.contains_key(&mapping_id) {
                return Err(AppError::NotFound(format!("Mapping {} not found", mapping_id)));
            }
        }
        
        // Update mapping
        {
            let mut mappings = self.mappings.write().await;
            mappings.insert(mapping_id.clone(), mapping.clone());
        }
        
        // Update domain index
        self.update_domain_index(&mapping).await;
        
        tracing::info!(mapping_id = %mapping_id, "Updated tenant mapping");
        Ok(())
    }
    
    async fn deactivate_mapping(&self, mapping_id: &str) -> Result<(), AppError> {
        let mut mappings = self.mappings.write().await;
        if let Some(mapping) = mappings.get_mut(mapping_id) {
            mapping.is_active = false;
            mapping.updated_at = Utc::now();
            tracing::info!(mapping_id = %mapping_id, "Deactivated tenant mapping");
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Mapping {} not found", mapping_id)))
        }
    }
    
    async fn list_active_mappings(&self) -> Result<Vec<TenantMapping>, AppError> {
        let mappings = self.mappings.read().await;
        let mut active_mappings: Vec<_> = mappings
            .values()
            .filter(|mapping| mapping.is_active)
            .cloned()
            .collect();
        
        // Sort by priority (highest first)
        active_mappings.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(active_mappings)
    }
    
    async fn get_resolution_stats(&self) -> Result<HashMap<String, serde_json::Value>, AppError> {
        let mappings = self.mappings.read().await;
        let domain_index = self.domain_index.read().await;
        
        let mut stats = HashMap::new();
        
        stats.insert("total_mappings".to_string(), serde_json::Value::Number(mappings.len().into()));
        
        let active_count = mappings.values().filter(|m| m.is_active).count();
        stats.insert("active_mappings".to_string(), serde_json::Value::Number(active_count.into()));
        
        let inactive_count = mappings.len() - active_count;
        stats.insert("inactive_mappings".to_string(), serde_json::Value::Number(inactive_count.into()));
        
        stats.insert("indexed_domains".to_string(), serde_json::Value::Number(domain_index.len().into()));
        
        // Strategy breakdown
        let mut strategy_counts = HashMap::new();
        for mapping in mappings.values() {
            let strategy_name = match &mapping.match_strategy {
                DomainMatchStrategy::Exact => "exact",
                DomainMatchStrategy::Subdomain => "subdomain",
                DomainMatchStrategy::Regex(_) => "regex",
            };
            *strategy_counts.entry(strategy_name).or_insert(0) += 1;
        }
        stats.insert("strategy_breakdown".to_string(), serde_json::json!(strategy_counts));
        
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::oidc::provider_registry::InMemoryProviderRegistry;
    
    #[tokio::test]
    async fn test_tenant_mapping_creation() {
        let mapping = TenantMapping::new(
            "test-tenant".to_string(),
            "example.com".to_string(),
            DomainMatchStrategy::Exact,
            "provider-1".to_string(),
        );
        
        assert_eq!(mapping.tenant_id, "test-tenant");
        assert_eq!(mapping.domain_pattern, "example.com");
        assert_eq!(mapping.provider_id, "provider-1");
        assert!(mapping.is_active);
    }
    
    #[tokio::test]
    async fn test_domain_matching() {
        let exact_mapping = TenantMapping::new(
            "tenant".to_string(),
            "example.com".to_string(),
            DomainMatchStrategy::Exact,
            "provider".to_string(),
        );
        
        assert!(exact_mapping.matches_email("user@example.com"));
        assert!(!exact_mapping.matches_email("user@sub.example.com"));
        assert!(!exact_mapping.matches_email("user@other.com"));
        
        let subdomain_mapping = TenantMapping::new(
            "tenant".to_string(),
            "*.example.com".to_string(),
            DomainMatchStrategy::Subdomain,
            "provider".to_string(),
        );
        
        assert!(subdomain_mapping.matches_email("user@example.com"));
        assert!(subdomain_mapping.matches_email("user@sub.example.com"));
        assert!(subdomain_mapping.matches_email("user@deep.sub.example.com"));
        assert!(!subdomain_mapping.matches_email("user@other.com"));
    }
    
    #[tokio::test]
    async fn test_tenant_resolver() {
        let provider_registry = Arc::new(InMemoryProviderRegistry::new());
        let resolver = InMemoryTenantResolver::new(provider_registry);
        
        let mapping = TenantMapping::new(
            "test-tenant".to_string(),
            "example.com".to_string(),
            DomainMatchStrategy::Exact,
            "provider-1".to_string(),
        );
        
        resolver.register_mapping(mapping).await.unwrap();
        
        let mappings = resolver.get_tenant_mappings("test-tenant").await.unwrap();
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].domain_pattern, "example.com");
    }
}