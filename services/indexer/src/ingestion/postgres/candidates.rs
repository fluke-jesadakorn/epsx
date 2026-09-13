use std::collections::BTreeMap;

use alloy::primitives::B256;
use sqlx::{postgres::PgRow, Acquire, Decode, Postgres, Row, Transaction, Type};

use super::super::{
    validate_block, BlockHash, BlockIdentity, BlockNumber, BlockRef, BlockRequest, ChainId,
    FetchedBlock, FetchedLog, FetchedReceipt, FetchedTransaction, SelectedChainRepositoryError,
    SelectionConflict, ValidatedBlockBatch, ValidationLimits,
};
use super::codec::{
    decode_address, decode_b256, decode_nonnegative_i32, decode_nonnegative_i64,
    decode_receipt_outcome, decode_timestamp_seconds, decode_u256_decimal, encode_address,
    encode_b256, encode_receipt_outcome, encode_u256_decimal, CodecError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CandidatePersistence {
    Inserted,
    MatchedExisting,
}

/// Persists one immutable candidate inside a rollback-on-cancellation
/// savepoint. The caller owns the surrounding transaction and its commit.
pub(super) async fn persist_or_compare_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    candidate: &ValidatedBlockBatch,
) -> Result<CandidatePersistence, SelectedChainRepositoryError> {
    let mut savepoint = transaction
        .begin()
        .await
        .map_err(|error| unavailable("begin candidate savepoint", error))?;

    let outcome = match persist_or_compare_inner(&mut savepoint, candidate).await {
        Ok(outcome) => outcome,
        Err(error) => {
            savepoint
                .rollback()
                .await
                .map_err(|rollback| unavailable("rollback candidate savepoint", rollback))?;
            return Err(error);
        }
    };
    savepoint
        .commit()
        .await
        .map_err(|error| unavailable("commit candidate savepoint", error))?;
    Ok(outcome)
}

