use super::{
    selection::SelectionChange, ApplyOutcome, BlockHash, BlockIdentity, BlockNumber, BlockRef,
    ChainId, ChainMutation, ChainRevision, ChainSnapshot, LeaseDuration, LeaseFence, LeaseGrant,
    LeaseOwner, MutationId, SelectedChainRepository, SelectedChainRepositoryError,
    SelectionConflict, ValidatedBlockBatch,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{Mutex, MutexGuard},
};

/// Test-only reference implementation for the repository's atomicity and
/// conflict contract. This is not a durable runtime adapter.
pub(super) struct MemoryBlockRepository {
    state: Mutex<MemoryState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemoryState {
    now: DateTime<Utc>,
    leases: HashMap<ChainId, LeaseState>,
    chains: HashMap<ChainId, ChainData>,
    journal: HashMap<(ChainId, MutationId), JournalEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct LeaseState {
    last_fence: Option<LeaseFence>,
    owner: Option<LeaseOwner>,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ChainData {
    revision: ChainRevision,
    selected: BTreeMap<BlockNumber, BlockHash>,
    candidates: HashMap<BlockHash, ValidatedBlockBatch>,
    candidates_by_height: BTreeMap<BlockNumber, BTreeSet<BlockHash>>,
    finalized_selection: Option<BlockRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JournalEntry {
    mutation: ChainMutation,
    outcome: ApplyOutcome,
}

impl Default for MemoryBlockRepository {
    fn default() -> Self {
        Self {
            state: Mutex::new(MemoryState {
                now: DateTime::from_timestamp(1_700_000_000, 0).expect("fixed test clock"),
                leases: HashMap::new(),
                chains: HashMap::new(),
                journal: HashMap::new(),
            }),
        }
    }
}

impl MemoryBlockRepository {
    fn lock(&self) -> Result<MutexGuard<'_, MemoryState>, SelectedChainRepositoryError> {
        self.state
            .lock()
            .map_err(|_| SelectedChainRepositoryError::Unavailable("test mutex poisoned".into()))
    }

    fn state_copy(&self) -> MemoryState {
        self.state
            .lock()
            .expect("test mutex must not be poisoned")
            .clone()
    }

    fn advance_clock(&self, duration: std::time::Duration) {
        let mut state = self.state.lock().expect("test mutex must not be poisoned");
        state.now += chrono::Duration::from_std(duration).expect("bounded test duration");
    }
}

#[async_trait]
impl SelectedChainRepository for MemoryBlockRepository {
    async fn acquire_lease(
        &self,
        chain_id: ChainId,
        owner: LeaseOwner,
        duration: LeaseDuration,
    ) -> Result<LeaseGrant, SelectedChainRepositoryError> {
        let mut state = self.lock()?;
        let now = state.now;
        let mut lease = state.leases.get(&chain_id).cloned().unwrap_or_default();
        if lease
            .expires_at
            .is_some_and(|expires_at| expires_at > now && lease.owner.is_some())
        {
            return Err(SelectionConflict::LeaseHeld { chain_id }.into());
        }
        let fence = LeaseFence::successor(lease.last_fence)?;
        let expires_at = add_duration(now, duration)?;
        lease.last_fence = Some(fence);
        lease.owner = Some(owner.clone());
        lease.expires_at = Some(expires_at);
        state.leases.insert(chain_id, lease);
        Ok(LeaseGrant::new(chain_id, owner, fence, expires_at))
    }

    async fn renew_lease(
        &self,
        grant: &LeaseGrant,
        duration: LeaseDuration,
    ) -> Result<LeaseGrant, SelectedChainRepositoryError> {
        let mut state = self.lock()?;
        validate_live_lease(&state, grant.chain_id(), grant.owner(), grant.fence())?;
        let expires_at = add_duration(state.now, duration)?;
        state
            .leases
            .get_mut(&grant.chain_id())
            .expect("validated lease exists")
            .expires_at = Some(expires_at);
        Ok(LeaseGrant::new(
            grant.chain_id(),
            grant.owner().clone(),
            grant.fence(),
            expires_at,
        ))
    }

    async fn release_lease(&self, grant: &LeaseGrant) -> Result<(), SelectedChainRepositoryError> {
        let mut state = self.lock()?;
        validate_live_lease(&state, grant.chain_id(), grant.owner(), grant.fence())?;
        let lease = state
            .leases
            .get_mut(&grant.chain_id())
            .expect("validated lease exists");
        lease.owner = None;
        lease.expires_at = None;
        Ok(())
    }

    async fn snapshot(
        &self,
        chain_id: ChainId,
    ) -> Result<ChainSnapshot, SelectedChainRepositoryError> {
        let state = self.lock()?;
        Ok(snapshot_for(&state, chain_id))
    }

    async fn load_candidate(
        &self,
        identity: BlockIdentity,
    ) -> Result<Option<ValidatedBlockBatch>, SelectedChainRepositoryError> {
        let state = self.lock()?;
        Ok(state
            .chains
            .get(&identity.chain_id())
            .and_then(|chain| chain.candidates.get(&identity.hash()))
            .cloned())
    }

    async fn selected_hash(
        &self,
        chain_id: ChainId,
        number: BlockNumber,
    ) -> Result<Option<BlockHash>, SelectedChainRepositoryError> {
        let state = self.lock()?;
        Ok(state
            .chains
            .get(&chain_id)
            .and_then(|chain| chain.selected.get(&number))
            .copied())
    }

    async fn candidates_at_height(
        &self,
        chain_id: ChainId,
        number: BlockNumber,
    ) -> Result<Vec<BlockRef>, SelectedChainRepositoryError> {
        let state = self.lock()?;
        Ok(state
            .chains
            .get(&chain_id)
            .and_then(|chain| chain.candidates_by_height.get(&number))
            .into_iter()
            .flatten()
            .copied()
            .map(|hash| BlockRef::new(chain_id, number, hash))
            .collect())
    }

    async fn apply(
        &self,
        mutation: ChainMutation,
    ) -> Result<ApplyOutcome, SelectedChainRepositoryError> {
        let mut state = self.lock()?;
        let journal_key = (mutation.chain_id(), mutation.mutation_id());
        if let Some(entry) = state.journal.get(&journal_key) {
            return if entry.mutation == mutation {
                Ok(entry.outcome.clone())
            } else {
                Err(SelectionConflict::MutationIdReuse {
                    mutation_id: mutation.mutation_id(),
                }
                .into())
            };
        }

        validate_live_lease(
            &state,
            mutation.chain_id(),
            mutation.owner(),
            mutation.fence(),
        )?;
        let actual = snapshot_for(&state, mutation.chain_id()).expected();
        if mutation.expected() != &actual {
            return Err(SelectionConflict::ExpectedState {
                expected: Box::new(mutation.expected().clone()),
                actual: Box::new(actual),
            }
            .into());
        }

        let mut working = state.clone();
        apply_new_mutation(&mut working, &mutation)?;
        let chain = working.chains.entry(mutation.chain_id()).or_default();
        chain.revision = chain.revision.next()?;
        let snapshot = snapshot_for(&working, mutation.chain_id());
        let outcome = ApplyOutcome::new(
            mutation.mutation_id(),
            snapshot.revision(),
            snapshot.selected_head(),
            snapshot.finalized_selection(),
            mutation.kind(),
        );
        working.journal.insert(
            journal_key,
            JournalEntry {
                mutation,
                outcome: outcome.clone(),
            },
        );
        *state = working;
        Ok(outcome)
    }
}

fn add_duration(
    now: DateTime<Utc>,
    duration: LeaseDuration,
) -> Result<DateTime<Utc>, SelectedChainRepositoryError> {
    let duration = chrono::Duration::from_std(duration.get())
        .map_err(|error| SelectedChainRepositoryError::Unavailable(error.to_string()))?;
    now.checked_add_signed(duration)
        .ok_or_else(|| SelectedChainRepositoryError::Unavailable("lease time overflow".into()))
}

fn validate_live_lease(
    state: &MemoryState,
    chain_id: ChainId,
    owner: &LeaseOwner,
    fence: LeaseFence,
) -> Result<(), SelectedChainRepositoryError> {
    let Some(lease) = state.leases.get(&chain_id) else {
        return Err(SelectionConflict::StaleLease.into());
    };
    if lease.owner.as_ref() != Some(owner)
        || lease.last_fence != Some(fence)
        || lease
            .expires_at
            .is_none_or(|expires_at| expires_at <= state.now)
    {
        return Err(SelectionConflict::StaleLease.into());
    }
    Ok(())
}

fn snapshot_for(state: &MemoryState, chain_id: ChainId) -> ChainSnapshot {
    let Some(chain) = state.chains.get(&chain_id) else {
        return ChainSnapshot::new(chain_id, ChainRevision::default(), None, None);
    };
    let selected_head = chain
        .selected
        .last_key_value()
        .map(|(number, hash)| BlockRef::new(chain_id, *number, *hash));
    ChainSnapshot::new(
        chain_id,
        chain.revision,
        selected_head,
        chain.finalized_selection,
    )
}

fn apply_new_mutation(
    state: &mut MemoryState,
    mutation: &ChainMutation,
) -> Result<(), SelectedChainRepositoryError> {
    let chain_id = mutation.chain_id();
    let chain = state.chains.entry(chain_id).or_default();
    match mutation.change() {
        SelectionChange::Initialize { attach } => {
            if !chain.selected.is_empty() {
                return Err(SelectionConflict::AlreadyInitialized.into());
            }
            insert_candidates(chain, attach)?;
            select_attachment(chain, attach);
        }
        SelectionChange::Extend { attach } => {
            if chain.selected.is_empty() {
                return Err(SelectionConflict::NotInitialized.into());
            }
            insert_candidates(chain, attach)?;
            select_attachment(chain, attach);
        }
        SelectionChange::Reorg {
            common_ancestor,
            detach,
            attach,
        } => {
            if chain.selected.get(&common_ancestor.number()) != Some(&common_ancestor.hash()) {
                return Err(SelectionConflict::CommonAncestorNotSelected.into());
            }
            let actual_detach: Vec<_> = chain
                .selected
                .range((
                    std::ops::Bound::Excluded(common_ancestor.number()),
                    std::ops::Bound::Unbounded,
                ))
                .map(|(number, hash)| BlockRef::new(chain_id, *number, *hash))
                .collect();
            if attach
                .iter()
                .map(BlockRef::from_batch)
                .eq(actual_detach.iter().copied())
            {
                return Err(SelectionConflict::ReorgNoop.into());
            }
            if &actual_detach != detach {
                return Err(SelectionConflict::DetachMismatch.into());
            }
            if chain
                .finalized_selection
                .is_some_and(|finalized| common_ancestor.number() < finalized.number())
            {
                return Err(SelectionConflict::FinalizedBoundary.into());
            }
            insert_candidates(chain, attach)?;
            for block in detach {
                chain.selected.remove(&block.number());
            }
            select_attachment(chain, attach);
        }
        SelectionChange::AdvanceFinalized { target } => {
            if chain.selected.get(&target.number()) != Some(&target.hash()) {
                return Err(SelectionConflict::FinalityTargetNotSelected.into());
            }
            chain.finalized_selection = Some(*target);
        }
    }
    Ok(())
}

fn insert_candidates(
    chain: &mut ChainData,
    attach: &[ValidatedBlockBatch],
) -> Result<(), SelectedChainRepositoryError> {
    for batch in attach {
        let identity = BlockIdentity::new(batch.block().chain_id(), batch.block().hash());
        if let Some(stored) = chain.candidates.get(&identity.hash()) {
            if stored != batch {
                return Err(SelectionConflict::CandidateContent { identity }.into());
            }
            continue;
        }
        chain.candidates.insert(identity.hash(), batch.clone());
        chain
            .candidates_by_height
            .entry(batch.block().number())
            .or_default()
            .insert(identity.hash());
    }
    Ok(())
}

fn select_attachment(chain: &mut ChainData, attach: &[ValidatedBlockBatch]) {
    for batch in attach {
        chain
            .selected
            .insert(batch.block().number(), batch.block().hash());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingestion::{
        validate_block, BlockNumber, BlockRequest, BoundaryError, ExpectedChainState, FetchedBlock,
        FetchedLog, FetchedReceipt, FetchedTransaction, LogIndex, MutationBuildError,
        ReceiptOutcome, SelectionBoundaryError, TransactionHash, TxIndex, ValidationError,
        ValidationLimits,
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
        assert!(LeaseOwner::new("").is_err());
        assert!(LeaseOwner::new("worker/unsafe").is_err());
        assert!(LeaseOwner::new("x".repeat(129)).is_err());
        assert_eq!(
            ChainRevision::new(i64::MAX as u64),
            Ok(ChainRevision::MAX_STORAGE)
        );
        assert_eq!(
            ChainRevision::new(i64::MAX as u64 + 1),
            Err(SelectionBoundaryError::ChainRevisionOutOfRange)
        );
        assert_eq!(
            ChainRevision::MAX_STORAGE.next(),
            Err(MutationBuildError::RevisionExhausted)
        );
        assert_eq!(
            LeaseFence::new(0),
            Err(SelectionBoundaryError::ZeroLeaseFence)
        );
        assert_eq!(
            LeaseFence::new(i64::MAX as u64),
            Ok(LeaseFence::MAX_STORAGE)
        );
        assert_eq!(
            LeaseFence::new(i64::MAX as u64 + 1),
            Err(SelectionBoundaryError::LeaseFenceOutOfRange)
        );
        assert_eq!(
            LeaseFence::successor(Some(LeaseFence::MAX_STORAGE)),
            Err(SelectionBoundaryError::LeaseFenceExhausted)
        );
        assert!(MutationId::new(B256::ZERO).is_err());
        assert!(LeaseDuration::new(std::time::Duration::ZERO).is_err());
        assert!(LeaseDuration::new(std::time::Duration::from_secs(86_401)).is_err());
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

    fn batch_at(
        chain_id: u64,
        number: u64,
        block_marker: u8,
        parent_marker: u8,
        tx_marker_delta: u8,
    ) -> ValidatedBlockBatch {
        let mut fetched = rebase(
            fetched_block(),
            chain_id,
            number,
            block_marker,
            tx_marker_delta,
        );
        fetched.parent_hash = hash(parent_marker);
        validate_block(
            request(chain_id, number),
            fetched,
            ValidationLimits::default(),
        )
        .unwrap()
    }

    fn chain() -> ChainId {
        ChainId::new(56).unwrap()
    }

    fn owner(marker: &str) -> LeaseOwner {
        LeaseOwner::new(marker).unwrap()
    }

    fn mutation_id(marker: u8) -> MutationId {
        MutationId::new(hash(marker)).unwrap()
    }

    fn lease_duration() -> LeaseDuration {
        LeaseDuration::new(std::time::Duration::from_secs(60)).unwrap()
    }

    async fn lease(repository: &MemoryBlockRepository, marker: &str) -> LeaseGrant {
        repository
            .acquire_lease(chain(), owner(marker), lease_duration())
            .await
            .unwrap()
    }

    async fn initialize(
        repository: &MemoryBlockRepository,
        grant: &LeaseGrant,
        id: u8,
        batches: Vec<ValidatedBlockBatch>,
    ) -> (ChainMutation, ApplyOutcome) {
        let mutation = ChainMutation::initialize(
            chain(),
            mutation_id(id),
            repository.snapshot(chain()).await.unwrap().expected(),
            grant.owner().clone(),
            grant.fence(),
            batches,
        )
        .unwrap();
        let outcome = repository.apply(mutation.clone()).await.unwrap();
        (mutation, outcome)
    }

    async fn extend(
        repository: &MemoryBlockRepository,
        grant: &LeaseGrant,
        id: u8,
        batches: Vec<ValidatedBlockBatch>,
    ) -> (ChainMutation, ApplyOutcome) {
        let mutation = ChainMutation::extend(
            chain(),
            mutation_id(id),
            repository.snapshot(chain()).await.unwrap().expected(),
            grant.owner().clone(),
            grant.fence(),
            batches,
        )
        .unwrap();
        let outcome = repository.apply(mutation.clone()).await.unwrap();
        (mutation, outcome)
    }

    #[test]
    fn initialize_requires_exact_zero_revision_empty_state() {
        let block = batch_at(56, 100, 10, 9, 0);
        let valid = ChainMutation::initialize(
            chain(),
            mutation_id(1),
            ExpectedChainState::empty(),
            owner("worker-a"),
            LeaseFence::new(1).unwrap(),
            vec![block.clone()],
        )
        .unwrap();
        assert_eq!(valid.expected(), &ExpectedChainState::empty());

        assert_eq!(
            ChainMutation::initialize(
                chain(),
                mutation_id(2),
                ExpectedChainState::new(ChainRevision::new(1).unwrap(), None, None),
                owner("worker-a"),
                LeaseFence::new(1).unwrap(),
                vec![block],
            ),
            Err(MutationBuildError::InitializeExpectedState)
        );
    }

    #[tokio::test]
    async fn fork_candidates_and_duplicate_transactions_are_retained_deterministically() {
        let repository = MemoryBlockRepository::default();
        let grant = lease(&repository, "worker-a").await;
        let block_100 = batch_at(56, 100, 10, 9, 0);
        initialize(&repository, &grant, 1, vec![block_100.clone()]).await;
        let block_101_a = batch_at(56, 101, 11, 10, 0);
        extend(&repository, &grant, 2, vec![block_101_a.clone()]).await;

        let block_101_b = batch_at(56, 101, 12, 10, 0);
        assert_eq!(
            block_101_a.transactions()[0].hash(),
            block_101_b.transactions()[0].hash()
        );
        let snapshot = repository.snapshot(chain()).await.unwrap();
        let mutation = ChainMutation::reorg(
            chain(),
            mutation_id(3),
            snapshot.expected(),
            grant.owner().clone(),
            grant.fence(),
            BlockRef::from_batch(&block_100),
            vec![BlockRef::from_batch(&block_101_a)],
            vec![block_101_b.clone()],
        )
        .unwrap();
        repository.apply(mutation).await.unwrap();

        assert_eq!(
            repository
                .selected_hash(chain(), BlockNumber::new(101).unwrap())
                .await
                .unwrap(),
            Some(block_101_b.block().hash())
        );
        let candidates = repository
            .candidates_at_height(chain(), BlockNumber::new(101).unwrap())
            .await
            .unwrap();
        assert_eq!(candidates.len(), 2);
        assert!(candidates
            .windows(2)
            .all(|pair| pair[0].hash() < pair[1].hash()));
        for block in [&block_101_a, &block_101_b] {
            assert_eq!(
                repository
                    .load_candidate(BlockRef::from_batch(block).identity())
                    .await
                    .unwrap(),
                Some(block.clone())
            );
        }
    }

    #[tokio::test]
    async fn lease_expiry_takeover_renew_release_and_stale_fence_fail_closed() {
        let repository = MemoryBlockRepository::default();
        let first = lease(&repository, "worker-a").await;
        assert!(matches!(
            repository
                .acquire_lease(chain(), owner("worker-b"), lease_duration())
                .await,
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::LeaseHeld { .. }
            ))
        ));
        let renewed = repository
            .renew_lease(&first, lease_duration())
            .await
            .unwrap();
        assert_eq!(renewed.fence(), first.fence());
        repository.advance_clock(std::time::Duration::from_secs(61));
        let second = lease(&repository, "worker-b").await;
        assert!(second.fence().get() > first.fence().get());

        let stale = ChainMutation::initialize(
            chain(),
            mutation_id(10),
            ExpectedChainState::empty(),
            first.owner().clone(),
            first.fence(),
            vec![batch_at(56, 100, 10, 9, 0)],
        )
        .unwrap();
        let before = repository.state_copy();
        assert_eq!(
            repository.apply(stale).await,
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::StaleLease
            ))
        );
        assert_eq!(repository.state_copy(), before);
        repository.release_lease(&second).await.unwrap();
        assert!(matches!(
            repository.renew_lease(&second, lease_duration()).await,
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::StaleLease
            ))
        ));
    }

    #[test]
    fn attachment_builders_reject_gaps_wrong_parents_and_wrong_chains() {
        let expected = ExpectedChainState::new(
            ChainRevision::new(1).unwrap(),
            Some(BlockRef::from_batch(&batch_at(56, 100, 10, 9, 0))),
            None,
        );
        let args = || {
            (
                chain(),
                mutation_id(20),
                expected.clone(),
                owner("worker-a"),
                LeaseFence::new(1).unwrap(),
            )
        };
        let (chain_id, id, expected, owner, fence) = args();
        assert_eq!(
            ChainMutation::extend(
                chain_id,
                id,
                expected,
                owner,
                fence,
                vec![batch_at(56, 102, 12, 10, 0)]
            ),
            Err(MutationBuildError::AttachmentGap)
        );
        let (chain_id, id, expected, owner, fence) = args();
        assert_eq!(
            ChainMutation::extend(
                chain_id,
                id,
                expected,
                owner,
                fence,
                vec![batch_at(56, 101, 11, 99, 0)]
            ),
            Err(MutationBuildError::AttachmentParent)
        );
        let (chain_id, id, expected, owner, fence) = args();
        assert_eq!(
            ChainMutation::extend(
                chain_id,
                id,
                expected,
                owner,
                fence,
                vec![batch_at(97, 101, 11, 10, 0)]
            ),
            Err(MutationBuildError::ChainMismatch)
        );

        let other_chain_head = ExpectedChainState::new(
            ChainRevision::new(1).unwrap(),
            Some(BlockRef::from_batch(&batch_at(97, 100, 10, 9, 0))),
            None,
        );
        assert_eq!(
            ChainMutation::extend(
                chain(),
                mutation_id(21),
                other_chain_head,
                LeaseOwner::new("worker-a").unwrap(),
                LeaseFence::new(1).unwrap(),
                vec![batch_at(56, 101, 11, 10, 0)]
            ),
            Err(MutationBuildError::ChainMismatch)
        );
    }

    #[tokio::test]
    async fn multi_block_reorg_requires_exact_suffix_and_preserves_all_facts() {
        let repository = MemoryBlockRepository::default();
        let grant = lease(&repository, "worker-a").await;
        let block_100 = batch_at(56, 100, 10, 9, 0);
        initialize(&repository, &grant, 30, vec![block_100.clone()]).await;
        let old = vec![
            batch_at(56, 101, 11, 10, 0),
            batch_at(56, 102, 12, 11, 0),
            batch_at(56, 103, 13, 12, 0),
        ];
        extend(&repository, &grant, 31, old.clone()).await;
        let replacement = vec![batch_at(56, 101, 21, 10, 10), batch_at(56, 102, 22, 21, 10)];
        let snapshot = repository.snapshot(chain()).await.unwrap();
        let invalid = ChainMutation::reorg(
            chain(),
            mutation_id(32),
            snapshot.expected(),
            grant.owner().clone(),
            grant.fence(),
            BlockRef::from_batch(&block_100),
            vec![BlockRef::from_batch(&old[1]), BlockRef::from_batch(&old[2])],
            replacement.clone(),
        );
        assert_eq!(invalid, Err(MutationBuildError::InvalidDetach));

        let mutation = ChainMutation::reorg(
            chain(),
            mutation_id(33),
            snapshot.expected(),
            grant.owner().clone(),
            grant.fence(),
            BlockRef::from_batch(&block_100),
            old.iter().map(BlockRef::from_batch).collect(),
            replacement.clone(),
        )
        .unwrap();
        repository.apply(mutation).await.unwrap();
        assert_eq!(
            repository.snapshot(chain()).await.unwrap().selected_head(),
            Some(BlockRef::from_batch(&replacement[1]))
        );
        for block in old.iter().chain(replacement.iter()) {
            assert!(repository
                .load_candidate(BlockRef::from_batch(block).identity())
                .await
                .unwrap()
                .is_some());
        }
    }

    #[tokio::test]
    async fn noop_reorg_is_typed_and_repository_rejection_is_atomic() {
        let repository = MemoryBlockRepository::default();
        let grant = lease(&repository, "worker-a").await;
        let blocks = [
            batch_at(56, 100, 10, 9, 0),
            batch_at(56, 101, 11, 10, 0),
            batch_at(56, 102, 12, 11, 0),
        ];
        initialize(&repository, &grant, 34, vec![blocks[0].clone()]).await;
        extend(&repository, &grant, 35, blocks[1..].to_vec()).await;
        let snapshot = repository.snapshot(chain()).await.unwrap();
        let exact_detach: Vec<_> = blocks[1..].iter().map(BlockRef::from_batch).collect();

        assert_eq!(
            ChainMutation::reorg(
                chain(),
                mutation_id(36),
                snapshot.expected(),
                grant.owner().clone(),
                grant.fence(),
                BlockRef::from_batch(&blocks[0]),
                exact_detach,
                blocks[1..].to_vec(),
            ),
            Err(MutationBuildError::ReorgNoop)
        );

        let repository_defense = ChainMutation::reorg(
            chain(),
            mutation_id(37),
            snapshot.expected(),
            grant.owner().clone(),
            grant.fence(),
            BlockRef::from_batch(&blocks[0]),
            vec![
                BlockRef::new(
                    chain(),
                    BlockNumber::new(101).unwrap(),
                    BlockHash::new(hash(99)).unwrap(),
                ),
                BlockRef::from_batch(&blocks[2]),
            ],
            blocks[1..].to_vec(),
        )
        .unwrap();
        let before = repository.state_copy();
        assert_eq!(
            repository.apply(repository_defense).await,
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::ReorgNoop
            ))
        );
        assert_eq!(repository.state_copy(), before);
        assert_eq!(
            repository.snapshot(chain()).await.unwrap().revision(),
            snapshot.revision()
        );
    }

    #[tokio::test]
    async fn transition_errors_leave_facts_selection_revision_and_journal_unchanged() {
        let repository = MemoryBlockRepository::default();
        let grant = lease(&repository, "worker-a").await;
        let block_100 = batch_at(56, 100, 10, 9, 0);
        let block_101 = batch_at(56, 101, 11, 10, 0);
        initialize(&repository, &grant, 35, vec![block_100.clone()]).await;
        extend(&repository, &grant, 36, vec![block_101.clone()]).await;
        let snapshot = repository.snapshot(chain()).await.unwrap();

        let wrong_detach = ChainMutation::reorg(
            chain(),
            mutation_id(37),
            snapshot.expected(),
            grant.owner().clone(),
            grant.fence(),
            BlockRef::new(
                chain(),
                BlockNumber::new(100).unwrap(),
                BlockHash::new(hash(99)).unwrap(),
            ),
            vec![BlockRef::from_batch(&block_101)],
            vec![batch_at(56, 101, 21, 99, 10)],
        )
        .unwrap();
        let before = repository.state_copy();
        assert_eq!(
            repository.apply(wrong_detach).await,
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::CommonAncestorNotSelected
            ))
        );
        assert_eq!(repository.state_copy(), before);

        let conflicting_fact = batch_at(56, 101, 11, 10, 10);
        let conflict = ChainMutation::reorg(
            chain(),
            mutation_id(38),
            snapshot.expected(),
            grant.owner().clone(),
            grant.fence(),
            BlockRef::from_batch(&block_100),
            vec![BlockRef::from_batch(&block_101)],
            vec![conflicting_fact, batch_at(56, 102, 12, 11, 10)],
        )
        .unwrap();
        assert!(matches!(
            repository.apply(conflict).await,
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::CandidateContent { .. }
            ))
        ));
        assert_eq!(repository.state_copy(), before);
    }

    #[tokio::test]
    async fn finalized_selection_advances_only_on_selection_and_blocks_deep_reorg() {
        let repository = MemoryBlockRepository::default();
        let grant = lease(&repository, "worker-a").await;
        let blocks = [
            batch_at(56, 100, 10, 9, 0),
            batch_at(56, 101, 11, 10, 0),
            batch_at(56, 102, 12, 11, 0),
        ];
        initialize(&repository, &grant, 40, vec![blocks[0].clone()]).await;
        extend(&repository, &grant, 41, blocks[1..].to_vec()).await;
        let snapshot = repository.snapshot(chain()).await.unwrap();
        let finalize = ChainMutation::advance_finalized(
            chain(),
            mutation_id(42),
            snapshot.expected(),
            grant.owner().clone(),
            grant.fence(),
            BlockRef::from_batch(&blocks[1]),
        )
        .unwrap();
        repository.apply(finalize).await.unwrap();
        let finalized = repository.snapshot(chain()).await.unwrap();
        assert_eq!(
            finalized.finalized_selection(),
            Some(BlockRef::from_batch(&blocks[1]))
        );
        assert_eq!(
            ChainMutation::advance_finalized(
                chain(),
                mutation_id(43),
                finalized.expected(),
                grant.owner().clone(),
                grant.fence(),
                BlockRef::from_batch(&blocks[0]),
            ),
            Err(MutationBuildError::FinalityDoesNotAdvance)
        );
        assert_eq!(
            ChainMutation::reorg(
                chain(),
                mutation_id(44),
                finalized.expected(),
                grant.owner().clone(),
                grant.fence(),
                BlockRef::from_batch(&blocks[0]),
                vec![
                    BlockRef::from_batch(&blocks[1]),
                    BlockRef::from_batch(&blocks[2])
                ],
                vec![batch_at(56, 101, 21, 10, 10)],
            ),
            Err(MutationBuildError::FinalizedBoundary)
        );

        let unselected = BlockRef::from_batch(&batch_at(56, 102, 22, 11, 10));
        let mutation = ChainMutation::advance_finalized(
            chain(),
            mutation_id(45),
            finalized.expected(),
            grant.owner().clone(),
            grant.fence(),
            unselected,
        )
        .unwrap();
        let before = repository.state_copy();
        assert_eq!(
            repository.apply(mutation).await,
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::FinalityTargetNotSelected
            ))
        );
        assert_eq!(repository.state_copy(), before);
    }

    #[tokio::test]
    async fn exact_replay_precedes_fence_checks_and_altered_reuse_conflicts() {
        let repository = MemoryBlockRepository::default();
        let first_grant = lease(&repository, "worker-a").await;
        let block_100 = batch_at(56, 100, 10, 9, 0);
        let (initialize_mutation, initialize_outcome) =
            initialize(&repository, &first_grant, 50, vec![block_100]).await;
        extend(
            &repository,
            &first_grant,
            51,
            vec![batch_at(56, 101, 11, 10, 0)],
        )
        .await;
        repository.advance_clock(std::time::Duration::from_secs(61));
        let _takeover = lease(&repository, "worker-b").await;
        assert_eq!(
            repository.apply(initialize_mutation.clone()).await.unwrap(),
            initialize_outcome
        );

        let altered = ChainMutation::initialize(
            chain(),
            initialize_mutation.mutation_id(),
            ExpectedChainState::empty(),
            first_grant.owner().clone(),
            first_grant.fence(),
            vec![batch_at(56, 100, 20, 9, 10)],
        )
        .unwrap();
        assert!(matches!(
            repository.apply(altered).await,
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::MutationIdReuse { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn concurrent_mutations_from_one_revision_have_one_winner() {
        let repository = Arc::new(MemoryBlockRepository::default());
        let grant = lease(&repository, "worker-a").await;
        initialize(&repository, &grant, 60, vec![batch_at(56, 100, 10, 9, 0)]).await;
        let expected = repository.snapshot(chain()).await.unwrap().expected();
        let mutations = [
            ChainMutation::extend(
                chain(),
                mutation_id(61),
                expected.clone(),
                grant.owner().clone(),
                grant.fence(),
                vec![batch_at(56, 101, 11, 10, 0)],
            )
            .unwrap(),
            ChainMutation::extend(
                chain(),
                mutation_id(62),
                expected,
                grant.owner().clone(),
                grant.fence(),
                vec![batch_at(56, 101, 12, 10, 10)],
            )
            .unwrap(),
        ];
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for mutation in mutations {
            let repository = repository.clone();
            let barrier = barrier.clone();
            handles.push(tokio::spawn(async move {
                barrier.wait().await;
                repository.apply(mutation).await
            }));
        }
        barrier.wait().await;
        let mut successes = 0;
        let mut conflicts = 0;
        for handle in handles {
            match handle.await.unwrap() {
                Ok(_) => successes += 1,
                Err(SelectedChainRepositoryError::Conflict(SelectionConflict::ExpectedState {
                    ..
                })) => conflicts += 1,
                other => panic!("unexpected result: {other:?}"),
            }
        }
        assert_eq!((successes, conflicts), (1, 1));
        assert_eq!(
            repository
                .candidates_at_height(chain(), BlockNumber::new(101).unwrap())
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn chain_state_and_candidate_identity_are_chain_scoped() {
        let repository = MemoryBlockRepository::default();
        let chain_56 = chain();
        let chain_97 = ChainId::new(97).unwrap();
        let grant_56 = lease(&repository, "worker-a").await;
        let grant_97 = repository
            .acquire_lease(chain_97, owner("worker-a"), lease_duration())
            .await
            .unwrap();
        let block_56 = batch_at(56, 100, 10, 9, 0);
        initialize(&repository, &grant_56, 70, vec![block_56.clone()]).await;
        let block_97 = batch_at(97, 100, 10, 9, 0);
        let mutation = ChainMutation::initialize(
            chain_97,
            mutation_id(70),
            ExpectedChainState::empty(),
            grant_97.owner().clone(),
            grant_97.fence(),
            vec![block_97.clone()],
        )
        .unwrap();
        repository.apply(mutation).await.unwrap();
        assert_eq!(
            repository
                .snapshot(chain_56)
                .await
                .unwrap()
                .revision()
                .get(),
            1
        );
        assert_eq!(
            repository
                .snapshot(chain_97)
                .await
                .unwrap()
                .revision()
                .get(),
            1
        );
        assert_eq!(
            repository
                .load_candidate(BlockRef::from_batch(&block_56).identity())
                .await
                .unwrap(),
            Some(block_56)
        );
        assert_eq!(
            repository
                .load_candidate(BlockRef::from_batch(&block_97).identity())
                .await
                .unwrap(),
            Some(block_97)
        );
    }
}
