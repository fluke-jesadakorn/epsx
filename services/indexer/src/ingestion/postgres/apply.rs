use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, Decode, PgPool, Postgres, Row, Transaction, Type};

use super::super::selection::SelectionChange;
use super::super::{
    ApplyOutcome, BlockHash, BlockNumber, BlockRef, ChainId, ChainMutation, ChainRevision,
    ExpectedChainState, LeaseFence, LeaseOwner, SelectedChainRepositoryError, SelectionConflict,
    ValidatedBlockBatch,
};
use super::candidates;
use super::codec::{decode_b256, decode_nonnegative_i64, encode_b256};
use super::journal;

const LOCK_CHAIN_STATE_SQL: &str = r#"
    SELECT
        revision,
        selected_head_number,
        selected_head_hash,
        finalized_selection_number,
        finalized_selection_hash,
        lease_owner,
        lease_fence,
        lease_expires_at,
        clock_timestamp() AS database_now
    FROM public.indexer_chain_state
    WHERE chain_id = $1
    FOR UPDATE
"#;

const LOAD_HIGHEST_SELECTED_SQL: &str = r#"
    SELECT number, block_hash, selected_revision
    FROM public.indexer_selected_blocks
    WHERE chain_id = $1
    ORDER BY number DESC
    LIMIT 1
"#;

const LOAD_EXACT_SELECTED_SQL: &str = r#"
    SELECT number, block_hash, selected_revision
    FROM public.indexer_selected_blocks
    WHERE chain_id = $1 AND number = $2
"#;

const LOAD_SELECTED_SUFFIX_SQL: &str = r#"
    SELECT number, block_hash, selected_revision
    FROM public.indexer_selected_blocks
    WHERE chain_id = $1 AND number > $2
    ORDER BY number ASC
"#;

const VERIFY_SELECTED_REVISIONS_SQL: &str = r#"
    SELECT EXISTS (
        SELECT 1
        FROM public.indexer_selected_blocks
        WHERE chain_id = $1 AND selected_revision > $2
    ) AS has_future_revision
"#;

const DELETE_SELECTED_SUFFIX_SQL: &str = r#"
    DELETE FROM public.indexer_selected_blocks
    WHERE chain_id = $1 AND number > $2
"#;

const INSERT_SELECTED_SQL: &str = r#"
    INSERT INTO public.indexer_selected_blocks (
        chain_id,
        number,
        block_hash,
        selected_revision
    )
    VALUES ($1, $2, $3, $4)
    ON CONFLICT (chain_id, number) DO NOTHING
"#;

const UPDATE_CHAIN_STATE_SQL: &str = r#"
    UPDATE public.indexer_chain_state
    SET revision = $2,
        selected_head_number = $3,
        selected_head_hash = $4,
        finalized_selection_number = $5,
        finalized_selection_hash = $6,
        updated_at = clock_timestamp()
    WHERE chain_id = $1
      AND revision = $7
      AND selected_head_number IS NOT DISTINCT FROM $8
      AND selected_head_hash IS NOT DISTINCT FROM $9
      AND finalized_selection_number IS NOT DISTINCT FROM $10
      AND finalized_selection_hash IS NOT DISTINCT FROM $11
      AND lease_owner = $12
      AND lease_fence = $13
      AND lease_expires_at > clock_timestamp()
    RETURNING revision
"#;

#[derive(Debug)]
struct LockedStateFields {
    revision: i64,
    selected_head_number: Option<i64>,
    selected_head_hash: Option<Vec<u8>>,
    finalized_selection_number: Option<i64>,
    finalized_selection_hash: Option<Vec<u8>>,
    lease_owner: Option<String>,
    lease_fence: i64,
    lease_expires_at: Option<DateTime<Utc>>,
    database_now: DateTime<Utc>,
}

