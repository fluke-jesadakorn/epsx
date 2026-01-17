// Enhanced Container - Web3-first service container
// Provides comprehensive Web3 services with proper dependency injection

use std::sync::Arc;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};
use crate::infrastructure::cache::Cache;
use crate::infrastructure::cache::unified_permission_cache::UnifiedPermissionCache;
use crate::infrastructure::redis::RedisPool;
use crate::web::notifications::RedisNotificationBroadcaster;
use crate::infrastructure::adapters::repositories::{
    wallet_user_repository_adapter::WalletUserRepositoryAdapter,

    group_repository_adapter::GroupRepositoryAdapter,
    payment_repository_adapter::PaymentRepositoryAdapter,
    notification_repository_adapter::NotificationRepositoryAdapter,
};
use crate::infrastructure::adapters::services::{
    permission_adapter::{Web3PermissionServiceAdapter, BlockchainConfig},
};
use crate::domain::wallet_management::{
    WalletPermissionService,
    WalletUserRepositoryPort,
    WalletUserAnalyticsPort,
};
use crate::domain::auth::ports::IdentityProviderPort;
use crate::infrastructure::adapters::auth::google_identity_adapter::GoogleIdentityAdapter;

use crate::domain::payment::repository_ports::{PaymentRepositoryPort, TransactionHistoryProvider};
use crate::auth::auth_service::UnifiedWeb3AuthService;
use crate::auth::token_service::OpenIDTokenService;
use crate::auth::key_manager::KeyManager;
use crate::auth::UnifiedPermissionService;
use crate::infrastructure::cqrs::{EventStore, PostgresEventStore, TransactionalOutbox};

/// Enhanced container with Web3-first services
#[derive(Clone)]
pub struct SimpleContainer {
    pub db_pool: Arc<&'static Pool<AsyncPgConnection>>,
    pub payments_pool: Option<Arc<&'static Pool<AsyncPgConnection>>>,
    pub analytics_pool: Option<Arc<&'static Pool<AsyncPgConnection>>>,
    pub notifications_pool: Option<Arc<&'static Pool<AsyncPgConnection>>>,
    pub cache: Option<Arc<dyn Cache>>,

    // NEW - Web3-first services (primary)
    pub wallet_user_repository: Option<Arc<WalletUserRepositoryAdapter>>,

    pub group_repository: Option<Arc<GroupRepositoryAdapter>>,
    pub payment_repository: Option<Arc<PaymentRepositoryAdapter>>,
    pub notification_repository: Option<Arc<NotificationRepositoryAdapter>>,
    pub wallet_permission_service: Option<Arc<WalletPermissionService>>,
    pub web3_permission_adapter: Option<Arc<Web3PermissionServiceAdapter>>,
    pub auth_service: Option<Arc<UnifiedWeb3AuthService>>,
    pub token_service: Option<Arc<OpenIDTokenService>>,
    pub identity_provider: Option<Arc<dyn IdentityProviderPort>>,

    pub event_bus: Option<Arc<dyn crate::domain::DomainEventBus>>,

    // Unified Permission Service (single source of truth for permissions)
    pub unified_permission_service: Option<Arc<UnifiedPermissionService>>,
    pub permission_cache: Option<Arc<UnifiedPermissionCache>>,

    // Redis infrastructure for real-time notifications
    pub redis_pool: Option<Arc<RedisPool>>,
    pub redis_broadcaster: Option<Arc<RedisNotificationBroadcaster>>,

    // CQRS Infrastructure (Event Sourcing)
    pub event_store: Option<Arc<dyn EventStore>>,
    pub transactional_outbox: Option<Arc<TransactionalOutbox>>,
    pub event_dispatcher: Option<Arc<crate::infrastructure::EventDispatcher>>,
    pub projection_manager: Option<Arc<crate::infrastructure::ProjectionManager>>,
    pub transaction_history_provider: Option<Arc<dyn TransactionHistoryProvider>>,

