use alloy::primitives::{Address, B256, U256};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChainId(u64);

impl ChainId {
    pub fn new(value: u64) -> Result<Self, BoundaryError> {
        if value == 0 || value.to_string().len() > 10 {
            return Err(BoundaryError::InvalidChainId(value));
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockNumber(i64);

impl BlockNumber {
    pub fn new(value: u64) -> Result<Self, BoundaryError> {
        let value = i64::try_from(value).map_err(|_| BoundaryError::BlockNumber(value))?;
        Ok(Self(value))
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TxIndex(i32);

impl TxIndex {
    pub fn new(value: u64) -> Result<Self, BoundaryError> {
        let value = i32::try_from(value).map_err(|_| BoundaryError::TransactionIndex(value))?;
        Ok(Self(value))
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LogIndex(i32);

impl LogIndex {
    pub fn new(value: u64) -> Result<Self, BoundaryError> {
        let value = i32::try_from(value).map_err(|_| BoundaryError::LogIndex(value))?;
        Ok(Self(value))
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockHash(B256);

impl BlockHash {
    pub fn new(value: B256) -> Result<Self, BoundaryError> {
        if value == B256::ZERO {
            return Err(BoundaryError::ZeroBlockHash);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> B256 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionHash(B256);

impl TransactionHash {
    pub fn new(value: B256) -> Result<Self, BoundaryError> {
        if value == B256::ZERO {
            return Err(BoundaryError::ZeroTransactionHash);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> B256 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockRequest {
    chain_id: ChainId,
    number: BlockNumber,
}

impl BlockRequest {
    pub const fn new(chain_id: ChainId, number: BlockNumber) -> Self {
        Self { chain_id, number }
    }

    pub const fn chain_id(self) -> ChainId {
        self.chain_id
    }

    pub const fn number(self) -> BlockNumber {
        self.number
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BoundaryError {
    #[error("chain id must be non-zero and no more than ten decimal digits: {0}")]
    InvalidChainId(u64),
    #[error("block number exceeds signed 64-bit storage: {0}")]
    BlockNumber(u64),
    #[error("transaction index exceeds signed 32-bit storage: {0}")]
    TransactionIndex(u64),
    #[error("log index exceeds signed 32-bit storage: {0}")]
    LogIndex(u64),
    #[error("block hash must not be zero")]
    ZeroBlockHash,
    #[error("transaction hash must not be zero")]
    ZeroTransactionHash,
}

/// Provider-shaped, untrusted input. Public fields are intentional: only
/// `validate_block` can turn this value into a checked batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedBlock {
    pub chain_id: u64,
    pub number: u64,
    pub hash: B256,
    pub parent_hash: B256,
    pub timestamp: u64,
    pub beneficiary: Option<Address>,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub transactions: Vec<FetchedTransaction>,
    pub receipts: Vec<FetchedReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedTransaction {
    pub hash: B256,
    pub block_hash: B256,
    pub block_number: u64,
    pub transaction_index: u64,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub input: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptOutcome {
    Succeeded,
    Reverted,
    PostStateRoot(B256),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedReceipt {
    pub transaction_hash: B256,
    pub transaction_index: u64,
    pub block_hash: B256,
    pub block_number: u64,
    pub outcome: ReceiptOutcome,
    pub gas_used: u64,
    pub cumulative_gas_used: u64,
    pub logs: Vec<FetchedLog>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedLog {
    pub block_hash: B256,
    pub block_number: u64,
    pub transaction_hash: B256,
    pub transaction_index: u64,
    pub log_index: u64,
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Vec<u8>,
    pub removed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationLimits {
    max_transactions: usize,
    max_logs: usize,
    max_input_bytes: usize,
    max_log_data_bytes: usize,
    max_total_payload_bytes: usize,
}

impl ValidationLimits {
    pub const fn new(
        max_transactions: usize,
        max_logs: usize,
        max_input_bytes: usize,
        max_log_data_bytes: usize,
        max_total_payload_bytes: usize,
    ) -> Self {
        Self {
            max_transactions,
            max_logs,
            max_input_bytes,
            max_log_data_bytes,
            max_total_payload_bytes,
        }
    }
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self::new(10_000, 100_000, 1_048_576, 1_048_576, 64 * 1_048_576)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedBlock {
    chain_id: ChainId,
    number: BlockNumber,
    hash: BlockHash,
    parent_hash: B256,
    timestamp: DateTime<Utc>,
    beneficiary: Option<Address>,
    gas_used: i64,
    gas_limit: i64,
}

impl ValidatedBlock {
    pub const fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub const fn number(&self) -> BlockNumber {
        self.number
    }

    pub const fn hash(&self) -> BlockHash {
        self.hash
    }

    pub const fn parent_hash(&self) -> B256 {
        self.parent_hash
    }

    pub const fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub const fn beneficiary(&self) -> Option<Address> {
        self.beneficiary
    }

    pub const fn gas_used(&self) -> i64 {
        self.gas_used
    }

    pub const fn gas_limit(&self) -> i64 {
        self.gas_limit
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTransaction {
    hash: TransactionHash,
    block_hash: BlockHash,
    block_number: BlockNumber,
    transaction_index: TxIndex,
    from: Address,
    to: Option<Address>,
    value: U256,
    input: Vec<u8>,
}

impl ValidatedTransaction {
    pub const fn hash(&self) -> TransactionHash {
        self.hash
    }

    pub const fn block_hash(&self) -> BlockHash {
        self.block_hash
    }

    pub const fn block_number(&self) -> BlockNumber {
        self.block_number
    }

    pub const fn transaction_index(&self) -> TxIndex {
        self.transaction_index
    }

    pub const fn from(&self) -> Address {
        self.from
    }

    pub const fn to(&self) -> Option<Address> {
        self.to
    }

    pub const fn value(&self) -> U256 {
        self.value
    }

    pub fn input(&self) -> &[u8] {
        &self.input
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedReceipt {
    transaction_hash: TransactionHash,
    transaction_index: TxIndex,
    block_hash: BlockHash,
    block_number: BlockNumber,
    outcome: ReceiptOutcome,
    gas_used: i64,
    cumulative_gas_used: i64,
}

impl ValidatedReceipt {
    pub const fn transaction_hash(&self) -> TransactionHash {
        self.transaction_hash
    }

    pub const fn transaction_index(&self) -> TxIndex {
        self.transaction_index
    }

    pub const fn block_hash(&self) -> BlockHash {
        self.block_hash
    }

    pub const fn block_number(&self) -> BlockNumber {
        self.block_number
    }

    pub const fn outcome(&self) -> &ReceiptOutcome {
        &self.outcome
    }

    pub const fn gas_used(&self) -> i64 {
        self.gas_used
    }

    pub const fn cumulative_gas_used(&self) -> i64 {
        self.cumulative_gas_used
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedLog {
    block_hash: BlockHash,
    block_number: BlockNumber,
    transaction_hash: TransactionHash,
    transaction_index: TxIndex,
    log_index: LogIndex,
    address: Address,
    topics: Vec<B256>,
    data: Vec<u8>,
}

impl ValidatedLog {
    pub const fn block_hash(&self) -> BlockHash {
        self.block_hash
    }

    pub const fn block_number(&self) -> BlockNumber {
        self.block_number
    }

    pub const fn transaction_hash(&self) -> TransactionHash {
        self.transaction_hash
    }

    pub const fn transaction_index(&self) -> TxIndex {
        self.transaction_index
    }

    pub const fn log_index(&self) -> LogIndex {
        self.log_index
    }

    pub const fn address(&self) -> Address {
        self.address
    }

    pub fn topics(&self) -> &[B256] {
        &self.topics
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedBlockBatch {
    block: ValidatedBlock,
    transactions: Vec<ValidatedTransaction>,
    receipts: Vec<ValidatedReceipt>,
    logs: Vec<ValidatedLog>,
}

impl ValidatedBlockBatch {
    pub const fn block(&self) -> &ValidatedBlock {
        &self.block
    }

    pub fn transactions(&self) -> &[ValidatedTransaction] {
        &self.transactions
    }

    pub fn receipts(&self) -> &[ValidatedReceipt] {
        &self.receipts
    }

    pub fn logs(&self) -> &[ValidatedLog] {
        &self.logs
    }

    pub const fn request(&self) -> BlockRequest {
        BlockRequest::new(self.block.chain_id, self.block.number)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    #[error(transparent)]
    Boundary(#[from] BoundaryError),
    #[error("fetched chain {actual:?} does not match requested chain {expected:?}")]
    ChainMismatch { expected: ChainId, actual: ChainId },
    #[error("fetched height {actual:?} does not match requested height {expected:?}")]
    HeightMismatch {
        expected: BlockNumber,
        actual: BlockNumber,
    },
    #[error("block timestamp is outside the supported UTC range: {0}")]
    Timestamp(u64),
    #[error("{field} exceeds signed 64-bit storage: {value}")]
    GasStorage { field: &'static str, value: u64 },
    #[error("block gas used exceeds gas limit")]
    BlockGasBounds,
    #[error("{kind} limit exceeded: actual {actual}, limit {limit}")]
    LimitExceeded {
        kind: &'static str,
        actual: usize,
        limit: usize,
    },
    #[error("payload byte count overflowed")]
    PayloadOverflow,
    #[error("duplicate transaction hash")]
    DuplicateTransactionHash,
    #[error("duplicate transaction index")]
    DuplicateTransactionIndex,
    #[error("transaction indices are not dense from zero")]
    NonDenseTransactionIndices,
    #[error("transaction block reference does not match the fetched block")]
    TransactionBlockReference,
    #[error("receipt count does not exactly cover transactions")]
    ReceiptCoverage,
    #[error("duplicate receipt transaction hash")]
    DuplicateReceiptHash,
    #[error("duplicate receipt transaction index")]
    DuplicateReceiptIndex,
    #[error("receipt does not match its transaction hash and index")]
    ReceiptTransactionReference,
    #[error("receipt block reference does not match the fetched block")]
    ReceiptBlockReference,
    #[error("receipt cumulative gas decreased")]
    CumulativeGasDecreased,
    #[error("receipt gas does not equal the cumulative gas delta")]
    ReceiptGasDelta,
    #[error("final cumulative gas does not equal block gas used")]
    FinalCumulativeGas,
    #[error("removed logs cannot be stored in a fetched block batch")]
    RemovedLog,
    #[error("a log contains more than four topics")]
    TooManyTopics,
    #[error("log block reference does not match the fetched block")]
    LogBlockReference,
    #[error("log transaction reference does not match its receipt")]
    LogTransactionReference,
    #[error("duplicate global log index")]
    DuplicateLogIndex,
    #[error("global log indices are not dense from zero")]
    NonDenseLogIndices,
}

/// Validates storage safety and internal consistency only. Passing this check
/// does not imply that a block is fresh, canonical, finalized, linked to a
/// stored parent, or that its consensus header hash has been recomputed.
pub fn validate_block(
    request: BlockRequest,
    fetched: FetchedBlock,
    limits: ValidationLimits,
) -> Result<ValidatedBlockBatch, ValidationError> {
    let chain_id = ChainId::new(fetched.chain_id)?;
    let number = BlockNumber::new(fetched.number)?;
    if chain_id != request.chain_id {
        return Err(ValidationError::ChainMismatch {
            expected: request.chain_id,
            actual: chain_id,
        });
    }
    if number != request.number {
        return Err(ValidationError::HeightMismatch {
            expected: request.number,
            actual: number,
        });
    }

    let hash = BlockHash::new(fetched.hash)?;
    let timestamp_seconds = i64::try_from(fetched.timestamp)
        .map_err(|_| ValidationError::Timestamp(fetched.timestamp))?;
    let timestamp = DateTime::from_timestamp(timestamp_seconds, 0)
        .ok_or(ValidationError::Timestamp(fetched.timestamp))?;
    let gas_used = checked_gas("block.gas_used", fetched.gas_used)?;
    let gas_limit = checked_gas("block.gas_limit", fetched.gas_limit)?;
    if gas_used > gas_limit {
        return Err(ValidationError::BlockGasBounds);
    }
    check_limit(
        "transactions",
        fetched.transactions.len(),
        limits.max_transactions,
    )?;

    let mut payload_bytes = 0usize;
    let mut tx_hashes = HashSet::with_capacity(fetched.transactions.len());
    let mut tx_indices = HashSet::with_capacity(fetched.transactions.len());
    let mut transactions = Vec::with_capacity(fetched.transactions.len());
    for transaction in fetched.transactions {
        let tx_hash = TransactionHash::new(transaction.hash)?;
        if !tx_hashes.insert(tx_hash) {
            return Err(ValidationError::DuplicateTransactionHash);
        }
        let tx_index = TxIndex::new(transaction.transaction_index)?;
        if !tx_indices.insert(tx_index) {
            return Err(ValidationError::DuplicateTransactionIndex);
        }
        if transaction.block_hash != fetched.hash || transaction.block_number != fetched.number {
            return Err(ValidationError::TransactionBlockReference);
        }
        check_limit(
            "transaction input bytes",
            transaction.input.len(),
            limits.max_input_bytes,
        )?;
        add_payload(&mut payload_bytes, transaction.input.len(), &limits)?;
        transactions.push(ValidatedTransaction {
            hash: tx_hash,
            block_hash: hash,
            block_number: number,
            transaction_index: tx_index,
            from: transaction.from,
            to: transaction.to,
            value: transaction.value,
            input: transaction.input,
        });
    }
    transactions.sort_by_key(ValidatedTransaction::transaction_index);
    ensure_dense_transactions(&transactions)?;

    if fetched.receipts.len() != transactions.len() {
        return Err(ValidationError::ReceiptCoverage);
    }
    let total_logs = fetched.receipts.iter().try_fold(0usize, |total, receipt| {
        total
            .checked_add(receipt.logs.len())
            .ok_or(ValidationError::PayloadOverflow)
    })?;
    check_limit("logs", total_logs, limits.max_logs)?;
    let transactions_by_hash: HashMap<_, _> = transactions
        .iter()
        .map(|transaction| (transaction.hash(), transaction.transaction_index()))
        .collect();
    let mut receipt_hashes = HashSet::with_capacity(fetched.receipts.len());
    let mut receipt_indices = HashSet::with_capacity(fetched.receipts.len());
    let mut receipts = Vec::with_capacity(fetched.receipts.len());
    let mut logs = Vec::with_capacity(total_logs);
    let mut log_indices = HashSet::new();

    for receipt in fetched.receipts {
        let tx_hash = TransactionHash::new(receipt.transaction_hash)?;
        if !receipt_hashes.insert(tx_hash) {
            return Err(ValidationError::DuplicateReceiptHash);
        }
        let tx_index = TxIndex::new(receipt.transaction_index)?;
        if !receipt_indices.insert(tx_index) {
            return Err(ValidationError::DuplicateReceiptIndex);
        }
        if transactions_by_hash.get(&tx_hash) != Some(&tx_index) {
            return Err(ValidationError::ReceiptTransactionReference);
        }
        if receipt.block_hash != fetched.hash || receipt.block_number != fetched.number {
            return Err(ValidationError::ReceiptBlockReference);
        }
        let receipt_gas = checked_gas("receipt.gas_used", receipt.gas_used)?;
        let cumulative_gas =
            checked_gas("receipt.cumulative_gas_used", receipt.cumulative_gas_used)?;

        for log in receipt.logs {
            if log.removed {
                return Err(ValidationError::RemovedLog);
            }
            if log.topics.len() > 4 {
                return Err(ValidationError::TooManyTopics);
            }
            if log.block_hash != fetched.hash || log.block_number != fetched.number {
                return Err(ValidationError::LogBlockReference);
            }
            if log.transaction_hash != receipt.transaction_hash
                || log.transaction_index != receipt.transaction_index
            {
                return Err(ValidationError::LogTransactionReference);
            }
            let log_index = LogIndex::new(log.log_index)?;
            if !log_indices.insert(log_index) {
                return Err(ValidationError::DuplicateLogIndex);
            }
            check_limit("log data bytes", log.data.len(), limits.max_log_data_bytes)?;
            add_payload(&mut payload_bytes, log.data.len(), &limits)?;
            let topic_bytes = log
                .topics
                .len()
                .checked_mul(32)
                .ok_or(ValidationError::PayloadOverflow)?;
            add_payload(&mut payload_bytes, topic_bytes, &limits)?;
            logs.push(ValidatedLog {
                block_hash: hash,
                block_number: number,
                transaction_hash: tx_hash,
                transaction_index: tx_index,
                log_index,
                address: log.address,
                topics: log.topics,
                data: log.data,
            });
        }

        receipts.push(ValidatedReceipt {
            transaction_hash: tx_hash,
            transaction_index: tx_index,
            block_hash: hash,
            block_number: number,
            outcome: receipt.outcome,
            gas_used: receipt_gas,
            cumulative_gas_used: cumulative_gas,
        });
    }

    receipts.sort_by_key(ValidatedReceipt::transaction_index);
    validate_receipt_gas(&receipts, gas_used)?;
    logs.sort_by_key(ValidatedLog::log_index);
    ensure_dense_logs(&logs)?;

    Ok(ValidatedBlockBatch {
        block: ValidatedBlock {
            chain_id,
            number,
            hash,
            parent_hash: fetched.parent_hash,
            timestamp,
            beneficiary: fetched.beneficiary,
            gas_used,
            gas_limit,
        },
        transactions,
        receipts,
        logs,
    })
}

fn checked_gas(field: &'static str, value: u64) -> Result<i64, ValidationError> {
    i64::try_from(value).map_err(|_| ValidationError::GasStorage { field, value })
}

fn check_limit(kind: &'static str, actual: usize, limit: usize) -> Result<(), ValidationError> {
    if actual > limit {
        return Err(ValidationError::LimitExceeded {
            kind,
            actual,
            limit,
        });
    }
    Ok(())
}

fn add_payload(
    total: &mut usize,
    amount: usize,
    limits: &ValidationLimits,
) -> Result<(), ValidationError> {
    *total = total
        .checked_add(amount)
        .ok_or(ValidationError::PayloadOverflow)?;
    check_limit(
        "total payload bytes",
        *total,
        limits.max_total_payload_bytes,
    )
}

fn ensure_dense_transactions(transactions: &[ValidatedTransaction]) -> Result<(), ValidationError> {
    for (expected, transaction) in transactions.iter().enumerate() {
        let expected =
            i32::try_from(expected).map_err(|_| ValidationError::NonDenseTransactionIndices)?;
        if transaction.transaction_index().get() != expected {
            return Err(ValidationError::NonDenseTransactionIndices);
        }
    }
    Ok(())
}

fn validate_receipt_gas(
    receipts: &[ValidatedReceipt],
    block_gas_used: i64,
) -> Result<(), ValidationError> {
    let mut previous = 0i64;
    for receipt in receipts {
        let cumulative = receipt.cumulative_gas_used();
        if cumulative < previous {
            return Err(ValidationError::CumulativeGasDecreased);
        }
        let delta = cumulative - previous;
        if delta != receipt.gas_used() {
            return Err(ValidationError::ReceiptGasDelta);
        }
        previous = cumulative;
    }
    if previous != block_gas_used {
        return Err(ValidationError::FinalCumulativeGas);
    }
    Ok(())
}

fn ensure_dense_logs(logs: &[ValidatedLog]) -> Result<(), ValidationError> {
    for (expected, log) in logs.iter().enumerate() {
        let expected = i32::try_from(expected).map_err(|_| ValidationError::NonDenseLogIndices)?;
        if log.log_index().get() != expected {
            return Err(ValidationError::NonDenseLogIndices);
        }
    }
    Ok(())
}
