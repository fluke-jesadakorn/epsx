use crate::prelude::TlsPool;
// Application State for Authentication Layer
// Centralized dependency injection for auth handlers and middleware

use std::sync::Arc;

use crate::infrastructure::cache::Cache;
use crate::infrastructure::container::DomainContainer;
use crate::infrastructure::redis::RedisPool;
use crate::infrastructure::services::audit_service::AuditService;
use crate::web::notifications::RedisNotificationBroadcaster;
use crate::domain::payment::repository_ports::{TransactionHistoryProvider};
use crate::domain::auth::ports::IdentityProviderPort;

use crate::infrastructure::adapters::repositories::permission_plan_repository_adapter::PermissionPlanRepositoryAdapter;
// use crate::infrastructure::adapters::repositories::payment_repository_adapter::PaymentRepositoryAdapter; // Temporarily disabled

/// Application State for Dependency Injection
/// Provides centralized access to infrastructure dependencies for auth layer
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<&'static TlsPool>,
    pub cache: Arc<dyn Cache>,
    pub domain_container: Arc<DomainContainer>,
    pub redis_pool: Option<Arc<RedisPool>>,
    pub redis_broadcaster: Option<Arc<RedisNotificationBroadcaster>>,
    pub plan_repo: Arc<PermissionPlanRepositoryAdapter>,
    pub transaction_history_provider: Option<Arc<dyn TransactionHistoryProvider>>,
    pub identity_provider: Option<Arc<dyn IdentityProviderPort>>,

    pub analytics_db_pool: Option<Arc<&'static TlsPool>>,
    pub audit: Arc<AuditService>,
    // pub payment_repository: Arc<PaymentRepositoryAdapter>, // Temporarily disabled
    // Stub for backwards compatibility with admin handlers
    pub user_repo: Option<String>,
}

impl AppState {
    /// Create new AppState with required dependencies
    /// Redis pool and broadcaster are optional - if not provided, notifications won't work
    pub fn new(
        db_pool: Arc<&'static TlsPool>,
        cache: Arc<dyn Cache>,
        domain_container: Arc<DomainContainer>,
        redis_pool: Option<Arc<RedisPool>>,
        redis_broadcaster: Option<Arc<RedisNotificationBroadcaster>>,
        analytics_db_pool: Option<Arc<&'static TlsPool>>,
    ) -> Self {
        // Diesel-based repository
        let plan_repo = domain_container
            .permission_plan_repository
            .clone()
            .expect("PermissionPlanRepository not initialized in DomainContainer");

        // let _payment_repository = domain_container.payment_repository.clone(); // Temporarily disabled - field removed

        let transaction_history_provider = domain_container.transaction_history_provider.clone();
        let identity_provider = domain_container.identity_provider.clone();

        let audit_pool = analytics_db_pool.clone().unwrap_or_else(|| db_pool.clone());
        let audit = Arc::new(AuditService::new(audit_pool));

        Self {
            db_pool,
            cache,
            domain_container,
            redis_pool,
            redis_broadcaster,
            plan_repo,
            transaction_history_provider,
            identity_provider,

            analytics_db_pool,
            audit,
            // payment_repository, // Temporarily disabled
            user_repo: None, // Placeholder for backwards compatibility
        }
    }
}