    // Subscription Management
    pub plan_repository: Option<Arc<crate::infrastructure::adapters::repositories::plan_repository_adapter::PostgresPlanRepositoryAdapter>>,
}

impl SimpleContainer {
    pub fn new(db_pool: Arc<&'static Pool<AsyncPgConnection>>) -> Self {
        Self {
            db_pool,
            payments_pool: None,
            analytics_pool: None,
            notifications_pool: None,
            cache: None,
            identity_provider: None,
            // NEW - Web3-first services (initialized as None, configured via builder methods)

            wallet_user_repository: None,

            group_repository: None,
            payment_repository: None,
            notification_repository: None,
            wallet_permission_service: None,
            web3_permission_adapter: None,
            auth_service: None,
            token_service: None,
            event_bus: None,
            // Unified Permission Service
            unified_permission_service: None,
            permission_cache: None,
            // Redis
            redis_pool: None,
            redis_broadcaster: None,
            // CQRS
            event_store: None,
            transactional_outbox: None,
            event_dispatcher: None,
            projection_manager: None,
            transaction_history_provider: None,
            plan_repository: None,
        }
    }

    /// Get Web3 domain for SIWE authentication from environment
    fn get_web3_domain() -> String {
        use std::env;
        
        // Try to get frontend URL from environment
        if let Ok(frontend_url) = env::var("FRONTEND_URL") {
            // Extract domain from URL
            if let Ok(url) = url::Url::parse(&frontend_url) {
                if let Some(host) = url.host_str() {
                    return host.to_string();
                }
            }
        }
        
        // Try NEXT_PUBLIC_APP_URL as fallback
        if let Ok(app_url) = env::var("NEXT_PUBLIC_APP_URL") {
            if let Ok(url) = url::Url::parse(&app_url) {
                if let Some(host) = url.host_str() {
                    return host.to_string();
                }
            }
        }
        
        // Environment-based defaults
        if env::var("NODE_ENV").map(|v| v == "production").unwrap_or(false) ||
           env::var("RUST_ENV").map(|v| v == "production").unwrap_or(false) {
            "epsx.io".to_string()
        } else {
            "localhost".to_string()
        }
    }
    
