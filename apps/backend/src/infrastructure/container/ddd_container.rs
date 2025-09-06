use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

// Infrastructure dependencies
use crate::infrastructure::adapters::repositories::diesel::DbPool;
use crate::infrastructure::cache::{Cache, CacheFactory};
use crate::infrastructure::adapters::repositories::diesel::repos::user_repo::DieselUserRepository;
use crate::application::ports::repositories::{UserRepository, UserPermissionRepository, SessionRepository};
use crate::infrastructure::adapters::repositories::{
    UserRepositoryAdapter, SessionRepositoryAdapter, UserPermissionRepositoryAdapter,
    PaymentRepositoryAdapter, TransactionRepositoryAdapter,
    CryptoAddressRepositoryAdapter, PaymentMethodRepositoryAdapter,
    RealtimeEventRepositoryAdapter, ConnectionRepositoryAdapter,
};
use crate::infrastructure::event_bus::SimpleEventBus;

// Domain layer imports
use crate::domain::shared_kernel::DomainEventBus;
use crate::domain::user_management::{UserRepositoryPort, SessionRepositoryPort};
use crate::domain::session_management::repositories::{SessionManagerRepositoryPort, SessionMetadataRepositoryPort};

// Application layer imports
use crate::application::shared::CommandHandler;
use crate::application::user_management::{
    CreateUserCommandHandler, 
    GrantPermissionCommandHandler,
    CreateSessionCommandHandler,
};
use crate::application::user_management::services::{UserApplicationService, UserQueryService};

// Integration layer imports
use crate::infrastructure::integration::{
    PaymentServiceIntegration,
    RealtimeEventsServiceIntegration,
};

/// DDD Container for dependency injection
/// Properly wires hexagonal architecture with clean separation of concerns
#[derive(Clone)]
pub struct DDDContainer {
    // Core infrastructure
    db_pool: Arc<DbPool>,
    event_bus: Arc<dyn DomainEventBus>,
    
    // Repository ports (domain interfaces)
    user_repository_port: Arc<dyn UserRepositoryPort>,
    session_repository_port: Arc<dyn SessionRepositoryPort>,
    user_permission_repository_port: Arc<dyn UserPermissionRepository<Error = crate::infrastructure::adapters::repositories::user_permission_repository_adapter::LegacyPermissionRepositoryError>>,
    
    // Command handlers
    create_user_handler: Arc<CreateUserCommandHandler>,
    grant_permission_handler: Arc<GrantPermissionCommandHandler>, 
    create_session_handler: Arc<CreateSessionCommandHandler>,
    
    // Application services
    user_application_service: Arc<UserApplicationService>,
    user_query_service: Arc<UserQueryService>,
    
    // Integration services (high-level orchestration)
    pub payment_service: Arc<PaymentServiceIntegration>,
    pub realtime_events_service: Arc<RealtimeEventsServiceIntegration>,
}

impl DDDContainer {
    /// Create new DDD container with all dependencies properly wired
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        // 1. Create infrastructure components
        let event_bus: Arc<dyn DomainEventBus> = Arc::new(SimpleEventBus::new());
        let cache: Arc<dyn Cache> = tokio::task::block_in_place(|| {
            Arc::from(tokio::runtime::Handle::current().block_on(CacheFactory::with_fallback()))
        });
        
        // Legacy user repository for compatibility - using LegacyRepositoryError for trait compliance
        let legacy_user_repository: Arc<dyn UserRepository<Error = crate::infrastructure::adapters::repositories::user_repository_adapter::LegacyRepositoryError>> = Arc::new(UserRepositoryAdapter::new(db_pool.clone()));
        
        // 2. Create repository adapters (infrastructure layer)
        let user_repository_port: Arc<dyn UserRepositoryPort> = 
            Arc::new(UserRepositoryAdapter::new(db_pool.clone()));
        let session_repository_port: Arc<dyn SessionRepositoryPort> = 
            Arc::new(SessionRepositoryAdapter::new(db_pool.clone()));
        let user_permission_repository_port: Arc<dyn UserPermissionRepository<Error = crate::infrastructure::adapters::repositories::user_permission_repository_adapter::LegacyPermissionRepositoryError>> = 
            Arc::new(UserPermissionRepositoryAdapter::new(db_pool.clone()));
        
        // Payment repository adapters
        let payment_repository_adapter = Arc::new(PaymentRepositoryAdapter::new(
            db_pool.clone(),
            legacy_user_repository.clone(),
        ));
        let transaction_repository_adapter = Arc::new(TransactionRepositoryAdapter::new(
            db_pool.clone(),
            cache.clone(),
        ));
        let crypto_address_repository_adapter = Arc::new(CryptoAddressRepositoryAdapter::new(db_pool.clone()));
        let payment_method_repository_adapter = Arc::new(PaymentMethodRepositoryAdapter::new(db_pool.clone()));
        
        // Real-time Events repository adapters
        let realtime_event_repository_adapter = Arc::new(RealtimeEventRepositoryAdapter::new(db_pool.clone()));
        let connection_repository_adapter = Arc::new(ConnectionRepositoryAdapter::new());
        
