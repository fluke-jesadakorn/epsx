use super::{
    BlockHash, BlockRepository, BlockRepositoryError, BlockRequest, ChainId, CommitOutcome,
    RepositoryConflict, TransactionHash, TxIndex, ValidatedBlockBatch,
};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

/// Test-only reference implementation for the repository's atomicity and
/// conflict contract. This is not a durable runtime adapter.
#[derive(Default)]
pub(super) struct MemoryBlockRepository {
    state: Mutex<MemoryState>,
}

#[derive(Default)]
struct MemoryState {
    heights: HashMap<(ChainId, super::BlockNumber), BlockHash>,
    blocks: HashMap<(ChainId, BlockHash), ValidatedBlockBatch>,
    transactions: HashMap<(ChainId, TransactionHash), (BlockHash, TxIndex)>,
}

impl MemoryBlockRepository {
    fn lock(&self) -> Result<MutexGuard<'_, MemoryState>, BlockRepositoryError> {
        self.state
            .lock()
            .map_err(|_| BlockRepositoryError::Unavailable("test mutex poisoned".into()))
    }

    fn counts(&self) -> (usize, usize, usize) {
        let state = self.state.lock().expect("test mutex must not be poisoned");
        (
            state.heights.len(),
            state.blocks.len(),
            state.transactions.len(),
        )
    }

    fn remove_transaction(&self, chain_id: ChainId, hash: TransactionHash) {
        self.state
            .lock()
            .expect("test mutex must not be poisoned")
            .transactions
            .remove(&(chain_id, hash));
    }
}

#[async_trait]
impl BlockRepository for MemoryBlockRepository {
    async fn commit(
        &self,
        batch: ValidatedBlockBatch,
    ) -> Result<CommitOutcome, BlockRepositoryError> {
        let chain_id = batch.block().chain_id();
        let number = batch.block().number();
        let block_hash = batch.block().hash();
        let height_key = (chain_id, number);
        let block_key = (chain_id, block_hash);
        let mut state = self.lock()?;

        if let Some(stored_hash) = state.heights.get(&height_key) {
            if *stored_hash != block_hash {
                return Err(RepositoryConflict::Height {
                    chain_id,
                    number,
                    stored: *stored_hash,
                    candidate: block_hash,
                }
                .into());
            }
            return match state.blocks.get(&block_key) {
                Some(stored) if stored == &batch => {
                    let linked_transactions = state
                        .transactions
                        .iter()
                        .filter(|((stored_chain, _), (stored_block, _))| {
                            *stored_chain == chain_id && *stored_block == block_hash
                        })
                        .count();
                    if linked_transactions != batch.transactions().len()
                        || batch.transactions().iter().any(|transaction| {
                            state.transactions.get(&(chain_id, transaction.hash()))
                                != Some(&(block_hash, transaction.transaction_index()))
                        })
                    {
                        return Err(BlockRepositoryError::CorruptState(
                            "exact block replay has an inconsistent transaction index".into(),
                        ));
                    }
                    Ok(CommitOutcome::AlreadyStored)
                }
                Some(stored) => Err(RepositoryConflict::BlockHash {
                    chain_id,
                    hash: block_hash,
                    stored_number: stored.block().number(),
                    candidate_number: number,
                }
                .into()),
                None => Err(BlockRepositoryError::CorruptState(
                    "height points to a missing block".into(),
                )),
            };
        }

        if let Some(stored) = state.blocks.get(&block_key) {
            if stored != &batch {
                return Err(RepositoryConflict::BlockHash {
                    chain_id,
                    hash: block_hash,
                    stored_number: stored.block().number(),
                    candidate_number: number,
                }
                .into());
            }
            return Err(BlockRepositoryError::CorruptState(
                "block exists without its height index".into(),
            ));
        }

        for transaction in batch.transactions() {
            let tx_key = (chain_id, transaction.hash());
            let expected = (block_hash, transaction.transaction_index());
            if let Some(stored) = state.transactions.get(&tx_key) {
                if *stored == expected {
                    return Err(BlockRepositoryError::CorruptState(
                        "transaction exists without its block".into(),
                    ));
                }
                return Err(RepositoryConflict::TransactionHash {
                    chain_id,
                    hash: transaction.hash(),
                    stored_block: stored.0,
                    candidate_block: block_hash,
                }
                .into());
            }
        }

        // All conflicts are checked before the first mutation.
        state.heights.insert(height_key, block_hash);
        for transaction in batch.transactions() {
            state.transactions.insert(
                (chain_id, transaction.hash()),
                (block_hash, transaction.transaction_index()),
            );
        }
        state.blocks.insert(block_key, batch);
        Ok(CommitOutcome::Inserted)
    }