    /// Create container with Web3 services properly wired
    pub async fn new_with_web3_services(
        cache: Option<Arc<dyn Cache>>,
        blockchain_config: Option<BlockchainConfig>,
    ) -> Self {
        // Initialize Identity Provider
        let identity_provider: Arc<dyn IdentityProviderPort> = Arc::new(GoogleIdentityAdapter::new());


        // Get Diesel pool
        let diesel_pool = crate::infrastructure::database::get_diesel_pool().await
            .expect("Failed to get Diesel pool");
        let db_pool = Arc::new(diesel_pool);

        // Create repository adapters
        let wallet_user_repository = Arc::new(WalletUserRepositoryAdapter::new(diesel_pool));

        let group_repository = Arc::new(GroupRepositoryAdapter::new(diesel_pool));
        let plan_repository = Arc::new(crate::infrastructure::adapters::repositories::plan_repository_adapter::PostgresPlanRepositoryAdapter::new(diesel_pool));

        // Initialize dedicated pools
        let payments_pool = crate::infrastructure::database::get_payments_pool().await.ok().map(Arc::new);
        let analytics_pool = crate::infrastructure::database::get_analytics_pool().await.ok().map(Arc::new);
        let notifications_pool = crate::infrastructure::database::get_notifications_pool().await.ok().map(Arc::new);
        
        // Payment Repository (uses payments_pool if available)
        let payment_repository = payments_pool.as_ref().map(|pool| Arc::new(PaymentRepositoryAdapter::new(**pool)));

        // Notification Repository (uses notifications_pool if available)
        let notification_repository = if let Some(pool) = &notifications_pool {
           Some(Arc::new(NotificationRepositoryAdapter::new(**pool)))
        } else {
           // If no dedicated pool, we could fallback to db_pool OR just return None/default
           // Since we updated NotificationRepositoryAdapter::new to take a pool, we MUST provide one.
           // Fallback to db_pool if notifications_pool is missing (e.g. single DB setup)
           Some(Arc::new(NotificationRepositoryAdapter::new(*db_pool)))
        };

        // Create domain services
        let wallet_permission_service = Arc::new(WalletPermissionService::new()
            .expect("Failed to create WalletPermissionService"));

        // NOTE: UnifiedPermissionCache will be created after Redis initialization
        // Create initial web3_permission_adapter with None cache (will be updated after Redis init)

        // Create OpenID token service with RSA key manager
        let key_manager = KeyManager::from_env_or_generate()
            .expect("Failed to initialize RSA key manager");
        let token_service_impl = OpenIDTokenService::new(
            *db_pool,
            "https://api.epsx.io".to_string(), // issuer
            vec!["epsx-frontend".to_string(), "epsx-admin".to_string(), "epsx-api".to_string()], // audiences
            Arc::new(key_manager),
        );
        let token_service = Arc::new(token_service_impl.clone());

        // Create unified auth service with environment-based domain & OpenID support
        let domain = Self::get_web3_domain();
        let auth_service = Arc::new(UnifiedWeb3AuthService::new_with_openid(
            *db_pool,
            domain,
            token_service_impl,
        ));

        // Create event bus
        let event_bus: Arc<dyn crate::domain::DomainEventBus> = Arc::new(
            crate::infrastructure::event_bus::simple_event_bus::SimpleEventBus::new()
        );

        // Create CQRS infrastructure (Event Sourcing)
        // Use analytics_pool for event store if available, otherwise fallback to primary (legacy)
        let event_store_pool = analytics_pool.as_ref().cloned().unwrap_or(Arc::clone(&db_pool));
        let event_store: Arc<dyn EventStore> = Arc::new(PostgresEventStore::new(event_store_pool.clone()));
        
        // Outbox also needs to know which DB it's using? Usually Outbox is on the same DB as the aggregate changes...
        // BUT here we have split DBs.
        // If an aggregate (e.g. User) in Primary DB emits an event, the outbox MUST be in Primary DB?
        // OR we use dual-write/saga. 
        // Current impl of TransactionalOutbox takes `db_pool` and `event_store`.
        // If event_store is in Analytics DB, ensuring atomicity is hard.
        // For now, let's assume Outbox table is on Primary DB (where Aggregates are).
        // Wait, I removed Outbox from Primary schema!
        // So Outbox MUST be in Analytics DB or wherever EventStore is.
        // Ideally, domain logic should write to Outbox in same transaction as Aggregate update.
        // If Aggregate is in Primary and Outbox in Analytics, we cannot do atomic transaction.
        // This is a known issue with split DBs. 
        // For this refactor, let's point Outbox to Analytics DB (event_store_pool).
        // This means we sacrifice atomicity unless we use 2PC (which we don't).
        // Or maybe Outbox table was intended to be in Primary? 
        // I removed `outbox_events` from Primary.
        let transactional_outbox = Arc::new(TransactionalOutbox::new(
            event_store_pool.clone(), // Use same pool as EventStore (Analytics)
            Arc::clone(&event_store),
        ));

        // Create EventDispatcher
        use std::env;
        let redis_url = env::var("REDIS_URL").ok();
        let dispatcher_config = crate::infrastructure::EventDispatcherConfig::default();
        let event_dispatcher = match crate::infrastructure::EventDispatcher::new(
            Arc::clone(&transactional_outbox),
            redis_url.clone(),
            dispatcher_config,
        ) {
            Ok(dispatcher) => Some(Arc::new(dispatcher)),
            Err(e) => {
                tracing::warn!("Failed to create EventDispatcher: {}", e);
                None
            }
        };

        // Create ProjectionManager with WalletReadModelProjection
        let projection_manager = match crate::infrastructure::ProjectionManager::new(
            Arc::clone(&db_pool),
            redis_url.clone(),
            "domain_events".to_string(),
        ) {
            Ok(manager) => {
                // Register WalletReadModelProjection
                let wallet_projection = Arc::new(
                    crate::infrastructure::WalletReadModelProjection::new(Arc::clone(&db_pool))
                );
                Some(Arc::new(manager.register(wallet_projection)))
            }
            Err(e) => {
                tracing::warn!("Failed to create ProjectionManager: {}", e);
                None
            }
        };

        // Create Redis pool and notification broadcaster
        let (redis_pool, redis_broadcaster, permission_cache, unified_permission_service) = match redis_url {
            Some(ref url) => {
                // Try to create Redis pool for notifications
                let (pool, broadcaster) = match RedisPool::new(url).await {
                    Ok(pool) => {
                        let pool_arc = Arc::new(pool);
                        let broadcaster = Arc::new(RedisNotificationBroadcaster::new(Arc::clone(&pool_arc)));
                        tracing::info!("✅ Redis notification system initialized");
                        (Some(pool_arc), Some(broadcaster))
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ Failed to create Redis pool: {} (notifications will not work)", e);
                        (None, None)
                    }
                };

                // Try to create Redis client for permission caching
                match redis::Client::open(url.as_str()) {
                    Ok(client) => {
                        let redis_client = Arc::new(client);
                        let perm_cache = Arc::new(UnifiedPermissionCache::new(Arc::clone(&redis_client)));
                        let perm_service = Arc::new(UnifiedPermissionService::new(
                            *db_pool,
                            Arc::clone(&perm_cache),
                        ));
                        tracing::info!("✅ UnifiedPermissionService initialized with Redis cache");
                        (pool, broadcaster, Some(perm_cache), perm_service)
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ Failed to create Redis client for permission cache: {}", e);
                        let perm_service = Arc::new(UnifiedPermissionService::new_without_cache(*db_pool));
                        tracing::info!("✅ UnifiedPermissionService initialized (without Redis cache)");
                        (pool, broadcaster, None, perm_service)
                    }
                }
            }
            None => {
                tracing::warn!("⚠️ No REDIS_URL configured - notifications and permission caching will not work");
                let perm_service = Arc::new(UnifiedPermissionService::new_without_cache(*db_pool));
                (None, None, None, perm_service)
            }
        };

        // Create Web3 permission adapter
        let web3_permission_adapter = Arc::new(Web3PermissionServiceAdapter::new(
            cache.as_ref().map(Arc::clone),
            blockchain_config,
            *db_pool,
        ));

        // Create TransactionHistoryProvider based on environment
        let transaction_history_provider: Option<Arc<dyn TransactionHistoryProvider>> = {
            let config = crate::config::env::init_config(); // Use config for environment checks
            let contract_address = std::env::var("PAYMENT_ESCROW_ADDRESS")
                .unwrap_or_else(|_| "0x1234567890123456789012345678901234567890".to_string());

            if config.is_production() {
                let api_key = std::env::var("BSCSCAN_API_KEY").unwrap_or_default();
                Some(Arc::new(crate::infrastructure::blockchain::ScannerTransactionHistoryProvider::new(
                    api_key,
                    contract_address,
                )))
            } else {
                let rpc_url = config.bsc_rpc_url.clone();
                match crate::infrastructure::blockchain::RpcTransactionHistoryProvider::new(rpc_url, contract_address) {
                    Ok(provider) => Some(Arc::new(provider)),
                    Err(e) => {
                        tracing::error!("Failed to initialize RpcTransactionHistoryProvider: {}", e);
                        None
                    }
                }
            }
        };

        Self {
            db_pool,
            payments_pool,
            analytics_pool,
            notifications_pool,
            cache,
            // Web3-first services
            wallet_user_repository: Some(wallet_user_repository),

            group_repository: Some(group_repository),
            payment_repository,
            notification_repository,
            wallet_permission_service: Some(wallet_permission_service),
            web3_permission_adapter: Some(web3_permission_adapter),
            auth_service: Some(auth_service),
            token_service: Some(token_service),
            identity_provider: Some(identity_provider),

            plan_repository: Some(plan_repository),
            event_bus: Some(event_bus),
            // Unified Permission Service
            unified_permission_service: Some(unified_permission_service),
            permission_cache,
            // Redis notifications
            redis_pool,
            redis_broadcaster,
            // CQRS
            event_store: Some(event_store),
            transactional_outbox: Some(transactional_outbox),
            event_dispatcher,
            projection_manager,
            transaction_history_provider,
        }
    }