#[derive(Debug)]
struct StoredState {
    expected: ExpectedChainState,
    lease_owner: Option<LeaseOwner>,
    lease_fence: Option<LeaseFence>,
    lease_expires_at: Option<DateTime<Utc>>,
    database_now: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectionWrite<'a> {
    Initialize {
        attach: &'a [ValidatedBlockBatch],
    },
    Extend {
        attach: &'a [ValidatedBlockBatch],
    },
    Reorg {
        common_ancestor: BlockRef,
        detach: &'a [BlockRef],
        attach: &'a [ValidatedBlockBatch],
    },
    AdvanceFinalized {
        target: BlockRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ApplyPlan<'a> {
    outcome: ApplyOutcome,
    write: SelectionWrite<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectedMapping {
    reference: BlockRef,
    selected_revision: ChainRevision,
}

/// Applies one selected-chain mutation in one PostgreSQL transaction.
///
/// Dropping the transaction on any error or cancellation rolls back all work.
/// A commit transport error is intentionally returned as unavailable: retrying
/// the same mutation ID reconciles an ambiguous commit through the journal
/// lookup, which occurs before lease and expected-state validation.
pub(super) async fn apply(
    pool: &PgPool,
    mutation: &ChainMutation,
) -> Result<ApplyOutcome, SelectedChainRepositoryError> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| unavailable("begin selected-chain mutation", error))?;

    let locked_fields = lock_chain_state(&mut transaction, mutation.chain_id()).await?;

    // Replay has precedence over lease expiry and expected-state drift. This
    // is the retry path for a commit whose acknowledgement was lost.
    if let Some(outcome) = journal::replay_if_present(&mut transaction, mutation).await? {
        transaction
            .commit()
            .await
            .map_err(|error| unavailable("commit mutation replay read", error))?;
        return Ok(outcome);
    }

    let fields = locked_fields.ok_or(SelectionConflict::StaleLease)?;
    let stored = decode_locked_state(mutation.chain_id(), fields)?;
    require_live_lease(&stored, mutation)?;

    if mutation.expected() != &stored.expected {
        return Err(SelectionConflict::ExpectedState {
            expected: mutation.expected().clone(),
            actual: stored.expected,
        }
        .into());
    }

    verify_selected_state(&mut transaction, mutation.chain_id(), mutation.expected()).await?;

    let plan = plan_mutation(mutation)?;
    for candidate in attached_candidates(&plan.write) {
        candidates::persist_or_compare_candidate(&mut transaction, candidate).await?;
    }

    validate_transition(&mut transaction, mutation, &plan).await?;
    write_selection(&mut transaction, mutation.chain_id(), &plan).await?;
    compare_and_swap_state(&mut transaction, mutation, &plan.outcome).await?;
    journal::append_applied_mutation(&mut transaction, mutation, &plan.outcome).await?;

    transaction
        .commit()
        .await
        .map_err(|error| unavailable("commit selected-chain mutation", error))?;
    Ok(plan.outcome)
}

async fn lock_chain_state(
    transaction: &mut Transaction<'_, Postgres>,
    chain_id: ChainId,
) -> Result<Option<LockedStateFields>, SelectedChainRepositoryError> {
    let row = sqlx::query(LOCK_CHAIN_STATE_SQL)
        .bind(chain_id_storage(chain_id))
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| unavailable("lock selected-chain state", error))?;
    row.map(locked_state_fields).transpose()
}

fn locked_state_fields(row: PgRow) -> Result<LockedStateFields, SelectedChainRepositoryError> {
    Ok(LockedStateFields {
        revision: column(&row, "revision")?,
        selected_head_number: column(&row, "selected_head_number")?,
        selected_head_hash: column(&row, "selected_head_hash")?,
        finalized_selection_number: column(&row, "finalized_selection_number")?,
        finalized_selection_hash: column(&row, "finalized_selection_hash")?,
        lease_owner: column(&row, "lease_owner")?,
        lease_fence: column(&row, "lease_fence")?,
        lease_expires_at: column(&row, "lease_expires_at")?,
        database_now: column(&row, "database_now")?,
    })
}