async fn persist_or_compare_inner(
    transaction: &mut Transaction<'_, Postgres>,
    candidate: &ValidatedBlockBatch,
) -> Result<CandidatePersistence, SelectedChainRepositoryError> {
    let block = candidate.block();
    let inserted = sqlx::query(
        r#"
        INSERT INTO public.indexer_block_candidates (
            chain_id,
            block_hash,
            number,
            parent_hash,
            block_timestamp,
            beneficiary,
            gas_used,
            gas_limit
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (chain_id, block_hash) DO NOTHING
        "#,
    )
    .bind(chain_id_storage(block.chain_id()))
    .bind(encode_b256(block.hash().get()))
    .bind(block.number().get())
    .bind(encode_b256(block.parent_hash()))
    .bind(block.timestamp())
    .bind(block.beneficiary().map(encode_address))
    .bind(block.gas_used())
    .bind(block.gas_limit())
    .execute(&mut **transaction)
    .await
    .map_err(|error| unavailable("insert block candidate", error))?
    .rows_affected()
        == 1;

    if !inserted {
        let identity = BlockIdentity::new(block.chain_id(), block.hash());
        let stored = load_candidate(transaction, identity)
            .await?
            .ok_or_else(|| {
                corrupt("candidate identity conflicted but the stored row could not be loaded")
            })?;
        if stored == *candidate {
            return Ok(CandidatePersistence::MatchedExisting);
        }
        return Err(SelectionConflict::CandidateContent { identity }.into());
    }

    for included in candidate.transactions() {
        sqlx::query(
            r#"
            INSERT INTO public.indexer_transaction_inclusions (
                chain_id,
                block_hash,
                transaction_index,
                transaction_hash,
                from_address,
                to_address,
                value,
                input_data
            )
            VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS NUMERIC(78, 0)), $8)
            "#,
        )
        .bind(chain_id_storage(block.chain_id()))
        .bind(encode_b256(block.hash().get()))
        .bind(included.transaction_index().get())
        .bind(encode_b256(included.hash().get()))
        .bind(encode_address(included.from()))
        .bind(included.to().map(encode_address))
        .bind(encode_u256_decimal(included.value()))
        .bind(included.input())
        .execute(&mut **transaction)
        .await
        .map_err(|error| unavailable("insert transaction inclusion", error))?;
    }

    for receipt in candidate.receipts() {
        let (outcome, post_state_root) = encode_receipt_outcome(receipt.outcome());
        sqlx::query(
            r#"
            INSERT INTO public.indexer_receipts (
                chain_id,
                block_hash,
                transaction_index,
                outcome,
                post_state_root,
                gas_used,
                cumulative_gas_used
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(chain_id_storage(block.chain_id()))
        .bind(encode_b256(block.hash().get()))
        .bind(receipt.transaction_index().get())
        .bind(outcome)
        .bind(post_state_root)
        .bind(receipt.gas_used())
        .bind(receipt.cumulative_gas_used())
        .execute(&mut **transaction)
        .await
        .map_err(|error| unavailable("insert receipt", error))?;
    }

    for log in candidate.logs() {
        let mut topics = log.topics().iter().copied().map(encode_b256);
        let topic0 = topics.next();
        let topic1 = topics.next();
        let topic2 = topics.next();
        let topic3 = topics.next();
        debug_assert!(topics.next().is_none());
        sqlx::query(
            r#"
            INSERT INTO public.indexer_raw_logs (
                chain_id,
                block_hash,
                log_index,
                transaction_index,
                address,
                topic0,
                topic1,
                topic2,
                topic3,
                data
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(chain_id_storage(block.chain_id()))
        .bind(encode_b256(block.hash().get()))
        .bind(log.log_index().get())
        .bind(log.transaction_index().get())
        .bind(encode_address(log.address()))
        .bind(topic0)
        .bind(topic1)
        .bind(topic2)
        .bind(topic3)
        .bind(log.data())
        .execute(&mut **transaction)
        .await
        .map_err(|error| unavailable("insert raw log", error))?;
    }

    Ok(CandidatePersistence::Inserted)
}

pub(super) async fn load_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    identity: BlockIdentity,
) -> Result<Option<ValidatedBlockBatch>, SelectedChainRepositoryError> {
    let block_row = sqlx::query(
        r#"
        SELECT
            chain_id,
            block_hash,
            number,
            parent_hash,
            block_timestamp,
            beneficiary,
            gas_used,
            gas_limit
        FROM public.indexer_block_candidates
        WHERE chain_id = $1 AND block_hash = $2
        "#,
    )
    .bind(chain_id_storage(identity.chain_id()))
    .bind(encode_b256(identity.hash().get()))
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| unavailable("load block candidate", error))?;

    let Some(block_row) = block_row else {
        return Ok(None);
    };

    let transaction_rows = sqlx::query(
        r#"
        SELECT
            transaction_index,
            transaction_hash,
            from_address,
            to_address,
            value::text AS value_text,
            input_data
        FROM public.indexer_transaction_inclusions
        WHERE chain_id = $1 AND block_hash = $2
        ORDER BY transaction_index ASC
        "#,
    )
    .bind(chain_id_storage(identity.chain_id()))
    .bind(encode_b256(identity.hash().get()))
    .fetch_all(&mut **transaction)
    .await
    .map_err(|error| unavailable("load transaction inclusions", error))?;

    let receipt_rows = sqlx::query(
        r#"
        SELECT
            transaction_index,
            outcome,
            post_state_root,
            gas_used,
            cumulative_gas_used
        FROM public.indexer_receipts
        WHERE chain_id = $1 AND block_hash = $2
        ORDER BY transaction_index ASC
        "#,
    )
    .bind(chain_id_storage(identity.chain_id()))
    .bind(encode_b256(identity.hash().get()))
    .fetch_all(&mut **transaction)
    .await
    .map_err(|error| unavailable("load receipts", error))?;

    let log_rows = sqlx::query(
        r#"
        SELECT
            log_index,
            transaction_index,
            address,
            topic0,
            topic1,
            topic2,
            topic3,
            data
        FROM public.indexer_raw_logs
        WHERE chain_id = $1 AND block_hash = $2
        ORDER BY log_index ASC
        "#,
    )
    .bind(chain_id_storage(identity.chain_id()))
    .bind(encode_b256(identity.hash().get()))
    .fetch_all(&mut **transaction)
    .await
    .map_err(|error| unavailable("load raw logs", error))?;

    decode_candidate(
        identity,
        block_row,
        transaction_rows,
        receipt_rows,
        log_rows,
    )
    .map(Some)
}

pub(super) async fn load_candidate_refs_at_height(
    transaction: &mut Transaction<'_, Postgres>,
    chain_id: ChainId,
    number: BlockNumber,
) -> Result<Vec<BlockRef>, SelectedChainRepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT block_hash
        FROM public.indexer_block_candidates
        WHERE chain_id = $1 AND number = $2
        ORDER BY block_hash ASC
        "#,
    )
    .bind(chain_id_storage(chain_id))
    .bind(number.get())
    .fetch_all(&mut **transaction)
    .await
    .map_err(|error| unavailable("load candidate identities at height", error))?;

    rows.into_iter()
        .map(|row| {
            let hash = decode_b256(
                "indexer_block_candidates.block_hash",
                column(&row, "block_hash")?,
            )?;
            let hash = BlockHash::new(hash)
                .map_err(|error| corrupt(format!("stored candidate hash is invalid: {error}")))?;
            Ok(BlockRef::new(chain_id, number, hash))
        })
        .collect()
}