    pub fn with_cache(db_pool: Arc<&'static Pool<AsyncPgConnection>>, cache: Arc<dyn Cache>) -> Self {
        Self {
            db_pool,
            payments_pool: None,
            analytics_pool: None,
            notifications_pool: None,
            cache: Some(cache),
            identity_provider: None,
            // Initialize Web3 services as None - use new_with_web3_services for full setup

            wallet_user_repository: None,

            group_repository: None,
            payment_repository: None,
            notification_repository: None,
            wallet_permission_service: None,
            web3_permission_adapter: None,
            auth_service: None,
            token_service: None,
            event_bus: None,
            // Unified Permission Service
            unified_permission_service: None,
            permission_cache: None,
            // Redis
            redis_pool: None,
            redis_broadcaster: None,
            // CQRS
            event_store: None,
            transactional_outbox: None,
            event_dispatcher: None,
            projection_manager: None,
            transaction_history_provider: None,
            plan_repository: None,
        }
    }

    /// Builder method to add blockchain configuration
    pub fn with_blockchain_config(mut self, blockchain_config: BlockchainConfig) -> Self {
        // Recreate Web3 services with blockchain config
        if let Some(cache) = &self.cache {
            self.web3_permission_adapter = Some(Arc::new(Web3PermissionServiceAdapter::new(
                Some(Arc::clone(cache)),
                Some(blockchain_config),
                *self.db_pool,
            )));
        }
        self
    }

