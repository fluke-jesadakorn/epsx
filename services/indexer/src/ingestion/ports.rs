use super::{
    ApplyOutcome, BlockHash, BlockIdentity, BlockNumber, BlockRef, BlockRequest, ChainId,
    ChainMutation, ChainSnapshot, ExpectedChainState, FetchedBlock, LeaseDuration, LeaseGrant,
    LeaseOwner, MutationBuildError, MutationId, SelectionBoundaryError, ValidatedBlockBatch,
};
use async_trait::async_trait;
use thiserror::Error;

/// Fetches untrusted provider data. No implementation is wired to runtime.
#[async_trait]
pub trait BlockProvider: Send + Sync {
    async fn fetch_block(
        &self,
        request: BlockRequest,
    ) -> Result<Option<FetchedBlock>, BlockProviderError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BlockProviderError {
    #[error("provider does not support chain {0:?}")]
    UnsupportedChain(ChainId),
    #[error("block provider request timed out")]
    Timeout,
    #[error("block provider rate limit exceeded")]
    RateLimited,
    #[error("block provider rejected its credentials")]
    Unauthorized,
    #[error("block provider transport failed: {0}")]
    Transport(String),
    #[error("block provider returned an invalid protocol response: {0}")]
    Protocol(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SelectionConflict {
    #[error("lease for chain {chain_id:?} is held by another live owner")]
    LeaseHeld { chain_id: ChainId },
    #[error("lease owner or fence no longer matches the live grant")]
    StaleLease,
    #[error("mutation id {mutation_id:?} was reused with different content")]
    MutationIdReuse { mutation_id: MutationId },
    #[error("expected chain state does not match stored state")]
    ExpectedState {
        expected: Box<ExpectedChainState>,
        actual: Box<ExpectedChainState>,
    },
    #[error("candidate block {identity:?} already has different immutable content")]
    CandidateContent { identity: BlockIdentity },
    #[error("chain selection has already been initialized")]
    AlreadyInitialized,
    #[error("chain selection has not been initialized")]
    NotInitialized,
    #[error("reorg common ancestor is not the selected block at its height")]
    CommonAncestorNotSelected,
    #[error("reorg detach is not the exact currently selected suffix")]
    DetachMismatch,
    #[error("reorg attachment is identical to the selected suffix it would detach")]
    ReorgNoop,
    #[error("reorg would replace or remove the finalized selection")]
    FinalizedBoundary,
    #[error("finality target is not the selected block at its height")]
    FinalityTargetNotSelected,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SelectedChainRepositoryError {
    #[error(transparent)]
    Conflict(#[from] SelectionConflict),
    #[error(transparent)]
    InvalidMutation(#[from] MutationBuildError),
    #[error(transparent)]
    InvalidBoundary(#[from] SelectionBoundaryError),
    #[error("selected-chain repository unavailable: {0}")]
    Unavailable(String),
    #[error("selected-chain repository state is internally inconsistent: {0}")]
    CorruptState(String),
}

/// Fork-preserving storage for immutable candidates and an internally selected
/// chain. Selection and finalized-selection markers do not prove external
/// canonicality or consensus finality.
///
/// Every mutating operation must use repository time and be atomic for errors
/// and cancellation. Exact mutation replay is checked before lease validity;
/// altered mutation-ID reuse must fail. Successful new mutations increment the
/// chain revision exactly once.
#[async_trait]
pub trait SelectedChainRepository: Send + Sync {
    async fn acquire_lease(
        &self,
        chain_id: ChainId,
        owner: LeaseOwner,
        duration: LeaseDuration,
    ) -> Result<LeaseGrant, SelectedChainRepositoryError>;

    async fn renew_lease(
        &self,
        grant: &LeaseGrant,
        duration: LeaseDuration,
    ) -> Result<LeaseGrant, SelectedChainRepositoryError>;

    async fn release_lease(&self, grant: &LeaseGrant) -> Result<(), SelectedChainRepositoryError>;

    async fn snapshot(
        &self,
        chain_id: ChainId,
    ) -> Result<ChainSnapshot, SelectedChainRepositoryError>;

    async fn load_candidate(
        &self,
        identity: BlockIdentity,
    ) -> Result<Option<ValidatedBlockBatch>, SelectedChainRepositoryError>;

    async fn selected_hash(
        &self,
        chain_id: ChainId,
        number: BlockNumber,
    ) -> Result<Option<BlockHash>, SelectedChainRepositoryError>;

    async fn candidates_at_height(
        &self,
        chain_id: ChainId,
        number: BlockNumber,
    ) -> Result<Vec<BlockRef>, SelectedChainRepositoryError>;

    async fn apply(
        &self,
        mutation: ChainMutation,
    ) -> Result<ApplyOutcome, SelectedChainRepositoryError>;
}
