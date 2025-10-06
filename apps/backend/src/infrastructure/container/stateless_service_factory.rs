// Stateless Service Factory for Serverless Architecture
// Creates services per request without shared state containers
// Designed for serverless environments (AWS Lambda, Google Cloud Functions, etc.)

use sqlx::PgPool;
use anyhow::Result;
use std::sync::Arc;
use crate::infrastructure::cache::{Cache, ServerlessCacheFactory};
use crate::infrastructure::database::{ServerlessConnectionManager, get_db_pool};
use crate::infrastructure::adapters::repositories::wallet_user_repository_adapter::WalletUserRepositoryAdapter;
use crate::infrastructure::adapters::services::web3_permission_service_adapter::{
    Web3PermissionServiceAdapter, BlockchainConfig
};
use crate::domain::wallet_management::{
    WalletPermissionService,
    WalletUserRepositoryPort,
    WalletUserAnalyticsPort,
};
use crate::auth::unified_web3_auth_service::UnifiedWeb3AuthService;
use crate::auth::openid_token_service::OpenIDTokenService;
use crate::auth::key_manager::KeyManager;
use crate::auth::{
    CentralizedPermissionAuthority, DatabasePermissionRegistry, PermissionState,
    create_permission_authority, create_permission_registry
};

/// Stateless configuration for service factory
#[derive(Clone)]
pub struct StatelessConfig {
    pub database_url: String,
    pub domain: String,
    pub issuer_url: String,
    pub oidc_audiences: Vec<String>,
    pub redis_url: Option<String>,
    pub blockchain_config: Option<BlockchainConfig>,
}