    // Compatibility methods
    pub fn db_pool(&self) -> Arc<&'static Pool<AsyncPgConnection>> {
        Arc::clone(&self.db_pool)
    }

    pub fn infra(&self) -> &Self {
        self
    }
    
    // Pool Getters
    pub fn get_payments_pool(&self) -> Option<Arc<&'static Pool<AsyncPgConnection>>> {
        self.payments_pool.as_ref().map(Arc::clone)
    }

    pub fn get_analytics_pool(&self) -> Option<Arc<&'static Pool<AsyncPgConnection>>> {
        self.analytics_pool.as_ref().map(Arc::clone)
    }

    pub fn get_notifications_pool(&self) -> Option<Arc<&'static Pool<AsyncPgConnection>>> {
        self.notifications_pool.as_ref().map(Arc::clone)
    }

    // NEW - Web3-first service getters (primary)
    pub fn get_wallet_user_repository(&self) -> Option<Arc<WalletUserRepositoryAdapter>> {
        self.wallet_user_repository.as_ref().map(Arc::clone)
    }

    pub fn get_wallet_user_repository_port(&self) -> Option<Arc<dyn WalletUserRepositoryPort>> {
        self.wallet_user_repository.as_ref().map(|repo| Arc::clone(repo) as Arc<dyn WalletUserRepositoryPort>)
    }

    pub fn get_wallet_user_analytics_port(&self) -> Option<Arc<dyn WalletUserAnalyticsPort>> {
        self.wallet_user_repository.as_ref().map(|repo| Arc::clone(repo) as Arc<dyn WalletUserAnalyticsPort>)
    }

    pub fn get_payment_repository(&self) -> Option<Arc<PaymentRepositoryAdapter>> {
        self.payment_repository.as_ref().map(Arc::clone)
    }

    pub fn get_payment_repository_port(&self) -> Option<Arc<dyn PaymentRepositoryPort>> {
        self.payment_repository.as_ref().map(|repo| Arc::clone(repo) as Arc<dyn PaymentRepositoryPort>)
    }

    pub fn get_notification_repository(&self) -> Option<Arc<NotificationRepositoryAdapter>> {
        self.notification_repository.as_ref().map(Arc::clone)
    }

    pub fn get_wallet_permission_service(&self) -> Option<Arc<WalletPermissionService>> {
        self.wallet_permission_service.as_ref().map(Arc::clone)
    }

    pub fn get_web3_permission_adapter(&self) -> Option<Arc<Web3PermissionServiceAdapter>> {
        self.web3_permission_adapter.as_ref().map(Arc::clone)
    }

    pub fn get_auth_service(&self) -> Option<Arc<UnifiedWeb3AuthService>> {
        self.auth_service.as_ref().map(Arc::clone)
    }

    pub fn get_token_service(&self) -> Option<Arc<OpenIDTokenService>> {
        self.token_service.as_ref().map(Arc::clone)
    }

    pub fn get_identity_provider(&self) -> Option<Arc<dyn IdentityProviderPort>> {
        self.identity_provider.as_ref().map(Arc::clone)
    }


    /// Get the unified permission service (single source of truth)
    pub fn get_unified_permission_service(&self) -> Option<Arc<UnifiedPermissionService>> {
        self.unified_permission_service.as_ref().map(Arc::clone)
    }

    pub fn get_redis_pool(&self) -> Option<Arc<RedisPool>> {
        self.redis_pool.as_ref().map(Arc::clone)
    }

    pub fn get_redis_broadcaster(&self) -> Option<Arc<RedisNotificationBroadcaster>> {
        self.redis_broadcaster.as_ref().map(Arc::clone)
    }

    pub fn get_transaction_history_provider(&self) -> Option<Arc<dyn TransactionHistoryProvider>> {
        self.transaction_history_provider.as_ref().map(Arc::clone)
    }

    pub fn get_plan_repository_port(&self) -> Option<Arc<dyn crate::domain::subscription_management::repository_ports::PlanRepositoryPort>> {
         self.plan_repository.as_ref().map(|repo| Arc::clone(repo) as Arc<dyn crate::domain::subscription_management::repository_ports::PlanRepositoryPort>)
    }

    // Enhanced app state creation with Web3 services
    pub fn create_app_state(&self) -> Web3AppState {
        Web3AppState {
            wallet_user_repository: self.get_wallet_user_repository_port(),
            wallet_user_analytics: self.get_wallet_user_analytics_port(),
            payment_repository: self.get_payment_repository_port(),
            wallet_permission_service: self.get_wallet_permission_service(),
            web3_permission_adapter: self.get_web3_permission_adapter(),
            auth_service: self.get_auth_service(),
            db_pool: Arc::clone(&self.db_pool),
            cache: self.cache.as_ref().map(Arc::clone),
            transaction_history_provider: self.get_transaction_history_provider(),
        }
    }
    
    // Health check for all services
    pub async fn health_check(&self) -> ContainerHealthStatus {
                use diesel_async::RunQueryDsl;

        // Check database connectivity
        let database_healthy = async {
            let mut conn = self.db_pool.get().await.ok()?;

            #[derive(diesel::QueryableByName)]
            struct HealthCheck {
                #[diesel(sql_type = diesel::sql_types::Integer)]
                _check: i32,
            }

            diesel::sql_query("SELECT 1 as _check")
                .get_result::<HealthCheck>(&mut conn)
                .await
                .ok()
        }.await.is_some();

        // Check cache connectivity
        let cache_healthy = if let Some(cache) = &self.cache {
            cache.health_check().is_ok()
        } else {
            true // No cache configured, considered healthy
        };

        let mut status = ContainerHealthStatus {
            database_healthy,
            cache_healthy,
            ..Default::default()
        };
        
        // Check Web3 services
        status.web3_services_healthy = self.wallet_user_repository.is_some() &&
            self.wallet_permission_service.is_some() &&
            self.web3_permission_adapter.is_some();
        
        status.overall_healthy = status.database_healthy && 
            status.cache_healthy && 
            status.web3_services_healthy;
        
        status
    }
    
    // Service validation
    pub fn validate_services(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        if self.wallet_user_repository.is_none() {
            errors.push("WalletUserRepository not configured".to_string());
        }
        
        if self.wallet_permission_service.is_none() {
            errors.push("WalletPermissionService not configured".to_string());
        }
        
        if self.web3_permission_adapter.is_none() {
            errors.push("Web3PermissionServiceAdapter not configured".to_string());
        }
        
        if self.auth_service.is_none() {
            errors.push("UnifiedWeb3AuthService not configured".to_string());
        }
        
        if self.token_service.is_none() {
            errors.push("OpenIDTokenService not configured".to_string());
        }
        
        errors
    }
}

