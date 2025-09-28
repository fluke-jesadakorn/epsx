// Enhanced Container - Web3-first service container
// Provides comprehensive Web3 services with proper dependency injection

use std::sync::Arc;
use sqlx::PgPool;
use crate::infrastructure::cache::Cache;
use crate::infrastructure::adapters::repositories::{
    wallet_user_repository_adapter::WalletUserRepositoryAdapter,
};
use crate::infrastructure::adapters::services::{
    web3_permission_service_adapter::{Web3PermissionServiceAdapter, BlockchainConfig},
};
use crate::domain::user_management::{
    WalletPermissionService,
    WalletUserRepositoryPort,
    WalletUserAnalyticsPort,
};
use crate::auth::unified_web3_auth_service::UnifiedWeb3AuthService;
use crate::auth::openid_token_service::OpenIDTokenService;

/// Enhanced container with Web3-first services
#[derive(Clone)]
pub struct SimpleContainer {
    pub db_pool: Arc<PgPool>,
    pub cache: Option<Arc<dyn Cache>>,
    
    // NEW - Web3-first services (primary)
    pub wallet_user_repository: Option<Arc<WalletUserRepositoryAdapter>>,
    pub wallet_permission_service: Option<Arc<WalletPermissionService>>,
    pub web3_permission_adapter: Option<Arc<Web3PermissionServiceAdapter>>,
    pub unified_web3_auth_service: Option<Arc<UnifiedWeb3AuthService>>,
    pub openid_token_service: Option<Arc<OpenIDTokenService>>,
    
    
    // Compatibility fields for compilation
    pub permission_service: Option<String>,
    pub auth_trigger_service: Option<String>,
}

