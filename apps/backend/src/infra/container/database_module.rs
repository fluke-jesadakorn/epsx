// Database Module - Handles database pool and repository creation
// Focused module following Single Responsibility Principle

use std::sync::Arc;
use crate::app::ports::repositories::*;
use crate::infra::db::diesel::{
    DbPool, create_pool,
    repos::{
        DieselUserRepository, DieselUserPermissionRepository, DieselSessionRepository, DieselAuditRepository,
        RefreshTokenRepository, RevokedTokenRepository, UserNotificationRepository,
    }
};

/// Database module responsible for database pool and repository creation
#[derive(Clone)]
pub struct DatabaseModule {
    pub database_pool: Arc<DbPool>,
    pub user_repo: Arc<dyn UserRepository>,
    pub user_permission_repo: Arc<dyn UserPermissionRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub audit_repo: Arc<dyn AuditRepository>,
    pub refresh_token_repo: Arc<RefreshTokenRepository>,
    pub revoked_token_repo: Arc<RevokedTokenRepository>,
    pub user_notification_repo: Arc<UserNotificationRepository>,
}

impl DatabaseModule {
    /// Create a new database module with the given pool
    pub async fn new(database_pool: Arc<DbPool>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("🔧 Creating repository layer with Diesel...");
        
        // Create all Diesel repositories
        let user_repo = Arc::new(DieselUserRepository::new(database_pool.clone())) as Arc<dyn UserRepository>;
        let user_permission_repo = Arc::new(DieselUserPermissionRepository::new(database_pool.clone())) as Arc<dyn UserPermissionRepository>;
        let session_repo = Arc::new(DieselSessionRepository::new(database_pool.clone())) as Arc<dyn SessionRepository>;
        let audit_repo = Arc::new(DieselAuditRepository::new(database_pool.clone())) as Arc<dyn AuditRepository>;
        let refresh_token_repo = Arc::new(RefreshTokenRepository::new(database_pool.clone()));
        let revoked_token_repo = Arc::new(RevokedTokenRepository::new(database_pool.clone()));
        let user_notification_repo = Arc::new(UserNotificationRepository::new(database_pool.clone()));
        
        tracing::info!("✅ Repository layer created successfully with user permissions and notifications");
        
        Ok(DatabaseModule {
            database_pool,
            user_repo,
            user_permission_repo,
            session_repo,
            audit_repo,
            refresh_token_repo,
            revoked_token_repo,
            user_notification_repo,
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

}