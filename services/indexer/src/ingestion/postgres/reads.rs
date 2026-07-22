use sqlx::{PgPool, Postgres, Row, Transaction};

use super::super::{
    BlockHash, BlockIdentity, BlockNumber, BlockRef, ChainId, ChainRevision, ChainSnapshot,
    SelectedChainRepositoryError, ValidatedBlockBatch,
};
use super::candidates;
use super::codec::{decode_b256, decode_nonnegative_i64};

struct SnapshotFields {
    revision: i64,
    selected_head_number: Option<i64>,
    selected_head_hash: Option<Vec<u8>>,
    finalized_selection_number: Option<i64>,
    finalized_selection_hash: Option<Vec<u8>>,
}

struct SelectedMappingFields {
    number: i64,
    block_hash: Vec<u8>,
    selected_revision: i64,
}

struct SelectedHashFields {
    block_hash: Vec<u8>,
    selected_revision: i64,
    chain_revision: Option<i64>,
}

pub(super) async fn snapshot(
    pool: &PgPool,
    chain_id: ChainId,
) -> Result<ChainSnapshot, SelectedChainRepositoryError> {
    let mut transaction = begin_consistent_read(pool, "begin chain snapshot").await?;
    let row = sqlx::query(
        r#"
        SELECT
            revision,
            selected_head_number,
            selected_head_hash,
            finalized_selection_number,
            finalized_selection_hash
        FROM public.indexer_chain_state
        WHERE chain_id = $1
        "#,
    )
    .bind(chain_id_storage(chain_id))
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| unavailable("load chain snapshot", error))?;

    let snapshot = match row {
        Some(row) => {
            let fields = SnapshotFields {
                revision: column(&row, "revision")?,
                selected_head_number: column(&row, "selected_head_number")?,
                selected_head_hash: column(&row, "selected_head_hash")?,
                finalized_selection_number: column(&row, "finalized_selection_number")?,
                finalized_selection_hash: column(&row, "finalized_selection_hash")?,
            };
            let snapshot = decode_snapshot(chain_id, fields)?;
            verify_snapshot_mapping(&mut transaction, &snapshot).await?;
            snapshot
        }
        None => {
            require_no_orphaned_selection_state(&mut transaction, chain_id).await?;
            ChainSnapshot::new(chain_id, ChainRevision::ZERO, None, None)
        }
    };

    transaction
        .commit()
        .await
        .map_err(|error| unavailable("commit chain snapshot read", error))?;
    Ok(snapshot)
}

pub(super) async fn load_candidate(
    pool: &PgPool,
    identity: BlockIdentity,
) -> Result<Option<ValidatedBlockBatch>, SelectedChainRepositoryError> {
    // Candidate reconstruction spans the immutable block, inclusion, receipt,
    // and log tables. A single PostgreSQL snapshot prevents a concurrent
    // commit from being observed only by a suffix of those reads.
    let mut transaction = begin_consistent_read(pool, "begin candidate read").await?;
    let candidate = candidates::load_candidate(&mut transaction, identity).await?;
    transaction
        .commit()
        .await
        .map_err(|error| unavailable("commit candidate read", error))?;
    Ok(candidate)
}

pub(super) async fn selected_hash(
    pool: &PgPool,
    chain_id: ChainId,
    number: BlockNumber,
) -> Result<Option<BlockHash>, SelectedChainRepositoryError> {
    let mut transaction = begin_consistent_read(pool, "begin selected block read").await?;
    let row = sqlx::query(
        r#"
        SELECT
            selected.block_hash,
            selected.selected_revision,
            state.revision AS chain_revision
        FROM public.indexer_selected_blocks AS selected
        LEFT JOIN public.indexer_chain_state AS state
          ON state.chain_id = selected.chain_id
        WHERE selected.chain_id = $1 AND selected.number = $2
        "#,
    )
    .bind(chain_id_storage(chain_id))
    .bind(number.get())
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| unavailable("load selected block hash", error))?;

    let hash = row
        .map(|row| {
            decode_selected_hash(SelectedHashFields {
                block_hash: column(&row, "block_hash")?,
                selected_revision: column(&row, "selected_revision")?,
                chain_revision: column(&row, "chain_revision")?,
            })
        })
        .transpose()?;
    transaction
        .commit()
        .await
        .map_err(|error| unavailable("commit selected block read", error))?;
    Ok(hash)
}

