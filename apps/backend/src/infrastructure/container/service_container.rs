// Unified Service Container
// Merges: AppContainer + DDDContainer into single dependency injection container

use std::sync::Arc;
use sqlx::PgPool;
use anyhow::Result;

use crate::infrastructure::cache::Cache;
use crate::auth::{AuthenticationService, AuthorizationService, SessionService, KeyManager};
use crate::domain::user_management::UserRepositoryPort;
use crate::infrastructure::adapters::repositories::UserRepositoryAdapter;
use crate::application::user_management::services::{UserQueryService, UserApplicationService};

/// Unified Service Container
/// Replaces both AppContainer and DDDContainer with single dependency injection system
#[derive(Clone)]
pub struct ServiceContainer {
    // Core infrastructure
    pub db_pool: Arc<PgPool>,
    pub cache: Arc<dyn Cache>,
    
    // New unified auth services
    pub auth_service: Arc<AuthenticationService>,
    pub authorization_service: Arc<AuthorizationService>,
    pub session_service: Arc<SessionService>,
    
    // Domain services
    pub user_repository: Arc<dyn UserRepositoryPort>,
    pub user_query_service: Arc<UserQueryService>,
    pub user_application_service: Arc<UserApplicationService>,
    
    // Notification service
    pub notification_service: Arc<dyn crate::application::ports::outbound::service_ports::NotificationServicePort<Error = crate::infrastructure::adapters::services::fcm_service::FcmServiceError>>,
}

impl ServiceContainer {
    /// Create new service container with all dependencies
    pub async fn new(
        db_pool: Arc<PgPool>,
        cache: Arc<dyn Cache>,
        notification_service: Arc<dyn crate::application::ports::outbound::service_ports::NotificationServicePort<Error = crate::infrastructure::adapters::services::fcm_service::FcmServiceError>>,
    ) -> Result<Self> {
        // Create new unified auth services with KeyManager
        let key_manager = Arc::new(KeyManager::new().map_err(|e| anyhow::anyhow!("Failed to create KeyManager: {}", e))?);
        let auth_service = Arc::new(AuthenticationService::new(
            key_manager.clone(),
            cache.clone(),
            24, // 24 hour token expiry
            "epsx-backend".to_string(),
            "epsx-frontend".to_string(),
        ));
        
        let authorization_service = Arc::new(AuthorizationService::new());
        
        let session_service = Arc::new(SessionService::new(
            cache.clone(),
            24, // 24 hour session expiry
            5,  // max 5 sessions per user
            24, // cleanup every 24 hours
        ));
        
        // Create domain repositories
        let user_repository: Arc<dyn UserRepositoryPort> = Arc::new(
            UserRepositoryAdapter::new(db_pool.clone())
        );
        
        // Create application services
        let user_query_service = Arc::new(UserQueryService::new(user_repository.clone()));
        
        // Create a simple event bus for now
        let event_bus: Arc<dyn crate::domain::shared_kernel::domain_event::DomainEventBus> = 
            Arc::new(crate::infrastructure::container::ddd_container::NoOpEventBus);
        
        // Create session repository port (stub for now)
        let session_repository: Arc<dyn crate::domain::user_management::repository_ports::session_repository_port::SessionRepositoryPort> =
            Arc::new(crate::infrastructure::adapters::repositories::database_types::SessionRepository::new(db_pool.clone()));
        
        let user_application_service = Arc::new(UserApplicationService::new(
            user_repository.clone(),
            session_repository,
            event_bus,
        ));
        
        Ok(Self {
            db_pool,
            cache,
            auth_service,
            authorization_service,
            session_service,
            user_repository,
            user_query_service,
            user_application_service,
            notification_service,
        })
    }
    
    /// Health check for all services
    pub async fn health_check(&self) -> ServiceHealthStatus {
        let mut status = ServiceHealthStatus::new();
        
        // Check database
        status.database = match sqlx::query("SELECT 1").fetch_one(self.db_pool.as_ref()).await {
            Ok(_) => ServiceStatus::Healthy,
            Err(e) => {
                status.errors.push(format!("Database: {}", e));
                ServiceStatus::Unhealthy
            }
        };
        
        // Check cache
        status.cache = match self.cache.get("health_check") {
            Some(_) | None => ServiceStatus::Healthy, // None is also valid (key not found)
            // Cache trait doesn't return errors in current interface
        };
        
        // Check auth service
        status.auth = if self.auth_service.health_check().await {
            ServiceStatus::Healthy
        } else {
            status.errors.push("Authentication service health check failed".to_string());
            ServiceStatus::Unhealthy
        };
        
        // Overall status
        status.overall = if status.database == ServiceStatus::Healthy 
            && status.cache == ServiceStatus::Healthy 
            && status.auth == ServiceStatus::Healthy {
            ServiceStatus::Healthy
        } else {
            ServiceStatus::Unhealthy
        };
        
        status
    }
    