fn decode_locked_state(
    chain_id: ChainId,
    fields: LockedStateFields,
) -> Result<StoredState, SelectedChainRepositoryError> {
    let revision_value = decode_nonnegative_i64("indexer_chain_state.revision", fields.revision)?;
    let revision = ChainRevision::new(revision_value)
        .map_err(|error| corrupt(format!("stored chain revision is invalid: {error}")))?;
    let selected_head = decode_optional_ref(
        chain_id,
        "indexer_chain_state.selected_head",
        fields.selected_head_number,
        fields.selected_head_hash,
    )?;
    let finalized_selection = decode_optional_ref(
        chain_id,
        "indexer_chain_state.finalized_selection",
        fields.finalized_selection_number,
        fields.finalized_selection_hash,
    )?;

    if (revision == ChainRevision::ZERO) != selected_head.is_none() {
        return Err(corrupt(
            "stored revision zero and selected-head absence must be equivalent",
        ));
    }
    if finalized_selection.is_some() && selected_head.is_none() {
        return Err(corrupt("stored finalized selection has no selected head"));
    }
    if finalized_selection
        .zip(selected_head)
        .is_some_and(|(finalized, head)| finalized.number() > head.number())
    {
        return Err(corrupt(
            "stored finalized selection is above the selected head",
        ));
    }

    let fence_value =
        decode_nonnegative_i64("indexer_chain_state.lease_fence", fields.lease_fence)?;
    let lease_fence = if fence_value == 0 {
        None
    } else {
        Some(
            LeaseFence::new(fence_value)
                .map_err(|error| corrupt(format!("stored lease fence is invalid: {error}")))?,
        )
    };
    let (lease_owner, lease_expires_at) = match (fields.lease_owner, fields.lease_expires_at) {
        (None, None) => (None, None),
        (Some(owner), Some(expires_at)) => {
            if lease_fence.is_none() {
                return Err(corrupt("stored live lease has a zero fence"));
            }
            let owner = LeaseOwner::new(owner)
                .map_err(|error| corrupt(format!("stored lease owner is invalid: {error}")))?;
            (Some(owner), Some(expires_at))
        }
        _ => {
            return Err(corrupt("stored lease owner and expiration are not paired"));
        }
    };

    Ok(StoredState {
        expected: ExpectedChainState::new(revision, selected_head, finalized_selection),
        lease_owner,
        lease_fence,
        lease_expires_at,
        database_now: fields.database_now,
    })
}

fn require_live_lease(
    stored: &StoredState,
    mutation: &ChainMutation,
) -> Result<(), SelectedChainRepositoryError> {
    if stored.lease_owner.as_ref() != Some(mutation.owner())
        || stored.lease_fence != Some(mutation.fence())
        || !stored
            .lease_expires_at
            .is_some_and(|expires_at| expires_at > stored.database_now)
    {
        return Err(SelectionConflict::StaleLease.into());
    }
    Ok(())
}

fn plan_mutation(mutation: &ChainMutation) -> Result<ApplyPlan<'_>, SelectedChainRepositoryError> {
    let revision = mutation.expected().revision().next()?;
    let (selected_head, finalized_selection, write) = match mutation.change() {
        SelectionChange::Initialize { attach } => (
            attach.last().map(BlockRef::from_batch),
            None,
            SelectionWrite::Initialize { attach },
        ),
        SelectionChange::Extend { attach } => (
            attach.last().map(BlockRef::from_batch),
            mutation.expected().finalized_selection(),
            SelectionWrite::Extend { attach },
        ),
        SelectionChange::Reorg {
            common_ancestor,
            detach,
            attach,
        } => (
            attach.last().map(BlockRef::from_batch),
            mutation.expected().finalized_selection(),
            SelectionWrite::Reorg {
                common_ancestor: *common_ancestor,
                detach,
                attach,
            },
        ),
        SelectionChange::AdvanceFinalized { target } => (
            mutation.expected().selected_head(),
            Some(*target),
            SelectionWrite::AdvanceFinalized { target: *target },
        ),
    };
    let outcome = ApplyOutcome::new(
        mutation.mutation_id(),
        revision,
        selected_head,
        finalized_selection,
        mutation.kind(),
    );
    Ok(ApplyPlan { outcome, write })
}