pub(super) async fn candidates_at_height(
    pool: &PgPool,
    chain_id: ChainId,
    number: BlockNumber,
) -> Result<Vec<BlockRef>, SelectedChainRepositoryError> {
    let mut transaction = begin_consistent_read(pool, "begin candidate identity read").await?;
    let candidates =
        candidates::load_candidate_refs_at_height(&mut transaction, chain_id, number).await?;
    transaction
        .commit()
        .await
        .map_err(|error| unavailable("commit candidate identity read", error))?;
    Ok(candidates)
}

async fn begin_consistent_read<'pool>(
    pool: &'pool PgPool,
    context: &str,
) -> Result<Transaction<'pool, Postgres>, SelectedChainRepositoryError> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| unavailable(context, error))?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *transaction)
        .await
        .map_err(|error| unavailable("configure consistent read transaction", error))?;
    Ok(transaction)
}

fn decode_snapshot(
    chain_id: ChainId,
    fields: SnapshotFields,
) -> Result<ChainSnapshot, SelectedChainRepositoryError> {
    let revision = decode_revision(fields.revision)?;
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

    if revision == ChainRevision::ZERO && selected_head.is_some() {
        return Err(corrupt("revision zero cannot have a selected head"));
    }
    if revision != ChainRevision::ZERO && selected_head.is_none() {
        return Err(corrupt("a non-zero revision must have a selected head"));
    }
    if finalized_selection.is_some() && selected_head.is_none() {
        return Err(corrupt(
            "a finalized selection cannot exist without a selected head",
        ));
    }
    if finalized_selection
        .zip(selected_head)
        .is_some_and(|(finalized, head)| finalized.number() > head.number())
    {
        return Err(corrupt(
            "the finalized selection is above the selected head",
        ));
    }

    Ok(ChainSnapshot::new(
        chain_id,
        revision,
        selected_head,
        finalized_selection,
    ))
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
            let hash = decode_block_hash(&format!("{field}_hash"), hash)?;
            Ok(Some(BlockRef::new(chain_id, number, hash)))
        }
        _ => Err(corrupt(format!(
            "{field} number and hash must either both be null or both be present"
        ))),
    }
}

fn decode_revision(value: i64) -> Result<ChainRevision, SelectedChainRepositoryError> {
    let value = decode_nonnegative_i64("indexer_chain_state.revision", value)?;
    ChainRevision::new(value)
        .map_err(|error| corrupt(format!("stored chain revision is invalid: {error}")))
}

fn decode_selected_revision(value: i64) -> Result<ChainRevision, SelectedChainRepositoryError> {
    let value = decode_nonnegative_i64("indexer_selected_blocks.selected_revision", value)?;
    if value == 0 {
        return Err(corrupt("stored selected-block revision must be non-zero"));
    }
    ChainRevision::new(value).map_err(|error| {
        corrupt(format!(
            "stored selected-block revision is invalid: {error}"
        ))
    })
}

fn decode_selected_hash(
    fields: SelectedHashFields,
) -> Result<BlockHash, SelectedChainRepositoryError> {
    let chain_revision = fields
        .chain_revision
        .ok_or_else(|| corrupt("stored selected block has no chain-state row"))?;
    let chain_revision = decode_revision(chain_revision)?;
    let selected_revision = decode_selected_revision(fields.selected_revision)?;
    if selected_revision > chain_revision {
        return Err(corrupt(
            "stored selected-block revision exceeds the chain revision",
        ));
    }
    decode_block_hash("indexer_selected_blocks.block_hash", fields.block_hash)
}

fn decode_block_hash(
    field: &str,
    value: Vec<u8>,
) -> Result<BlockHash, SelectedChainRepositoryError> {
    let value = decode_b256(field, value)?;
    BlockHash::new(value).map_err(|error| corrupt(format!("{field} is invalid: {error}")))
}

