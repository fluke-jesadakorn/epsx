use sqlx::PgPool;

mod candidates;
mod codec;

mod leases;
mod reads;

/// Dormant PostgreSQL repository foundation.
///
/// Construction does not connect to PostgreSQL, run a migration, start a
/// worker, or activate an ingestion route. Durable repository behavior is
/// added only by later reviewed slices.
pub(super) struct PostgresSelectedChainRepository {
    pool: PgPool,
}

impl PostgresSelectedChainRepository {
    pub(super) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) fn pool(&self) -> &PgPool {
        &self.pool
    }
}
