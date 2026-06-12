use crate::prelude::TlsPool;
// Application State for Authentication Layer
// Centralized dependency injection for auth handlers and middleware

use std::sync::Arc;

use crate::infrastructure::cache::Cache;
use crate::infrastructure::container::DomainContainer;
use crate::infrastructure::redis::RedisPool;
use crate::infrastructure::services::audit_service::AuditService;
use crate::domain::payment::repository_ports::{TransactionHistoryProvider};
use crate::domain::auth::ports::IdentityProviderPort;
use epsx_contracts::pubsub_port::PubsubPort;

use crate::infrastructure::adapters::repositories::permission_plan_repository_adapter::PermissionPlanRepositoryAdapter;
use crate::infrastructure::storage::S3Storage;
// use crate::infrastructure::adapters::repositories::payment_repository_adapter::PaymentRepositoryAdapter; // Temporarily disabled

/// Application State for Dependency Injection
/// Provides centralized access to infrastructure dependencies for auth layer
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<&'static TlsPool>,
    pub cache: Arc<dyn Cache>,
    pub domain_container: Arc<DomainContainer>,
    pub redis_pool: Option<Arc<RedisPool>>,
    /// Generic pubsub port. Notifications + chat both publish/subscribe
    /// through this single port. Hoisting the previous
    /// notifications-typed `RedisNotificationBroadcaster` to this
    /// kernel-level `PubsubPort` was wave-10 R2 — see
    /// `docs/wave8-service-boundary/ROADMAP.md` §5 R2 and
    /// `docs/wave8-service-boundary/audit-notifications.md` §3c.
    pub pubsub: Option<Arc<dyn PubsubPort>>,
    pub plan_repo: Arc<PermissionPlanRepositoryAdapter>,
    pub transaction_history_provider: Option<Arc<dyn TransactionHistoryProvider>>,
    pub identity_provider: Option<Arc<dyn IdentityProviderPort>>,

    pub analytics_db_pool: Option<Arc<&'static TlsPool>>,
    pub audit: Arc<AuditService>,
    pub s3: Option<Arc<S3Storage>>,
    // Stub for backwards compatibility with admin handlers
    pub user_repo: Option<String>,
}

impl AppState {
    /// Create new AppState with required dependencies
    /// Redis pool and pubsub port are optional - if not provided, real-time features won't work
    pub fn new(
        db_pool: Arc<&'static TlsPool>,
        cache: Arc<dyn Cache>,
        domain_container: Arc<DomainContainer>,
        redis_pool: Option<Arc<RedisPool>>,
        pubsub: Option<Arc<dyn PubsubPort>>,
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

        let s3 = domain_container.s3.clone();

        Self {
            db_pool,
            cache,
            domain_container,
            redis_pool,
            pubsub,
            plan_repo,
            transaction_history_provider,
            identity_provider,

            analytics_db_pool,
            audit,
            s3,
            user_repo: None,
        }
    }
}