fn attached_candidates<'a>(write: &'a SelectionWrite<'a>) -> &'a [ValidatedBlockBatch] {
    match write {
        SelectionWrite::Initialize { attach }
        | SelectionWrite::Extend { attach }
        | SelectionWrite::Reorg { attach, .. } => attach,
        SelectionWrite::AdvanceFinalized { .. } => &[],
    }
}

async fn verify_selected_state(
    transaction: &mut Transaction<'_, Postgres>,
    chain_id: ChainId,
    expected: &ExpectedChainState,
) -> Result<(), SelectedChainRepositoryError> {
    let highest = sqlx::query(LOAD_HIGHEST_SELECTED_SQL)
        .bind(chain_id_storage(chain_id))
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| unavailable("load highest selected block", error))?
        .map(|row| decode_selected_mapping(chain_id, &row, expected.revision()))
        .transpose()?;
    if highest.map(|mapping| mapping.reference) != expected.selected_head() {
        return Err(corrupt(
            "stored selected head is not the highest exact selected mapping",
        ));
    }

    if let Some(finalized) = expected.finalized_selection() {
        if load_exact_selected(transaction, finalized, expected.revision()).await?
            != Some(finalized)
        {
            return Err(corrupt(
                "stored finalized selection does not match its exact selected mapping",
            ));
        }
    }

    let row = sqlx::query(VERIFY_SELECTED_REVISIONS_SQL)
        .bind(chain_id_storage(chain_id))
        .bind(revision_storage(expected.revision()))
        .fetch_one(&mut **transaction)
        .await
        .map_err(|error| unavailable("verify selected mapping revisions", error))?;
    let has_future_revision: bool = column(&row, "has_future_revision")?;
    if has_future_revision {
        return Err(corrupt(
            "a selected mapping was written after the stored chain revision",
        ));
    }
    Ok(())
}

async fn validate_transition(
    transaction: &mut Transaction<'_, Postgres>,
    mutation: &ChainMutation,
    plan: &ApplyPlan<'_>,
) -> Result<(), SelectedChainRepositoryError> {
    match &plan.write {
        SelectionWrite::Initialize { .. } => {
            if mutation.expected().selected_head().is_some() {
                return Err(SelectionConflict::AlreadyInitialized.into());
            }
        }
        SelectionWrite::Extend { .. } => {
            let Some(head) = mutation.expected().selected_head() else {
                return Err(SelectionConflict::NotInitialized.into());
            };
            if load_exact_selected(transaction, head, mutation.expected().revision()).await?
                != Some(head)
            {
                return Err(SelectionConflict::NotInitialized.into());
            }
        }
        SelectionWrite::Reorg {
            common_ancestor,
            detach,
            attach,
        } => {
            if load_exact_selected(
                transaction,
                *common_ancestor,
                mutation.expected().revision(),
            )
            .await?
                != Some(*common_ancestor)
            {
                return Err(SelectionConflict::CommonAncestorNotSelected.into());
            }
            let actual = load_selected_suffix(
                transaction,
                mutation.chain_id(),
                common_ancestor.number(),
                mutation.expected().revision(),
            )
            .await?;
            let attached = attach.iter().map(BlockRef::from_batch).collect::<Vec<_>>();
            if attached == actual {
                return Err(SelectionConflict::ReorgNoop.into());
            }
            if *detach != actual.as_slice() {
                return Err(SelectionConflict::DetachMismatch.into());
            }
            if mutation
                .expected()
                .finalized_selection()
                .is_some_and(|finalized| common_ancestor.number() < finalized.number())
            {
                return Err(SelectionConflict::FinalizedBoundary.into());
            }
        }
        SelectionWrite::AdvanceFinalized { target } => {
            if load_exact_selected(transaction, *target, mutation.expected().revision()).await?
                != Some(*target)
            {
                return Err(SelectionConflict::FinalityTargetNotSelected.into());
            }
        }
    }
    Ok(())
}