fn decode_candidate(
    identity: BlockIdentity,
    block_row: PgRow,
    transaction_rows: Vec<PgRow>,
    receipt_rows: Vec<PgRow>,
    log_rows: Vec<PgRow>,
) -> Result<ValidatedBlockBatch, SelectedChainRepositoryError> {
    let stored_chain = decode_nonnegative_i64(
        "indexer_block_candidates.chain_id",
        column(&block_row, "chain_id")?,
    )?;
    let number = decode_nonnegative_i64(
        "indexer_block_candidates.number",
        column(&block_row, "number")?,
    )?;
    let number_boundary = BlockNumber::new(number)
        .map_err(|error| corrupt(format!("stored block number is invalid: {error}")))?;
    let block_hash = decode_b256(
        "indexer_block_candidates.block_hash",
        column(&block_row, "block_hash")?,
    )?;
    let parent_hash = decode_b256(
        "indexer_block_candidates.parent_hash",
        column(&block_row, "parent_hash")?,
    )?;
    let timestamp = decode_timestamp_seconds(
        "indexer_block_candidates.block_timestamp",
        column(&block_row, "block_timestamp")?,
    )?;
    let beneficiary = column::<Option<Vec<u8>>>(&block_row, "beneficiary")?
        .map(|value| decode_address("indexer_block_candidates.beneficiary", value))
        .transpose()?;
    let gas_used = decode_nonnegative_i64(
        "indexer_block_candidates.gas_used",
        column(&block_row, "gas_used")?,
    )?;
    let gas_limit = decode_nonnegative_i64(
        "indexer_block_candidates.gas_limit",
        column(&block_row, "gas_limit")?,
    )?;

    let mut transactions = Vec::with_capacity(transaction_rows.len());
    let mut transaction_hashes = BTreeMap::new();
    let mut max_input_bytes = 0usize;
    let mut total_payload_bytes = 0usize;
    for row in transaction_rows {
        let transaction_index = decode_nonnegative_i32(
            "indexer_transaction_inclusions.transaction_index",
            column(&row, "transaction_index")?,
        )?;
        let hash = decode_b256(
            "indexer_transaction_inclusions.transaction_hash",
            column(&row, "transaction_hash")?,
        )?;
        let from = decode_address(
            "indexer_transaction_inclusions.from_address",
            column(&row, "from_address")?,
        )?;
        let to = column::<Option<Vec<u8>>>(&row, "to_address")?
            .map(|value| decode_address("indexer_transaction_inclusions.to_address", value))
            .transpose()?;
        let value_text: String = column(&row, "value_text")?;
        let value = decode_u256_decimal("indexer_transaction_inclusions.value", &value_text)?;
        let input: Vec<u8> = column(&row, "input_data")?;
        max_input_bytes = max_input_bytes.max(input.len());
        total_payload_bytes = total_payload_bytes
            .checked_add(input.len())
            .ok_or_else(|| corrupt("stored candidate payload byte count overflowed"))?;
        if transaction_hashes.insert(transaction_index, hash).is_some() {
            return Err(corrupt(
                "stored candidate has duplicate transaction indices",
            ));
        }
        transactions.push(FetchedTransaction {
            hash,
            block_hash,
            block_number: number,
            transaction_index,
            from,
            to,
            value,
            input,
        });
    }

    let mut logs_by_transaction: BTreeMap<u64, Vec<FetchedLog>> = BTreeMap::new();
    let mut max_log_data_bytes = 0usize;
    for row in log_rows {
        let log_index =
            decode_nonnegative_i32("indexer_raw_logs.log_index", column(&row, "log_index")?)?;
        let transaction_index = decode_nonnegative_i32(
            "indexer_raw_logs.transaction_index",
            column(&row, "transaction_index")?,
        )?;
        let transaction_hash = transaction_hashes
            .get(&transaction_index)
            .copied()
            .ok_or_else(|| corrupt("stored log references a missing transaction inclusion"))?;
        let address = decode_address("indexer_raw_logs.address", column(&row, "address")?)?;
        let topics = decode_topics(&row)?;
        let data: Vec<u8> = column(&row, "data")?;
        max_log_data_bytes = max_log_data_bytes.max(data.len());
        let topic_bytes = topics
            .len()
            .checked_mul(32)
            .ok_or_else(|| corrupt("stored log topic byte count overflowed"))?;
        total_payload_bytes = total_payload_bytes
            .checked_add(topic_bytes)
            .and_then(|total| total.checked_add(data.len()))
            .ok_or_else(|| corrupt("stored candidate payload byte count overflowed"))?;
        logs_by_transaction
            .entry(transaction_index)
            .or_default()
            .push(FetchedLog {
                block_hash,
                block_number: number,
                transaction_hash,
                transaction_index,
                log_index,
                address,
                topics,
                data,
                removed: false,
            });
    }

    let mut receipts = Vec::with_capacity(receipt_rows.len());
    for row in receipt_rows {
        let transaction_index = decode_nonnegative_i32(
            "indexer_receipts.transaction_index",
            column(&row, "transaction_index")?,
        )?;
        let transaction_hash = transaction_hashes
            .get(&transaction_index)
            .copied()
            .ok_or_else(|| corrupt("stored receipt references a missing transaction inclusion"))?;
        let outcome =
            decode_receipt_outcome(column(&row, "outcome")?, column(&row, "post_state_root")?)?;
        let receipt_gas_used =
            decode_nonnegative_i64("indexer_receipts.gas_used", column(&row, "gas_used")?)?;
        let cumulative_gas_used = decode_nonnegative_i64(
            "indexer_receipts.cumulative_gas_used",
            column(&row, "cumulative_gas_used")?,
        )?;
        receipts.push(FetchedReceipt {
            transaction_hash,
            transaction_index,
            block_hash,
            block_number: number,
            outcome,
            gas_used: receipt_gas_used,
            cumulative_gas_used,
            logs: logs_by_transaction
                .remove(&transaction_index)
                .unwrap_or_default(),
        });
    }
    if !logs_by_transaction.is_empty() {
        return Err(corrupt("stored logs are not covered by stored receipts"));
    }

    let limits = ValidationLimits::new(
        transactions.len(),
        log_count(&receipts)?,
        max_input_bytes,
        max_log_data_bytes,
        total_payload_bytes,
    );
    let request = BlockRequest::new(identity.chain_id(), number_boundary);
    let candidate = validate_block(
        request,
        FetchedBlock {
            chain_id: stored_chain,
            number,
            hash: block_hash,
            parent_hash,
            timestamp,
            beneficiary,
            gas_used,
            gas_limit,
            transactions,
            receipts,
        },
        limits,
    )
    .map_err(|error| corrupt(format!("stored candidate failed validation: {error}")))?;

    if candidate.block().hash() != identity.hash() {
        return Err(corrupt(
            "loaded candidate hash does not match its requested identity",
        ));
    }
    Ok(candidate)
}