        // 3. Create application services (application layer)
        // UserApplicationService creates handlers internally
        let user_application_service = Arc::new(UserApplicationService::new(
            user_repository_port.clone(),
            session_repository_port.clone(),
            event_bus.clone(),
        ));
        
        let user_query_service = Arc::new(UserQueryService::new(
            user_repository_port.clone(),
        ));
        
        // 4. Extract handlers from application service for direct access if needed
        // (In a real implementation, we might not need this level of access)
        let create_user_handler = Arc::new(CreateUserCommandHandler::new(
            user_repository_port.clone(),
            event_bus.clone(),
        ));
        
        let grant_permission_handler = Arc::new(GrantPermissionCommandHandler::new(
            user_repository_port.clone(),
            event_bus.clone(),
        ));
        
        let create_session_handler = Arc::new(CreateSessionCommandHandler::new(
            user_repository_port.clone(),
            session_repository_port.clone(),
            event_bus.clone(),
        ));
        
        // 5. Create integration services (high-level orchestration)
        let payment_service = Arc::new(PaymentServiceIntegration::new(
            payment_repository_adapter,
            transaction_repository_adapter,
            crypto_address_repository_adapter,
            payment_method_repository_adapter,
            event_bus.clone(),
        ));
        
        let realtime_events_service = Arc::new(RealtimeEventsServiceIntegration::new(
            realtime_event_repository_adapter,
            connection_repository_adapter,
        ));
        
        Self {
            db_pool,
            event_bus,
            user_repository_port,
            session_repository_port,
            user_permission_repository_port,
            create_user_handler,
            grant_permission_handler,
            create_session_handler,
            user_application_service,
            user_query_service,
            payment_service,
            realtime_events_service,
        }
    }
    
    // === Getters for dependency injection ===
    
    /// Get user repository port (domain interface)
    pub fn user_repository(&self) -> Arc<dyn UserRepositoryPort> {
        self.user_repository_port.clone()
    }
    
    /// Get session repository port (domain interface)
    pub fn session_repository(&self) -> Arc<dyn SessionRepositoryPort> {
        self.session_repository_port.clone()
    }
    
    /// Get user permission repository port (domain interface)
    pub fn user_permission_repository(&self) -> Arc<dyn UserPermissionRepository<Error = crate::infrastructure::adapters::repositories::user_permission_repository_adapter::LegacyPermissionRepositoryError>> {
        self.user_permission_repository_port.clone()
    }
    
    /// Get domain event bus
    pub fn event_bus(&self) -> Arc<dyn DomainEventBus> {
        self.event_bus.clone()
    }
    
    /// Get create user command handler
    pub fn create_user_handler(&self) -> Arc<CreateUserCommandHandler> {
        self.create_user_handler.clone()
    }
    
    /// Get grant permission command handler
    pub fn grant_permission_handler(&self) -> Arc<GrantPermissionCommandHandler> {
        self.grant_permission_handler.clone()
    }
    
    /// Get create session command handler
    pub fn create_session_handler(&self) -> Arc<CreateSessionCommandHandler> {
        self.create_session_handler.clone()
    }
    
    /// Get user application service
    pub fn user_application_service(&self) -> Arc<UserApplicationService> {
        self.user_application_service.clone()
    }
    
    /// Get user query service
    pub fn user_query_service(&self) -> Arc<UserQueryService> {
        self.user_query_service.clone()
    }
    
    /// Get payment service integration
    pub fn payment_service(&self) -> Arc<PaymentServiceIntegration> {
        self.payment_service.clone()
    }
    
    /// Get real-time events service integration  
    pub fn realtime_events_service(&self) -> Arc<RealtimeEventsServiceIntegration> {
        self.realtime_events_service.clone()
    }
    
    /// Get authentication service integration
    pub fn authentication_service_integration(&self) -> Arc<crate::infrastructure::AuthenticationServiceIntegration> {
        // Create authentication service integration on-demand
        Arc::new(crate::infrastructure::AuthenticationServiceIntegration::new(
            Arc::new(crate::infrastructure::adapters::repositories::user_repository_adapter::UserRepositoryAdapter::new(self.db_pool.clone())),
            Arc::new(crate::infrastructure::adapters::repositories::session_repository_adapter::SessionRepositoryAdapter::new(self.db_pool.clone())),
            self.create_session_handler.clone(),
        ))
    }
    
    /// Get database pool for direct access when needed
    pub fn db_pool(&self) -> Arc<DbPool> {
        self.db_pool.clone()
    }
}

/// Builder for DDD Container to support configuration
pub struct DDDContainerBuilder {
    db_pool: Option<Arc<DbPool>>,
}

impl DDDContainerBuilder {
    pub fn new() -> Self {
        Self {
            db_pool: None,
        }
    }
    
    pub fn with_db_pool(mut self, db_pool: Arc<DbPool>) -> Self {
        self.db_pool = Some(db_pool);
        self
    }
    
    pub fn build(self) -> Result<DDDContainer, String> {
        let db_pool = self.db_pool
            .ok_or("Database pool is required".to_string())?;
            
        Ok(DDDContainer::new(db_pool))
    }
}

impl Default for DDDContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}