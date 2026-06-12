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
use crate::infrastructure::storage::S3Storage;
// use crate::infrastructure::adapters::repositories::payment_repository_adapter::PaymentRepositoryAdapter; // Temporarily disabled

use epsx_contracts::notification_port::NotificationPort;

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
    pub s3: Option<Arc<S3Storage>>,
    // Stub for backwards compatibility with admin handlers
    pub user_repo: Option<String>,

    /// Wave 10 / R3: cross-cutting port that the 8 publisher call
    /// sites (payments, chat, admin permissions, plan expiration)
    /// go through instead of the concrete `NotificationService`.
    ///
    /// `None` until the in-process adapter is wired in `bootstrap.rs`
    /// / `simple_container.rs` / `stateless_service_factory.rs`. The
    /// legacy `NotificationService::send` shim returns
    /// `AppError::Configuration` when this is `None`, so a missing
    /// port fails fast instead of silently dropping notifications.
    pub notification_port: Option<Arc<dyn NotificationPort>>,
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

        let s3 = domain_container.s3.clone();

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
            s3,
            user_repo: None,
            // Default to None — the container factories set this in
            // their `create_auth_app_state` after constructing the
            // in-process adapter. A `None` here means the server has
            // not finished initializing; the legacy
            // `NotificationService` shim returns
            // `AppError::Configuration` rather than silently dropping
            // notifications.
            notification_port: None,
        }
    }

    /// Attach a `NotificationPort` to the AppState. Called by the
    /// container factories after they construct the in-process
    /// adapter. Returns a new `AppState` with the port set.
    pub fn with_notification_port(mut self, port: Arc<dyn NotificationPort>) -> Self {
        self.notification_port = Some(port);
        self
    }

    /// Attach an `Option<Arc<dyn NotificationPort>>` to the AppState.
    /// Use this from the container factories where the in-process
    /// adapter construction can fail (missing
    /// `NOTIFICATIONS_DATABASE_URL`); `None` means notifications are
    /// not wired and the publisher call sites will log a warning
    /// instead of writing notifications to the wrong pool.
    pub fn with_notification_port_opt(
        mut self,
        port: Option<Arc<dyn NotificationPort>>,
    ) -> Self {
        self.notification_port = port;
        self
    }
}
