use super::{BlockHash, BlockNumber, BlockRequest, ChainId, TransactionHash, ValidatedBlockBatch};
use async_trait::async_trait;
use thiserror::Error;

/// Fetches an untrusted block bundle. Implementations are deliberately absent
/// until chain configuration and RPC authority have been decided.
#[async_trait]
pub trait BlockProvider: Send + Sync {
    async fn fetch_block(
        &self,
        request: BlockRequest,
    ) -> Result<Option<super::FetchedBlock>, BlockProviderError>;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitOutcome {
    Inserted,
    AlreadyStored,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositoryConflict {
    #[error("height {chain_id:?}/{number:?} is already associated with another block")]
    Height {
        chain_id: ChainId,
        number: BlockNumber,
        stored: BlockHash,
        candidate: BlockHash,
    },
    #[error("block hash {hash:?} on {chain_id:?} is associated with different block data")]
    BlockHash {
        chain_id: ChainId,
        hash: BlockHash,
        stored_number: BlockNumber,
        candidate_number: BlockNumber,
    },
    #[error("transaction hash {hash:?} on {chain_id:?} is associated with another block")]
    TransactionHash {
        chain_id: ChainId,
        hash: TransactionHash,
        stored_block: BlockHash,
        candidate_block: BlockHash,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BlockRepositoryError {
    #[error(transparent)]
    Conflict(#[from] RepositoryConflict),
    #[error("block repository unavailable: {0}")]
    Unavailable(String),
    #[error("block repository state is internally inconsistent: {0}")]
    CorruptState(String),
}

/// Stores or loads a complete validated batch.
///
/// `commit` must be all-or-nothing for every error and for cancellation: no
/// block, height, or transaction subset may become visible unless the complete
/// batch commits. `AlreadyStored` is valid only when the normalized batch is an
/// exact match and every repository invariant and secondary mapping remains
/// intact. Differing content must return a typed conflict or corrupt-state
/// error, never `AlreadyStored`.
#[async_trait]
pub trait BlockRepository: Send + Sync {
    async fn commit(
        &self,
        batch: ValidatedBlockBatch,
    ) -> Result<CommitOutcome, BlockRepositoryError>;

    async fn load(
        &self,
        request: BlockRequest,
    ) -> Result<Option<ValidatedBlockBatch>, BlockRepositoryError>;
}
