// Application State for Authentication Layer
// Centralized dependency injection for auth handlers and middleware

use std::sync::Arc;
use sqlx::PgPool as DbPool;

use crate::infrastructure::cache::Cache;
use crate::infrastructure::container::DomainContainer;
use crate::infrastructure::redis::RedisPool;
use crate::web::notifications::RedisNotificationBroadcaster;
use crate::infrastructure::adapters::repositories::database_types::PermissionGroupRepository;

/// Application State for Dependency Injection
/// Provides centralized access to infrastructure dependencies for auth layer
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub cache: Arc<dyn Cache>,
    pub domain_container: Arc<DomainContainer>,
    pub redis_pool: Arc<RedisPool>,
    pub redis_broadcaster: Arc<RedisNotificationBroadcaster>,
    pub permission_group_repo: PermissionGroupRepository,
    // Stub for backwards compatibility with admin handlers
    pub user_repo: Option<String>,
}

impl AppState {
    /// Create new AppState with required dependencies
    pub fn new(
        db_pool: Arc<DbPool>,
        cache: Arc<dyn Cache>,
        domain_container: Arc<DomainContainer>,
        redis_pool: Arc<RedisPool>,
        redis_broadcaster: Arc<RedisNotificationBroadcaster>,
    ) -> Self {
        let permission_group_repo = PermissionGroupRepository::new(db_pool.clone());
        Self {
            db_pool,
            cache,
            domain_container,
            redis_pool,
            redis_broadcaster,
            permission_group_repo,
            user_repo: None, // Placeholder for backwards compatibility
        }
    }
}
