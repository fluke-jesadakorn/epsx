// Database Module - Handles database pool and repository creation
// Focused module following Single Responsibility Principle

use std::sync::Arc;
use crate::app::ports::repositories::*;
use crate::infra::db::diesel::{
    DbPool, create_pool,
    repos::{
        DieselUserRepo, DieselSessionRepo, DieselAuditRepo,
        DieselStockRepo, DieselIamRepo, DieselPermissionProfileRepo,
        StubTemporaryPermissionRepo, StubModuleRepo
    }
};

/// Database module responsible for database pool and repository creation
#[derive(Clone)]
pub struct DatabaseModule {
    pub database_pool: Arc<DbPool>,
    pub user_repo: Arc<dyn UserRepo>,
    pub session_repo: Arc<dyn SessRepo>,
    pub audit_repo: Arc<dyn AuditRepo>,
    pub stock_repo: Arc<dyn StockRepo>,
    pub iam_repo: Arc<dyn IamRepo>,
    pub permission_profile_repo: Arc<dyn PermissionProfileRepo>,
}

impl DatabaseModule {
    /// Create a new database module with the given pool
    pub async fn new(database_pool: Arc<DbPool>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("🔧 Creating repository layer with Diesel...");
        
        // Create all Diesel repositories
        let user_repo = Arc::new(DieselUserRepo::new(database_pool.clone())) as Arc<dyn UserRepo>;
        let session_repo = Arc::new(DieselSessionRepo::new(database_pool.clone())) as Arc<dyn SessRepo>;
        let audit_repo = Arc::new(DieselAuditRepo::new(database_pool.clone())) as Arc<dyn AuditRepo>;
        let stock_repo = Arc::new(DieselStockRepo::new(database_pool.clone())) as Arc<dyn StockRepo>;
        let iam_repo = Arc::new(DieselIamRepo::new(database_pool.clone())) as Arc<dyn IamRepo>;
        let permission_profile_repo = Arc::new(DieselPermissionProfileRepo::new(database_pool.clone())) as Arc<dyn PermissionProfileRepo>;
        
        tracing::info!("✅ Repository layer created successfully");
        
        Ok(DatabaseModule {
            database_pool,
            user_repo,
            session_repo,
            audit_repo,
            stock_repo,
            iam_repo,
            permission_profile_repo,
        })
    }

    /// Create database module from database URL
    pub async fn from_url(database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("🔧 Creating Diesel connection pool...");
        let database_pool = Arc::new(create_pool(database_url).await?);
        tracing::info!("✅ Diesel connection pool created");
        
        Self::new(database_pool).await
    }

    /// Create database module from environment variable
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable is required")?;
        
        Self::from_url(&database_url).await
    }

    /// Create stub repositories for testing and development
    pub fn create_stub_repos(&self) -> (Arc<dyn TemporaryPermissionRepo>, Arc<dyn ModuleRepo>, Arc<dyn UsageRepo>) {
        let temporary_permission_repo = Arc::new(StubTemporaryPermissionRepo::new()) as Arc<dyn TemporaryPermissionRepo>;
        let module_repo = Arc::new(StubModuleRepo::new()) as Arc<dyn ModuleRepo>;
        let usage_repo = Arc::new(crate::infra::db::diesel::repos::DieselUsageRepo::new()) as Arc<dyn UsageRepo>;
        
        (temporary_permission_repo, module_repo, usage_repo)
    }
}