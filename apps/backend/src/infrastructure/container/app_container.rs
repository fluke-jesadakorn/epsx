use std::sync::Arc;
use sqlx::PgPool;
use crate::infrastructure::container::ddd_container::DDDContainer;
use crate::infrastructure::adapters::services::firebase::firebase_admin::FirebaseAdmin;
use crate::infrastructure::cache::Cache;
use crate::web::auth::AppState;

type DbPool = PgPool;

/// Modern AppContainer with unified database abstraction
#[derive(Clone)]
pub struct AppContainer {
    ddd_container: DDDContainer,
    firebase_admin: Arc<FirebaseAdmin>,
    cache: Arc<dyn Cache>,
    pub infra: InfraFactory,
    pub fcm_service: Arc<dyn crate::application::ports::outbound::service_ports::NotificationServicePort<Error = crate::infrastructure::adapters::services::fcm_service::FcmServiceError>>,
    pub fcm_topic_service: Arc<crate::infrastructure::adapters::services::fcm_service::FcmTopicService>,
    pub user_notification_repo: Arc<UserNotificationRepository>,
}

/// Infrastructure factory for creating infrastructure components
#[derive(Clone)]
pub struct InfraFactory {
    pub db_pool: Arc<DbPool>,
    pub cache: Arc<dyn Cache>,
    pub firebase_admin: Arc<FirebaseAdmin>,
}

impl AppContainer {
    /// Create new AppContainer with database connection pool
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize database connection pool
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable is required")?;
        let db_pool = sqlx::PgPool::connect(&database_url).await
            .map_err(|e| format!("Failed to create database connection pool: {}", e))?;
        
        let db_pool = Arc::new(db_pool);
        
        // Create DDD container
        let ddd_container = DDDContainer::new(db_pool.clone());
        
        // Initialize Firebase Admin
        let project_id = std::env::var("FIREBASE_PROJECT_ID")
            .map_err(|_| "FIREBASE_PROJECT_ID environment variable is required")?;
        let firebase_admin = Arc::new(FirebaseAdmin::new(project_id));
        
        // Initialize cache
        let cache_impl = crate::infrastructure::cache::CacheFactory::with_fallback().await;
        let cache: Arc<dyn crate::infrastructure::cache::Cache> = Arc::from(cache_impl);
        
        // Create infrastructure factory
        let infra = InfraFactory {
            db_pool: db_pool.clone(),
            cache: cache.clone(),
            firebase_admin: firebase_admin.clone(),
        };
        
        // Initialize FCM topic service
        let fcm_topic_service = Arc::new(crate::infrastructure::adapters::services::fcm_service::FcmTopicService::new());
        
        // Initialize FCM service
        let fcm_service_concrete = Arc::new(crate::infrastructure::adapters::services::fcm_service::FcmService::new(firebase_admin.clone()));
        let fcm_service: Arc<dyn crate::application::ports::outbound::service_ports::NotificationServicePort<Error = crate::infrastructure::adapters::services::fcm_service::FcmServiceError>> = fcm_service_concrete;
        
        // Create placeholder user notification repository
        let user_notification_repo = Arc::new(UserNotificationRepository::new());

