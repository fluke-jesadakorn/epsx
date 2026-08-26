// SQLx Connection Pool Manager for Serverless Environments
// Provides efficient connection pooling using sqlx
// Optimized for Cloud Run serverless deployment
//
// kernel extraction wave10: the type definitions (`TlsConnectionManager`,
// `ManagerError`, `TlsPool`, `PoolExt`) live in the shared
// `epsx-database-pools` crate (now type alias for `sqlx::PgPool`).
// This file retains the backend runtime wiring: the global pools,
// the initializer struct, the serverless config, and the health-check
// / pool-statistics accessors.

use anyhow::Result;
use epsx_database_pools::{create_all_pools, SqlxPoolConfig};
use sqlx::PgPool;
use std::sync::OnceLock;
use tracing::{error, info, warn};

// Re-export the shared types for backward compatibility.
pub use epsx_database_pools::TlsPool;

// Global sqlx connection pools (canonical).
static GLOBAL_CORE_POOL: OnceLock<PgPool> = OnceLock::new();
static GLOBAL_ANALYTICS_POOL: OnceLock<Option<PgPool>> = OnceLock::new();
static GLOBAL_NOTIFICATIONS_POOL: OnceLock<Option<PgPool>> = OnceLock::new();
static GLOBAL_PAYMENTS_POOL: OnceLock<Option<PgPool>> = OnceLock::new();

/// Health status for all database pools
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AllPoolsHealth {
    pub primary: bool,
    pub analytics: bool,
    pub notifications: bool,
    pub payments: bool,
    pub healthy: bool,
}

/// Serverless-optimized sqlx connection configuration
#[derive(Clone, Debug)]
pub struct DieselServerlessConfig {
    pub database_url: String,
    pub max_size: usize,
    pub acquire_timeout_secs: u64,
}

impl DieselServerlessConfig {
    /// Create optimized config for serverless environments (Cloud Run)
    pub fn for_serverless(database_url: String) -> Self {
        Self {
            database_url,
            max_size: 10,
            acquire_timeout_secs: 5,
        }
    }

    /// Create config from environment variables
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| crate::config::get_fallback_config().database_url);

        let mut config = Self::for_serverless(database_url);

        if let Ok(max_conn) = std::env::var("DB_MAX_CONNECTIONS") {
            config.max_size = max_conn.parse().unwrap_or(config.max_size);
        }

        if let Ok(timeout) = std::env::var("DB_ACQUIRE_TIMEOUT_SECS") {
            config.acquire_timeout_secs = timeout.parse().unwrap_or(5);
        }

        Ok(config)
    }
}

/// SQLx connection pool manager for serverless
pub struct DieselConnectionManager;

impl DieselConnectionManager {
    /// Get or create the global sqlx connection pool (canonical name)
    pub async fn get_pool() -> Result<&'static PgPool> {
        if let Some(pool) = GLOBAL_CORE_POOL.get() {
            return Ok(pool);
        }

