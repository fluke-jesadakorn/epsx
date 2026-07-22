//! Dormant, provider-neutral block-ingestion contracts.
//!
//! This module does not start a worker, call an RPC endpoint, write SQL, or
//! assert canonicality/finality. It only validates a fetched block as one
//! internally consistent storage batch and defines ports for later adapters.

mod domain;
mod ports;

#[cfg(test)]
mod memory;

pub use domain::{
    validate_block, BlockHash, BlockNumber, BlockRequest, BoundaryError, ChainId, FetchedBlock,
    FetchedLog, FetchedReceipt, FetchedTransaction, LogIndex, ReceiptOutcome, TransactionHash,
    TxIndex, ValidatedBlock, ValidatedBlockBatch, ValidatedLog, ValidatedReceipt,
    ValidatedTransaction, ValidationError, ValidationLimits,
};
pub use ports::{
    BlockProvider, BlockProviderError, BlockRepository, BlockRepositoryError, CommitOutcome,
    RepositoryConflict,
};
