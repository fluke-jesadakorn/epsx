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

use crate::domain::payment::repository_ports::payment_context_port::PaymentContextRepositoryPort;
use crate::domain::payment::repository_ports::subscription_port::SubscriptionRepositoryPort;

use epsx_contracts::notification_port::NotificationPort;

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

    /// Wave 11 / Track B: payment-context (V2 dynamic payment
    /// link) port. Used by the public
    /// `GET /api/public/payment-links/{slug}` route and the
    /// admin `web/payments/payment_link_handlers.rs` CRUD.
    /// In-process impl is the existing
    /// `PaymentContextRepositoryAdapter` (see
    /// `infrastructure::adapters::repositories::payment_context_repository_adapter`).
    /// `None` means the container did not wire the adapter; the
    /// handlers return `503 SERVICE_UNAVAILABLE` rather than
    /// silently 500.
    pub payment_context_repository_port: Option<Arc<dyn PaymentContextRepositoryPort>>,

    /// Wave 11 / Track B: subscription port. Backed by the
    /// `PaymentSubscriptionRepositoryAdapter` (see
    /// `infrastructure::adapters::repositories::payment::subscription_repository_adapter`).
    /// Used by the admin plans editor and the
    /// market_analytics `get_stock_ranking_assignments` query
    /// (the audit's row-4 leak closed in this track).
    /// `None` means the container did not wire the adapter.
    pub subscription_repository_port: Option<Arc<dyn SubscriptionRepositoryPort>>,
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
            // Default to None — the container factories set this in
            // their `create_auth_app_state` after constructing the
            // in-process adapter. A `None` here means the server has
            // not finished initializing; the legacy
            // `NotificationService` shim returns
            // `AppError::Configuration` rather than silently dropping
            // notifications.
            notification_port: None,
            // wave11(track-b) ports. Default to None; the
            // container factories wire them after constructing
            // the in-process adapters. A `None` value means the
            // server has not finished initializing; the handler
            // paths return 503 (payment-link) or fall back to
            // the legacy shim (subscription) rather than
            // panicking.
            payment_context_repository_port: None,
            subscription_repository_port: None,
        }
    }

    /// Attach a `PaymentContextRepositoryPort` to the AppState.
    ///
    /// Called by the container factories after they construct
    /// the in-process `PaymentContextRepositoryAdapter`.
    /// `web/payments/payment_link_handlers::get_port` reads
    /// this field; a missing port fails fast with `503`.
    pub fn with_payment_context_repository_port(
        mut self,
        port: Arc<dyn PaymentContextRepositoryPort>,
    ) -> Self {
        self.payment_context_repository_port = Some(port);
        self
    }

    /// Attach an `Option<Arc<dyn PaymentContextRepositoryPort>>`
    /// to the AppState. Use this from the container factories
    /// where the in-process adapter construction can fail
    /// (missing `PAYMENTS_DATABASE_URL`); `None` means the
    /// payment-link handlers will return 503.
    pub fn with_payment_context_repository_port_opt(
        mut self,
        port: Option<Arc<dyn PaymentContextRepositoryPort>>,
    ) -> Self {
        self.payment_context_repository_port = port;
        self
    }

    /// Attach a `SubscriptionRepositoryPort` to the AppState.
    ///
    /// Called by the container factories after they construct
    /// the in-process `PaymentSubscriptionRepositoryAdapter`.
    /// The market_analytics stock-ranking-assignments query
    /// reads this field; a missing port surfaces as a 500
    /// (the handler still runs, but the query path returns
    /// empty).
    pub fn with_subscription_repository_port(
        mut self,
        port: Arc<dyn SubscriptionRepositoryPort>,
    ) -> Self {
        self.subscription_repository_port = Some(port);
        self
    }

    /// Attach an `Option<Arc<dyn SubscriptionRepositoryPort>>`
    /// to the AppState. Use this from the container factories
    /// where the in-process adapter construction can fail
    /// (missing `PAYMENTS_DATABASE_URL`); `None` means the
    /// stock-ranking query returns empty.
    pub fn with_subscription_repository_port_opt(
        mut self,
        port: Option<Arc<dyn SubscriptionRepositoryPort>>,
    ) -> Self {
        self.subscription_repository_port = port;
        self
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