impl SimpleContainer {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self {
            db_pool,
            cache: None,
            // NEW - Web3-first services (initialized as None, configured via builder methods)
            wallet_user_repository: None,
            wallet_permission_service: None,
            web3_permission_adapter: None,
            unified_web3_auth_service: None,
            openid_token_service: None,
            // Compatibility
            permission_service: None,
            auth_trigger_service: None,
        }
    }

    /// Get Web3 domain for SIWE authentication from environment
    fn get_web3_domain() -> String {
        use std::env;
        
        // Try to get frontend URL from environment
        if let Ok(frontend_url) = env::var("FRONTEND_URL") {
            // Extract domain from URL
            if let Ok(url) = url::Url::parse(&frontend_url) {
                if let Some(host) = url.host_str() {
                    return host.to_string();
                }
            }
        }
        
        // Try NEXT_PUBLIC_APP_URL as fallback
        if let Ok(app_url) = env::var("NEXT_PUBLIC_APP_URL") {
            if let Ok(url) = url::Url::parse(&app_url) {
                if let Some(host) = url.host_str() {
                    return host.to_string();
                }
            }
        }
        
        // Environment-based defaults
        if env::var("NODE_ENV").map(|v| v == "production").unwrap_or(false) ||
           env::var("RUST_ENV").map(|v| v == "production").unwrap_or(false) {
            "epsx.io".to_string()
        } else {
            "localhost".to_string()
        }
    }
    
    /// Create container with Web3 services properly wired
    pub fn new_with_web3_services(
        db_pool: Arc<PgPool>,
        cache: Option<Arc<dyn Cache>>,
        blockchain_config: Option<BlockchainConfig>,
    ) -> Self {
        // Create repository adapters
        let wallet_user_repository = Arc::new(WalletUserRepositoryAdapter::new(db_pool.clone()));
        
        // Create domain services
        let wallet_permission_service = Arc::new(WalletPermissionService);
        
        // Create infrastructure adapters
        let web3_permission_adapter = Arc::new(Web3PermissionServiceAdapter::new(
            cache.clone(),
            blockchain_config,
            (*db_pool).clone(),
        ));
        
        // Create unified auth service with environment-based domain
        let domain = Self::get_web3_domain();
        let unified_web3_auth_service = Arc::new(UnifiedWeb3AuthService::new(
            (*db_pool).clone(),
            domain,
        ));
        
        // Create OpenID token service
        let openid_token_service = Arc::new(OpenIDTokenService::new(
            (*db_pool).clone(),
            "https://api.epsx.io".to_string(), // issuer
            vec!["epsx-frontend".to_string(), "epsx-admin".to_string()], // audiences
            // TODO: Load actual RSA private key from environment
            jsonwebtoken::EncodingKey::from_secret(b"dev-secret-key"), // placeholder for development
        ));
        
        Self {
            db_pool,
            cache,
            // Web3-first services
            wallet_user_repository: Some(wallet_user_repository),
            wallet_permission_service: Some(wallet_permission_service),
            web3_permission_adapter: Some(web3_permission_adapter),
            unified_web3_auth_service: Some(unified_web3_auth_service),
            openid_token_service: Some(openid_token_service),
            // Compatibility
            permission_service: None,
            auth_trigger_service: None,
        }
    }

    pub fn with_cache(db_pool: Arc<PgPool>, cache: Arc<dyn Cache>) -> Self {
        Self {
            db_pool,
            cache: Some(cache),
            // Initialize Web3 services as None - use new_with_web3_services for full setup
            wallet_user_repository: None,
            wallet_permission_service: None,
            web3_permission_adapter: None,
            unified_web3_auth_service: None,
            openid_token_service: None,
            permission_service: None,
            auth_trigger_service: None,
        }
    }

    /// Builder method to add blockchain configuration
    pub fn with_blockchain_config(mut self, blockchain_config: BlockchainConfig) -> Self {
        // Recreate Web3 services with blockchain config
        if let Some(cache) = &self.cache {
            self.web3_permission_adapter = Some(Arc::new(Web3PermissionServiceAdapter::new(
                Some(cache.clone()),
                Some(blockchain_config),
                (*self.db_pool).clone(),
            )));
        }
        self
    }

    // Compatibility methods
    pub fn db_pool(&self) -> Arc<PgPool> {
        self.db_pool.clone()
    }

    pub fn infra(&self) -> &Self {
        self
    }

    // NEW - Web3-first service getters (primary)
    pub fn get_wallet_user_repository(&self) -> Option<Arc<WalletUserRepositoryAdapter>> {
        self.wallet_user_repository.clone()
    }
    
    pub fn get_wallet_user_repository_port(&self) -> Option<Arc<dyn WalletUserRepositoryPort>> {
        self.wallet_user_repository.as_ref().map(|repo| repo.clone() as Arc<dyn WalletUserRepositoryPort>)
    }
    
    pub fn get_wallet_user_analytics_port(&self) -> Option<Arc<dyn WalletUserAnalyticsPort>> {
        self.wallet_user_repository.as_ref().map(|repo| repo.clone() as Arc<dyn WalletUserAnalyticsPort>)
    }

    pub fn get_wallet_permission_service(&self) -> Option<Arc<WalletPermissionService>> {
        self.wallet_permission_service.clone()
    }

    pub fn get_web3_permission_adapter(&self) -> Option<Arc<Web3PermissionServiceAdapter>> {
        self.web3_permission_adapter.clone()
    }

    pub fn get_unified_web3_auth_service(&self) -> Option<Arc<UnifiedWeb3AuthService>> {
        self.unified_web3_auth_service.clone()
    }

    pub fn get_openid_token_service(&self) -> Option<Arc<OpenIDTokenService>> {
        self.openid_token_service.clone()
    }
    

    // Enhanced app state creation with Web3 services
    pub fn create_app_state(&self) -> Web3AppState {
        Web3AppState {
            wallet_user_repository: self.get_wallet_user_repository_port(),
            wallet_user_analytics: self.get_wallet_user_analytics_port(),
            wallet_permission_service: self.get_wallet_permission_service(),
            web3_permission_adapter: self.get_web3_permission_adapter(),
            unified_web3_auth_service: self.get_unified_web3_auth_service(),
            db_pool: self.db_pool.clone(),
            cache: self.cache.clone(),
        }
    }
    
    // Health check for all services
    pub async fn health_check(&self) -> ContainerHealthStatus {
        let mut status = ContainerHealthStatus::default();
        
        // Check database connectivity
        status.database_healthy = sqlx::query!("SELECT 1 as health_check").fetch_one(&*self.db_pool).await.is_ok();
        
        // Check cache connectivity
        if let Some(cache) = &self.cache {
            status.cache_healthy = cache.health_check().is_ok();
        } else {
            status.cache_healthy = true; // No cache configured, considered healthy
        }
        
        // Check Web3 services
        status.web3_services_healthy = self.wallet_user_repository.is_some() &&
            self.wallet_permission_service.is_some() &&
            self.web3_permission_adapter.is_some();
        
        status.overall_healthy = status.database_healthy && 
            status.cache_healthy && 
            status.web3_services_healthy;
        
        status
    }
    
    // Service validation
    pub fn validate_services(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        if self.wallet_user_repository.is_none() {
            errors.push("WalletUserRepository not configured".to_string());
        }
        
        if self.wallet_permission_service.is_none() {
            errors.push("WalletPermissionService not configured".to_string());
        }
        
        if self.web3_permission_adapter.is_none() {
            errors.push("Web3PermissionServiceAdapter not configured".to_string());
        }
        
        if self.unified_web3_auth_service.is_none() {
            errors.push("UnifiedWeb3AuthService not configured".to_string());
        }
        
        if self.openid_token_service.is_none() {
            errors.push("OpenIDTokenService not configured".to_string());
        }
        
        errors
    }
}

