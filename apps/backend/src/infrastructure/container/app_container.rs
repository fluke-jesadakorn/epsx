use std::sync::Arc;
use crate::infrastructure::adapters::repositories::diesel::DbPool;
use crate::infrastructure::container::ddd_container::DDDContainer;
use crate::infrastructure::adapters::services::firebase::firebase_admin::FirebaseAdmin;
use crate::infrastructure::cache::Cache;
use crate::web::auth::AppState;
use crate::infrastructure::adapters::repositories::diesel::repos::UserNotificationRepository;

/// AppContainer is a container for all application dependencies
/// This container is used throughout the application for dependency injection
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
    /// Create a new AppContainer with all dependencies properly wired
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize database connection pool
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/epsx".to_string());
        let db_pool = crate::infrastructure::adapters::repositories::diesel::create_diesel_pool(&database_url).await
            .map_err(|e| format!("Failed to create database connection pool: {}", e))?;
        
        let db_pool = Arc::new(db_pool);
        
        // Create DDD container
        let ddd_container = DDDContainer::new(db_pool.clone());
        
        // Initialize Firebase Admin
        let project_id = std::env::var("FIREBASE_PROJECT_ID")
            .unwrap_or_else(|_| "epsx-test".to_string());
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
        
        // Initialize user notification repository  
        let fcm_service_concrete = Arc::new(crate::infrastructure::adapters::services::fcm_service::FcmService::new(firebase_admin.clone()));
        let user_notification_repo = Arc::new(crate::infrastructure::adapters::repositories::notification_repository_adapter::NotificationRepositoryAdapter::new(
            fcm_service_concrete.clone(),
            Arc::new(crate::infrastructure::adapters::services::email_service::SendGridEmailService::new(
                crate::config::Config::from_env().expect("Failed to load configuration").email.sendgrid_api_key,
            )),
        ));
        
        // Update fcm_service to use the concrete type for trait object compatibility
        let fcm_service: Arc<dyn crate::application::ports::outbound::service_ports::NotificationServicePort<Error = crate::infrastructure::adapters::services::fcm_service::FcmServiceError>> = fcm_service_concrete;
        
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
        // Use the DDD container from this container
        let ddd_container = Arc::new(self.ddd_container.clone());
        
        // Get user repository port from domain (expected by AppState) 
        let user_repo: Arc<dyn crate::domain::user_management::UserRepositoryPort> = 
            Arc::new(crate::infrastructure::adapters::repositories::user_repository_adapter::UserRepositoryAdapter::new(
                self.db_pool()
            )) as Arc<dyn crate::domain::user_management::UserRepositoryPort>;

        // Get session repository port from domain (expected by AppState)
        let session_repo: Arc<dyn crate::domain::user_management::SessionRepositoryPort> =
            Arc::new(crate::infrastructure::adapters::repositories::session_repository_adapter::SessionRepositoryAdapter::new(
                self.db_pool()
            )) as Arc<dyn crate::domain::user_management::SessionRepositoryPort>;
        
        // Create rate limiting service with integrated resource tracking
        let rate_limiting_service = self.create_rate_limiting_service().await?;
        
        Ok(AppState {
            db_pool: self.db_pool(),
            firebase_admin: self.firebase_admin(),
            cache: self.cache(),
            notification_service: self.notification_service(),
            ddd_container,
            user_repo,
            session_repo,
            permission_application_service: None, // Placeholder until implemented
            rate_limiting_service: Some(rate_limiting_service),
        })
    }
    
    /// Get the underlying DDD container
    pub fn ddd_container(&self) -> &DDDContainer {
        &self.ddd_container
    }
    
    /// Get the Firebase Admin instance
    pub fn firebase_admin(&self) -> Arc<FirebaseAdmin> {
        self.firebase_admin.clone()
    }
    
    /// Get the cache instance
    pub fn cache(&self) -> Arc<dyn Cache> {
        self.cache.clone()
    }
    
    /// Get database pool for direct access when needed
    pub fn db_pool(&self) -> Arc<DbPool> {
        self.ddd_container.db_pool()
    }
    
    /// Get notification service port
    pub fn notification_service(&self) -> Arc<dyn crate::application::ports::outbound::service_ports::NotificationServicePort<Error = crate::infrastructure::adapters::services::fcm_service::FcmServiceError>> {
        self.fcm_service.clone()
    }
    
    /// Get user repository port (domain interface)
    pub fn user_repository(&self) -> Arc<dyn crate::domain::user_management::UserRepositoryPort> {
        self.ddd_container.user_repository()
    }
    
    /// Get session repository port (domain interface)
    pub fn session_repository(&self) -> Arc<dyn crate::domain::user_management::SessionRepositoryPort> {
        self.ddd_container.session_repository()
    }
    
    /// Get user permission repository port (domain interface)
    pub fn user_permission_repository(&self) -> Arc<dyn crate::application::ports::repositories::UserPermissionRepository<Error = crate::infrastructure::adapters::repositories::user_permission_repository_adapter::LegacyPermissionRepositoryError>> {
        self.ddd_container.user_permission_repository()
    }
    
    /// Get domain event bus
    pub fn event_bus(&self) -> Arc<dyn crate::domain::shared_kernel::DomainEventBus> {
        self.ddd_container.event_bus()
    }
    
    /// Get FCM topic service
    pub fn fcm_topic_service(&self) -> Arc<crate::infrastructure::adapters::services::fcm_service::FcmTopicService> {
        self.fcm_topic_service.clone()
    }
    
    /// Get user notification repository
    pub fn user_notification_repository(&self) -> Arc<UserNotificationRepository> {
        self.user_notification_repo.clone()
    }

    /// Get user query service
    pub fn user_query_service(&self) -> Arc<crate::application::user_management::services::UserApplicationService> {
        // Create user application service which handles user queries
        self.ddd_container.user_application_service()
    }
    
    /// Create rate limiting service with all necessary dependencies
    async fn create_rate_limiting_service(&self) -> Result<Arc<crate::domain::resource_management::services::RateLimitingService>, Box<dyn std::error::Error + Send + Sync>> {
        // Create real-time cache port implementation using Redis/memory cache
        let real_time_cache = Arc::new(crate::infrastructure::adapters::cache::RealTimeCacheAdapter::new(
            self.cache()
        ));
        
        // Create plan repository port for plan-based limits
        let plan_repository = Arc::new(crate::infrastructure::adapters::repositories::plan_repository_adapter::PlanRepositoryAdapter::new(
            self.db_pool()
        ));
        
        // Create rate limit configuration
        let rate_limit_config = Arc::new(crate::domain::resource_management::services::RateLimitConfig::default());
        
        // Create the rate limiting service
        let rate_limiting_service = crate::domain::resource_management::services::RateLimitingService::new(
            real_time_cache,
            plan_repository,
            rate_limit_config,
        );
        
        Ok(Arc::new(rate_limiting_service))
    }
}