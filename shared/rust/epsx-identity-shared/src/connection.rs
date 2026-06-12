//! Async Postgres connection manager for `epsx-identity-shared`.
//!
//! Implements a real `deadpool::managed::Manager` over
//! `diesel_async::AsyncPgConnection` so the moved auth source files
//! (which perform Diesel queries through `&mut conn` references) can
//! compile and link. The real backend's `TlsConnectionManager` (with
//! TLS) is a separate type and stays in `apps/backend`.

use async_trait::async_trait;
use deadpool::managed::{Manager, RecycleResult};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::str::FromStr;
use std::time::Duration;
use tokio_postgres::NoTls;

pub struct AsyncPgConnectionManager {
    database_url: String,
}

impl AsyncPgConnectionManager {
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }
}

#[async_trait]
impl Manager for AsyncPgConnectionManager {
    type Type = AsyncPgConnection;
    type Error = String;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let config = tokio_postgres::Config::from_str(&self.database_url)
            .map_err(|e| format!("bad DATABASE_URL: {e}"))?;
        let connect_timeout = Duration::from_secs(5);
        let (client, connection) = tokio::time::timeout(connect_timeout, config.connect(NoTls))
            .await
            .map_err(|_| "Database connection timed out".to_string())?
            .map_err(|e| format!("postgres connect: {e}"))?;
        tokio::spawn(async move {
            let _ = connection.await;
        });
        AsyncPgConnection::try_from(client)
            .await
            .map_err(|e| format!("AsyncPgConversion: {e}"))
    }

    async fn recycle(&self, conn: &mut Self::Type) -> RecycleResult<Self::Error> {
        diesel::sql_query("SELECT 1")
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| deadpool::managed::RecycleError::Backend(format!("recycle: {e}")))
    }
}