    /// Get database statistics
    pub async fn get_db_stats(&self) -> Result<DatabaseStats> {
        // Get connection pool stats
        let pool_size = self.db_pool.size() as u32;
        let num_idle = self.db_pool.num_idle() as u32;
        
        let pool_stats = DatabaseStats {
            active_connections: pool_size - num_idle,
            idle_connections: num_idle,
            max_connections: self.db_pool.options().get_max_connections(),
            total_connections: pool_size,
        };
        
        Ok(pool_stats)
    }
    
    /// Create application state for web handlers
    pub fn create_app_state(&self) -> crate::web::auth::routes::AppState {
        // Create missing services that are required for AppState
        // Get Web3 configuration from environment
        let web3_domain = std::env::var("WEB3_DOMAIN")
            .unwrap_or_else(|_| {
                match std::env::var("NODE_ENV").unwrap_or_else(|_| "development".to_string()).as_str() {
                    "development" => "localhost:3000".to_string(),
                    _ => "epsx.io".to_string(),
                }
            });
        
        let web3_chain_id = std::env::var("WEB3_CHAIN_ID")
            .unwrap_or_else(|_| {
                match std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK").unwrap_or_else(|_| "testnet".to_string()).as_str() {
                    "mainnet" => "56".to_string(), // BSC mainnet
                    _ => "97".to_string(), // BSC testnet (default)
                }
            })
            .parse::<u64>()
            .unwrap_or(97); // Default to BSC testnet
            
        let web3_auth_service = Arc::new(crate::auth::Web3AuthService::new((*self.db_pool).clone(), web3_domain, web3_chain_id));
        let web3_permission_service = Arc::new(crate::auth::Web3PermissionService::new((*self.db_pool).clone(), "https://eth.llamarpc.com".to_string(), "https://polygon.llamarpc.com".to_string()));
        let jwt_service = Arc::new(crate::auth::JWTService::new().expect("Failed to create JWT service"));
        
        crate::web::auth::routes::AppState::new(
            self.db_pool.clone(),
            self.cache.clone(),
            Arc::new(crate::infrastructure::container::ddd_container::DDDContainer::new(self.db_pool.clone())),
            self.user_repository.clone(),
            Arc::new(crate::domain::authorization::services::stateless_permission_service::StatelessPermissionService::new()),
            None, // Rate limiting service - optional
            web3_auth_service,
            web3_permission_service,
            jwt_service,
        )
    }
}

/// Service health status
#[derive(Debug, Clone)]
pub struct ServiceHealthStatus {
    pub overall: ServiceStatus,
    pub database: ServiceStatus,
    pub cache: ServiceStatus,
    pub auth: ServiceStatus,
    pub errors: Vec<String>,
}

impl ServiceHealthStatus {
    pub fn new() -> Self {
        Self {
            overall: ServiceStatus::Unknown,
            database: ServiceStatus::Unknown,
            cache: ServiceStatus::Unknown,
            auth: ServiceStatus::Unknown,
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub total_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::memory_cache::MemoryCache;
    use crate::infrastructure::adapters::services::notification_service_adapter::NotificationServiceAdapter;
    
    #[tokio::test]
    async fn test_service_container_creation() {
        // This test would require a database connection
        // For now, we'll just test the structure
        
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/test".to_string());
        
        if let Ok(pool) = PgPool::connect(&db_url).await {
            let db_pool = Arc::new(pool);
            let cache = Arc::new(MemoryCache::new());
            let firebase_admin_stub = Arc::new(FirebaseAdminStub::new("test-project".to_string()));
            let notification_service = Arc::new(NotificationServiceAdapter::new());
            
            let container = ServiceContainer::new(
                db_pool,
                cache,
                firebase_admin_stub,
                notification_service,
            ).await;
            
            assert!(container.is_ok());
        }
    }
}