async fn verify_snapshot_mapping(
    transaction: &mut Transaction<'_, Postgres>,
    snapshot: &ChainSnapshot,
) -> Result<(), SelectedChainRepositoryError> {
    let latest = sqlx::query(
        r#"
        SELECT number, block_hash, selected_revision
        FROM public.indexer_selected_blocks
        WHERE chain_id = $1
        ORDER BY number DESC
        LIMIT 1
        "#,
    )
    .bind(chain_id_storage(snapshot.chain_id()))
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| unavailable("load selected head mapping", error))?
    .map(|row| {
        decode_selected_mapping(
            snapshot.chain_id(),
            SelectedMappingFields {
                number: column(&row, "number")?,
                block_hash: column(&row, "block_hash")?,
                selected_revision: column(&row, "selected_revision")?,
            },
            snapshot.revision(),
        )
    })
    .transpose()?;

    if latest.map(|(reference, _)| reference) != snapshot.selected_head() {
        return Err(corrupt(
            "stored selected head is not the highest exact selected-block mapping",
        ));
    }

    if let Some(finalized) = snapshot.finalized_selection() {
        let row = sqlx::query(
            r#"
            SELECT number, block_hash, selected_revision
            FROM public.indexer_selected_blocks
            WHERE chain_id = $1 AND number = $2
            "#,
        )
        .bind(chain_id_storage(snapshot.chain_id()))
        .bind(finalized.number().get())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|error| unavailable("load finalized selection mapping", error))?
        .ok_or_else(|| corrupt("stored finalized selection has no selected-block mapping"))?;
        let (mapped, _) = decode_selected_mapping(
            snapshot.chain_id(),
            SelectedMappingFields {
                number: column(&row, "number")?,
                block_hash: column(&row, "block_hash")?,
                selected_revision: column(&row, "selected_revision")?,
            },
            snapshot.revision(),
        )?;
        if mapped != finalized {
            return Err(corrupt(
                "stored finalized selection does not match its selected-block mapping",
            ));
        }
    }

    let row = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM public.indexer_selected_blocks
            WHERE chain_id = $1 AND selected_revision > $2
        ) AS has_future_revision
        "#,
    )
    .bind(chain_id_storage(snapshot.chain_id()))
    .bind(revision_storage(snapshot.revision()))
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| unavailable("verify selected-block revisions", error))?;
    let has_future_revision: bool = column(&row, "has_future_revision")?;
    if has_future_revision {
        return Err(corrupt(
            "a selected-block mapping was written after the stored chain revision",
        ));
    }

    Ok(())
}

fn decode_selected_mapping(
    chain_id: ChainId,
    fields: SelectedMappingFields,
    chain_revision: ChainRevision,
) -> Result<(BlockRef, ChainRevision), SelectedChainRepositoryError> {
    let number = decode_nonnegative_i64("indexer_selected_blocks.number", fields.number)?;
    let number = BlockNumber::new(number)
        .map_err(|error| corrupt(format!("stored selected block number is invalid: {error}")))?;
    let hash = decode_block_hash("indexer_selected_blocks.block_hash", fields.block_hash)?;
    let selected_revision = decode_selected_revision(fields.selected_revision)?;
    if selected_revision > chain_revision {
        return Err(corrupt(
            "stored selected-block revision exceeds the chain revision",
        ));
    }
    Ok((BlockRef::new(chain_id, number, hash), selected_revision))
}

async fn require_no_orphaned_selection_state(
    transaction: &mut Transaction<'_, Postgres>,
    chain_id: ChainId,
) -> Result<(), SelectedChainRepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            EXISTS (
                SELECT 1 FROM public.indexer_selected_blocks WHERE chain_id = $1
            ) AS has_selected_blocks,
            EXISTS (
                SELECT 1 FROM public.indexer_mutation_journal WHERE chain_id = $1
            ) AS has_mutation_journal
        "#,
    )
    .bind(chain_id_storage(chain_id))
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| unavailable("check state-less chain references", error))?;
    let has_selected_blocks: bool = column(&row, "has_selected_blocks")?;
    let has_mutation_journal: bool = column(&row, "has_mutation_journal")?;
    if has_selected_blocks || has_mutation_journal {
        return Err(corrupt(
            "chain state row is missing while selected blocks or mutation journal rows exist",
        ));
    }
    Ok(())
}

fn chain_id_storage(chain_id: ChainId) -> i64 {
    i64::try_from(chain_id.get()).expect("validated chain IDs fit signed storage")
}

fn revision_storage(revision: ChainRevision) -> i64 {
    i64::try_from(revision.get()).expect("validated chain revisions fit signed storage")
}

fn column<T>(
    row: &sqlx::postgres::PgRow,
    name: &'static str,
) -> Result<T, SelectedChainRepositoryError>
where
    T: for<'row> sqlx::Decode<'row, Postgres> + sqlx::Type<Postgres>,
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

#[cfg(test)]
mod tests {
    use alloy::primitives::B256;

    use super::*;

    fn chain() -> ChainId {
        ChainId::new(1).expect("chain")
    }

    fn hash(byte: u8) -> Vec<u8> {
        B256::repeat_byte(byte).as_slice().to_vec()
    }