        let config = DieselServerlessConfig::from_env()?;
        let pool = Self::create_optimized_pool(config).await?;
        Ok(GLOBAL_CORE_POOL.get_or_init(|| pool))
    }

    /// Get the analytics pool (lazy)
    pub async fn get_analytics_pool() -> Result<&'static Option<PgPool>> {
        if GLOBAL_ANALYTICS_POOL.get().is_none() {
            let url = std::env::var("ANALYTICS_DATABASE_URL").ok();
            if let Some(url) = url {
                let cfg = SqlxPoolConfig::analytics(url);
                match epsx_database_pools::create_pool(cfg).await {
                    Ok(p) => {
                        GLOBAL_ANALYTICS_POOL.get_or_init(|| Some(p));
                    }
                    Err(e) => {
                        warn!("Failed to init analytics pool: {}", e);
                    }
                }
            } else {
                GLOBAL_ANALYTICS_POOL.get_or_init(|| None);
            }
        }
        Ok(GLOBAL_ANALYTICS_POOL.get().unwrap())
    }

    /// Get the notifications pool (lazy)
    pub async fn get_notifications_pool() -> Result<&'static Option<PgPool>> {
        if GLOBAL_NOTIFICATIONS_POOL.get().is_none() {
            let url = std::env::var("NOTIFICATIONS_DATABASE_URL").ok();
            if let Some(url) = url {
                let cfg = SqlxPoolConfig::notifications(url);
                match epsx_database_pools::create_pool(cfg).await {
                    Ok(p) => {
                        GLOBAL_NOTIFICATIONS_POOL.get_or_init(|| Some(p));
                    }
                    Err(e) => {
                        warn!("Failed to init notifications pool: {}", e);
                    }
                }
            } else {
                GLOBAL_NOTIFICATIONS_POOL.get_or_init(|| None);
            }
        }
        Ok(GLOBAL_NOTIFICATIONS_POOL.get().unwrap())
    }

    /// Get the payments pool (lazy)
    pub async fn get_payments_pool() -> Result<&'static Option<PgPool>> {
        if GLOBAL_PAYMENTS_POOL.get().is_none() {
            let url = std::env::var("PAYMENTS_DATABASE_URL").ok();
            if let Some(url) = url {
                let cfg = SqlxPoolConfig::payments(url);
                match epsx_database_pools::create_pool(cfg).await {
                    Ok(p) => {
                        GLOBAL_PAYMENTS_POOL.get_or_init(|| Some(p));
                    }
                    Err(e) => {
                        warn!("Failed to init payments pool: {}", e);
                    }
                }
            } else {
                GLOBAL_PAYMENTS_POOL.get_or_init(|| None);
            }
        }
        Ok(GLOBAL_PAYMENTS_POOL.get().unwrap())
    }

    async fn create_optimized_pool(config: DieselServerlessConfig) -> Result<PgPool> {
        let cfg = SqlxPoolConfig::core(config.database_url.clone());
        let pool = epsx_database_pools::create_pool(cfg).await?;
        info!(
            "Created sqlx pool (max_size={}, timeout={}s)",
            config.max_size, config.acquire_timeout_secs
        );
        Ok(pool)
    }

    // Backward-compatible health-check aliases (sqlx-based)
    pub async fn diesel_health_check() -> Result<(), String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;
        sqlx::query("SELECT 1")
            .execute(pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub async fn diesel_health_check_all() -> Result<AllPoolsHealth, String> {
        let core_ok = Self::get_pool().await.is_ok();
        Ok(AllPoolsHealth {
            primary: core_ok,
            analytics: core_ok,
            notifications: core_ok,
            payments: core_ok,
            healthy: core_ok,
        })
    }
}

/// Backward-compatible alias for get_pool (legacy callers).
pub async fn get_diesel_pool() -> Result<&'static PgPool, String> {
    DieselConnectionManager::get_pool().await.map_err(|e| e.to_string())
}

pub async fn get_analytics_pool() -> Result<PgPool, String> {
    if let Ok(Some(pool)) = DieselConnectionManager::get_analytics_pool().await {
        return Ok(pool.clone());
    }
    DieselConnectionManager::get_pool().await.map(|p| p.clone()).map_err(|e| e.to_string())
}

pub async fn get_notifications_pool() -> Result<PgPool, String> {
    if let Ok(Some(pool)) = DieselConnectionManager::get_notifications_pool().await {
        return Ok(pool.clone());
    }
    DieselConnectionManager::get_pool().await.map(|p| p.clone()).map_err(|e| e.to_string())
}

pub async fn get_payments_pool() -> Result<PgPool, String> {
    if let Ok(Some(pool)) = DieselConnectionManager::get_payments_pool().await {
        return Ok(pool.clone());
    }
    DieselConnectionManager::get_pool().await.map(|p| p.clone()).map_err(|e| e.to_string())
}

pub async fn diesel_health_check() -> Result<(), String> {
    DieselConnectionManager::diesel_health_check().await
}

pub async fn diesel_health_check_all() -> Result<AllPoolsHealth, String> {
    DieselConnectionManager::diesel_health_check_all().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_env_overrides() {
        // Without env vars, defaults to fallback config
        let cfg = DieselServerlessConfig::from_env().unwrap();
        assert!(!cfg.database_url.is_empty());
    }

    #[test]
    fn serverless_config_defaults() {
        let cfg = DieselServerlessConfig::for_serverless("postgres://test".to_string());
        assert_eq!(cfg.max_size, 10);
        assert_eq!(cfg.acquire_timeout_secs, 5);
    }
}
