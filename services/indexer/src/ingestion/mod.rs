//! Dormant, provider-neutral block-ingestion contracts.
//!
//! This module does not start a worker, call an RPC endpoint, write SQL, or
//! assert canonicality/finality. It only validates a fetched block as one
//! internally consistent storage batch and defines ports for later adapters.

mod domain;
mod ports;
mod selection;

#[cfg(test)]
mod memory;

pub use domain::{
    validate_block, BlockHash, BlockNumber, BlockRequest, BoundaryError, ChainId, FetchedBlock,
    FetchedLog, FetchedReceipt, FetchedTransaction, LogIndex, ReceiptOutcome, TransactionHash,
    TxIndex, ValidatedBlock, ValidatedBlockBatch, ValidatedLog, ValidatedReceipt,
    ValidatedTransaction, ValidationError, ValidationLimits,
};
pub use ports::{
    BlockProvider, BlockProviderError, SelectedChainRepository, SelectedChainRepositoryError,
    SelectionConflict,
};
pub use selection::{
    ApplyOutcome, BlockIdentity, BlockRef, ChainMutation, ChainRevision, ChainSnapshot,
    ExpectedChainState, LeaseDuration, LeaseFence, LeaseGrant, LeaseOwner, MutationBuildError,
    MutationId, MutationKind, SelectionBoundaryError,
};
