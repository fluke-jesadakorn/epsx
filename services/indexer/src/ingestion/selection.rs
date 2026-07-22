use super::{BlockHash, BlockNumber, ChainId, ValidatedBlockBatch};
use alloy::primitives::B256;
use chrono::{DateTime, Utc};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockIdentity {
    chain_id: ChainId,
    hash: BlockHash,
}

impl BlockIdentity {
    pub const fn new(chain_id: ChainId, hash: BlockHash) -> Self {
        Self { chain_id, hash }
    }

    pub const fn chain_id(self) -> ChainId {
        self.chain_id
    }

    pub const fn hash(self) -> BlockHash {
        self.hash
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockRef {
    identity: BlockIdentity,
    number: BlockNumber,
}

impl BlockRef {
    pub const fn new(chain_id: ChainId, number: BlockNumber, hash: BlockHash) -> Self {
        Self {
            identity: BlockIdentity::new(chain_id, hash),
            number,
        }
    }

    pub const fn identity(self) -> BlockIdentity {
        self.identity
    }

    pub const fn chain_id(self) -> ChainId {
        self.identity.chain_id
    }

    pub const fn number(self) -> BlockNumber {
        self.number
    }

    pub const fn hash(self) -> BlockHash {
        self.identity.hash
    }

    pub fn from_batch(batch: &ValidatedBlockBatch) -> Self {
        Self::new(
            batch.block().chain_id(),
            batch.block().number(),
            batch.block().hash(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ChainRevision(u64);

impl ChainRevision {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn next(self) -> Result<Self, MutationBuildError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(MutationBuildError::RevisionExhausted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LeaseOwner(String);

impl LeaseOwner {
    pub fn new(value: impl Into<String>) -> Result<Self, SelectionBoundaryError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
            })
        {
            return Err(SelectionBoundaryError::InvalidLeaseOwner);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LeaseFence(u64);

impl LeaseFence {
    pub fn new(value: u64) -> Result<Self, SelectionBoundaryError> {
        if value == 0 {
            return Err(SelectionBoundaryError::ZeroLeaseFence);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn successor(previous: Option<Self>) -> Result<Self, SelectionBoundaryError> {
        let next = previous
            .map_or(Some(1), |fence| fence.0.checked_add(1))
            .ok_or(SelectionBoundaryError::LeaseFenceExhausted)?;
        Self::new(next)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MutationId(B256);

impl MutationId {
    pub fn new(value: B256) -> Result<Self, SelectionBoundaryError> {
        if value == B256::ZERO {
            return Err(SelectionBoundaryError::ZeroMutationId);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> B256 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseDuration(Duration);

impl LeaseDuration {
    pub fn new(value: Duration) -> Result<Self, SelectionBoundaryError> {
        if value.is_zero() || value > Duration::from_secs(24 * 60 * 60) {
            return Err(SelectionBoundaryError::InvalidLeaseDuration);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> Duration {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseGrant {
    chain_id: ChainId,
    owner: LeaseOwner,
    fence: LeaseFence,
    expires_at: DateTime<Utc>,
}

impl LeaseGrant {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn new(
        chain_id: ChainId,
        owner: LeaseOwner,
        fence: LeaseFence,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            chain_id,
            owner,
            fence,
            expires_at,
        }
    }

    pub const fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub const fn owner(&self) -> &LeaseOwner {
        &self.owner
    }

    pub const fn fence(&self) -> LeaseFence {
        self.fence
    }

    pub const fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedChainState {
    revision: ChainRevision,
    selected_head: Option<BlockRef>,
    finalized_selection: Option<BlockRef>,
}

impl ExpectedChainState {
    pub const fn new(
        revision: ChainRevision,
        selected_head: Option<BlockRef>,
        finalized_selection: Option<BlockRef>,
    ) -> Self {
        Self {
            revision,
            selected_head,
            finalized_selection,
        }
    }

    pub const fn empty() -> Self {
        Self::new(ChainRevision::new(0), None, None)
    }

    pub const fn revision(&self) -> ChainRevision {
        self.revision
    }

    pub const fn selected_head(&self) -> Option<BlockRef> {
        self.selected_head
    }

    pub const fn finalized_selection(&self) -> Option<BlockRef> {
        self.finalized_selection
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainSnapshot {
    chain_id: ChainId,
    revision: ChainRevision,
    selected_head: Option<BlockRef>,
    finalized_selection: Option<BlockRef>,
}

impl ChainSnapshot {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn new(
        chain_id: ChainId,
        revision: ChainRevision,
        selected_head: Option<BlockRef>,
        finalized_selection: Option<BlockRef>,
    ) -> Self {
        Self {
            chain_id,
            revision,
            selected_head,
            finalized_selection,
        }
    }

    pub const fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub const fn revision(&self) -> ChainRevision {
        self.revision
    }

    pub const fn selected_head(&self) -> Option<BlockRef> {
        self.selected_head
    }

    pub const fn finalized_selection(&self) -> Option<BlockRef> {
        self.finalized_selection
    }

    pub const fn expected(&self) -> ExpectedChainState {
        ExpectedChainState::new(self.revision, self.selected_head, self.finalized_selection)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationKind {
    Initialize,
    Extend,
    Reorg,
    AdvanceFinalized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyOutcome {
    mutation_id: MutationId,
    revision: ChainRevision,
    selected_head: Option<BlockRef>,
    finalized_selection: Option<BlockRef>,
    kind: MutationKind,
}

impl ApplyOutcome {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn new(
        mutation_id: MutationId,
        revision: ChainRevision,
        selected_head: Option<BlockRef>,
        finalized_selection: Option<BlockRef>,
        kind: MutationKind,
    ) -> Self {
        Self {
            mutation_id,
            revision,
            selected_head,
            finalized_selection,
            kind,
        }
    }

    pub const fn mutation_id(&self) -> MutationId {
        self.mutation_id
    }

    pub const fn revision(&self) -> ChainRevision {
        self.revision
    }

    pub const fn selected_head(&self) -> Option<BlockRef> {
        self.selected_head
    }

    pub const fn finalized_selection(&self) -> Option<BlockRef> {
        self.finalized_selection
    }

    pub const fn kind(&self) -> MutationKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainMutation {
    chain_id: ChainId,
    mutation_id: MutationId,
    expected: ExpectedChainState,
    owner: LeaseOwner,
    fence: LeaseFence,
    change: SelectionChange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SelectionChange {
    Initialize {
        attach: Vec<ValidatedBlockBatch>,
    },
    Extend {
        attach: Vec<ValidatedBlockBatch>,
    },
    Reorg {
        common_ancestor: BlockRef,
        detach: Vec<BlockRef>,
        attach: Vec<ValidatedBlockBatch>,
    },
    AdvanceFinalized {
        target: BlockRef,
    },
}

impl ChainMutation {
    pub fn initialize(
        chain_id: ChainId,
        mutation_id: MutationId,
        expected: ExpectedChainState,
        owner: LeaseOwner,
        fence: LeaseFence,
        attach: Vec<ValidatedBlockBatch>,
    ) -> Result<Self, MutationBuildError> {
        if expected.selected_head.is_some() || expected.finalized_selection.is_some() {
            return Err(MutationBuildError::InitializeExpectedState);
        }
        validate_attachment(chain_id, &attach)?;
        Ok(Self {
            chain_id,
            mutation_id,
            expected,
            owner,
            fence,
            change: SelectionChange::Initialize { attach },
        })
    }

    pub fn extend(
        chain_id: ChainId,
        mutation_id: MutationId,
        expected: ExpectedChainState,
        owner: LeaseOwner,
        fence: LeaseFence,
        attach: Vec<ValidatedBlockBatch>,
    ) -> Result<Self, MutationBuildError> {
        validate_expected_chain(chain_id, &expected)?;
        let head = expected
            .selected_head
            .ok_or(MutationBuildError::MissingSelectedHead)?;
        validate_attachment(chain_id, &attach)?;
        validate_first_after(head, &attach[0])?;
        Ok(Self {
            chain_id,
            mutation_id,
            expected,
            owner,
            fence,
            change: SelectionChange::Extend { attach },
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reorg(
        chain_id: ChainId,
        mutation_id: MutationId,
        expected: ExpectedChainState,
        owner: LeaseOwner,
        fence: LeaseFence,
        common_ancestor: BlockRef,
        detach: Vec<BlockRef>,
        attach: Vec<ValidatedBlockBatch>,
    ) -> Result<Self, MutationBuildError> {
        validate_expected_chain(chain_id, &expected)?;
        let head = expected
            .selected_head
            .ok_or(MutationBuildError::MissingSelectedHead)?;
        if common_ancestor.chain_id() != chain_id || head.chain_id() != chain_id {
            return Err(MutationBuildError::ChainMismatch);
        }
        validate_detach(common_ancestor, head, &detach)?;
        validate_attachment(chain_id, &attach)?;
        validate_first_after(common_ancestor, &attach[0])?;
        if attach
            .iter()
            .map(BlockRef::from_batch)
            .eq(detach.iter().copied())
        {
            return Err(MutationBuildError::ReorgNoop);
        }
        if expected
            .finalized_selection
            .is_some_and(|finalized| common_ancestor.number() < finalized.number())
        {
            return Err(MutationBuildError::FinalizedBoundary);
        }
        Ok(Self {
            chain_id,
            mutation_id,
            expected,
            owner,
            fence,
            change: SelectionChange::Reorg {
                common_ancestor,
                detach,
                attach,
            },
        })
    }

    pub fn advance_finalized(
        chain_id: ChainId,
        mutation_id: MutationId,
        expected: ExpectedChainState,
        owner: LeaseOwner,
        fence: LeaseFence,
        target: BlockRef,
    ) -> Result<Self, MutationBuildError> {
        validate_expected_chain(chain_id, &expected)?;
        let head = expected
            .selected_head
            .ok_or(MutationBuildError::MissingSelectedHead)?;
        if target.chain_id() != chain_id || head.chain_id() != chain_id {
            return Err(MutationBuildError::ChainMismatch);
        }
        if target.number() > head.number()
            || expected
                .finalized_selection
                .is_some_and(|current| target.number() <= current.number())
        {
            return Err(MutationBuildError::FinalityDoesNotAdvance);
        }
        Ok(Self {
            chain_id,
            mutation_id,
            expected,
            owner,
            fence,
            change: SelectionChange::AdvanceFinalized { target },
        })
    }

    pub const fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub const fn mutation_id(&self) -> MutationId {
        self.mutation_id
    }

    pub const fn expected(&self) -> &ExpectedChainState {
        &self.expected
    }

    pub const fn owner(&self) -> &LeaseOwner {
        &self.owner
    }

    pub const fn fence(&self) -> LeaseFence {
        self.fence
    }

    pub const fn kind(&self) -> MutationKind {
        match self.change {
            SelectionChange::Initialize { .. } => MutationKind::Initialize,
            SelectionChange::Extend { .. } => MutationKind::Extend,
            SelectionChange::Reorg { .. } => MutationKind::Reorg,
            SelectionChange::AdvanceFinalized { .. } => MutationKind::AdvanceFinalized,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) const fn change(&self) -> &SelectionChange {
        &self.change
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SelectionBoundaryError {
    #[error("lease owner must be 1..=128 safe ASCII characters")]
    InvalidLeaseOwner,
    #[error("lease duration must be nonzero and no more than 24 hours")]
    InvalidLeaseDuration,
    #[error("lease fence must be nonzero")]
    ZeroLeaseFence,
    #[error("lease fence is exhausted")]
    LeaseFenceExhausted,
    #[error("mutation id must be nonzero")]
    ZeroMutationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MutationBuildError {
    #[error("chain revision is exhausted")]
    RevisionExhausted,
    #[error("attachment must contain at least one block")]
    EmptyAttachment,
    #[error("attachment contains another chain")]
    ChainMismatch,
    #[error("attachment block numbers are not dense")]
    AttachmentGap,
    #[error("attachment parent hashes are not linked")]
    AttachmentParent,
    #[error("initialize requires an expected state without heads")]
    InitializeExpectedState,
    #[error("transition requires an expected selected head")]
    MissingSelectedHead,
    #[error("reorg detach must be the dense suffix after the common ancestor")]
    InvalidDetach,
    #[error("reorg attachment must differ from the detached selection")]
    ReorgNoop,
    #[error("reorg would cross the finalized selection")]
    FinalizedBoundary,
    #[error("finalized selection must strictly advance within the selected head")]
    FinalityDoesNotAdvance,
}

fn validate_expected_chain(
    chain_id: ChainId,
    expected: &ExpectedChainState,
) -> Result<(), MutationBuildError> {
    if expected
        .selected_head
        .into_iter()
        .chain(expected.finalized_selection)
        .any(|block| block.chain_id() != chain_id)
    {
        return Err(MutationBuildError::ChainMismatch);
    }
    Ok(())
}

fn validate_attachment(
    chain_id: ChainId,
    attach: &[ValidatedBlockBatch],
) -> Result<(), MutationBuildError> {
    if attach.is_empty() {
        return Err(MutationBuildError::EmptyAttachment);
    }
    for (index, batch) in attach.iter().enumerate() {
        if batch.block().chain_id() != chain_id {
            return Err(MutationBuildError::ChainMismatch);
        }
        if let Some(previous) = index.checked_sub(1).map(|previous| &attach[previous]) {
            validate_first_after(BlockRef::from_batch(previous), batch)?;
        }
    }
    Ok(())
}

fn validate_first_after(
    previous: BlockRef,
    next: &ValidatedBlockBatch,
) -> Result<(), MutationBuildError> {
    let expected_number = previous
        .number()
        .get()
        .checked_add(1)
        .ok_or(MutationBuildError::AttachmentGap)?;
    if next.block().number().get() != expected_number {
        return Err(MutationBuildError::AttachmentGap);
    }
    if next.block().parent_hash() != previous.hash().get() {
        return Err(MutationBuildError::AttachmentParent);
    }
    Ok(())
}

fn validate_detach(
    common_ancestor: BlockRef,
    expected_head: BlockRef,
    detach: &[BlockRef],
) -> Result<(), MutationBuildError> {
    if detach.is_empty() || detach.last() != Some(&expected_head) {
        return Err(MutationBuildError::InvalidDetach);
    }
    let mut previous = common_ancestor;
    for block in detach {
        let expected_number = previous.number().get().checked_add(1);
        if block.chain_id() != common_ancestor.chain_id()
            || Some(block.number().get()) != expected_number
        {
            return Err(MutationBuildError::InvalidDetach);
        }
        previous = *block;
    }
    Ok(())
}
