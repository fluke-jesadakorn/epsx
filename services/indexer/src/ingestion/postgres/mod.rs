use sqlx::PgPool;

use async_trait::async_trait;

use super::{
    ApplyOutcome, BlockHash, BlockIdentity, BlockNumber, BlockRef, ChainId, ChainMutation,
    ChainSnapshot, LeaseDuration, LeaseGrant, LeaseOwner, SelectedChainRepository,
    SelectedChainRepositoryError, ValidatedBlockBatch,
};

mod apply;
mod candidates;
mod codec;
mod journal;

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

#[async_trait]
impl SelectedChainRepository for PostgresSelectedChainRepository {
    async fn acquire_lease(
        &self,
        chain_id: ChainId,
        owner: LeaseOwner,
        duration: LeaseDuration,
    ) -> Result<LeaseGrant, SelectedChainRepositoryError> {
        leases::acquire(self.pool(), chain_id, owner, duration).await
    }

    async fn renew_lease(
        &self,
        grant: &LeaseGrant,
        duration: LeaseDuration,
    ) -> Result<LeaseGrant, SelectedChainRepositoryError> {
        leases::renew(self.pool(), grant, duration).await
    }

    async fn release_lease(&self, grant: &LeaseGrant) -> Result<(), SelectedChainRepositoryError> {
        leases::release(self.pool(), grant).await
    }

    async fn snapshot(
        &self,
        chain_id: ChainId,
    ) -> Result<ChainSnapshot, SelectedChainRepositoryError> {
        reads::snapshot(self.pool(), chain_id).await
    }

    async fn load_candidate(
        &self,
        identity: BlockIdentity,
    ) -> Result<Option<ValidatedBlockBatch>, SelectedChainRepositoryError> {
        reads::load_candidate(self.pool(), identity).await
    }

    async fn selected_hash(
        &self,
        chain_id: ChainId,
        number: BlockNumber,
    ) -> Result<Option<BlockHash>, SelectedChainRepositoryError> {
        reads::selected_hash(self.pool(), chain_id, number).await
    }

    async fn candidates_at_height(
        &self,
        chain_id: ChainId,
        number: BlockNumber,
    ) -> Result<Vec<BlockRef>, SelectedChainRepositoryError> {
        reads::candidates_at_height(self.pool(), chain_id, number).await
    }

    async fn apply(
        &self,
        mutation: ChainMutation,
    ) -> Result<ApplyOutcome, SelectedChainRepositoryError> {
        apply::apply(self.pool(), &mutation).await
    }
}