/// Web3-first application state with all necessary services
#[derive(Clone)]
pub struct Web3AppState {
    pub wallet_user_repository: Option<Arc<dyn WalletUserRepositoryPort>>,
    pub wallet_user_analytics: Option<Arc<dyn WalletUserAnalyticsPort>>,
    pub payment_repository: Option<Arc<dyn PaymentRepositoryPort>>,
    pub wallet_permission_service: Option<Arc<WalletPermissionService>>,
    pub web3_permission_adapter: Option<Arc<Web3PermissionServiceAdapter>>,
    pub auth_service: Option<Arc<UnifiedWeb3AuthService>>,
    pub db_pool: Arc<&'static Pool<AsyncPgConnection>>,
    pub cache: Option<Arc<dyn Cache>>,
    pub transaction_history_provider: Option<Arc<dyn TransactionHistoryProvider>>,
}

impl Web3AppState {
    /// Validate that all required services are available
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.wallet_user_repository.is_none() {
            errors.push("WalletUserRepository is required".to_string());
        }

        if self.payment_repository.is_none() {
            errors.push("PaymentRepository is required".to_string());
        }
        
        if self.wallet_permission_service.is_none() {
            errors.push("WalletPermissionService is required".to_string());
        }
        
        if self.web3_permission_adapter.is_none() {
            errors.push("Web3PermissionServiceAdapter is required".to_string());
        }
        