impl StatelessConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            domain: Self::get_web3_domain(),
            issuer_url: std::env::var("BACKEND_URL")
                .unwrap_or_else(|_| "https://api.epsx.io".to_string()),
            oidc_audiences: vec![
                "epsx-frontend".to_string(),
                "epsx-admin".to_string(),
            ],
            redis_url: std::env::var("REDIS_URL").ok(),
            blockchain_config: None, // Can be added later if needed
        })
    }

    /// Get Web3 domain for SIWE authentication from environment
    fn get_web3_domain() -> String {
        use std::env;
        
        // Try to get frontend URL from environment
        if let Ok(frontend_url) = env::var("FRONTEND_URL") {
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
}

/// Stateless Service Factory - Creates services per request without shared state
#[derive(Clone)]
pub struct StatelessServiceFactory {
    config: StatelessConfig,
}

impl StatelessServiceFactory {
    pub fn new(config: StatelessConfig) -> Self {
        Self { config }
    }

    /// Create all services for a single request
    /// This is called once per HTTP request in serverless environments
    pub async fn create_request_services(&self) -> Result<RequestServices> {
        // Get global serverless-optimized database pool (reuses connections)
        let db_pool = get_db_pool().await
            .map_err(|e| anyhow::anyhow!("Failed to get database pool: {}", e))?;

        // Clone pool once for Arc wrapping (PgPool clone is cheap - it's Arc internally)
        let db_pool_owned = db_pool.clone();
        let db_pool_arc = Arc::new(db_pool_owned.clone());

        // Create cache (Redis ONLY - no fallback to memory for serverless)
        let cache = if let Some(redis_url) = &self.config.redis_url {
            Some(ServerlessCacheFactory::redis_with_url(redis_url.clone()).await?)
        } else {
            None
        };

        // Create repository adapters using Arc pool reference
        let wallet_user_repository = WalletUserRepositoryAdapter::new(db_pool_arc.clone());

        // Create domain services (stateless by design)
        let wallet_permission_service = WalletPermissionService;

        // Create infrastructure adapters using owned pool
        let web3_permission_adapter = Web3PermissionServiceAdapter::new(
            cache.clone(),
            self.config.blockchain_config.clone(),
            db_pool_owned.clone(),
        );

        // Create auth services using owned pool
        let unified_web3_auth_service = UnifiedWeb3AuthService::new(
            db_pool_owned.clone(),
            self.config.domain.clone(),
        );

        // Create OpenID token service using owned pool and RSA key manager
        let key_manager = KeyManager::from_env_or_generate()
            .expect("Failed to initialize RSA key manager");
        let openid_token_service = OpenIDTokenService::new(
            db_pool_owned.clone(),
            self.config.issuer_url.clone(),
            self.config.oidc_audiences.clone(),
            Arc::new(key_manager),
        );

        // Create centralized permission services
        let permission_authority = Arc::new(create_permission_authority(db_pool_owned.clone()));
        let permission_registry = Arc::new(create_permission_registry(db_pool_owned));

        // Initialize permission registry with default routes
        if let Err(e) = permission_registry.initialize().await {
            tracing::warn!("Failed to initialize permission registry: {}", e);
        }

        // Create permission state for dependency injection
        let permission_state = PermissionState::new(
            permission_authority.clone(),
            permission_registry.clone(),
        );

        Ok(RequestServices {
            db_pool: db_pool_arc,
            cache,
            wallet_user_repository: Arc::new(wallet_user_repository),
            wallet_permission_service,
            web3_permission_adapter: Arc::new(web3_permission_adapter),
            unified_web3_auth_service: Arc::new(unified_web3_auth_service),
            openid_token_service: Arc::new(openid_token_service),

            // New centralized permission services
            permission_authority,
            permission_registry,
            permission_state: Arc::new(permission_state),
        })
    }

    // Redis cache creation methods removed - now using ServerlessCacheFactory

    /// Create minimal services for health checks (faster cold start)
    pub async fn create_health_services(&self) -> Result<HealthServices> {
        // Use global serverless connection manager for health checks
        let db_pool = get_db_pool().await
            .map_err(|e| anyhow::anyhow!("Failed to get database pool: {}", e))?;

        Ok(HealthServices {
            db_pool: Arc::new(db_pool.clone()),
        })
    }
}

/// Services created per request - no shared state
pub struct RequestServices {
    pub db_pool: Arc<PgPool>,
    pub cache: Option<Arc<dyn Cache>>,
    
    // Service instances (owned by this request)
    pub wallet_user_repository: Arc<WalletUserRepositoryAdapter>,
    pub wallet_permission_service: WalletPermissionService,
    pub web3_permission_adapter: Arc<Web3PermissionServiceAdapter>,
    pub unified_web3_auth_service: Arc<UnifiedWeb3AuthService>,
    pub openid_token_service: Arc<OpenIDTokenService>,
    
    // Centralized permission services (v2.0)
    pub permission_authority: Arc<CentralizedPermissionAuthority>,
    pub permission_registry: Arc<DatabasePermissionRegistry>,
    pub permission_state: Arc<PermissionState>,
}

impl RequestServices {
    /// Get wallet user repository port
    pub fn get_wallet_user_repository_port(&self) -> Arc<dyn WalletUserRepositoryPort> {
        self.wallet_user_repository.clone() as Arc<dyn WalletUserRepositoryPort>
    }

    /// Get wallet user analytics port
    pub fn get_wallet_user_analytics_port(&self) -> Arc<dyn WalletUserAnalyticsPort> {
        self.wallet_user_repository.clone() as Arc<dyn WalletUserAnalyticsPort>
    }

    /// Create app state for auth routes
    pub fn create_auth_app_state(&self) -> crate::web::auth::AppState {
        crate::web::auth::AppState::new(
            self.db_pool.clone(),
            self.cache.as_ref().unwrap().clone(), // Auth requires cache
            // Convert to legacy container format for compatibility
            Arc::new(crate::infrastructure::container::DomainContainer::new(self.db_pool.clone())),
        )
    }

    /// Validate that all required services are available
    pub fn validate(&self) -> Result<()> {
        if self.cache.is_none() {
            return Err(anyhow::anyhow!("Cache is required for request services"));
        }
        Ok(())
    }
}

/// Minimal services for health checks only
pub struct HealthServices {
    pub db_pool: Arc<PgPool>,
}

impl HealthServices {
    /// Health check - test database connectivity using serverless manager
    pub async fn health_check(&self) -> bool {
        // Use the optimized health check from ServerlessConnectionManager
        ServerlessConnectionManager::health_check().await
    }
}

/// Service factory trait for dependency injection
pub trait ServiceFactory: Send + Sync + Clone {
    type Services;
    type Error;

    fn create_services(&self) -> impl std::future::Future<Output = Result<Self::Services, Self::Error>> + Send;
}

impl ServiceFactory for StatelessServiceFactory {
    type Services = RequestServices;
    type Error = anyhow::Error;

    fn create_services(&self) -> impl std::future::Future<Output = Result<Self::Services, Self::Error>> + Send {
        self.create_request_services()
    }
}

/// Health status for stateless services
#[derive(Debug)]
pub struct StatelessHealthStatus {
    pub database_healthy: bool,
    pub cache_available: bool,
    pub services_ready: bool,
    pub error_details: Vec<String>,
}

impl StatelessHealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.database_healthy && self.services_ready
    }

    pub fn add_error(&mut self, error: String) {
        self.error_details.push(error);
        self.services_ready = false;
    }
}