async fn write_selection(
    transaction: &mut Transaction<'_, Postgres>,
    chain_id: ChainId,
    plan: &ApplyPlan<'_>,
) -> Result<(), SelectedChainRepositoryError> {
    if let SelectionWrite::Reorg {
        common_ancestor,
        detach,
        ..
    } = &plan.write
    {
        let deleted = sqlx::query(DELETE_SELECTED_SUFFIX_SQL)
            .bind(chain_id_storage(chain_id))
            .bind(common_ancestor.number().get())
            .execute(&mut **transaction)
            .await
            .map_err(|error| unavailable("delete selected reorg suffix", error))?
            .rows_affected();
        if deleted != usize_to_u64(detach.len(), "reorg detach count")? {
            return Err(corrupt(
                "selected reorg suffix changed after exact validation",
            ));
        }
    }

    for candidate in attached_candidates(&plan.write) {
        let reference = BlockRef::from_batch(candidate);
        let inserted = sqlx::query(INSERT_SELECTED_SQL)
            .bind(chain_id_storage(chain_id))
            .bind(reference.number().get())
            .bind(encode_b256(reference.hash().get()))
            .bind(revision_storage(plan.outcome.revision()))
            .execute(&mut **transaction)
            .await
            .map_err(|error| unavailable("insert selected block mapping", error))?
            .rows_affected();
        if inserted != 1 {
            return Err(corrupt(
                "selected attachment collided with an existing height",
            ));
        }
    }
    Ok(())
}

async fn compare_and_swap_state(
    transaction: &mut Transaction<'_, Postgres>,
    mutation: &ChainMutation,
    outcome: &ApplyOutcome,
) -> Result<(), SelectedChainRepositoryError> {
    let result_head = outcome.selected_head();
    let result_finalized = outcome.finalized_selection();
    let expected_head = mutation.expected().selected_head();
    let expected_finalized = mutation.expected().finalized_selection();
    let revision = sqlx::query_scalar::<_, i64>(UPDATE_CHAIN_STATE_SQL)
        .bind(chain_id_storage(mutation.chain_id()))
        .bind(revision_storage(outcome.revision()))
        .bind(reference_number(result_head))
        .bind(reference_hash(result_head))
        .bind(reference_number(result_finalized))
        .bind(reference_hash(result_finalized))
        .bind(revision_storage(mutation.expected().revision()))
        .bind(reference_number(expected_head))
        .bind(reference_hash(expected_head))
        .bind(reference_number(expected_finalized))
        .bind(reference_hash(expected_finalized))
        .bind(mutation.owner().as_str())
        .bind(fence_storage(mutation.fence()))
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| unavailable("compare and swap selected-chain state", error))?;
    if revision != Some(revision_storage(outcome.revision())) {
        return Err(SelectionConflict::StaleLease.into());
    }
    Ok(())
}

async fn load_exact_selected(
    transaction: &mut Transaction<'_, Postgres>,
    reference: BlockRef,
    chain_revision: ChainRevision,
) -> Result<Option<BlockRef>, SelectedChainRepositoryError> {
    let row = sqlx::query(LOAD_EXACT_SELECTED_SQL)
        .bind(chain_id_storage(reference.chain_id()))
        .bind(reference.number().get())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| unavailable("load exact selected mapping", error))?;
    row.map(|row| decode_selected_mapping(reference.chain_id(), &row, chain_revision))
        .transpose()
        .map(|mapping| mapping.map(|mapping| mapping.reference))
}

async fn load_selected_suffix(
    transaction: &mut Transaction<'_, Postgres>,
    chain_id: ChainId,
    after: BlockNumber,
    chain_revision: ChainRevision,
) -> Result<Vec<BlockRef>, SelectedChainRepositoryError> {
    let rows = sqlx::query(LOAD_SELECTED_SUFFIX_SQL)
        .bind(chain_id_storage(chain_id))
        .bind(after.get())
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| unavailable("load selected reorg suffix", error))?;
    rows.iter()
        .map(|row| {
            decode_selected_mapping(chain_id, row, chain_revision).map(|mapping| mapping.reference)
        })
        .collect()
}