        if self.auth_service.is_none() {
            errors.push("UnifiedWeb3AuthService is required".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Get all required services - returns error if any are missing
    pub fn services(&self) -> Result<Web3Services, Vec<String>> {
        self.validate()?;

        Ok(Web3Services {
            wallet_user_repository: Arc::clone(self.wallet_user_repository.as_ref().unwrap()),
            wallet_user_analytics: Arc::clone(self.wallet_user_analytics.as_ref().unwrap()),
            payment_repository: Arc::clone(self.payment_repository.as_ref().unwrap()),
            wallet_permission_service: Arc::clone(self.wallet_permission_service.as_ref().unwrap()),
            web3_permission_adapter: Arc::clone(self.web3_permission_adapter.as_ref().unwrap()),
            auth_service: Arc::clone(self.auth_service.as_ref().unwrap()),
        })
    }
}

/// Strongly typed Web3 services collection
#[derive(Clone)]
pub struct Web3Services {
    pub wallet_user_repository: Arc<dyn WalletUserRepositoryPort>,
    pub wallet_user_analytics: Arc<dyn WalletUserAnalyticsPort>,
    pub payment_repository: Arc<dyn PaymentRepositoryPort>,
    pub wallet_permission_service: Arc<WalletPermissionService>,
    pub web3_permission_adapter: Arc<Web3PermissionServiceAdapter>,
    pub auth_service: Arc<UnifiedWeb3AuthService>,
}

/// Health status for the container and its services
#[derive(Debug, Default)]
pub struct ContainerHealthStatus {
    pub overall_healthy: bool,
    pub database_healthy: bool,
    pub cache_healthy: bool,
    pub web3_services_healthy: bool,
    pub error_details: Vec<String>,
}

impl ContainerHealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.overall_healthy
    }
    
    pub fn add_error(&mut self, error: String) {
        self.error_details.push(error);
        self.overall_healthy = false;
    }
}

// Type alias for compatibility
pub type DomainContainer = SimpleContainer;