    #[test]
    fn snapshot_fields_decode_checked_revision_and_refs() {
        let snapshot = decode_snapshot(
            chain(),
            SnapshotFields {
                revision: 2,
                selected_head_number: Some(11),
                selected_head_hash: Some(hash(1)),
                finalized_selection_number: Some(10),
                finalized_selection_hash: Some(hash(2)),
            },
        )
        .expect("valid snapshot");

        assert_eq!(snapshot.revision().get(), 2);
        assert_eq!(snapshot.selected_head().expect("head").number().get(), 11);
        assert_eq!(
            snapshot
                .finalized_selection()
                .expect("finalized")
                .number()
                .get(),
            10
        );
    }

    #[test]
    fn snapshot_fields_reject_negative_or_malformed_storage_values() {
        let cases = [
            SnapshotFields {
                revision: -1,
                selected_head_number: None,
                selected_head_hash: None,
                finalized_selection_number: None,
                finalized_selection_hash: None,
            },
            SnapshotFields {
                revision: 1,
                selected_head_number: Some(-1),
                selected_head_hash: Some(hash(1)),
                finalized_selection_number: None,
                finalized_selection_hash: None,
            },
            SnapshotFields {
                revision: 1,
                selected_head_number: Some(1),
                selected_head_hash: None,
                finalized_selection_number: None,
                finalized_selection_hash: None,
            },
            SnapshotFields {
                revision: 1,
                selected_head_number: Some(1),
                selected_head_hash: Some(vec![0; 32]),
                finalized_selection_number: None,
                finalized_selection_hash: None,
            },
        ];

        for fields in cases {
            assert!(decode_snapshot(chain(), fields).is_err());
        }
    }

    #[test]
    fn snapshot_fields_enforce_revision_and_finalized_relationships() {
        let zero_with_head = SnapshotFields {
            revision: 0,
            selected_head_number: Some(1),
            selected_head_hash: Some(hash(1)),
            finalized_selection_number: None,
            finalized_selection_hash: None,
        };
        let finalized_above_head = SnapshotFields {
            revision: 2,
            selected_head_number: Some(10),
            selected_head_hash: Some(hash(1)),
            finalized_selection_number: Some(11),
            finalized_selection_hash: Some(hash(2)),
        };

        assert!(decode_snapshot(chain(), zero_with_head).is_err());
        assert!(decode_snapshot(chain(), finalized_above_head).is_err());
    }

    #[test]
    fn selected_mapping_rejects_zero_or_future_revisions() {
        let revision = ChainRevision::new(2).expect("revision");
        for selected_revision in [0, 3] {
            assert!(decode_selected_mapping(
                chain(),
                SelectedMappingFields {
                    number: 10,
                    block_hash: hash(1),
                    selected_revision,
                },
                revision,
            )
            .is_err());
        }
    }

    #[test]
    fn selected_hash_fields_reject_orphaned_and_future_rows() {
        let orphaned = SelectedHashFields {
            block_hash: hash(1),
            selected_revision: 1,
            chain_revision: None,
        };
        let future = SelectedHashFields {
            block_hash: hash(1),
            selected_revision: 3,
            chain_revision: Some(2),
        };
        let revision_zero = SelectedHashFields {
            block_hash: hash(1),
            selected_revision: 1,
            chain_revision: Some(0),
        };

        assert!(decode_selected_hash(orphaned).is_err());
        assert!(decode_selected_hash(future).is_err());
        assert!(decode_selected_hash(revision_zero).is_err());
    }

    #[test]
    fn selected_hash_fields_accept_a_current_checked_row() {
        let expected = BlockHash::new(B256::repeat_byte(1)).expect("hash");
        let decoded = decode_selected_hash(SelectedHashFields {
            block_hash: hash(1),
            selected_revision: 2,
            chain_revision: Some(2),
        })
        .expect("current selected hash");

        assert_eq!(decoded, expected);
    }

    #[test]
    fn empty_snapshot_shape_is_valid_for_a_lease_only_state_row() {
        let snapshot = decode_snapshot(
            chain(),
            SnapshotFields {
                revision: 0,
                selected_head_number: None,
                selected_head_hash: None,
                finalized_selection_number: None,
                finalized_selection_hash: None,
            },
        )
        .expect("empty snapshot");
        assert_eq!(snapshot.revision(), ChainRevision::ZERO);
        assert_eq!(snapshot.selected_head(), None);
        assert_eq!(snapshot.finalized_selection(), None);
    }
}
