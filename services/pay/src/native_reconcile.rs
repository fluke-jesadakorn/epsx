//! Contract-scoped scanner with transactional PostgreSQL checkpoints and reorg replay.
use crate::{
    native_chain::{self as chain, Chain, Error},
    native_pay::Deal,
    AppState,
};
use alloy::{
    primitives::{keccak256, Address, B256, U256},
    sol_types::{SolCall, SolValue},
};
use serde_json::{json, Value};
use sqlx::Row;
use std::{str::FromStr, time::Duration};
fn text(v: &Value, key: &str) -> Result<String, Error> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| format!("missing {key}").into())
}
fn topic(v: &Value, index: usize) -> Result<B256, Error> {
    Ok(B256::from_str(
        v["topics"][index].as_str().ok_or("missing event topic")?,
    )?)
}
fn topic_address(v: &Value, index: usize) -> Result<Address, Error> {
    let b = topic(v, index)?;
    if b[..12].iter().any(|b| *b != 0) {
        return Err("invalid address topic".into());
    }
    Ok(Address::from_slice(&b[12..]))
}
struct Evidence {
    status: &'static str,
    fee: String,
    actor: String,
    kind: &'static str,
}
fn decode(c: &Chain, d: &Deal, log: &Value, transaction: &Value) -> Result<Evidence, Error> {
    let data = chain::bytes(&log["data"])?;
    let id = B256::from_str(&d.on_chain_id)?;
    let amount = chain::amount(&d.amount)?;
    let payer = chain::address(&d.payer)?;
    let payee = chain::address(&d.payee)?;
    let token = chain::address(&d.token_address)?;
    let from = chain::address(&text(transaction, "from")?)?;
    let input = chain::bytes(&transaction["input"])?;
    let signature = topic(log, 0)?;
    if topic(log, 1)? != id {
        return Err("escrow ID mismatch".into());
    }
    let (status, fee, kind, expected) = if signature
        == keccak256("Deposited(bytes32,address,address,address,uint256)")
    {
        if topic_address(log, 2)? != payer
            || topic_address(log, 3)? != payee
            || <(Address, U256)>::abi_decode(&data)? != (token, amount)
            || from != payer
        {
            return Err("deposit evidence mismatch".into());
        }
        let expected = chain::depositCall {
            salt: B256::from_str(&d.salt)?,
            payee,
            token,
            amount,
        }
        .abi_encode();
        if U256::from_str(
            transaction["value"]
                .as_str()
                .ok_or("missing transaction value")?,
        )? != if token.is_zero() { amount } else { U256::ZERO }
        {
            return Err("deposit value mismatch".into());
        }
        ("active", "0".into(), "deposit", expected)
    } else if signature == keccak256("Released(bytes32,address,address,address,uint256,uint256)") {
        let expected_fee = chain::fee(amount);
        if topic_address(log, 2)? != payer
            || topic_address(log, 3)? != payee
            || <(Address, U256, U256)>::abi_decode(&data)? != (token, amount, expected_fee)
        {
            return Err("release evidence mismatch".into());
        }
        if from == payer && input == (chain::releaseCall { id }).abi_encode() {
            (
                "released",
                expected_fee.to_string(),
                "release",
                chain::releaseCall { id }.abi_encode(),
            )
        } else if from == c.admin {
            (
                "released",
                expected_fee.to_string(),
                "resolve-release",
                chain::resolveCall { id, toPayee: true }.abi_encode(),
            )
        } else {
            return Err("unauthorized release sender".into());
        }
    } else if signature == keccak256("Refunded(bytes32,address,address,address,uint256)") {
        if topic_address(log, 2)? != payer
            || topic_address(log, 3)? != payee
            || <(Address, U256)>::abi_decode(&data)? != (token, amount)
        {
            return Err("refund evidence mismatch".into());
        }
        if from == payee && input == (chain::refundCall { id }).abi_encode() {
            (
                "refunded",
                "0".into(),
                "refund",
                chain::refundCall { id }.abi_encode(),
            )
        } else if from == c.admin {
            (
                "refunded",
                "0".into(),
                "resolve-refund",
                chain::resolveCall { id, toPayee: false }.abi_encode(),
            )
        } else {
            return Err("unauthorized refund sender".into());
        }
    } else if signature == keccak256("Disputed(bytes32,address)") {
        if !data.is_empty() || (from != payer && from != payee) || topic_address(log, 2)? != from {
            return Err("dispute evidence mismatch".into());
        }
        (
            "disputed",
            "0".into(),
            "dispute",
            chain::disputeCall { id }.abi_encode(),
        )
    } else {
        return Err("unknown escrow event".into());
    };
    if input != expected {
        return Err("transaction input mismatch".into());
    }
    Ok(Evidence {
        status,
        fee,
        actor: from.to_string().to_ascii_lowercase(),
        kind,
    })
}
async fn receipt(c: &Chain, hash: &str, height: u64, block_hash: &str) -> Result<Value, Error> {
    let r = c.rpc("eth_getTransactionReceipt", json!([hash])).await?;
    if chain::number(&r["status"])? != 1
        || chain::number(&r["blockNumber"])? != height
        || text(&r, "blockHash")? != block_hash
        || text(&r, "transactionHash")? != hash
        || chain::address(&text(&r, "to")?)? != c.contract
    {
        return Err("receipt evidence mismatch".into());
    }
    Ok(r)
}
async fn apply_log(
    s: &AppState,
    c: &Chain,
    tx: &mut sqlx::PgConnection,
    log: Value,
) -> Result<(), Error> {
    if log["removed"].as_bool() == Some(true)
        || chain::address(&text(&log, "address")?)? != c.contract
    {
        return Err("removed or foreign log".into());
    }
    let sig = topic(&log, 0)?;
    if ![
        "Deposited(bytes32,address,address,address,uint256)",
        "Released(bytes32,address,address,address,uint256,uint256)",
        "Refunded(bytes32,address,address,address,uint256)",
        "Disputed(bytes32,address)",
    ]
    .iter()
    .any(|s| keccak256(s) == sig)
    {
        return Ok(());
    }
    let id = format!("{:#x}", topic(&log, 1)?);
    let deal:Option<Deal>=sqlx::query_as("SELECT * FROM pay_v1_deals WHERE chain_id=$1 AND contract_address=$2 AND on_chain_id=$3 FOR UPDATE").bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase()).bind(&id).fetch_optional(&mut *tx).await?;
    // Direct deposits not created by EPSX cannot grant a package or invent an EPSX intent.
    let Some(deal) = deal else {
        tracing::warn!(escrow_id=%id,"unmatched escrow event; no application status changed");
        return Ok(());
    };
    let height = chain::number(&log["blockNumber"])?;
    let block_hash = text(&log, "blockHash")?;
    let hash = text(&log, "transactionHash")?;
    if c.block_hash(height).await? != block_hash {
        return Err("noncanonical event block".into());
    }
    let r = receipt(c, &hash, height, &block_hash).await?;
    if !r["logs"]
        .as_array()
        .ok_or("receipt logs missing")?
        .iter()
        .any(|l| {
            l["logIndex"] == log["logIndex"]
                && l["topics"] == log["topics"]
                && l["data"] == log["data"]
                && l["address"] == log["address"]
        })
    {
        return Err("event absent from receipt".into());
    }
    let transaction = c.rpc("eth_getTransactionByHash", json!([hash])).await?;
    if chain::number(&transaction["chainId"])? != c.chain_id
        || chain::address(&text(&transaction, "to")?)? != c.contract
        || text(&transaction, "hash")? != hash
    {
        return Err("transaction chain or destination mismatch".into());
    }
    let evidence = match decode(c, &deal, &log, &transaction) {
        Ok(evidence) => evidence,
        Err(error) => {
            tracing::error!(%error,deal_id=%deal.id,tx_hash=%hash,"Escrow event conflicts with immutable quote");
            sqlx::query("UPDATE pay_v1_deals SET status='verification_required',verification_error='quote_evidence_mismatch',updated_at=now() WHERE id=$1").bind(&deal.id).execute(&mut *tx).await?;
            return Ok(());
        }
    };
    sqlx::query("INSERT INTO pay_v1_chain_events(chain_id,contract_address,block_number,block_hash,tx_hash,log_index,payload) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT DO NOTHING").bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase()).bind(height as i64).bind(&block_hash).bind(&hash).bind(chain::number(&log["logIndex"])? as i64).bind(&log).execute(&mut *tx).await?;
    sqlx::query("UPDATE pay_v1_deals SET status=$2,fee_amount=$3,tx_hash=$4,verified_block=$5,verified_block_hash=$6,updated_at=now() WHERE id=$1").bind(&deal.id).bind(evidence.status).bind(evidence.fee).bind(&hash).bind(height as i64).bind(&block_hash).execute(&mut *tx).await?;
    // Parameters and actor must match even if the browser crashed before posting the hash.
    sqlx::query("UPDATE pay_v1_operations SET status='confirmed',tx_hash=$4,confirmed_block=$5,confirmed_block_hash=$6,updated_at=now() WHERE id=(SELECT id FROM pay_v1_operations WHERE deal_id=$1 AND kind=$2 AND actor=$3 AND (tx_hash=$4 OR tx_hash IS NULL) ORDER BY (tx_hash IS NOT NULL) DESC,created_at LIMIT 1)").bind(&deal.id).bind(evidence.kind).bind(evidence.actor).bind(hash).bind(height as i64).bind(block_hash).execute(&mut *tx).await?;
    let _ = s;
    Ok(())
}
pub async fn tick(s: &AppState, c: &Chain) -> Result<(), Error> {
    c.validate().await?;
    let scope = c.contract.to_string().to_ascii_lowercase();
    let mut tx = s.db.begin().await?;
    if !sqlx::query_scalar::<_, bool>("SELECT pg_try_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("pay-scanner:{}:{scope}", c.chain_id))
        .fetch_one(&mut *tx)
        .await?
    {
        return Ok(());
    }
    sqlx::query("INSERT INTO pay_v1_chain_checkpoints(chain_id,contract_address,next_block) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(c.chain_id as i64).bind(&scope).bind(c.deployment_block as i64).execute(&mut *tx).await?;
    let checkpoint=sqlx::query("SELECT next_block,last_block_hash FROM pay_v1_chain_checkpoints WHERE chain_id=$1 AND contract_address=$2 FOR UPDATE").bind(c.chain_id as i64).bind(&scope).fetch_one(&mut *tx).await?;
    let next = checkpoint.get::<i64, _>("next_block") as u64;
    let last: Option<String> = checkpoint.get("last_block_hash");
    if let Some(last) = last {
        let changed = if next == 0 {
            true
        } else {
            match c.block_hash(next - 1).await {
                Ok(hash) => hash != last,
                Err(error) => {
                    let head = chain::number(&c.rpc("eth_blockNumber", json!([])).await?)?;
                    if head < next - 1 {
                        true
                    } else {
                        return Err(error);
                    }
                }
            }
        };
        if changed {
            // Fail closed immediately, then replay canonical history from deployment.
            // This preserves intent IDs, checkout reservations and idempotency records.
            sqlx::query("UPDATE pay_v1_deals SET status='verification_required',fee_amount='0',verified_block=NULL,verified_block_hash=NULL WHERE chain_id=$1 AND contract_address=$2 AND verified_block IS NOT NULL").bind(c.chain_id as i64).bind(&scope).execute(&mut *tx).await?;
            sqlx::query("UPDATE pay_v1_operations SET status='verification_required',confirmed_block=NULL,confirmed_block_hash=NULL WHERE deal_id IN(SELECT id FROM pay_v1_deals WHERE chain_id=$1 AND contract_address=$2) AND status='confirmed'").bind(c.chain_id as i64).bind(&scope).execute(&mut *tx).await?;
            sqlx::query("UPDATE pay_v1_contract_operations SET status='verification_required',confirmed_block=NULL,confirmed_block_hash=NULL WHERE chain_id=$1 AND contract_address=$2 AND status='confirmed'").bind(c.chain_id as i64).bind(&scope).execute(&mut *tx).await?;
            sqlx::query(
                "DELETE FROM pay_v1_chain_events WHERE chain_id=$1 AND contract_address=$2",
            )
            .bind(c.chain_id as i64)
            .bind(&scope)
            .execute(&mut *tx)
            .await?;
            sqlx::query("UPDATE pay_v1_chain_checkpoints SET next_block=$3,last_block_hash=NULL,healthy=false,last_checked_at=now() WHERE chain_id=$1 AND contract_address=$2").bind(c.chain_id as i64).bind(&scope).bind(c.deployment_block as i64).execute(&mut *tx).await?;
            tx.commit().await?;
            return Ok(());
        }
    }
    let head = chain::number(&c.rpc("eth_blockNumber", json!([])).await?)?;
    if head.saturating_add(1) < c.confirmations {
        return Err("waiting for confirmations".into());
    }
    let finalized = head + 1 - c.confirmations;
    if next <= finalized {
        let end = next.saturating_add(c.scan_blocks - 1).min(finalized);
        let end_hash = c.block_hash(end).await?;
        let logs=c.rpc("eth_getLogs",json!([{"address":c.contract,"fromBlock":format!("0x{next:x}"),"toBlock":format!("0x{end:x}")}])).await?;
        let mut logs = logs.as_array().ok_or("invalid logs response")?.clone();
        logs.sort_by_key(|l| {
            (
                chain::number(&l["blockNumber"]).unwrap_or(u64::MAX),
                chain::number(&l["transactionIndex"]).unwrap_or(u64::MAX),
                chain::number(&l["logIndex"]).unwrap_or(u64::MAX),
            )
        });
        for log in logs {
            let n = chain::number(&log["blockNumber"])?;
            if n < next || n > end {
                return Err("log outside requested range".into());
            }
            apply_log(s, c, &mut tx, log).await?;
        }
        if c.block_hash(end).await? != end_hash {
            return Err("reorg during scan".into());
        }
        sqlx::query("UPDATE pay_v1_chain_checkpoints SET next_block=$3,last_block_hash=$4,healthy=$5,last_checked_at=now() WHERE chain_id=$1 AND contract_address=$2").bind(c.chain_id as i64).bind(&scope).bind((end+1) as i64).bind(end_hash).bind(end==finalized).execute(&mut *tx).await?;
    } else {
        sqlx::query("UPDATE pay_v1_chain_checkpoints SET healthy=true,last_checked_at=now() WHERE chain_id=$1 AND contract_address=$2").bind(c.chain_id as i64).bind(&scope).execute(&mut *tx).await?;
    }
    let caught_up: bool = sqlx::query_scalar(
        "SELECT healthy FROM pay_v1_chain_checkpoints WHERE chain_id=$1 AND contract_address=$2",
    )
    .bind(c.chain_id as i64)
    .bind(&scope)
    .fetch_one(&mut *tx)
    .await?;
    if caught_up {
        sqlx::query("UPDATE pay_v1_deals SET status='pending',tx_hash=NULL WHERE chain_id=$1 AND contract_address=$2 AND status='verification_required' AND verified_block IS NULL AND verification_error IS NULL").bind(c.chain_id as i64).bind(&scope).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    reconcile_controls(s, c, finalized).await?;
    // A failed or unrelated submitted transaction never changes a deal's money state.
    let operations=sqlx::query("SELECT o.id,o.tx_hash FROM pay_v1_operations o JOIN pay_v1_deals d ON d.id=o.deal_id WHERE d.chain_id=$1 AND d.contract_address=$2 AND o.status IN ('pending','verification_required') AND o.tx_hash IS NOT NULL LIMIT 100").bind(c.chain_id as i64).bind(&scope).fetch_all(&s.db).await?;
    for op in operations {
        let hash: String = op.get("tx_hash");
        let r = c.rpc("eth_getTransactionReceipt", json!([hash])).await?;
        if r.is_null() {
            continue;
        }
        let height = chain::number(&r["blockNumber"])?;
        if height > finalized || c.block_hash(height).await? != text(&r, "blockHash")? {
            continue;
        }
        let scanned:bool=sqlx::query_scalar("SELECT next_block>$3 FROM pay_v1_chain_checkpoints WHERE chain_id=$1 AND contract_address=$2").bind(c.chain_id as i64).bind(&scope).bind(height as i64).fetch_one(&s.db).await?;
        if scanned {
            sqlx::query("UPDATE pay_v1_operations SET status='failed',updated_at=now() WHERE id=$1 AND status IN ('pending','verification_required')").bind(op.get::<String,_>("id")).execute(&s.db).await?;
        }
    }
    Ok(())
}
pub fn spawn(s: AppState) {
    tokio::spawn(async move {
        let Some(c) = s.native_chain.clone() else {
            return;
        };
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let _ = checked_tick(&s, &c).await;
        }
    });
}

pub async fn checked_tick(s: &AppState, c: &Chain) -> Result<(), Error> {
    let result = tick(s, c).await;
    if let Err(error) = &result {
        tracing::error!(%error,"Pay chain reconciliation unavailable");
        sqlx::query("UPDATE pay_v1_chain_checkpoints SET healthy=false WHERE chain_id=$1 AND contract_address=$2")
            .bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase())
            .execute(&s.db).await?;
    }
    result
}

async fn reconcile_controls(s: &AppState, c: &Chain, finalized: u64) -> Result<(), Error> {
    let ops=sqlx::query("SELECT id,actor,paused,tx_hash,transaction_parameters FROM pay_v1_contract_operations WHERE chain_id=$1 AND contract_address=$2 AND status IN ('pending','verification_required') AND tx_hash IS NOT NULL LIMIT 100").bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase()).fetch_all(&s.db).await?;
    for op in ops {
        let hash: String = op.get("tx_hash");
        let r = c.rpc("eth_getTransactionReceipt", json!([hash])).await?;
        if r.is_null() {
            continue;
        }
        let height = chain::number(&r["blockNumber"])?;
        let block_hash = text(&r, "blockHash")?;
        if height > finalized || c.block_hash(height).await? != block_hash {
            continue;
        }
        let t = c.rpc("eth_getTransactionByHash", json!([hash])).await?;
        let params: Value = op.get("transaction_parameters");
        let event = keccak256(if op.get::<bool, _>("paused") {
            "Paused(address)"
        } else {
            "Unpaused(address)"
        });
        let valid = text(&r, "transactionHash")? == hash
            && text(&t, "hash")? == hash
            && chain::address(&text(&t, "to")?)? == c.contract
            && chain::number(&r["status"])? == 1
            && chain::number(&t["chainId"])? == c.chain_id
            && chain::address(&text(&r, "to")?)? == c.contract
            && chain::address(&text(&t, "from")?)? == c.admin
            && t["input"] == params["data"]
            && r["logs"].as_array().is_some_and(|logs| {
                logs.iter().any(|l| {
                    chain::address(l["address"].as_str().unwrap_or("")).ok() == Some(c.contract)
                        && topic(l, 0).ok() == Some(event)
                        && chain::bytes(&l["data"])
                            .ok()
                            .and_then(|b| Address::abi_decode(&b).ok())
                            == Some(c.admin)
                })
            });
        sqlx::query("UPDATE pay_v1_contract_operations SET status=$2,confirmed_block=$3,confirmed_block_hash=$4 WHERE id=$1").bind(op.get::<String,_>("id")).bind(if valid{"confirmed"}else{"failed"}).bind(height as i64).bind(block_hash).execute(&s.db).await?;
    }
    Ok(())
}
