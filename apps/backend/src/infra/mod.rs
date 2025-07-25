// Infrastructure layer implementations

pub mod auth;
pub mod cache;
pub mod db;
pub mod repos;
pub mod services;
pub mod events;
pub mod firebase_admin;
pub mod jobs;

// Re-export commonly used implementations
pub use db::{InMemoryLevelHistoryRepo, PostgresUserRepo, PostgresSessRepo, PostgresPayRepo, PostgresStockRepo, PostgresIamRepo, PostgresAuditRepo, PostgresPermissionProfileRepo, DatabasePool, create_pool, DatabaseConfig};
pub use repos::{IamRepoImpl, AuditRepoImpl, PermissionProfileRepoImpl};
pub use services::{SendGridEmailService, MockEmailService, notification::*};
pub use events::{SimpleEventDispatcher};
pub use firebase_admin::FirebaseAdmin;
pub use jobs::{JobScheduler, ExpirationChecker, NotificationService as JobNotificationService};

use std::sync::Arc;
use crate::app::ports::{repositories::*, services::*, events::*};
use crate::core::plugins::{PluginManager, PluginRegistry};
use crate::dom::services::feature_expiration::FeatureExpirationService;

/// Database backend type
#[derive(Debug, Clone)]
pub enum DatabaseBackend {
    PostgreSQL,
}

impl Default for DatabaseBackend {
    fn default() -> Self {
        Self::PostgreSQL
    }
}

/// Factory for creating infrastructure implementations
pub struct InfraFactory {
    pub database_backend: DatabaseBackend,
    pub postgres_pool: DatabasePool,
}

impl InfraFactory {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let database_backend = DatabaseBackend::PostgreSQL;
        let config = DatabaseConfig::default();
        let postgres_pool = create_pool(config).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        Ok(Self {
            database_backend,
            postgres_pool,
        })
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Note: This is a simplified version - in practice you'd handle async properly
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                Self::new()
            )
        })
    }

    // Repository factories
    pub fn create_user_repo(&self) -> Arc<dyn UserRepo> {
        Arc::new(PostgresUserRepo::new(self.postgres_pool.clone()))
    }

    pub fn create_session_repo(&self) -> Arc<dyn SessRepo> {
        Arc::new(PostgresSessRepo::new(self.postgres_pool.clone()))
    }

    pub fn create_payment_repo(&self) -> Arc<dyn PayRepo> {
        Arc::new(PostgresPayRepo::new((*self.postgres_pool).clone()))
    }

    pub fn create_stock_repo(&self) -> Arc<dyn StockRepo> {
        Arc::new(PostgresStockRepo::new((*self.postgres_pool).clone()))
    }

    pub fn create_level_history_repo(&self) -> Arc<dyn LevelHistoryRepo> {
        Arc::new(InMemoryLevelHistoryRepo::new())
    }

    pub fn create_iam_repo(&self) -> Arc<dyn IamRepo> {
        Arc::new(PostgresIamRepo::new((*self.postgres_pool).clone()))
    }

    pub fn create_audit_repo(&self) -> Arc<dyn AuditRepo> {
        Arc::new(PostgresAuditRepo::new((*self.postgres_pool).clone()))
    }

    pub fn create_permission_profile_repo(&self) -> Arc<dyn PermissionProfileRepo> {
        Arc::new(PostgresPermissionProfileRepo::new((*self.postgres_pool).clone()))
    }

    // Service factories

    pub fn create_email_svc(&self) -> Arc<dyn EmailSvc> {
        // Use environment variable to determine which email service to use
        if std::env::var("USE_MOCK_EMAIL").unwrap_or_else(|_| "false".to_string()) == "true" {
            Arc::new(MockEmailService::new())
        } else {
            match SendGridEmailService::from_env() {
                Ok(service) => Arc::new(service),
                Err(_) => {
                    tracing::warn!("SendGrid configuration not found, using mock email service");
                    Arc::new(MockEmailService::new())
                }
            }
        }
    }

    pub fn create_payment_gateway(&self) -> Arc<dyn PayGw> {
        // TODO: Implement PayGw when needed
        unimplemented!("PayGw not yet implemented")
    }

    pub fn create_stock_data_svc(&self) -> Arc<dyn StockDataSvc> {
        // TODO: Implement StockDataSvc when needed
        unimplemented!("StockDataSvc not yet implemented")
    }

    pub fn create_websocket_svc(&self) -> Arc<dyn WebSocketSvc> {
        // TODO: Implement WebSocketSvc when needed
        unimplemented!("WebSocketSvc not yet implemented")
    }

    pub fn create_firebase_admin(&self) -> Result<Arc<FirebaseAdmin>, Box<dyn std::error::Error>> {
        Ok(Arc::new(FirebaseAdmin::new()?))
    }

    pub fn create_notification_service(&self) -> Arc<dyn NotificationService> {
        // Use in-memory service for now, can be replaced with database-backed service
        Arc::new(InMemoryNotificationService::new())
    }

    pub fn create_feature_expiration_service(
        &self,
        user_repo: Arc<dyn UserRepo>,
        permission_profile_repo: Arc<dyn PermissionProfileRepo>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Arc<dyn FeatureExpirationService> {
        use crate::dom::services::feature_expiration::{FeatureExpirationServiceImpl, ExpirationConfig};
        
        Arc::new(FeatureExpirationServiceImpl::new(
            user_repo,
            permission_profile_repo,
            notification_service,
            Some(ExpirationConfig::default()),
        ))
    }

    // Event services
    pub fn create_event_dispatcher(&self) -> Arc<dyn EventDispatcher> {
        Arc::new(SimpleEventDispatcher::new())
    }
    
    // Plugin services
    pub fn create_plugin_manager(&self) -> PluginManager {
        PluginManager::new()
    }
    
    pub fn create_plugin_registry(&self) -> PluginRegistry {
        PluginRegistry::new()
    }
}

