// Application State for Authentication Layer
// Centralized dependency injection for auth handlers and middleware

use std::sync::Arc;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};

use crate::infrastructure::cache::Cache;
use crate::infrastructure::container::DomainContainer;
use crate::infrastructure::redis::RedisPool;
use crate::web::notifications::RedisNotificationBroadcaster;
use crate::infrastructure::adapters::repositories::permission_group_repository_adapter::PermissionGroupRepositoryAdapter;
// use crate::infrastructure::adapters::repositories::payment_repository_adapter::PaymentRepositoryAdapter; // Temporarily disabled

/// Application State for Dependency Injection
/// Provides centralized access to infrastructure dependencies for auth layer
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<&'static Pool<AsyncPgConnection>>,
    pub cache: Arc<dyn Cache>,
    pub domain_container: Arc<DomainContainer>,
    pub redis_pool: Option<Arc<RedisPool>>,
    pub redis_broadcaster: Option<Arc<RedisNotificationBroadcaster>>,
    pub permission_group_repo: Arc<PermissionGroupRepositoryAdapter>,
    // pub payment_repository: Arc<PaymentRepositoryAdapter>, // Temporarily disabled
    // Stub for backwards compatibility with admin handlers
    pub user_repo: Option<String>,
}

impl AppState {
    /// Create new AppState with required dependencies
    /// Redis pool and broadcaster are optional - if not provided, notifications won't work
    pub fn new(
        db_pool: Arc<&'static Pool<AsyncPgConnection>>,
        cache: Arc<dyn Cache>,
        domain_container: Arc<DomainContainer>,
        redis_pool: Option<Arc<RedisPool>>,
        redis_broadcaster: Option<Arc<RedisNotificationBroadcaster>>,
    ) -> Self {
        // Diesel-based repository
        let permission_group_repo = domain_container
            .permission_group_repository
            .clone()
            .expect("PermissionGroupRepository not initialized in DomainContainer");

        // let _payment_repository = domain_container.payment_repository.clone(); // Temporarily disabled - field removed

        Self {
            db_pool,
            cache,
            domain_container,
            redis_pool,
            redis_broadcaster,
            permission_group_repo,
            // payment_repository, // Temporarily disabled
            user_repo: None, // Placeholder for backwards compatibility
        }
    }
}