fn decode_topics(row: &PgRow) -> Result<Vec<B256>, SelectedChainRepositoryError> {
    let stored = [
        column::<Option<Vec<u8>>>(row, "topic0")?,
        column::<Option<Vec<u8>>>(row, "topic1")?,
        column::<Option<Vec<u8>>>(row, "topic2")?,
        column::<Option<Vec<u8>>>(row, "topic3")?,
    ];
    let mut topics = Vec::with_capacity(4);
    let mut found_gap = false;
    for (index, value) in stored.into_iter().enumerate() {
        match value {
            Some(_) if found_gap => {
                return Err(corrupt(format!(
                    "stored log topic{index} appears after a null topic"
                )));
            }
            Some(value) => topics.push(decode_b256(
                &format!("indexer_raw_logs.topic{index}"),
                value,
            )?),
            None => found_gap = true,
        }
    }
    Ok(topics)
}

fn log_count(receipts: &[FetchedReceipt]) -> Result<usize, SelectedChainRepositoryError> {
    receipts.iter().try_fold(0usize, |count, receipt| {
        count
            .checked_add(receipt.logs.len())
            .ok_or_else(|| corrupt("stored candidate log count overflowed"))
    })
}

fn chain_id_storage(chain_id: ChainId) -> i64 {
    i64::try_from(chain_id.get()).expect("validated chain IDs fit signed storage")
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

impl From<CodecError> for SelectedChainRepositoryError {
    fn from(error: CodecError) -> Self {
        corrupt(error.to_string())
    }
}
