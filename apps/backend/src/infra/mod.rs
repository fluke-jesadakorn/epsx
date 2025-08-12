// Infrastructure layer implementations

// pub mod auth;
pub mod cache;
pub mod db;
// pub mod repos;
pub mod services;
pub mod events;
pub mod firebase_admin;
pub mod jobs;

// Re-export commonly used implementations
pub use db::{PostgresUserRepo, PostgresSessRepo, PostgresPayRepo, PostgresStockRepo, PostgresIamRepo, PostgresAuditRepo, PostgresPermissionProfileRepo, PostgresTemporaryPermissionRepo, PostgresEPSRepository, DatabasePool, create_pool, DatabaseConfig};
// pub use repos::{IamRepoImpl, AuditRepoImpl, PermissionProfileRepoImpl};
pub use services::{SendGridEmailService, MockEmailService, notification::*};
pub use events::{SimpleEventDispatcher};
pub use firebase_admin::FirebaseAdmin;
pub use jobs::{JobScheduler, ExpirationChecker, NotificationService as JobNotificationService};

use std::sync::Arc;
use crate::app::ports::{repositories::*, services::*, events::*};
use crate::dom::ports::NotificationPort;
// use crate::core::plugins::{PluginManager, PluginRegistry};
use crate::dom::services::feature_expiration::FeatureExpirationService;
use crate::dom::services::eps_ranking_service::EPSRankingService;
use crate::infra::db::MigrationRunner;

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
        Arc::new(crate::infra::db::InMemoryLevelHistoryRepo::new())
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

    pub fn create_temporary_permission_repo(&self) -> Arc<dyn TemporaryPermissionRepo> {
        Arc::new(PostgresTemporaryPermissionRepo::new((*self.postgres_pool).clone()))
    }

    pub fn create_eps_repo(&self) -> Arc<PostgresEPSRepository> {
        Arc::new(PostgresEPSRepository::new(self.postgres_pool.clone()))
    }

    pub fn create_eps_ranking_service(&self) -> Arc<EPSRankingService> {
        let eps_repo = self.create_eps_repo();
        Arc::new(EPSRankingService::new(eps_repo))
    }

    // Service factories

    pub fn create_email_svc(&self, config: Arc<crate::config::Config>) -> Arc<dyn EmailSvc> {
        // Use environment variable to determine which email service to use
        if std::env::var("USE_MOCK_EMAIL").unwrap_or_else(|_| "false".to_string()) == "true" {
            Arc::new(MockEmailService::new())
        } else {
            if config.email.enabled && !config.email.sendgrid_api_key.is_empty() {
                Arc::new(SendGridEmailService::from_config(config))
            } else {
                tracing::warn!("SendGrid configuration not found or disabled, using mock email service");
                Arc::new(MockEmailService::new())
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

    pub async fn create_firebase_admin(&self) -> Result<Arc<FirebaseAdmin>, Box<dyn std::error::Error>> {
        Ok(Arc::new(FirebaseAdmin::new().await?))
    }

    pub fn create_notification_service(&self) -> Arc<dyn NotificationService> {
        // Use in-memory service for now, can be replaced with database-backed service
        Arc::new(InMemoryNotificationService::new())
    }

    pub fn create_feature_expiration_service(
        &self,
        user_repo: Arc<dyn UserRepo>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Arc<dyn FeatureExpirationService> {
        use crate::dom::services::feature_expiration::{FeatureExpirationServiceImpl, ExpirationConfig};
        use crate::infra::services::NotificationPortAdapter;
        
        // Create adapter to convert NotificationService to NotificationPort
        let notification_port: Arc<dyn NotificationPort> = Arc::new(
            NotificationPortAdapter::new(notification_service)
        );
        
        Arc::new(FeatureExpirationServiceImpl::new(
            user_repo,
            notification_port,
            Some(ExpirationConfig::default()),
        ))
    }

    // Event services
    pub fn create_event_dispatcher(&self) -> Arc<dyn EventDispatcher> {
        Arc::new(SimpleEventDispatcher::new())
    }
    
    // Plugin services
    // pub fn create_plugin_manager(&self) -> PluginManager {
    //     PluginManager::new()
    // }
    
    // pub fn create_plugin_registry(&self) -> PluginRegistry {
    //     PluginRegistry::new()
    // }
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
    pub temporary_permission_repo: Arc<dyn TemporaryPermissionRepo>,
    
    // Services
    pub email_svc: Arc<dyn EmailSvc>,
    pub event_dispatcher: Arc<dyn EventDispatcher>,
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub notification_service: Arc<dyn NotificationService>,
    pub feature_expiration_service: Arc<dyn FeatureExpirationService>,
    
    // Plugin system
    // pub plugin_manager: Arc<tokio::sync::Mutex<PluginManager>>,
    // pub plugin_registry: Arc<tokio::sync::Mutex<PluginRegistry>>,
}

impl AppContainer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(crate::config::Config::from_env());
        let infra = InfraFactory::from_env()?;
        
        // Run database migrations automatically
        // Self::run_migrations(&infra.postgres_pool).await?; // Temporarily disabled
        
        let user_repo = infra.create_user_repo();
        let session_repo = infra.create_session_repo();
        let payment_repo = infra.create_payment_repo();
        let stock_repo = infra.create_stock_repo();
        let level_history_repo = infra.create_level_history_repo();
        let iam_repo = infra.create_iam_repo();
        let audit_repo = infra.create_audit_repo();
        let permission_profile_repo = infra.create_permission_profile_repo();
        let temporary_permission_repo = infra.create_temporary_permission_repo();
        let email_svc = infra.create_email_svc(config.clone());
        let event_dispatcher = infra.create_event_dispatcher();
        let firebase_admin = infra.create_firebase_admin().await?;
        let notification_service = infra.create_notification_service();
        let feature_expiration_service = infra.create_feature_expiration_service(
            user_repo.clone(),
            notification_service.clone(),
        );
        // let plugin_manager = Arc::new(tokio::sync::Mutex::new(infra.create_plugin_manager()));
        // let plugin_registry = Arc::new(tokio::sync::Mutex::new(infra.create_plugin_registry()));
        
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
            temporary_permission_repo,
            email_svc,
            event_dispatcher,
            firebase_admin,
            notification_service,
            feature_expiration_service,
            // plugin_manager,
            // plugin_registry,
        })
    }
    
    pub async fn from_infra(infra: InfraFactory) -> Self {
        let config = Arc::new(crate::config::Config::from_env());
        let user_repo = infra.create_user_repo();
        let session_repo = infra.create_session_repo();
        let payment_repo = infra.create_payment_repo();
        let stock_repo = infra.create_stock_repo();
        let level_history_repo = infra.create_level_history_repo();
        let iam_repo = infra.create_iam_repo();
        let audit_repo = infra.create_audit_repo();
        let permission_profile_repo = infra.create_permission_profile_repo();
        let temporary_permission_repo = infra.create_temporary_permission_repo();
        let email_svc = infra.create_email_svc(config.clone());
        let event_dispatcher = infra.create_event_dispatcher();
        let firebase_admin = infra.create_firebase_admin().await.unwrap_or_else(|e| {
            tracing::warn!("Failed to create Firebase Admin: {}, creating default", e);
            // Create a default/mock Firebase admin instance
            use crate::infra::firebase_admin::FirebaseAdmin;
            Arc::new(FirebaseAdmin {
                client: reqwest::Client::new(),
                project_id: "default-project".to_string(),
                service_account_key: None,
                jwks_cache: std::collections::HashMap::new(),
                jwks_cache_expiry: chrono::Utc::now(),
            })
        });
        let notification_service = infra.create_notification_service();
        let feature_expiration_service = infra.create_feature_expiration_service(
            user_repo.clone(),
            notification_service.clone(),
        );
        // let plugin_manager = Arc::new(tokio::sync::Mutex::new(infra.create_plugin_manager()));
        // let plugin_registry = Arc::new(tokio::sync::Mutex::new(infra.create_plugin_registry()));
        
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
            temporary_permission_repo,
            email_svc,
            event_dispatcher,
            firebase_admin,
            notification_service,
            feature_expiration_service,
            // plugin_manager,
            // plugin_registry,
        }
    }
    
    /// Initialize and start all plugins
    // pub async fn initialize_plugins(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     use crate::core::plugin_examples::{
    //         SimpleAnalysisPlugin, MockDataProviderPlugin, EmailNotificationPlugin
    //     };
    //     use crate::core::plugins::PluginConfig;
    //     use std::collections::HashMap;
    //     
    //     let mut plugin_manager = self.plugin_manager.lock().await;
    //     
    //     // Create plugin configurations
    //     let mut settings = HashMap::new();
    //     settings.insert("environment".to_string(), serde_json::Value::String("development".to_string()));
    //     
    //     let config = PluginConfig {
    //         enabled: true,
    //         settings,
    //         environment: "development".to_string(),
    //     };
    //     
    //     // Register example plugins
    //     plugin_manager.register_plugin(
    //         Arc::new(SimpleAnalysisPlugin::new()),
    //         config.clone(),
    //     ).await?;
    //     
    //     plugin_manager.register_plugin(
    //         Arc::new(MockDataProviderPlugin::new()),
    //         config.clone(),
    //     ).await?;
    //     
    //     plugin_manager.register_plugin(
    //         Arc::new(EmailNotificationPlugin::new()),
    //         config.clone(),
    //     ).await?;
    //     
    //     // Initialize and start all plugins
    //     plugin_manager.initialize_all().await?;
    //     plugin_manager.start_all().await?;
    //     
    //     tracing::info!("Plugin system initialized with {} plugins", plugin_manager.list_plugins().len());
    //     Ok(())
    // }
    
    /// Stop all plugins gracefully
    // pub async fn shutdown_plugins(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut plugin_manager = self.plugin_manager.lock().await;
    //     plugin_manager.stop_all().await?;
    //     tracing::info!("Plugin system shut down");
    //     Ok(())
    // }
    
    /// Create database if it doesn't exist and run migrations automatically
    async fn run_migrations(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
        // First, try to create the database if it doesn't exist
        Self::ensure_database_exists().await?;
        
        tracing::info!("Running database migrations...");
        
        let migrations_dir = std::env::var("MIGRATIONS_DIR")
            .unwrap_or_else(|_| "migrations".to_string());
            
        let runner = MigrationRunner::new((**pool).clone(), migrations_dir);
        
        match runner.migrate().await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("✅ Applied {} migrations successfully", count);
                } else {
                    tracing::info!("✅ Database is up to date");
                }
                Ok(())
            }
            Err(e) => {
                tracing::error!("❌ Migration failed: {}", e);
                Err(Box::new(e))
            }
        }
    }
    
    /// Ensure database exists by connecting to postgres and creating it if needed
    async fn ensure_database_exists() -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
        use std::str::FromStr;
        
        let database_url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("POSTGRES_URL"))
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost/epsx_db".to_string());
        
        // Parse the database URL to get connection details
        let opts = PgConnectOptions::from_str(&database_url)?;
        let db_name = opts.get_database().unwrap_or("epsx_db");
        
        // Extract connection details to build master URL
        let master_url = database_url.replace(&format!("/{}", db_name), "/postgres");
        
        tracing::info!("Checking if database '{}' exists...", db_name);
        
        // Connect to master postgres database
        let master_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&master_url)
            .await?;
        
        // Check if database exists
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)"
        )
        .bind(db_name)
        .fetch_one(&master_pool)
        .await?;
        
        if !exists {
            tracing::info!("Database '{}' does not exist, creating it...", db_name);
            
            // Create the database
            let create_query = format!("CREATE DATABASE \"{}\"", db_name);
            sqlx::query(&create_query)
                .execute(&master_pool)
                .await?;
            
            tracing::info!("✅ Database '{}' created successfully", db_name);
        } else {
            tracing::info!("✅ Database '{}' already exists", db_name);
        }
        
        master_pool.close().await;
        Ok(())
    }
    
}