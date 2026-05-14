use std::sync::{Arc, Once};

use anyhow::{anyhow, Result};
use axum::Router;
use tracing::info;

use crate::{
    config::env::{init_config, Config},
    infrastructure::{
        cache::{memory_cache::MemoryCache, redis_cache::RedisCache, Cache, CacheConfig},
        container::DomainContainer,
        database::get_diesel_pool,
    },
    prelude::TlsPool,
    web::create_router,
};

static LOGGER_INIT: Once = Once::new();
static CRYPTO_PROVIDER_INIT: Once = Once::new();

#[derive(Clone, Copy)]
pub struct BackendBootstrapOptions {
    pub seed_database: bool,
    pub start_background_workers: bool,
}

impl BackendBootstrapOptions {
    pub const fn stateful_server() -> Self {
        Self {
            seed_database: true,
            start_background_workers: true,
        }
    }

    pub const fn vercel_function() -> Self {
        Self {
            seed_database: false,
            start_background_workers: false,
        }
    }
}

pub struct BackendRuntime {
    pub config: Config,
    pub container: Arc<DomainContainer>,
    pub router: Router,
}

pub async fn build_runtime(options: BackendBootstrapOptions) -> Result<BackendRuntime> {
    let config = init_backend_process();
    let _db_pool = initialize_database(options.seed_database).await?;
    let cache = initialize_cache().await;

    let container = Arc::new(DomainContainer::new_with_web3_services(cache, None).await);

    if options.start_background_workers {
        start_background_workers(&container);
    }

    let router = create_router(Arc::clone(&container));

    Ok(BackendRuntime {
        config,
        container,
        router,
    })
}

pub fn init_backend_process() -> Config {
    let config = init_config();

    LOGGER_INIT.call_once(|| {
        crate::infrastructure::logger::init_logger(config.is_production(), &config.log_level);
    });

    CRYPTO_PROVIDER_INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });

    config
}

async fn initialize_database(seed_database: bool) -> Result<&'static TlsPool> {
    if std::env::var("DATABASE_URL").is_err() {
        return Err(anyhow!("DATABASE_URL must be set"));
    }

    info!("Connecting to database...");

    let db_pool = get_diesel_pool().await?;
    let connection_timeout = std::time::Duration::from_secs(10);

    match tokio::time::timeout(connection_timeout, db_pool.get()).await {
        Ok(Ok(_)) => info!("Database pool created and connection verified"),
        Ok(Err(e)) => return Err(anyhow!("Database connection failed: {}", e)),
        Err(_) => return Err(anyhow!("Database connection timed out")),
    }

    if seed_database {
        crate::infrastructure::services::seed_system_admin_plans(db_pool).await;
        crate::infrastructure::services::seed_production_news(db_pool).await;
    }

    Ok(db_pool)
}

async fn initialize_cache() -> Option<Arc<dyn Cache>> {
    let redis_timeout = std::time::Duration::from_secs(5);

    match std::env::var("REDIS_URL").ok() {
        Some(redis_url) => {
            match tokio::time::timeout(
                redis_timeout,
                RedisCache::new(redis_url, 10, CacheConfig::default()),
            )
            .await
            {
                Ok(Ok(cache)) => {
                    info!("Redis cache initialized");
                    Some(Arc::new(cache) as Arc<dyn Cache>)
                }
                Ok(Err(e)) => {
                    info!(
                        "Redis cache initialization failed, using memory cache: {}",
                        e
                    );
                    Some(Arc::new(MemoryCache::new()) as Arc<dyn Cache>)
                }
                Err(_) => {
                    info!("Redis connection timed out after 5s, using memory cache");
                    Some(Arc::new(MemoryCache::new()) as Arc<dyn Cache>)
                }
            }
        }
        None => {
            info!("No Redis URL configured, using memory cache");
            Some(Arc::new(MemoryCache::new()) as Arc<dyn Cache>)
        }
    }
}

fn start_background_workers(container: &Arc<DomainContainer>) {
    crate::infrastructure::blockchain::spawn_transaction_monitor();
    info!("Transaction Monitor background service started");

    if let Some(dispatcher) = &container.event_dispatcher {
        let dispatcher = Arc::clone(dispatcher);
        tokio::spawn(async move {
            match dispatcher.start().await {
                Ok(_) => {
                    info!("EventDispatcher started - events will be published to Redis Streams")
                }
                Err(e) => info!(
                    "EventDispatcher failed to start: {} (continuing without event publishing)",
                    e
                ),
            }
        });
    } else {
        info!("EventDispatcher not configured (Redis URL not set)");
    }

    if let Some(projection_manager) = &container.projection_manager {
        let projection_manager = Arc::clone(projection_manager);
        tokio::spawn(async move {
            match projection_manager.start().await {
                Ok(_) => {
                    info!("ProjectionManager started - read models will be updated from events")
                }
                Err(e) => info!(
                    "ProjectionManager failed to start: {} (continuing without projections)",
                    e
                ),
            }
        });
    } else {
        info!("ProjectionManager not configured");
    }

    let svc = crate::infrastructure::services::PlanExpirationService::new(
        Arc::clone(&container.db_pool),
        container.notifications_pool.as_ref().map(Arc::clone),
        container.redis_broadcaster.as_ref().map(Arc::clone),
    );
    svc.start();
    info!("PlanExpirationService background service started");
}