        Ok(Self {
            ddd_container,
            firebase_admin,
            cache,
            infra,
            fcm_service,
            fcm_topic_service,
            user_notification_repo,
        })
    }
    
    /// Create application state for web routes
    pub async fn create_app_state(&self) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
        let ddd_container = Arc::new(self.ddd_container.clone());
        
        // Get user repository from DDD container
        let user_repo = self.ddd_container.user_repository();

        // Create Web3 services
        let web3_auth_service = Arc::new(crate::auth::Web3AuthService::new(
            (*self.infra.db_pool).clone(),
            "epsx.io".to_string(), // TODO: Get from env
        ));

        let web3_permission_service = Arc::new(crate::auth::Web3PermissionService::new_multi_chain(
            (*self.infra.db_pool).clone(),
            std::env::var("ETHEREUM_RPC_URL").unwrap_or_else(|_| "https://eth.llamarpc.com".to_string()),
            std::env::var("POLYGON_RPC_URL").unwrap_or_else(|_| "https://polygon.llamarpc.com".to_string()),
            std::env::var("ARBITRUM_RPC_URL").unwrap_or_else(|_| "https://arbitrum.llamarpc.com".to_string()),
            std::env::var("OPTIMISM_RPC_URL").unwrap_or_else(|_| "https://optimism.llamarpc.com".to_string()),
            std::env::var("BASE_RPC_URL").unwrap_or_else(|_| "https://base.llamarpc.com".to_string()),
            std::env::var("BSC_RPC_URL").unwrap_or_else(|_| "https://bsc-dataseed.binance.org".to_string()),
        ));

        let jwt_service = Arc::new(crate::auth::JWTService::new()?);

        let firebase_admin_stub = Arc::new(crate::infrastructure::adapters::services::FirebaseAdminStub::new(
            std::env::var("FIREBASE_PROJECT_ID").unwrap_or_else(|_| "epsx-web3".to_string()),
        ));

        Ok(AppState {
            db_pool: self.infra.db_pool.clone(),
            cache: self.cache.clone(),
            ddd_container,
            user_repo: user_repo,
            permission_service: Arc::new(crate::domain::authorization::services::stateless_permission_service::StatelessPermissionService),
            rate_limiting_service: None, // TODO: Implement proper rate limiting service wrapper
            web3_auth_service,
            web3_permission_service,
            jwt_service,
            firebase_admin: firebase_admin_stub,
        })
    }
    
    /// Get database pool
    pub fn db_pool(&self) -> Arc<DbPool> {
        self.infra.db_pool.clone()
    }
    
    /// Get Web3 auth service
    pub fn web3_auth_service(&self) -> Arc<crate::auth::Web3AuthService> {
        Arc::new(crate::auth::Web3AuthService::new(
            (*self.infra.db_pool).clone(),
            "epsx.io".to_string(), // TODO: Get from env
        ))
    }
    
    /// Get Web3 permission service
    pub fn web3_permission_service(&self) -> Arc<crate::auth::Web3PermissionService> {
        Arc::new(crate::auth::Web3PermissionService::new_multi_chain(
            (*self.infra.db_pool).clone(),
            "https://eth.llamarpc.com".to_string(),
            "https://polygon.llamarpc.com".to_string(),
            "https://arbitrum.llamarpc.com".to_string(),
            "https://optimism.llamarpc.com".to_string(),
            "https://base.llamarpc.com".to_string(),
            "https://bsc-dataseed.binance.org".to_string(),
        ))
    }
    
    /// Get JWT service
    pub fn jwt_service(&self) -> Result<Arc<crate::auth::JWTService>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(crate::auth::JWTService::new()?))
    }
    
    /// Get unified auth service (legacy compatibility)
    pub fn unified_auth_service(&self) -> Arc<crate::auth::Web3AuthService> {
        self.web3_auth_service() // Delegate to Web3 auth service
    }
    
    // Helper methods for creating services
    async fn create_rate_limiting_service(&self) -> Result<Arc<dyn crate::domain::resource_management::services::rate_limiting_service::RateLimitingServicePort<Error = crate::infrastructure::adapters::services::combined_rate_limiting_service::CombinedRateLimitingError>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(
            crate::infrastructure::adapters::services::combined_rate_limiting_service::CombinedRateLimitingService::new(
                self.cache.clone(),
                self.infra.db_pool.clone()
            )
        ))
    }
    
    async fn create_security_monitoring_service(&self) -> Result<Arc<dyn crate::application::ports::outbound::service_ports::SecurityMonitoringServicePort<Error = crate::infrastructure::adapters::services::security_monitoring_service_adapter::SecurityMonitoringError>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(
            crate::infrastructure::adapters::services::security_monitoring_service_adapter::SecurityMonitoringServiceAdapter::new(
                self.cache.clone()
            )
        ))
    }
    
    async fn create_unified_admin_client(&self) -> Result<Arc<dyn crate::application::ports::outbound::service_ports::AdminClientPort<Error = crate::infrastructure::adapters::services::unified_admin_client_adapter::UnifiedAdminClientError>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(
            crate::infrastructure::adapters::services::unified_admin_client_adapter::UnifiedAdminClientAdapter::new(
                self.infra.db_pool.clone(),
                self.cache.clone()
            )
        ))
    }
    
    async fn create_granular_permissions_client(&self) -> Result<Arc<dyn crate::application::ports::outbound::service_ports::GranularPermissionsClientPort<Error = crate::infrastructure::adapters::services::granular_permissions_admin_client_adapter::GranularPermissionsClientError>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new(
            crate::infrastructure::adapters::services::granular_permissions_admin_client_adapter::GranularPermissionsAdminClientAdapter::new(
                self.infra.db_pool.clone(),
                self.cache.clone()
            )
        ))
    }
}

/// Placeholder User Notification Repository
#[derive(Clone)]
pub struct UserNotificationRepository;

impl UserNotificationRepository {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn get_notifications(&self, _user_id: &str) -> Result<Vec<Notification>, String> {
        // TODO: Implement notification retrieval
        Ok(vec![])
    }
}

/// Placeholder notification struct
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub message: String,
    pub read: bool,
}