/// Application-wide dependency injection container
pub struct AppContainer {
    pub infra: InfraFactory,
    
    // Repositories
    pub user_repo: Arc<dyn UserRepo>,
    pub session_repo: Arc<dyn SessRepo>,
    pub payment_repo: Arc<dyn PayRepo>,
    pub stock_repo: Arc<dyn StockRepo>,
    pub level_history_repo: Arc<dyn LevelHistoryRepo>,
    pub iam_repo: Arc<dyn IamRepo>,
    pub audit_repo: Arc<dyn AuditRepo>,
    pub permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    
    // Services
    pub email_svc: Arc<dyn EmailSvc>,
    pub event_dispatcher: Arc<dyn EventDispatcher>,
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub notification_service: Arc<dyn NotificationService>,
    pub feature_expiration_service: Arc<dyn FeatureExpirationService>,
    
    // Plugin system
    pub plugin_manager: Arc<tokio::sync::Mutex<PluginManager>>,
    pub plugin_registry: Arc<tokio::sync::Mutex<PluginRegistry>>,
}

impl AppContainer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let infra = InfraFactory::from_env()?;
        
        let user_repo = infra.create_user_repo();
        let session_repo = infra.create_session_repo();
        let payment_repo = infra.create_payment_repo();
        let stock_repo = infra.create_stock_repo();
        let level_history_repo = infra.create_level_history_repo();
        let iam_repo = infra.create_iam_repo();
        let audit_repo = infra.create_audit_repo();
        let permission_profile_repo = infra.create_permission_profile_repo();
        let email_svc = infra.create_email_svc();
        let event_dispatcher = infra.create_event_dispatcher();
        let firebase_admin = infra.create_firebase_admin()?;
        let notification_service = infra.create_notification_service();
        let feature_expiration_service = infra.create_feature_expiration_service(
            user_repo.clone(),
            permission_profile_repo.clone(),
            notification_service.clone(),
        );
        let plugin_manager = Arc::new(tokio::sync::Mutex::new(infra.create_plugin_manager()));
        let plugin_registry = Arc::new(tokio::sync::Mutex::new(infra.create_plugin_registry()));
        
        Ok(Self {
            infra,
            user_repo,
            session_repo,
            payment_repo,
            stock_repo,
            level_history_repo,
            iam_repo,
            audit_repo,
            permission_profile_repo,
            email_svc,
            event_dispatcher,
            firebase_admin,
            notification_service,
            feature_expiration_service,
            plugin_manager,
            plugin_registry,
        })
    }
    
    pub fn from_infra(infra: InfraFactory) -> Self {
        let user_repo = infra.create_user_repo();
        let session_repo = infra.create_session_repo();
        let payment_repo = infra.create_payment_repo();
        let stock_repo = infra.create_stock_repo();
        let level_history_repo = infra.create_level_history_repo();
        let iam_repo = infra.create_iam_repo();
        let audit_repo = infra.create_audit_repo();
        let permission_profile_repo = infra.create_permission_profile_repo();
        let email_svc = infra.create_email_svc();
        let event_dispatcher = infra.create_event_dispatcher();
        let firebase_admin = infra.create_firebase_admin().unwrap_or_else(|e| {
            tracing::warn!("Failed to create Firebase Admin: {}, using mock", e);
            Arc::new(FirebaseAdmin::new().unwrap())
        });
        let notification_service = infra.create_notification_service();
        let feature_expiration_service = infra.create_feature_expiration_service(
            user_repo.clone(),
            permission_profile_repo.clone(),
            notification_service.clone(),
        );
        let plugin_manager = Arc::new(tokio::sync::Mutex::new(infra.create_plugin_manager()));
        let plugin_registry = Arc::new(tokio::sync::Mutex::new(infra.create_plugin_registry()));
        
        Self {
            infra,
            user_repo,
            session_repo,
            payment_repo,
            stock_repo,
            level_history_repo,
            iam_repo,
            audit_repo,
            permission_profile_repo,
            email_svc,
            event_dispatcher,
            firebase_admin,
            notification_service,
            feature_expiration_service,
            plugin_manager,
            plugin_registry,
        }
    }
    
    /// Initialize and start all plugins
    pub async fn initialize_plugins(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::core::plugin_examples::{
            SimpleAnalysisPlugin, MockDataProviderPlugin, EmailNotificationPlugin
        };
        use crate::core::plugins::PluginConfig;
        use std::collections::HashMap;
        
        let mut plugin_manager = self.plugin_manager.lock().await;
        
        // Create plugin configurations
        let mut settings = HashMap::new();
        settings.insert("environment".to_string(), serde_json::Value::String("development".to_string()));
        
        let config = PluginConfig {
            enabled: true,
            settings,
            environment: "development".to_string(),
        };
        
        // Register example plugins
        plugin_manager.register_plugin(
            Arc::new(SimpleAnalysisPlugin::new()),
            config.clone(),
        ).await?;
        
        plugin_manager.register_plugin(
            Arc::new(MockDataProviderPlugin::new()),
            config.clone(),
        ).await?;
        
        plugin_manager.register_plugin(
            Arc::new(EmailNotificationPlugin::new()),
            config.clone(),
        ).await?;
        
        // Initialize and start all plugins
        plugin_manager.initialize_all().await?;
        plugin_manager.start_all().await?;
        
        tracing::info!("Plugin system initialized with {} plugins", plugin_manager.list_plugins().len());
        Ok(())
    }
    
    /// Stop all plugins gracefully
    pub async fn shutdown_plugins(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut plugin_manager = self.plugin_manager.lock().await;
        plugin_manager.stop_all().await?;
        tracing::info!("Plugin system shut down");
        Ok(())
    }
    
}