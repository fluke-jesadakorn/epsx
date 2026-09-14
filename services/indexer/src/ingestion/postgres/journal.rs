use std::collections::BTreeSet;

use sqlx::{postgres::PgRow, Decode, Postgres, Row, Transaction, Type};

use super::super::selection::SelectionChange;
use super::super::{
    ApplyOutcome, BlockHash, BlockNumber, BlockRef, ChainId, ChainMutation, ChainRevision,
    ExpectedChainState, LeaseFence, LeaseOwner, MutationId, MutationKind,
    SelectedChainRepositoryError, SelectionConflict, ValidatedBlockBatch,
};
use super::candidates;
use super::codec::{decode_b256, decode_nonnegative_i64, encode_b256};

const KIND_INITIALIZE: i16 = 0;
const KIND_EXTEND: i16 = 1;
const KIND_REORG: i16 = 2;
const KIND_ADVANCE_FINALIZED: i16 = 3;
const ROLE_DETACH: i16 = 0;
const ROLE_ATTACH: i16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
struct JournalHeader {
    chain_id: ChainId,
    mutation_id: MutationId,
    expected: ExpectedChainState,
    owner: LeaseOwner,
    fence: LeaseFence,
    common_ancestor: Option<BlockRef>,
    finality_target: Option<BlockRef>,
    outcome: ApplyOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HeaderStorage {
    chain_id: i64,
    mutation_id: Vec<u8>,
    kind: i16,
    expected_revision: i64,
    expected_selected_head_number: Option<i64>,
    expected_selected_head_hash: Option<Vec<u8>>,
    expected_finalized_selection_number: Option<i64>,
    expected_finalized_selection_hash: Option<Vec<u8>>,
    lease_owner: String,
    lease_fence: i64,
    common_ancestor_number: Option<i64>,
    common_ancestor_hash: Option<Vec<u8>>,
    finality_target_number: Option<i64>,
    finality_target_hash: Option<Vec<u8>>,
    result_revision: i64,
    result_selected_head_number: Option<i64>,
    result_selected_head_hash: Option<Vec<u8>>,
    result_finalized_selection_number: Option<i64>,
    result_finalized_selection_hash: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MemberRole {
    Detach,
    Attach,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JournalMember {
    role: MemberRole,
    ordinal: u64,
    reference: BlockRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemberStorage {
    role: i16,
    ordinal: i64,
    number: i64,
    block_hash: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MutationProjection {
    header: JournalHeader,
    members: Vec<JournalMember>,
}

/// Looks up an already-applied mutation inside the caller's transaction.
///
/// This must be called before lease validation by the later apply coordinator.
/// It never commits, starts a transaction, or mutates journal state.
#[allow(dead_code)]
pub(super) async fn replay_if_present(
    transaction: &mut Transaction<'_, Postgres>,
    mutation: &ChainMutation,
) -> Result<Option<ApplyOutcome>, SelectedChainRepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            chain_id,
            mutation_id,
            kind,
            expected_revision,
            expected_selected_head_number,
            expected_selected_head_hash,
            expected_finalized_selection_number,
            expected_finalized_selection_hash,
            lease_owner,
            lease_fence,
            common_ancestor_number,
            common_ancestor_hash,
            finality_target_number,
            finality_target_hash,
            result_revision,
            result_selected_head_number,
            result_selected_head_hash,
            result_finalized_selection_number,
            result_finalized_selection_hash
        FROM public.indexer_mutation_journal
        WHERE chain_id = $1 AND mutation_id = $2
        "#,
    )
    .bind(chain_id_storage(mutation.chain_id()))
    .bind(encode_b256(mutation.mutation_id().get()))
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| unavailable("load mutation journal header", error))?;

    let Some(row) = row else {
        return Ok(None);
    };

    let stored_header = decode_header(header_storage(&row)?)?;
    let member_rows = sqlx::query(
        r#"
        SELECT role, ordinal, number, block_hash
        FROM public.indexer_mutation_blocks
        WHERE chain_id = $1 AND mutation_id = $2
        ORDER BY role ASC, ordinal ASC
        "#,
    )
    .bind(chain_id_storage(mutation.chain_id()))
    .bind(encode_b256(mutation.mutation_id().get()))
    .fetch_all(&mut **transaction)
    .await
    .map_err(|error| unavailable("load mutation journal members", error))?;
    let stored_members = decode_members(
        stored_header.chain_id,
        member_rows
            .iter()
            .map(member_storage)
            .collect::<Result<Vec<_>, _>>()?,
    )?;
    validate_record_shape(&stored_header, &stored_members)?;

    if !stored_request_matches(mutation, &stored_header, &stored_members) {
        return Err(reused(mutation.mutation_id()));
    }
    let incoming = project_mutation(mutation)?;
    compare_record(
        mutation.mutation_id(),
        &stored_header,
        &stored_members,
        &incoming,
    )?;

    verify_candidate_facts(transaction, mutation, &stored_header, &stored_members).await?;
    Ok(Some(stored_header.outcome))
}

/// Appends the exact successful mutation record inside the caller's existing
/// transaction. The caller owns rollback/commit and must have completed the
/// selection writes before calling this function.
#[allow(dead_code)]
pub(super) async fn append_applied_mutation(
    transaction: &mut Transaction<'_, Postgres>,
    mutation: &ChainMutation,
    outcome: &ApplyOutcome,
) -> Result<(), SelectedChainRepositoryError> {
    let projection = project_mutation(mutation)?;
    if &projection.header.outcome != outcome {
        return Err(corrupt(
            "apply outcome does not exactly match the mutation-derived journal result",
        ));
    }
    validate_record_shape(&projection.header, &projection.members)?;

    let header = &projection.header;
    let expected_head = header.expected.selected_head();
    let expected_finalized = header.expected.finalized_selection();
    let result_head = header.outcome.selected_head();
    let result_finalized = header.outcome.finalized_selection();
    sqlx::query(
        r#"
        INSERT INTO public.indexer_mutation_journal (
            chain_id,
            mutation_id,
            kind,
            expected_revision,
            expected_selected_head_number,
            expected_selected_head_hash,
            expected_finalized_selection_number,
            expected_finalized_selection_hash,
            lease_owner,
            lease_fence,
            common_ancestor_number,
            common_ancestor_hash,
            finality_target_number,
            finality_target_hash,
            result_revision,
            result_selected_head_number,
            result_selected_head_hash,
            result_finalized_selection_number,
            result_finalized_selection_hash
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, $16, $17, $18, $19
        )
        "#,
    )
    .bind(chain_id_storage(header.chain_id))
    .bind(encode_b256(header.mutation_id.get()))
    .bind(encode_kind(header.outcome.kind()))
    .bind(revision_storage(header.expected.revision()))
    .bind(reference_number(expected_head))
    .bind(reference_hash(expected_head))
    .bind(reference_number(expected_finalized))
    .bind(reference_hash(expected_finalized))
    .bind(header.owner.as_str())
    .bind(fence_storage(header.fence))
    .bind(reference_number(header.common_ancestor))
    .bind(reference_hash(header.common_ancestor))
    .bind(reference_number(header.finality_target))
    .bind(reference_hash(header.finality_target))
    .bind(revision_storage(header.outcome.revision()))
    .bind(reference_number(result_head))
    .bind(reference_hash(result_head))
    .bind(reference_number(result_finalized))
    .bind(reference_hash(result_finalized))
    .execute(&mut **transaction)
    .await
    .map_err(|error| unavailable("append mutation journal header", error))?;

    for member in &projection.members {
        sqlx::query(
            r#"
            INSERT INTO public.indexer_mutation_blocks (
                chain_id,
                mutation_id,
                role,
                ordinal,
                number,
                block_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(chain_id_storage(header.chain_id))
        .bind(encode_b256(header.mutation_id.get()))
        .bind(encode_role(member.role))
        .bind(ordinal_storage(member.ordinal)?)
        .bind(member.reference.number().get())
        .bind(encode_b256(member.reference.hash().get()))
        .execute(&mut **transaction)
        .await
        .map_err(|error| unavailable("append mutation journal member", error))?;
    }

    Ok(())
}

fn project_mutation(
    mutation: &ChainMutation,
) -> Result<MutationProjection, SelectedChainRepositoryError> {
    let (common_ancestor, finality_target, detach, attach) = match mutation.change() {
        SelectionChange::Initialize { attach } | SelectionChange::Extend { attach } => {
            (None, None, &[][..], attach.as_slice())
        }
        SelectionChange::Reorg {
            common_ancestor,
            detach,
            attach,
        } => (
            Some(*common_ancestor),
            None,
            detach.as_slice(),
            attach.as_slice(),
        ),
        SelectionChange::AdvanceFinalized { target } => (None, Some(*target), &[][..], &[][..]),
    };

    let result_revision = mutation.expected().revision().next()?;
    let result_selected_head = match mutation.change() {
        SelectionChange::Initialize { attach }
        | SelectionChange::Extend { attach }
        | SelectionChange::Reorg { attach, .. } => attach.last().map(BlockRef::from_batch),
        SelectionChange::AdvanceFinalized { .. } => mutation.expected().selected_head(),
    };
    let result_finalized_selection = match mutation.change() {
        SelectionChange::Initialize { .. } => None,
        SelectionChange::Extend { .. } | SelectionChange::Reorg { .. } => {
            mutation.expected().finalized_selection()
        }
        SelectionChange::AdvanceFinalized { target } => Some(*target),
    };
    let outcome = ApplyOutcome::new(
        mutation.mutation_id(),
        result_revision,
        result_selected_head,
        result_finalized_selection,
        mutation.kind(),
    );

    let mut members = Vec::with_capacity(detach.len().saturating_add(attach.len()));
    for (ordinal, reference) in detach.iter().copied().enumerate() {
        members.push(JournalMember {
            role: MemberRole::Detach,
            ordinal: ordinal_u64(ordinal)?,
            reference,
        });
    }
    for (ordinal, batch) in attach.iter().enumerate() {
        members.push(JournalMember {
            role: MemberRole::Attach,
            ordinal: ordinal_u64(ordinal)?,
            reference: BlockRef::from_batch(batch),
        });
    }

    Ok(MutationProjection {
        header: JournalHeader {
            chain_id: mutation.chain_id(),
            mutation_id: mutation.mutation_id(),
            expected: mutation.expected().clone(),
            owner: mutation.owner().clone(),
            fence: mutation.fence(),
            common_ancestor,
            finality_target,
            outcome,
        },
        members,
    })
}

fn decode_header(fields: HeaderStorage) -> Result<JournalHeader, SelectedChainRepositoryError> {
    let chain_id_value =
        decode_nonnegative_i64("indexer_mutation_journal.chain_id", fields.chain_id)?;
    let chain_id = ChainId::new(chain_id_value)
        .map_err(|error| corrupt(format!("stored journal chain ID is invalid: {error}")))?;
    let mutation_id = MutationId::new(decode_b256(
        "indexer_mutation_journal.mutation_id",
        fields.mutation_id,
    )?)
    .map_err(|error| corrupt(format!("stored journal mutation ID is invalid: {error}")))?;
    let kind = decode_kind(fields.kind)?;
    let expected_revision = decode_revision(
        "indexer_mutation_journal.expected_revision",
        fields.expected_revision,
    )?;
    let expected_head = decode_optional_ref(
        chain_id,
        "indexer_mutation_journal.expected_selected_head",
        fields.expected_selected_head_number,
        fields.expected_selected_head_hash,
    )?;
    let expected_finalized = decode_optional_ref(
        chain_id,
        "indexer_mutation_journal.expected_finalized_selection",
        fields.expected_finalized_selection_number,
        fields.expected_finalized_selection_hash,
    )?;
    let owner = LeaseOwner::new(fields.lease_owner)
        .map_err(|error| corrupt(format!("stored journal lease owner is invalid: {error}")))?;
    let fence_value =
        decode_nonnegative_i64("indexer_mutation_journal.lease_fence", fields.lease_fence)?;
    let fence = LeaseFence::new(fence_value)
        .map_err(|error| corrupt(format!("stored journal lease fence is invalid: {error}")))?;
    let common_ancestor = decode_optional_ref(
        chain_id,
        "indexer_mutation_journal.common_ancestor",
        fields.common_ancestor_number,
        fields.common_ancestor_hash,
    )?;
    let finality_target = decode_optional_ref(
        chain_id,
        "indexer_mutation_journal.finality_target",
        fields.finality_target_number,
        fields.finality_target_hash,
    )?;
    let result_revision = decode_revision(
        "indexer_mutation_journal.result_revision",
        fields.result_revision,
    )?;
    let result_head = decode_optional_ref(
        chain_id,
        "indexer_mutation_journal.result_selected_head",
        fields.result_selected_head_number,
        fields.result_selected_head_hash,
    )?;
    let result_finalized = decode_optional_ref(
        chain_id,
        "indexer_mutation_journal.result_finalized_selection",
        fields.result_finalized_selection_number,
        fields.result_finalized_selection_hash,
    )?;

    Ok(JournalHeader {
        chain_id,
        mutation_id,
        expected: ExpectedChainState::new(expected_revision, expected_head, expected_finalized),
        owner,
        fence,
        common_ancestor,
        finality_target,
        outcome: ApplyOutcome::new(
            mutation_id,
            result_revision,
            result_head,
            result_finalized,
            kind,
        ),
    })
}

fn decode_members(
    chain_id: ChainId,
    rows: Vec<MemberStorage>,
) -> Result<Vec<JournalMember>, SelectedChainRepositoryError> {
    let mut next_detach = 0u64;
    let mut next_attach = 0u64;
    let mut members = Vec::with_capacity(rows.len());
    for row in rows {
        let role = decode_role(row.role)?;
        let ordinal = decode_nonnegative_i64("indexer_mutation_blocks.ordinal", row.ordinal)?;
        let expected_ordinal = match role {
            MemberRole::Detach => &mut next_detach,
            MemberRole::Attach => &mut next_attach,
        };
        if ordinal != *expected_ordinal {
            return Err(corrupt(format!(
                "stored {role:?} mutation members are not dense from ordinal zero"
            )));
        }
        *expected_ordinal = expected_ordinal
            .checked_add(1)
            .ok_or_else(|| corrupt("stored mutation member ordinal overflowed"))?;
        let number_value = decode_nonnegative_i64("indexer_mutation_blocks.number", row.number)?;
        let number = BlockNumber::new(number_value).map_err(|error| {
            corrupt(format!("stored mutation block number is invalid: {error}"))
        })?;
        let hash = BlockHash::new(decode_b256(
            "indexer_mutation_blocks.block_hash",
            row.block_hash,
        )?)
        .map_err(|error| corrupt(format!("stored mutation block hash is invalid: {error}")))?;
        members.push(JournalMember {
            role,
            ordinal,
            reference: BlockRef::new(chain_id, number, hash),
        });
    }
    Ok(members)
}

fn validate_record_shape(
    header: &JournalHeader,
    members: &[JournalMember],
) -> Result<(), SelectedChainRepositoryError> {
    if header.outcome.mutation_id() != header.mutation_id {
        return Err(corrupt(
            "journal result mutation ID differs from its header",
        ));
    }
    let next_revision = header
        .expected
        .revision()
        .next()
        .map_err(|error| corrupt(format!("stored journal revision cannot advance: {error}")))?;
    if header.outcome.revision() != next_revision {
        return Err(corrupt(
            "journal result revision is not exactly expected revision plus one",
        ));
    }
    if (header.expected.revision() == ChainRevision::ZERO)
        != header.expected.selected_head().is_none()
    {
        return Err(corrupt(
            "journal expected revision and selected-head presence are inconsistent",
        ));
    }

    let all_refs = header
        .expected
        .selected_head()
        .into_iter()
        .chain(header.expected.finalized_selection())
        .chain(header.common_ancestor)
        .chain(header.finality_target)
        .chain(header.outcome.selected_head())
        .chain(header.outcome.finalized_selection())
        .chain(members.iter().map(|member| member.reference));
    if all_refs
        .into_iter()
        .any(|reference| reference.chain_id() != header.chain_id)
    {
        return Err(corrupt(
            "journal contains a block reference for another chain",
        ));
    }
    validate_finalized_bounds(
        "expected",
        header.expected.selected_head(),
        header.expected.finalized_selection(),
    )?;
    validate_finalized_bounds(
        "result",
        header.outcome.selected_head(),
        header.outcome.finalized_selection(),
    )?;

    let detach = members_for_role(members, MemberRole::Detach);
    let attach = members_for_role(members, MemberRole::Attach);
    validate_dense_member_numbers(&detach)?;
    validate_dense_member_numbers(&attach)?;

    match header.outcome.kind() {
        MutationKind::Initialize => {
            if header.expected != ExpectedChainState::empty()
                || header.common_ancestor.is_some()
                || header.finality_target.is_some()
                || !detach.is_empty()
                || attach.is_empty()
                || header.outcome.selected_head() != attach.last().copied()
                || header.outcome.finalized_selection().is_some()
            {
                return Err(corrupt("stored initialize journal shape is invalid"));
            }
        }
        MutationKind::Extend => {
            let Some(expected_head) = header.expected.selected_head() else {
                return Err(corrupt(
                    "stored extend journal is missing its expected head",
                ));
            };
            if header.common_ancestor.is_some()
                || header.finality_target.is_some()
                || !detach.is_empty()
                || !members_begin_after(&attach, expected_head)
                || header.outcome.selected_head() != attach.last().copied()
                || header.outcome.finalized_selection() != header.expected.finalized_selection()
            {
                return Err(corrupt("stored extend journal shape is invalid"));
            }
        }
        MutationKind::Reorg => {
            let Some(expected_head) = header.expected.selected_head() else {
                return Err(corrupt("stored reorg journal is missing its expected head"));
            };
            let Some(common) = header.common_ancestor else {
                return Err(corrupt(
                    "stored reorg journal is missing its common ancestor",
                ));
            };
            if header.finality_target.is_some()
                || !members_begin_after(&detach, common)
                || detach.last().copied() != Some(expected_head)
                || !members_begin_after(&attach, common)
                || header.outcome.selected_head() != attach.last().copied()
                || header.outcome.finalized_selection() != header.expected.finalized_selection()
                || header
                    .expected
                    .finalized_selection()
                    .is_some_and(|finalized| common.number() < finalized.number())
                || attach == detach
            {
                return Err(corrupt("stored reorg journal shape is invalid"));
            }
        }
        MutationKind::AdvanceFinalized => {
            let Some(expected_head) = header.expected.selected_head() else {
                return Err(corrupt(
                    "stored finality journal is missing its expected head",
                ));
            };
            let Some(target) = header.finality_target else {
                return Err(corrupt("stored finality journal is missing its target"));
            };
            if header.common_ancestor.is_some()
                || !members.is_empty()
                || header.outcome.selected_head() != Some(expected_head)
                || header.outcome.finalized_selection() != Some(target)
                || target.number() > expected_head.number()
                || header
                    .expected
                    .finalized_selection()
                    .is_some_and(|finalized| target.number() <= finalized.number())
            {
                return Err(corrupt("stored finality journal shape is invalid"));
            }
        }
    }
    Ok(())
}

fn validate_finalized_bounds(
    label: &str,
    head: Option<BlockRef>,
    finalized: Option<BlockRef>,
) -> Result<(), SelectedChainRepositoryError> {
    if finalized.is_some() && head.is_none() {
        return Err(corrupt(format!(
            "journal {label} finalized selection has no selected head"
        )));
    }
    if finalized
        .zip(head)
        .is_some_and(|(finalized, head)| finalized.number() > head.number())
    {
        return Err(corrupt(format!(
            "journal {label} finalized selection is above its selected head"
        )));
    }
    Ok(())
}

fn members_for_role(members: &[JournalMember], role: MemberRole) -> Vec<BlockRef> {
    members
        .iter()
        .filter(|member| member.role == role)
        .map(|member| member.reference)
        .collect()
}

fn validate_dense_member_numbers(members: &[BlockRef]) -> Result<(), SelectedChainRepositoryError> {
    for pair in members.windows(2) {
        let expected = pair[0]
            .number()
            .get()
            .checked_add(1)
            .ok_or_else(|| corrupt("stored mutation block number overflowed"))?;
        if pair[1].number().get() != expected {
            return Err(corrupt("stored mutation block numbers are not dense"));
        }
    }
    Ok(())
}

fn members_begin_after(members: &[BlockRef], previous: BlockRef) -> bool {
    members.first().is_some_and(|first| {
        previous
            .number()
            .get()
            .checked_add(1)
            .is_some_and(|number| first.number().get() == number)
    })
}

async fn verify_candidate_facts(
    transaction: &mut Transaction<'_, Postgres>,
    mutation: &ChainMutation,
    header: &JournalHeader,
    members: &[JournalMember],
) -> Result<(), SelectedChainRepositoryError> {
    let attachments = match mutation.change() {
        SelectionChange::Initialize { attach }
        | SelectionChange::Extend { attach }
        | SelectionChange::Reorg { attach, .. } => attach.as_slice(),
        SelectionChange::AdvanceFinalized { .. } => &[],
    };

    let mut member_refs = BTreeSet::new();
    for member in members {
        member_refs.insert(member.reference);
        let stored = load_exact_candidate(transaction, member.reference).await?;
        if member.role == MemberRole::Attach {
            let incoming = attachments
                .get(usize::try_from(member.ordinal).map_err(|_| {
                    corrupt("stored attachment ordinal does not fit memory indexing")
                })?)
                .ok_or_else(|| corrupt("stored attachment ordinal has no incoming batch"))?;
            compare_attached_candidate(header.mutation_id, &stored, incoming)?;
        }
    }

    let header_refs = header
        .expected
        .selected_head()
        .into_iter()
        .chain(header.expected.finalized_selection())
        .chain(header.common_ancestor)
        .chain(header.finality_target)
        .chain(header.outcome.selected_head())
        .chain(header.outcome.finalized_selection())
        .collect::<BTreeSet<_>>();
    for reference in header_refs.difference(&member_refs) {
        load_exact_candidate(transaction, *reference).await?;
    }
    Ok(())
}

async fn load_exact_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    reference: BlockRef,
) -> Result<ValidatedBlockBatch, SelectedChainRepositoryError> {
    let candidate = candidates::load_candidate(transaction, reference.identity())
        .await?
        .ok_or_else(|| {
            corrupt(format!(
                "journal-owned candidate fact is missing: {:?}",
                reference.identity()
            ))
        })?;
    if BlockRef::from_batch(&candidate) != reference {
        return Err(corrupt(
            "journal block reference number differs from its immutable candidate fact",
        ));
    }
    Ok(candidate)
}

fn compare_attached_candidate(
    mutation_id: MutationId,
    stored: &ValidatedBlockBatch,
    incoming: &ValidatedBlockBatch,
) -> Result<(), SelectedChainRepositoryError> {
    if stored != incoming {
        return Err(reused(mutation_id));
    }
    Ok(())
}

fn compare_record(
    mutation_id: MutationId,
    stored_header: &JournalHeader,
    stored_members: &[JournalMember],
    incoming: &MutationProjection,
) -> Result<(), SelectedChainRepositoryError> {
    if stored_header != &incoming.header || stored_members != incoming.members {
        return Err(reused(mutation_id));
    }
    Ok(())
}

fn stored_request_matches(
    mutation: &ChainMutation,
    stored_header: &JournalHeader,
    stored_members: &[JournalMember],
) -> bool {
    if stored_header.chain_id != mutation.chain_id()
        || stored_header.mutation_id != mutation.mutation_id()
        || &stored_header.expected != mutation.expected()
        || &stored_header.owner != mutation.owner()
        || stored_header.fence != mutation.fence()
        || stored_header.outcome.kind() != mutation.kind()
    {
        return false;
    }

    let (common_ancestor, finality_target, detach, attach) = match mutation.change() {
        SelectionChange::Initialize { attach } | SelectionChange::Extend { attach } => {
            (None, None, &[][..], attach.as_slice())
        }
        SelectionChange::Reorg {
            common_ancestor,
            detach,
            attach,
        } => (
            Some(*common_ancestor),
            None,
            detach.as_slice(),
            attach.as_slice(),
        ),
        SelectionChange::AdvanceFinalized { target } => (None, Some(*target), &[][..], &[][..]),
    };
    if stored_header.common_ancestor != common_ancestor
        || stored_header.finality_target != finality_target
    {
        return false;
    }

    let stored_detach = stored_members
        .iter()
        .filter(|member| member.role == MemberRole::Detach)
        .map(|member| member.reference);
    let stored_attach = stored_members
        .iter()
        .filter(|member| member.role == MemberRole::Attach)
        .map(|member| member.reference);
    stored_detach.eq(detach.iter().copied())
        && stored_attach.eq(attach.iter().map(BlockRef::from_batch))
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
            let number_value = decode_nonnegative_i64(&format!("{field}_number"), number)?;
            let number = BlockNumber::new(number_value)
                .map_err(|error| corrupt(format!("stored {field} number is invalid: {error}")))?;
            let hash = BlockHash::new(decode_b256(&format!("{field}_hash"), hash)?)
                .map_err(|error| corrupt(format!("stored {field} hash is invalid: {error}")))?;
            Ok(Some(BlockRef::new(chain_id, number, hash)))
        }
        _ => Err(corrupt(format!(
            "stored {field} number and hash are not paired"
        ))),
    }
}

fn decode_revision(field: &str, value: i64) -> Result<ChainRevision, SelectedChainRepositoryError> {
    let value = decode_nonnegative_i64(field, value)?;
    ChainRevision::new(value)
        .map_err(|error| corrupt(format!("stored {field} is invalid: {error}")))
}

fn encode_kind(kind: MutationKind) -> i16 {
    match kind {
        MutationKind::Initialize => KIND_INITIALIZE,
        MutationKind::Extend => KIND_EXTEND,
        MutationKind::Reorg => KIND_REORG,
        MutationKind::AdvanceFinalized => KIND_ADVANCE_FINALIZED,
    }
}

fn decode_kind(value: i16) -> Result<MutationKind, SelectedChainRepositoryError> {
    match value {
        KIND_INITIALIZE => Ok(MutationKind::Initialize),
        KIND_EXTEND => Ok(MutationKind::Extend),
        KIND_REORG => Ok(MutationKind::Reorg),
        KIND_ADVANCE_FINALIZED => Ok(MutationKind::AdvanceFinalized),
        _ => Err(corrupt(format!(
            "stored mutation kind is outside 0..=3: {value}"
        ))),
    }
}

fn encode_role(role: MemberRole) -> i16 {
    match role {
        MemberRole::Detach => ROLE_DETACH,
        MemberRole::Attach => ROLE_ATTACH,
    }
}

fn decode_role(value: i16) -> Result<MemberRole, SelectedChainRepositoryError> {
    match value {
        ROLE_DETACH => Ok(MemberRole::Detach),
        ROLE_ATTACH => Ok(MemberRole::Attach),
        _ => Err(corrupt(format!(
            "stored mutation member role is outside 0..=1: {value}"
        ))),
    }
}

fn header_storage(row: &PgRow) -> Result<HeaderStorage, SelectedChainRepositoryError> {
    Ok(HeaderStorage {
        chain_id: column(row, "chain_id")?,
        mutation_id: column(row, "mutation_id")?,
        kind: column(row, "kind")?,
        expected_revision: column(row, "expected_revision")?,
        expected_selected_head_number: column(row, "expected_selected_head_number")?,
        expected_selected_head_hash: column(row, "expected_selected_head_hash")?,
        expected_finalized_selection_number: column(row, "expected_finalized_selection_number")?,
        expected_finalized_selection_hash: column(row, "expected_finalized_selection_hash")?,
        lease_owner: column(row, "lease_owner")?,
        lease_fence: column(row, "lease_fence")?,
        common_ancestor_number: column(row, "common_ancestor_number")?,
        common_ancestor_hash: column(row, "common_ancestor_hash")?,
        finality_target_number: column(row, "finality_target_number")?,
        finality_target_hash: column(row, "finality_target_hash")?,
        result_revision: column(row, "result_revision")?,
        result_selected_head_number: column(row, "result_selected_head_number")?,
        result_selected_head_hash: column(row, "result_selected_head_hash")?,
        result_finalized_selection_number: column(row, "result_finalized_selection_number")?,
        result_finalized_selection_hash: column(row, "result_finalized_selection_hash")?,
    })
}

fn member_storage(row: &PgRow) -> Result<MemberStorage, SelectedChainRepositoryError> {
    Ok(MemberStorage {
        role: column(row, "role")?,
        ordinal: column(row, "ordinal")?,
        number: column(row, "number")?,
        block_hash: column(row, "block_hash")?,
    })
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

fn ordinal_u64(ordinal: usize) -> Result<u64, SelectedChainRepositoryError> {
    u64::try_from(ordinal)
        .map_err(|_| corrupt("mutation member ordinal exceeds unsigned 64-bit storage"))
}

fn ordinal_storage(ordinal: u64) -> Result<i64, SelectedChainRepositoryError> {
    i64::try_from(ordinal)
        .map_err(|_| corrupt("mutation member ordinal exceeds signed 64-bit storage"))
}

fn column<T>(row: &PgRow, name: &'static str) -> Result<T, SelectedChainRepositoryError>
where
    T: for<'row> Decode<'row, Postgres> + Type<Postgres>,
{
    row.try_get(name)
        .map_err(|error| corrupt(format!("could not decode column {name}: {error}")))
}

fn unavailable(context: &str, error: sqlx::Error) -> SelectedChainRepositoryError {
    SelectedChainRepositoryError::Unavailable(format!("{context}: {error}"))
}

fn corrupt(message: impl Into<String>) -> SelectedChainRepositoryError {
    SelectedChainRepositoryError::CorruptState(message.into())
}

fn reused(mutation_id: MutationId) -> SelectedChainRepositoryError {
    SelectionConflict::MutationIdReuse { mutation_id }.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingestion::{validate_block, BlockRequest, FetchedBlock, ValidationLimits};
    use alloy::primitives::B256;

    fn hash(marker: u8) -> B256 {
        B256::repeat_byte(marker)
    }

    fn chain() -> ChainId {
        ChainId::new(56).expect("chain")
    }

    fn block_ref(number: u64, marker: u8) -> BlockRef {
        BlockRef::new(
            chain(),
            BlockNumber::new(number).expect("number"),
            BlockHash::new(hash(marker)).expect("hash"),
        )
    }

    fn batch(number: u64, marker: u8, parent: u8, timestamp: u64) -> ValidatedBlockBatch {
        validate_block(
            BlockRequest::new(chain(), BlockNumber::new(number).expect("number")),
            FetchedBlock {
                chain_id: chain().get(),
                number,
                hash: hash(marker),
                parent_hash: hash(parent),
                timestamp,
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

    fn owner(name: &str) -> LeaseOwner {
        LeaseOwner::new(name).expect("owner")
    }

    fn mutation_id(marker: u8) -> MutationId {
        MutationId::new(hash(marker)).expect("mutation id")
    }

    fn storage_from_header(header: &JournalHeader) -> HeaderStorage {
        HeaderStorage {
            chain_id: chain_id_storage(header.chain_id),
            mutation_id: encode_b256(header.mutation_id.get()),
            kind: encode_kind(header.outcome.kind()),
            expected_revision: revision_storage(header.expected.revision()),
            expected_selected_head_number: reference_number(header.expected.selected_head()),
            expected_selected_head_hash: reference_hash(header.expected.selected_head()),
            expected_finalized_selection_number: reference_number(
                header.expected.finalized_selection(),
            ),
            expected_finalized_selection_hash: reference_hash(
                header.expected.finalized_selection(),
            ),
            lease_owner: header.owner.as_str().to_string(),
            lease_fence: fence_storage(header.fence),
            common_ancestor_number: reference_number(header.common_ancestor),
            common_ancestor_hash: reference_hash(header.common_ancestor),
            finality_target_number: reference_number(header.finality_target),
            finality_target_hash: reference_hash(header.finality_target),
            result_revision: revision_storage(header.outcome.revision()),
            result_selected_head_number: reference_number(header.outcome.selected_head()),
            result_selected_head_hash: reference_hash(header.outcome.selected_head()),
            result_finalized_selection_number: reference_number(
                header.outcome.finalized_selection(),
            ),
            result_finalized_selection_hash: reference_hash(header.outcome.finalized_selection()),
        }
    }

    #[test]
    fn projection_encodes_exact_reorg_header_members_and_result() {
        let old_10 = block_ref(10, 10);
        let old_11 = block_ref(11, 11);
        let new_11 = batch(11, 21, 10, 1_700_000_011);
        let new_12 = batch(12, 22, 21, 1_700_000_012);
        let mutation = ChainMutation::reorg(
            chain(),
            mutation_id(1),
            ExpectedChainState::new(
                ChainRevision::new(7).expect("revision"),
                Some(old_11),
                Some(old_10),
            ),
            owner("worker-a"),
            LeaseFence::new(3).expect("fence"),
            old_10,
            vec![old_11],
            vec![new_11.clone(), new_12.clone()],
        )
        .expect("mutation");

        let projection = project_mutation(&mutation).expect("projection");
        assert_eq!(projection.header.common_ancestor, Some(old_10));
        assert_eq!(projection.header.finality_target, None);
        assert_eq!(projection.header.outcome.revision().get(), 8);
        assert_eq!(
            projection.header.outcome.selected_head(),
            Some(BlockRef::from_batch(&new_12))
        );
        assert_eq!(
            projection.members,
            vec![
                JournalMember {
                    role: MemberRole::Detach,
                    ordinal: 0,
                    reference: old_11,
                },
                JournalMember {
                    role: MemberRole::Attach,
                    ordinal: 0,
                    reference: BlockRef::from_batch(&new_11),
                },
                JournalMember {
                    role: MemberRole::Attach,
                    ordinal: 1,
                    reference: BlockRef::from_batch(&new_12),
                },
            ]
        );
        validate_record_shape(&projection.header, &projection.members).expect("shape");
        assert_eq!(
            decode_header(storage_from_header(&projection.header)).expect("decode"),
            projection.header
        );
    }

    #[test]
    fn header_decoder_rejects_unknown_kind_unpaired_refs_and_revision_drift() {
        let first = batch(10, 10, 9, 1_700_000_010);
        let mutation = ChainMutation::initialize(
            chain(),
            mutation_id(2),
            ExpectedChainState::empty(),
            owner("worker-a"),
            LeaseFence::new(1).expect("fence"),
            vec![first],
        )
        .expect("mutation");
        let projection = project_mutation(&mutation).expect("projection");
        let mut storage = storage_from_header(&projection.header);
        storage.kind = 4;
        assert!(matches!(
            decode_header(storage),
            Err(SelectedChainRepositoryError::CorruptState(_))
        ));

        let mut storage = storage_from_header(&projection.header);
        storage.result_selected_head_hash = None;
        assert!(matches!(
            decode_header(storage),
            Err(SelectedChainRepositoryError::CorruptState(_))
        ));

        let mut header = projection.header;
        header.outcome = ApplyOutcome::new(
            header.mutation_id,
            ChainRevision::new(2).expect("revision"),
            header.outcome.selected_head(),
            None,
            MutationKind::Initialize,
        );
        assert!(validate_record_shape(&header, &projection.members).is_err());
    }

    #[test]
    fn member_decoder_rejects_unknown_roles_and_non_dense_ordinals() {
        let valid = MemberStorage {
            role: ROLE_ATTACH,
            ordinal: 0,
            number: 10,
            block_hash: encode_b256(hash(10)),
        };
        let mut unknown = valid.clone();
        unknown.role = 2;
        assert!(decode_members(chain(), vec![unknown]).is_err());

        let mut gap = valid;
        gap.ordinal = 1;
        assert!(decode_members(chain(), vec![gap]).is_err());
    }

    #[test]
    fn shape_validation_rejects_kind_specific_member_drift() {
        let first = batch(10, 10, 9, 1_700_000_010);
        let mutation = ChainMutation::initialize(
            chain(),
            mutation_id(3),
            ExpectedChainState::empty(),
            owner("worker-a"),
            LeaseFence::new(1).expect("fence"),
            vec![first],
        )
        .expect("mutation");
        let mut projection = project_mutation(&mutation).expect("projection");
        projection.members[0].role = MemberRole::Detach;
        assert!(validate_record_shape(&projection.header, &projection.members).is_err());
    }

    #[test]
    fn full_attached_candidate_comparison_detects_same_identity_content_reuse() {
        let stored = batch(10, 10, 9, 1_700_000_010);
        let incoming = batch(10, 10, 8, 1_700_000_010);
        let id = mutation_id(4);
        assert_eq!(
            BlockRef::from_batch(&stored),
            BlockRef::from_batch(&incoming)
        );
        assert!(matches!(
            compare_attached_candidate(id, &stored, &incoming),
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::MutationIdReuse { mutation_id }
            )) if mutation_id == id
        ));
        assert!(compare_attached_candidate(id, &stored, &stored).is_ok());
    }

    #[test]
    fn exact_header_and_member_comparison_classifies_content_drift_as_reuse() {
        let first = batch(10, 10, 9, 1_700_000_010);
        let mutation = ChainMutation::initialize(
            chain(),
            mutation_id(5),
            ExpectedChainState::empty(),
            owner("worker-a"),
            LeaseFence::new(1).expect("fence"),
            vec![first],
        )
        .expect("mutation");
        let projection = project_mutation(&mutation).expect("projection");
        assert!(compare_record(
            mutation.mutation_id(),
            &projection.header,
            &projection.members,
            &projection,
        )
        .is_ok());

        let mut changed = projection.header.clone();
        changed.owner = owner("worker-b");
        assert!(matches!(
            compare_record(
                mutation.mutation_id(),
                &changed,
                &projection.members,
                &projection,
            ),
            Err(SelectedChainRepositoryError::Conflict(
                SelectionConflict::MutationIdReuse { mutation_id }
            )) if mutation_id == mutation.mutation_id()
        ));
        assert!(stored_request_matches(
            &mutation,
            &projection.header,
            &projection.members
        ));

        let exhausted = ChainMutation::extend(
            chain(),
            mutation.mutation_id(),
            ExpectedChainState::new(ChainRevision::MAX_STORAGE, Some(block_ref(10, 10)), None),
            owner("worker-a"),
            LeaseFence::new(1).expect("fence"),
            vec![batch(11, 11, 10, 1_700_000_011)],
        )
        .expect("mutation");
        assert!(!stored_request_matches(
            &exhausted,
            &projection.header,
            &projection.members
        ));
    }
}