    async fn load(
        &self,
        request: BlockRequest,
    ) -> Result<Option<ValidatedBlockBatch>, BlockRepositoryError> {
        let state = self.lock()?;
        let Some(hash) = state.heights.get(&(request.chain_id(), request.number())) else {
            return Ok(None);
        };
        state
            .blocks
            .get(&(request.chain_id(), *hash))
            .cloned()
            .map(Some)
            .ok_or_else(|| {
                BlockRepositoryError::CorruptState("height points to a missing block".into())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingestion::{
        validate_block, BlockNumber, BoundaryError, FetchedBlock, FetchedLog, FetchedReceipt,
        FetchedTransaction, LogIndex, ReceiptOutcome, TxIndex, ValidationError, ValidationLimits,
    };
    use alloy::primitives::{Address, B256, U256};
    use std::sync::Arc;
    use tokio::sync::Barrier;

    fn hash(marker: u8) -> B256 {
        B256::from([marker; 32])
    }

    fn address(marker: u8) -> Address {
        Address::from([marker; 20])
    }

    fn request(chain_id: u64, number: u64) -> BlockRequest {
        BlockRequest::new(
            ChainId::new(chain_id).unwrap(),
            BlockNumber::new(number).unwrap(),
        )
    }

    fn fetched_block() -> FetchedBlock {
        let block_hash = hash(10);
        let tx_zero = FetchedTransaction {
            hash: hash(20),
            block_hash,
            block_number: 100,
            transaction_index: 0,
            from: address(1),
            to: Some(address(2)),
            value: U256::from(100),
            input: vec![1, 2],
        };
        let tx_one = FetchedTransaction {
            hash: hash(21),
            block_hash,
            block_number: 100,
            transaction_index: 1,
            from: address(3),
            to: None,
            value: U256::from(200),
            input: vec![3, 4],
        };
        let receipt_zero = FetchedReceipt {
            transaction_hash: tx_zero.hash,
            transaction_index: 0,
            block_hash,
            block_number: 100,
            outcome: ReceiptOutcome::Succeeded,
            gas_used: 21,
            cumulative_gas_used: 21,
            logs: vec![FetchedLog {
                block_hash,
                block_number: 100,
                transaction_hash: tx_zero.hash,
                transaction_index: 0,
                log_index: 0,
                address: address(4),
                topics: vec![hash(30)],
                data: vec![5],
                removed: false,
            }],
        };
        let receipt_one = FetchedReceipt {
            transaction_hash: tx_one.hash,
            transaction_index: 1,
            block_hash,
            block_number: 100,
            outcome: ReceiptOutcome::PostStateRoot(hash(40)),
            gas_used: 21,
            cumulative_gas_used: 42,
            logs: vec![FetchedLog {
                block_hash,
                block_number: 100,
                transaction_hash: tx_one.hash,
                transaction_index: 1,
                log_index: 1,
                address: address(5),
                topics: vec![hash(31)],
                data: vec![6],
                removed: false,
            }],
        };
        FetchedBlock {
            chain_id: 56,
            number: 100,
            hash: block_hash,
            parent_hash: hash(9),
            timestamp: 1_700_000_000,
            beneficiary: Some(address(9)),
            gas_used: 42,
            gas_limit: 100,
            transactions: vec![tx_one, tx_zero],
            receipts: vec![receipt_one, receipt_zero],
        }
    }

    fn validated() -> ValidatedBlockBatch {
        validate_block(
            request(56, 100),
            fetched_block(),
            ValidationLimits::default(),
        )
        .unwrap()
    }

    fn rebase(
        mut fetched: FetchedBlock,
        chain_id: u64,
        number: u64,
        block_marker: u8,
        tx_marker_delta: u8,
    ) -> FetchedBlock {
        let old_tx_hashes: Vec<_> = fetched.transactions.iter().map(|tx| tx.hash).collect();
        let new_block_hash = hash(block_marker);
        fetched.chain_id = chain_id;
        fetched.number = number;
        fetched.hash = new_block_hash;
        for transaction in &mut fetched.transactions {
            let old_hash = transaction.hash;
            transaction.block_hash = new_block_hash;
            transaction.block_number = number;
            if tx_marker_delta != 0 {
                let position = old_tx_hashes
                    .iter()
                    .position(|hash| *hash == old_hash)
                    .unwrap();
                transaction.hash = hash(20 + position as u8 + tx_marker_delta);
            }
        }
        for receipt in &mut fetched.receipts {
            let position = old_tx_hashes
                .iter()
                .position(|hash| *hash == receipt.transaction_hash)
                .unwrap();
            receipt.block_hash = new_block_hash;
            receipt.block_number = number;
            if tx_marker_delta != 0 {
                receipt.transaction_hash = hash(20 + position as u8 + tx_marker_delta);
            }
            for log in &mut receipt.logs {
                log.block_hash = new_block_hash;
                log.block_number = number;
                if tx_marker_delta != 0 {
                    log.transaction_hash = receipt.transaction_hash;
                }
            }
        }
        fetched
    }

    fn assert_validation_error(fetched: FetchedBlock, expected: ValidationError) {
        assert_eq!(
            validate_block(request(56, 100), fetched, ValidationLimits::default()),
            Err(expected)
        );
    }

    #[test]
    fn checked_value_objects_reject_storage_unsafe_values() {
        assert_eq!(ChainId::new(0), Err(BoundaryError::InvalidChainId(0)));
        assert_eq!(
            ChainId::new(10_000_000_000),
            Err(BoundaryError::InvalidChainId(10_000_000_000))
        );
        assert_eq!(
            BlockNumber::new(i64::MAX as u64 + 1),
            Err(BoundaryError::BlockNumber(i64::MAX as u64 + 1))
        );
        assert_eq!(
            TxIndex::new(i32::MAX as u64 + 1),
            Err(BoundaryError::TransactionIndex(i32::MAX as u64 + 1))
        );
        assert_eq!(
            LogIndex::new(i32::MAX as u64 + 1),
            Err(BoundaryError::LogIndex(i32::MAX as u64 + 1))
        );
        assert_eq!(
            BlockHash::new(B256::ZERO),
            Err(BoundaryError::ZeroBlockHash)
        );
        assert_eq!(
            TransactionHash::new(B256::ZERO),
            Err(BoundaryError::ZeroTransactionHash)
        );
    }

    #[test]
    fn validator_rechecks_all_provider_storage_boundaries() {
        let mut fetched = fetched_block();
        fetched.chain_id = 0;
        assert_validation_error(
            fetched,
            ValidationError::Boundary(BoundaryError::InvalidChainId(0)),
        );

        let mut fetched = fetched_block();
        fetched.number = i64::MAX as u64 + 1;
        assert_validation_error(
            fetched,
            ValidationError::Boundary(BoundaryError::BlockNumber(i64::MAX as u64 + 1)),
        );

        let mut fetched = fetched_block();
        fetched.hash = B256::ZERO;
        assert_validation_error(
            fetched,
            ValidationError::Boundary(BoundaryError::ZeroBlockHash),
        );

        let mut fetched = fetched_block();
        fetched.transactions[0].hash = B256::ZERO;
        assert_validation_error(
            fetched,
            ValidationError::Boundary(BoundaryError::ZeroTransactionHash),
        );

        let mut fetched = fetched_block();
        fetched.transactions[0].transaction_index = i32::MAX as u64 + 1;
        assert_validation_error(
            fetched,
            ValidationError::Boundary(BoundaryError::TransactionIndex(i32::MAX as u64 + 1)),
        );

        let mut fetched = fetched_block();
        fetched.receipts[0].logs[0].log_index = i32::MAX as u64 + 1;
        assert_validation_error(
            fetched,
            ValidationError::Boundary(BoundaryError::LogIndex(i32::MAX as u64 + 1)),
        );

        let mut fetched = fetched_block();
        fetched.receipts[0].gas_used = i64::MAX as u64 + 1;
        assert_validation_error(
            fetched,
            ValidationError::GasStorage {
                field: "receipt.gas_used",
                value: i64::MAX as u64 + 1,
            },
        );
    }

    #[test]
    fn shuffled_provider_data_normalizes_to_identical_batches() {
        let shuffled = fetched_block();
        let mut ordered = shuffled.clone();
        ordered.transactions.sort_by_key(|tx| tx.transaction_index);
        ordered
            .receipts
            .sort_by_key(|receipt| receipt.transaction_index);
        let left = validate_block(request(56, 100), shuffled, ValidationLimits::default()).unwrap();
        let right = validate_block(request(56, 100), ordered, ValidationLimits::default()).unwrap();
        assert_eq!(left, right);
        assert_eq!(
            left.transactions()
                .iter()
                .map(|tx| tx.transaction_index().get())
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(
            left.logs()
                .iter()
                .map(|log| log.log_index().get())
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    #[test]
    fn empty_block_requires_zero_gas_and_is_accepted() {
        let mut fetched = fetched_block();
        fetched.transactions.clear();
        fetched.receipts.clear();
        fetched.gas_used = 0;
        let batch = validate_block(request(56, 100), fetched, ValidationLimits::default()).unwrap();
        assert!(batch.transactions().is_empty());
        assert!(batch.receipts().is_empty());
        assert!(batch.logs().is_empty());
    }

    #[test]
    fn request_timestamp_and_block_gas_invariants_fail_closed() {
        assert_eq!(
            validate_block(
                request(97, 100),
                fetched_block(),
                ValidationLimits::default()
            ),
            Err(ValidationError::ChainMismatch {
                expected: ChainId::new(97).unwrap(),
                actual: ChainId::new(56).unwrap(),
            })
        );
        assert_eq!(
            validate_block(
                request(56, 101),
                fetched_block(),
                ValidationLimits::default()
            ),
            Err(ValidationError::HeightMismatch {
                expected: BlockNumber::new(101).unwrap(),
                actual: BlockNumber::new(100).unwrap(),
            })
        );
        let mut fetched = fetched_block();
        fetched.timestamp = u64::MAX;
        assert_validation_error(fetched, ValidationError::Timestamp(u64::MAX));

        let mut fetched = fetched_block();
        fetched.gas_limit = i64::MAX as u64 + 1;
        assert_validation_error(
            fetched,
            ValidationError::GasStorage {
                field: "block.gas_limit",
                value: i64::MAX as u64 + 1,
            },
        );
        let mut fetched = fetched_block();
        fetched.gas_limit = 41;
        assert_validation_error(fetched, ValidationError::BlockGasBounds);
    }

    #[test]
    fn transaction_identity_order_and_block_references_are_exact() {
        let mut fetched = fetched_block();
        fetched.transactions[1].hash = fetched.transactions[0].hash;
        assert_validation_error(fetched, ValidationError::DuplicateTransactionHash);

        let mut fetched = fetched_block();
        fetched.transactions[1].transaction_index = fetched.transactions[0].transaction_index;
        assert_validation_error(fetched, ValidationError::DuplicateTransactionIndex);

        let mut fetched = fetched_block();
        fetched.transactions[0].transaction_index = 2;
        fetched.receipts[0].transaction_index = 2;
        fetched.receipts[0].logs[0].transaction_index = 2;
        assert_validation_error(fetched, ValidationError::NonDenseTransactionIndices);

        let mut fetched = fetched_block();
        fetched.transactions[0].block_hash = hash(99);
        assert_validation_error(fetched, ValidationError::TransactionBlockReference);
    }

    #[test]
    fn receipt_coverage_identity_and_block_references_are_exact() {
        let mut fetched = fetched_block();
        fetched.receipts.pop();
        assert_validation_error(fetched, ValidationError::ReceiptCoverage);

        let mut fetched = fetched_block();
        fetched.receipts[1].transaction_hash = fetched.receipts[0].transaction_hash;
        assert_validation_error(fetched, ValidationError::DuplicateReceiptHash);

        let mut fetched = fetched_block();
        fetched.receipts[1].transaction_index = fetched.receipts[0].transaction_index;
        assert_validation_error(fetched, ValidationError::DuplicateReceiptIndex);

        let mut fetched = fetched_block();
        fetched.receipts.swap(0, 1);
        let hashes: Vec<_> = fetched
            .receipts
            .iter()
            .map(|receipt| receipt.transaction_hash)
            .collect();
        fetched.receipts[0].transaction_hash = hashes[1];
        fetched.receipts[1].transaction_hash = hashes[0];
        assert_validation_error(fetched, ValidationError::ReceiptTransactionReference);

        let mut fetched = fetched_block();
        fetched.receipts[0].block_number = 99;
        assert_validation_error(fetched, ValidationError::ReceiptBlockReference);
    }

    #[test]
    fn receipt_gas_is_monotonic_delta_exact_and_equals_block_gas() {
        let mut fetched = fetched_block();
        fetched.receipts[0].cumulative_gas_used = 20;
        assert_validation_error(fetched, ValidationError::CumulativeGasDecreased);

        let mut fetched = fetched_block();
        fetched.receipts[1].gas_used = 20;
        assert_validation_error(fetched, ValidationError::ReceiptGasDelta);

        let mut fetched = fetched_block();
        fetched.gas_used = 43;
        assert_validation_error(fetched, ValidationError::FinalCumulativeGas);

        let mut fetched = fetched_block();
        fetched.transactions.clear();
        fetched.receipts.clear();
        assert_validation_error(fetched, ValidationError::FinalCumulativeGas);
    }

    #[test]
    fn logs_require_active_exact_dense_provider_metadata() {
        let mut fetched = fetched_block();
        fetched.receipts[0].logs[0].removed = true;
        assert_validation_error(fetched, ValidationError::RemovedLog);

        let mut fetched = fetched_block();
        fetched.receipts[0].logs[0].topics = vec![hash(1); 5];
        assert_validation_error(fetched, ValidationError::TooManyTopics);

        let mut fetched = fetched_block();
        fetched.receipts[0].logs[0].block_hash = hash(99);
        assert_validation_error(fetched, ValidationError::LogBlockReference);

        let mut fetched = fetched_block();
        fetched.receipts[0].logs[0].transaction_hash = hash(99);
        assert_validation_error(fetched, ValidationError::LogTransactionReference);

        let mut fetched = fetched_block();
        fetched.receipts[1].logs[0].log_index = fetched.receipts[0].logs[0].log_index;
        assert_validation_error(fetched, ValidationError::DuplicateLogIndex);

        let mut fetched = fetched_block();
        for receipt in &mut fetched.receipts {
            for log in &mut receipt.logs {
                log.log_index += 1;
            }
        }
        assert_validation_error(fetched, ValidationError::NonDenseLogIndices);
    }

    #[test]
    fn configured_count_and_payload_limits_are_enforced() {
        let fetched = fetched_block();
        for (limits, kind, actual, limit) in [
            (
                ValidationLimits::new(1, 10, 10, 10, 1_000),
                "transactions",
                2,
                1,
            ),
            (ValidationLimits::new(10, 1, 10, 10, 1_000), "logs", 2, 1),
            (
                ValidationLimits::new(10, 10, 1, 10, 1_000),
                "transaction input bytes",
                2,
                1,
            ),
            (
                ValidationLimits::new(10, 10, 10, 0, 1_000),
                "log data bytes",
                1,
                0,
            ),
            (
                ValidationLimits::new(10, 10, 10, 10, 69),
                "total payload bytes",
                70,
                69,
            ),
        ] {
            assert_eq!(
                validate_block(request(56, 100), fetched.clone(), limits),
                Err(ValidationError::LimitExceeded {
                    kind,
                    actual,
                    limit,
                })
            );
        }
    }

    #[tokio::test]
    async fn repository_commits_loads_and_replays_identical_batches() {
        let repository = MemoryBlockRepository::default();
        let batch = validated();
        assert_eq!(
            repository.commit(batch.clone()).await.unwrap(),
            CommitOutcome::Inserted
        );
        assert_eq!(
            repository.load(batch.request()).await.unwrap(),
            Some(batch.clone())
        );
        assert_eq!(
            repository.commit(batch).await.unwrap(),
            CommitOutcome::AlreadyStored
        );
        assert_eq!(repository.counts(), (1, 1, 2));
    }

    #[tokio::test]
    async fn exact_replay_rejects_a_missing_transaction_secondary_mapping() {
        let repository = MemoryBlockRepository::default();
        let batch = validated();
        repository.commit(batch.clone()).await.unwrap();
        repository.remove_transaction(batch.block().chain_id(), batch.transactions()[0].hash());
        let corrupted_counts = repository.counts();

        assert_eq!(
            repository.commit(batch).await,
            Err(BlockRepositoryError::CorruptState(
                "exact block replay has an inconsistent transaction index".into()
            ))
        );
        assert_eq!(repository.counts(), corrupted_counts);
    }

    #[tokio::test]
    async fn repository_reports_height_block_hash_and_transaction_conflicts() {
        let repository = MemoryBlockRepository::default();
        repository.commit(validated()).await.unwrap();

        let height = rebase(fetched_block(), 56, 100, 11, 10);
        assert!(matches!(
            repository
                .commit(
                    validate_block(request(56, 100), height, ValidationLimits::default()).unwrap()
                )
                .await,
            Err(BlockRepositoryError::Conflict(
                RepositoryConflict::Height { .. }
            ))
        ));

        let block_hash = rebase(fetched_block(), 56, 101, 10, 10);
        assert!(matches!(
            repository
                .commit(
                    validate_block(request(56, 101), block_hash, ValidationLimits::default(),)
                        .unwrap()
                )
                .await,
            Err(BlockRepositoryError::Conflict(
                RepositoryConflict::BlockHash {
                    chain_id,
                    hash: conflicting_hash,
                    stored_number,
                    candidate_number,
                }
            )) if chain_id == ChainId::new(56).unwrap()
                && conflicting_hash == BlockHash::new(hash(10)).unwrap()
                && stored_number == BlockNumber::new(100).unwrap()
                && candidate_number == BlockNumber::new(101).unwrap()
        ));

        let transaction = rebase(fetched_block(), 56, 101, 11, 0);
        assert!(matches!(
            repository
                .commit(
                    validate_block(request(56, 101), transaction, ValidationLimits::default(),)
                        .unwrap()
                )
                .await,
            Err(BlockRepositoryError::Conflict(
                RepositoryConflict::TransactionHash { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn rejected_commit_rolls_back_without_partial_state() {
        let repository = MemoryBlockRepository::default();
        repository.commit(validated()).await.unwrap();
        let before = repository.counts();

        let mut candidate = rebase(fetched_block(), 56, 101, 11, 10);
        let colliding_hash = hash(20);
        candidate.transactions[0].hash = colliding_hash;
        candidate.receipts[0].transaction_hash = colliding_hash;
        candidate.receipts[0].logs[0].transaction_hash = colliding_hash;
        let candidate =
            validate_block(request(56, 101), candidate, ValidationLimits::default()).unwrap();
        assert!(matches!(
            repository.commit(candidate).await,
            Err(BlockRepositoryError::Conflict(
                RepositoryConflict::TransactionHash { .. }
            ))
        ));
        assert_eq!(repository.counts(), before);
        assert_eq!(repository.load(request(56, 101)).await.unwrap(), None);
    }

    #[tokio::test]
    async fn repository_keys_are_chain_scoped_and_missing_load_is_none() {
        let repository = MemoryBlockRepository::default();
        let chain_56 = validated();
        let chain_97_fetched = rebase(fetched_block(), 97, 100, 10, 0);
        let chain_97 = validate_block(
            request(97, 100),
            chain_97_fetched,
            ValidationLimits::default(),
        )
        .unwrap();
        repository.commit(chain_56.clone()).await.unwrap();
        repository.commit(chain_97.clone()).await.unwrap();
        assert_eq!(
            repository.load(chain_56.request()).await.unwrap(),
            Some(chain_56)
        );
        assert_eq!(
            repository.load(chain_97.request()).await.unwrap(),
            Some(chain_97)
        );
        assert_eq!(repository.load(request(56, 999)).await.unwrap(), None);
        assert_eq!(repository.counts(), (2, 2, 4));
    }

    #[tokio::test]
    async fn repository_detects_an_internally_corrupt_height_index() {
        let repository = MemoryBlockRepository::default();
        let batch = validated();
        repository.commit(batch.clone()).await.unwrap();
        repository
            .state
            .lock()
            .unwrap()
            .blocks
            .remove(&(batch.block().chain_id(), batch.block().hash()));
        assert_eq!(
            repository.load(batch.request()).await,
            Err(BlockRepositoryError::CorruptState(
                "height points to a missing block".into()
            ))
        );
    }

    #[tokio::test]
    async fn concurrent_competing_height_commits_have_one_winner() {
        let repository = Arc::new(MemoryBlockRepository::default());
        let first = validated();
        let second_fetched = rebase(fetched_block(), 56, 100, 11, 10);
        let second = validate_block(
            request(56, 100),
            second_fetched,
            ValidationLimits::default(),
        )
        .unwrap();
        let barrier = Arc::new(Barrier::new(3));

        let one = {
            let repository = repository.clone();
            let barrier = barrier.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                repository.commit(first).await
            })
        };
        let two = {
            let repository = repository.clone();
            let barrier = barrier.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                repository.commit(second).await
            })
        };
        barrier.wait().await;
        let results = [one.await.unwrap(), two.await.unwrap()];
        assert_eq!(
            results
                .iter()
                .filter(|result| result == &&Ok(CommitOutcome::Inserted))
                .count(),
            1
        );
        assert_eq!(
            results
                .iter()
                .filter(|result| {
                    matches!(
                        result,
                        Err(BlockRepositoryError::Conflict(
                            RepositoryConflict::Height { .. }
                        ))
                    )
                })
                .count(),
            1
        );
        assert_eq!(repository.counts(), (1, 1, 2));
    }
}