fn decode_selected_mapping(
    chain_id: ChainId,
    row: &PgRow,
    chain_revision: ChainRevision,
) -> Result<SelectedMapping, SelectedChainRepositoryError> {
    let number_value =
        decode_nonnegative_i64("indexer_selected_blocks.number", column(row, "number")?)?;
    let number = BlockNumber::new(number_value)
        .map_err(|error| corrupt(format!("stored selected number is invalid: {error}")))?;
    let hash = BlockHash::new(decode_b256(
        "indexer_selected_blocks.block_hash",
        column(row, "block_hash")?,
    )?)
    .map_err(|error| corrupt(format!("stored selected hash is invalid: {error}")))?;
    let selected_revision_value = decode_nonnegative_i64(
        "indexer_selected_blocks.selected_revision",
        column(row, "selected_revision")?,
    )?;
    if selected_revision_value == 0 {
        return Err(corrupt("stored selected revision must be non-zero"));
    }
    let selected_revision = ChainRevision::new(selected_revision_value)
        .map_err(|error| corrupt(format!("stored selected revision is invalid: {error}")))?;
    if selected_revision > chain_revision {
        return Err(corrupt(
            "stored selected revision exceeds the chain revision",
        ));
    }
    Ok(SelectedMapping {
        reference: BlockRef::new(chain_id, number, hash),
        selected_revision,
    })
}

fn decode_optional_ref(
    chain_id: ChainId,
    field: &str,
    number: Option<i64>,
    hash: Option<Vec<u8>>,
) -> Result<Option<BlockRef>, SelectedChainRepositoryError> {
    match (number, hash) {
        (None, None) => Ok(None),
        (Some(number), Some(hash)) => {
            let number = decode_nonnegative_i64(&format!("{field}_number"), number)?;
            let number = BlockNumber::new(number)
                .map_err(|error| corrupt(format!("{field} number is invalid: {error}")))?;
            let hash = BlockHash::new(decode_b256(&format!("{field}_hash"), hash)?)
                .map_err(|error| corrupt(format!("{field} hash is invalid: {error}")))?;
            Ok(Some(BlockRef::new(chain_id, number, hash)))
        }
        _ => Err(corrupt(format!(
            "{field} number and hash must both be null or both be present"
        ))),
    }
}

fn reference_number(reference: Option<BlockRef>) -> Option<i64> {
    reference.map(|reference| reference.number().get())
}

fn reference_hash(reference: Option<BlockRef>) -> Option<Vec<u8>> {
    reference.map(|reference| encode_b256(reference.hash().get()))
}

fn chain_id_storage(chain_id: ChainId) -> i64 {
    i64::try_from(chain_id.get()).expect("validated chain IDs fit signed storage")
}

fn revision_storage(revision: ChainRevision) -> i64 {
    i64::try_from(revision.get()).expect("validated chain revisions fit signed storage")
}

fn fence_storage(fence: LeaseFence) -> i64 {
    i64::try_from(fence.get()).expect("validated lease fences fit signed storage")
}

fn usize_to_u64(value: usize, field: &'static str) -> Result<u64, SelectedChainRepositoryError> {
    u64::try_from(value).map_err(|_| corrupt(format!("{field} exceeds storage counting")))
}

