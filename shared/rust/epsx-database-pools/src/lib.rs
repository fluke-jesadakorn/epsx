//! `epsx-database-pools` — shared Postgres connection pool types.
//!
//! Extracted from `apps/backend/src/infrastructure/database/diesel_connection_manager.rs`
//! during the wave 10 prep pass so that both the backend and the shared
//! `epsx-identity-shared` auth crate see the *same* `TlsPool` type.
//!
//! Before this crate existed, `epsx-identity-shared::prelude::TlsPool` was
//! a placeholder alias for `Pool<AsyncPgConnectionManager>` (the non-TLS
//! Diesel manager) while the backend used `Pool<TlsConnectionManager>`
//! (the real TLS-enforcing one). The two aliases resolved to different
//! types, so any attempt to pass a backend pool into a shared-crate
//! service constructor was an E0308 mismatch.
//!
//! What lives here:
//!   - `TlsConnectionManager` — the custom `deadpool::managed::Manager`
//!     impl that creates `AsyncPgConnection` over a TLS-enforcing
//!     `tokio_postgres_rustls::MakeRustlsConnect`.
//!   - `ManagerError` — the `thiserror::Error` enum this manager produces.
//!   - `TlsPool` — `pub type TlsPool = Pool<TlsConnectionManager>`.
//!   - `PoolExt` — the `async fn conn(&self) -> AppResult<Object<...>>`
//!     extension trait that maps pool errors to the kernel `AppError`.
//!
//! What does NOT live here (still in the backend):
//!   - The `GLOBAL_*_POOL` `OnceLock` statics and the
//!     `DieselConnectionManager` initializer struct — these are
//!     runtime wiring, not type definitions, and they reach into
//!     `crate::config::get_fallback_config` and `utoipa`-derived
//!     health-check schemas that the backend (and only the backend)
//!     cares about.
//!   - The `DieselServerlessConfig`, `AllPoolsHealth`, `DieselPoolStats`
//!     types — backend-shaped configuration and observability structs.

use diesel_async::{AsyncPgConnection, RunQueryDsl};
use deadpool::managed::{Manager, Pool, RecycleResult, RecycleError};
use tracing::{debug, error};
use tokio_postgres_rustls::MakeRustlsConnect;
use rustls::ClientConfig;
use std::str::FromStr;
use async_trait::async_trait;

/// Custom Error type for the Connection Manager
#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("Database connection error: {0}")]
    Connection(#[from] tokio_postgres::Error),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Custom Connection Manager that enforces TLS
#[derive(Clone)]
pub struct TlsConnectionManager {
    database_url: String,
}

impl TlsConnectionManager {
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }
}

#[async_trait]
impl Manager for TlsConnectionManager {
    type Type = AsyncPgConnection;
    type Error = ManagerError;

    async fn create(&self) -> Result<AsyncPgConnection, ManagerError> {
        let config = tokio_postgres::Config::from_str(&self.database_url)
            .map_err(|e| ManagerError::Config(e.to_string()))?;

        let connect_timeout = std::time::Duration::from_secs(5);

        debug!("Connecting to database (SSL Mode: {:?})...", config.get_ssl_mode());

        let client = match config.get_ssl_mode() {
            tokio_postgres::config::SslMode::Disable => {
                let (client, connection) = tokio::time::timeout(connect_timeout, config.connect(tokio_postgres::NoTls))
                    .await
                    .map_err(|_| ManagerError::Config("Database connection timed out".to_string()))?
                    .map_err(|e| {
                        error!("Connection error: {:?}", e);
                        ManagerError::Connection(e)
                    })?;

                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("database connection error: {}", e);
                    }
                });
                client
            }
            _ => {
                let root_store = rustls::RootCertStore::from_iter(
                    webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
                );
                let client_config = ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                let tls = MakeRustlsConnect::new(client_config);

                let (client, connection) = tokio::time::timeout(connect_timeout, config.connect(tls))
                    .await
                    .map_err(|_| ManagerError::Config("Database connection timed out during TLS handshake".to_string()))?
                    .map_err(|e| {
                        error!("TLS Connection error: {:?}", e);
                        ManagerError::Connection(e)
                    })?;

                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("database connection error: {}", e);
                    }
                });
                client
            }
        };

        debug!("Wrapping in AsyncPgConnection...");
        tokio::time::timeout(connect_timeout, AsyncPgConnection::try_from(client))
            .await
            .map_err(|_| ManagerError::Config("AsyncPgConnection wrapper timed out".to_string()))?
            .map_err(|e| {
                error!("AsyncPgConnection conversion error: {}", e);
                ManagerError::Internal(e.to_string())
            })
    }

    async fn recycle(&self, conn: &mut AsyncPgConnection) -> RecycleResult<ManagerError> {
        // Simple health check query
        diesel::sql_query("SELECT 1")
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| RecycleError::Backend(ManagerError::Internal(e.to_string())))
    }
}

/// Pool type used by the backend and (post-wave-10) the shared auth
/// services. One source of truth so both sides resolve to the same
/// `Pool<TlsConnectionManager>` type.
pub type TlsPool = Pool<TlsConnectionManager>;

/// Extension trait for `TlsPool` that maps pool errors to the kernel
/// `AppError` for ergonomic `?` propagation in handler code.
#[async_trait]
pub trait PoolExt {
    /// Get a connection from the pool, mapping errors to `AppError`.
    async fn conn(&self) -> epsx_contracts::errors::AppResult<deadpool::managed::Object<TlsConnectionManager>>;
}

#[async_trait]
impl PoolExt for TlsPool {
    async fn conn(&self) -> epsx_contracts::errors::AppResult<deadpool::managed::Object<TlsConnectionManager>> {
        self.get().await
            .map_err(|e| epsx_contracts::errors::AppError::database_error(e.to_string()))
    }
}