/// Web3-first application state with all necessary services
#[derive(Clone)]
pub struct Web3AppState {
    pub wallet_user_repository: Option<Arc<dyn WalletUserRepositoryPort>>,
    pub wallet_user_analytics: Option<Arc<dyn WalletUserAnalyticsPort>>,
    pub wallet_permission_service: Option<Arc<WalletPermissionService>>,
    pub web3_permission_adapter: Option<Arc<Web3PermissionServiceAdapter>>,
    pub unified_web3_auth_service: Option<Arc<UnifiedWeb3AuthService>>,
    pub db_pool: Arc<PgPool>,
    pub cache: Option<Arc<dyn Cache>>,
}

impl Web3AppState {
    /// Validate that all required services are available
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.wallet_user_repository.is_none() {
            errors.push("WalletUserRepository is required".to_string());
        }
        
        if self.wallet_permission_service.is_none() {
            errors.push("WalletPermissionService is required".to_string());
        }
        
        if self.web3_permission_adapter.is_none() {
            errors.push("Web3PermissionServiceAdapter is required".to_string());
        }
        
        if self.unified_web3_auth_service.is_none() {
            errors.push("UnifiedWeb3AuthService is required".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Get all required services - panics if any are missing
    pub fn services(&self) -> Web3Services {
        Web3Services {
            wallet_user_repository: self.wallet_user_repository.as_ref()
                .expect("WalletUserRepository not configured")
                .clone(),
            wallet_user_analytics: self.wallet_user_analytics.as_ref()
                .expect("WalletUserAnalytics not configured")
                .clone(),
            wallet_permission_service: self.wallet_permission_service.as_ref()
                .expect("WalletPermissionService not configured")
                .clone(),
            web3_permission_adapter: self.web3_permission_adapter.as_ref()
                .expect("Web3PermissionServiceAdapter not configured")
                .clone(),
            unified_web3_auth_service: self.unified_web3_auth_service.as_ref()
                .expect("UnifiedWeb3AuthService not configured")
                .clone(),
        }
    }
}

/// Strongly typed Web3 services collection
#[derive(Clone)]
pub struct Web3Services {
    pub wallet_user_repository: Arc<dyn WalletUserRepositoryPort>,
    pub wallet_user_analytics: Arc<dyn WalletUserAnalyticsPort>,
    pub wallet_permission_service: Arc<WalletPermissionService>,
    pub web3_permission_adapter: Arc<Web3PermissionServiceAdapter>,
    pub unified_web3_auth_service: Arc<UnifiedWeb3AuthService>,
}

/// Health status for the container and its services
#[derive(Debug, Default)]
pub struct ContainerHealthStatus {
    pub overall_healthy: bool,
    pub database_healthy: bool,
    pub cache_healthy: bool,
    pub web3_services_healthy: bool,
    pub error_details: Vec<String>,
}

impl ContainerHealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.overall_healthy
    }
    
    pub fn add_error(&mut self, error: String) {
        self.error_details.push(error);
        self.overall_healthy = false;
    }
}

// Type alias for compatibility
pub type DomainContainer = SimpleContainer;