fn column<T>(row: &PgRow, name: &'static str) -> Result<T, SelectedChainRepositoryError>
where
    T: for<'row> Decode<'row, Postgres> + Type<Postgres>,
{
    row.try_get(name)
        .map_err(|error| corrupt(format!("could not decode column {name}: {error}")))
}

fn unavailable(context: &str, error: sqlx::Error) -> SelectedChainRepositoryError {
    match error {
        error @ (sqlx::Error::ColumnDecode { .. } | sqlx::Error::Decode(_)) => {
            corrupt(format!("{context}: {error}"))
        }
        error => SelectedChainRepositoryError::Unavailable(format!("{context}: {error}")),
    }
}

fn corrupt(message: impl Into<String>) -> SelectedChainRepositoryError {
    SelectedChainRepositoryError::CorruptState(message.into())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use alloy::primitives::B256;

    use super::*;
    use crate::ingestion::{
        validate_block, BlockRequest, FetchedBlock, LeaseDuration, MutationId, MutationKind,
        ValidationLimits,
    };

    fn hash(marker: u8) -> B256 {
        B256::repeat_byte(marker)
    }

    fn chain() -> ChainId {
        ChainId::new(56).expect("chain")
    }

    fn reference(number: u64, marker: u8) -> BlockRef {
        BlockRef::new(
            chain(),
            BlockNumber::new(number).expect("number"),
            BlockHash::new(hash(marker)).expect("hash"),
        )
    }

    fn batch(number: u64, marker: u8, parent: u8) -> ValidatedBlockBatch {
        validate_block(
            BlockRequest::new(chain(), BlockNumber::new(number).expect("number")),
            FetchedBlock {
                chain_id: chain().get(),
                number,
                hash: hash(marker),
                parent_hash: hash(parent),
                timestamp: 1_700_000_000 + number,
                beneficiary: None,
                gas_used: 0,
                gas_limit: 0,
                transactions: vec![],
                receipts: vec![],
            },
            ValidationLimits::default(),
        )
        .expect("batch")
    }

    fn owner() -> LeaseOwner {
        LeaseOwner::new("apply-test").expect("owner")
    }

    fn fence() -> LeaseFence {
        LeaseFence::new(7).expect("fence")
    }

    fn mutation_id(marker: u8) -> MutationId {
        MutationId::new(hash(marker)).expect("mutation id")
    }

    fn database_now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("database time")
    }

    #[test]
    fn locked_state_decoder_enforces_state_and_live_lease_shapes() {
        let valid = LockedStateFields {
            revision: 1,
            selected_head_number: Some(10),
            selected_head_hash: Some(hash(10).as_slice().to_vec()),
            finalized_selection_number: None,
            finalized_selection_hash: None,
            lease_owner: Some("worker-a".into()),
            lease_fence: 1,
            lease_expires_at: Some(database_now() + chrono::Duration::seconds(10)),
            database_now: database_now(),
        };
        let decoded = decode_locked_state(chain(), valid).expect("valid state");
        assert_eq!(decoded.expected.selected_head(), Some(reference(10, 10)));

        let invalid = LockedStateFields {
            revision: 0,
            selected_head_number: Some(10),
            selected_head_hash: Some(hash(10).as_slice().to_vec()),
            finalized_selection_number: None,
            finalized_selection_hash: None,
            lease_owner: Some("worker-a".into()),
            lease_fence: 0,
            lease_expires_at: Some(database_now() + chrono::Duration::seconds(10)),
            database_now: database_now(),
        };
        assert!(decode_locked_state(chain(), invalid).is_err());
    }

    #[test]
    fn lease_validation_uses_database_time_and_exact_owner_fence() {
        let mutation = ChainMutation::initialize(
            chain(),
            mutation_id(1),
            ExpectedChainState::empty(),
            owner(),
            fence(),
            vec![batch(10, 10, 9)],
        )
        .expect("mutation");
        let live = StoredState {
            expected: ExpectedChainState::empty(),
            lease_owner: Some(owner()),
            lease_fence: Some(fence()),
            lease_expires_at: Some(database_now() + chrono::Duration::seconds(1)),
            database_now: database_now(),
        };
        require_live_lease(&live, &mutation).expect("live");

        let expired = StoredState {
            lease_expires_at: Some(database_now()),
            ..live
        };
        assert!(matches!(
            require_live_lease(&expired, &mutation),
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::StaleLease
            ))
        ));
    }

    #[test]
    fn transition_plans_exact_reorg_and_finality_results() {
        let common = reference(10, 10);
        let old_head = reference(11, 11);
        let finalized = reference(9, 9);
        let replacement = batch(11, 21, 10);
        let reorg = ChainMutation::reorg(
            chain(),
            mutation_id(2),
            ExpectedChainState::new(
                ChainRevision::new(4).expect("revision"),
                Some(old_head),
                Some(finalized),
            ),
            owner(),
            fence(),
            common,
            vec![old_head],
            vec![replacement.clone()],
        )
        .expect("reorg");
        let plan = plan_mutation(&reorg).expect("plan");
        assert_eq!(plan.outcome.revision().get(), 5);
        assert_eq!(
            plan.outcome.selected_head(),
            Some(BlockRef::from_batch(&replacement))
        );
        assert_eq!(plan.outcome.finalized_selection(), Some(finalized));
        assert!(matches!(
            plan.write,
            SelectionWrite::Reorg {
                common_ancestor,
                detach,
                ..
            } if common_ancestor == common && detach == [old_head]
        ));

        let finality = ChainMutation::advance_finalized(
            chain(),
            mutation_id(3),
            ExpectedChainState::new(
                ChainRevision::new(5).expect("revision"),
                Some(BlockRef::from_batch(&replacement)),
                Some(finalized),
            ),
            owner(),
            fence(),
            common,
        )
        .expect("finality");
        let plan = plan_mutation(&finality).expect("plan");
        assert_eq!(plan.outcome.kind(), MutationKind::AdvanceFinalized);
        assert_eq!(plan.outcome.revision().get(), 6);
        assert_eq!(plan.outcome.finalized_selection(), Some(common));
    }

    #[test]
    fn sql_protocol_is_locked_cas_driven_and_does_not_touch_legacy_state() {
        assert!(LOCK_CHAIN_STATE_SQL.contains("clock_timestamp() AS database_now"));
        assert!(LOCK_CHAIN_STATE_SQL.contains("FOR UPDATE"));
        assert!(UPDATE_CHAIN_STATE_SQL.contains("revision = $7"));
        assert!(UPDATE_CHAIN_STATE_SQL.contains("IS NOT DISTINCT FROM $8"));
        assert!(UPDATE_CHAIN_STATE_SQL.contains("lease_owner = $12"));
        assert!(UPDATE_CHAIN_STATE_SQL.contains("lease_fence = $13"));
        assert!(UPDATE_CHAIN_STATE_SQL.contains("lease_expires_at > clock_timestamp()"));
        assert!(INSERT_SELECTED_SQL.contains("selected_revision"));

        let sql = [
            LOCK_CHAIN_STATE_SQL,
            LOAD_HIGHEST_SELECTED_SQL,
            LOAD_EXACT_SELECTED_SQL,
            LOAD_SELECTED_SUFFIX_SQL,
            VERIFY_SELECTED_REVISIONS_SQL,
            DELETE_SELECTED_SUFFIX_SQL,
            INSERT_SELECTED_SQL,
            UPDATE_CHAIN_STATE_SQL,
        ]
        .join("\n")
        .to_ascii_lowercase();
        for forbidden in [
            "pg_advisory",
            "public.blocks",
            "public.transactions",
            "public.token_transfers",
            "payment",
            "fingerprint",
        ] {
            assert!(!sql.contains(forbidden), "forbidden SQL token: {forbidden}");
        }
    }

    #[test]
    fn revision_planning_rejects_storage_exhaustion() {
        let mutation = ChainMutation::extend(
            chain(),
            mutation_id(4),
            ExpectedChainState::new(ChainRevision::MAX_STORAGE, Some(reference(10, 10)), None),
            owner(),
            fence(),
            vec![batch(11, 11, 10)],
        )
        .expect("mutation");
        assert!(matches!(
            plan_mutation(&mutation),
            Err(SelectedChainRepositoryError::InvalidMutation(_))
        ));
    }

    #[test]
    fn test_fixture_lease_duration_remains_within_domain() {
        LeaseDuration::new(Duration::from_secs(1)).expect("duration");
    }
}
