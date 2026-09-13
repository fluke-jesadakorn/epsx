use super::{
    api::Payment,
    chain::{self, Network},
    emit, Platform,
};
use crate::native_chain::{self as rpc, Error};
use alloy::{
    primitives::{keccak256, Address, B256, U256},
    sol_types::SolValue,
};
use serde_json::{json, Value};
use sqlx::Row;
use std::{str::FromStr, time::Duration};

fn text(v: &Value, k: &str) -> std::result::Result<String, Error> {
    v[k].as_str()
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| format!("missing {k}").into())
}
fn topic(v: &Value, i: usize) -> std::result::Result<B256, Error> {
    Ok(B256::from_str(
        v["topics"][i].as_str().ok_or("missing topic")?,
    )?)
}
fn who(v: &Value, i: usize) -> std::result::Result<Address, Error> {
    let b = topic(v, i)?;
    if b[..12].iter().any(|b| *b != 0) {
        return Err("invalid address topic".into());
    }
    Ok(Address::from_slice(&b[12..]))
}
fn decode(
    n: &Network,
    d: &Payment,
    log: &Value,
    t: &Value,
) -> std::result::Result<(&'static str, String, &'static str, &'static str), Error> {
    let payer = Address::from_str(d.payer.as_deref().ok_or("unbound payer")?)?;
    let payee = Address::from_str(&d.payee)?;
    let token = Address::from_str(&d.token_address)?;
    let amount = rpc::amount(&d.amount)?;
    let fee = chain::fee(amount, d.fee_bps as u16);
    let sender = Address::from_str(&text(t, "from")?)?;
    let input = rpc::bytes(&t["input"])?;
    let data = rpc::bytes(&log["data"])?;
    let sig = topic(log, 0)?;
    let (status, fee_amount, kind, event) =
        if sig == keccak256("Paid(bytes32,address,address,address,uint256,uint256)") {
            if d.mode != "direct"
                || sender != payer
                || who(log, 2)? != payer
                || who(log, 3)? != payee
                || <(Address, U256, U256)>::abi_decode(&data)? != (token, amount, fee)
            {
                return Err("direct payment mismatch".into());
            }
            ("succeeded", fee.to_string(), "pay", "payment.succeeded")
        } else if sig == keccak256("Funded(bytes32,address,address,address,uint256)") {
            if d.mode != "escrow"
                || sender != payer
                || who(log, 2)? != payer
                || who(log, 3)? != payee
                || <(Address, U256)>::abi_decode(&data)? != (token, amount)
            {
                return Err("escrow funding mismatch".into());
            }
            ("funded", "0".into(), "deposit", "escrow.funded")
        } else if sig == keccak256("Released(bytes32,address,address,address,uint256,uint256)") {
            if d.mode != "escrow"
                || who(log, 2)? != payer
                || who(log, 3)? != payee
                || <(Address, U256, U256)>::abi_decode(&data)? != (token, amount, fee)
            {
                return Err("escrow release mismatch".into());
            }
            let kind = if sender == payer
                && input == chain::input(d, "release").map_err(|_| "invalid release")?
            {
                "release"
            } else if sender == n.admin {
                "resolve-release"
            } else {
                return Err("unauthorized release".into());
            };
            ("succeeded", fee.to_string(), kind, "payment.succeeded")
        } else if sig == keccak256("Refunded(bytes32,address,address,address,uint256)") {
            if who(log, 2)? != payer
                || who(log, 3)? != payee
                || <(Address, U256)>::abi_decode(&data)? != (token, amount)
            {
                return Err("refund mismatch".into());
            }
            let kind = if sender == payee
                && input == chain::input(d, "refund").map_err(|_| "invalid refund")?
            {
                "refund"
            } else if d.mode == "escrow" && sender == n.admin {
                "resolve-refund"
            } else {
                return Err("unauthorized refund".into());
            };
            (
                "refunded",
                if d.mode == "direct" {
                    fee.to_string()
                } else {
                    "0".into()
                },
                kind,
                "refund.succeeded",
            )
        } else if sig == keccak256("Disputed(bytes32,address)") {
            if d.mode != "escrow"
                || !data.is_empty()
                || who(log, 2)? != sender
                || (sender != payer && sender != payee)
            {
                return Err("dispute mismatch".into());
            }
            ("disputed", "0".into(), "dispute", "escrow.disputed")
        } else {
            return Err("unknown event".into());
        };
    let expected =
        chain::input(d, kind).map_err(|e| format!("invalid issued parameters: {e:?}"))?;
    let funding = matches!(kind, "pay" | "deposit") || kind == "refund" && d.mode == "direct";
    if input != expected
        || U256::from_str(t["value"].as_str().ok_or("missing value")?)?
            != if funding && token.is_zero() {
                amount
            } else {
                U256::ZERO
            }
    {
        return Err("transaction input/value mismatch".into());
    }
    Ok((status, fee_amount, kind, event))
}
async fn apply(
    p: &Platform,
    n: &Network,
    mode: &str,
    tx: &mut sqlx::PgConnection,
    log: &Value,
) -> std::result::Result<(), Error> {
    let c = n.contract(mode).address;
    let sig = topic(log, 0)?;
    if ![
        "Paid(bytes32,address,address,address,uint256,uint256)",
        "Funded(bytes32,address,address,address,uint256)",
        "Released(bytes32,address,address,address,uint256,uint256)",
        "Refunded(bytes32,address,address,address,uint256)",
        "Disputed(bytes32,address)",
    ]
    .iter()
    .any(|s| keccak256(s) == sig)
    {
        return Ok(());
    }
    if log["removed"].as_bool() == Some(true) || Address::from_str(&text(log, "address")?)? != c {
        return Err("foreign/removed log".into());
    }
    let chain_id = format!("{:#x}", topic(log, 1)?);
    let d:Option<Payment>=sqlx::query_as("SELECT * FROM pay_merchant_intents WHERE chain_id=$1 AND contract_address=$2 AND on_chain_id=$3 FOR UPDATE").bind(n.chain_id as i64).bind(c.to_string().to_ascii_lowercase()).bind(chain_id).fetch_optional(&mut *tx).await?;
    let Some(d) = d else { return Ok(()) };
    let height = rpc::number(&log["blockNumber"])?;
    let block_hash = text(log, "blockHash")?;
    let hash = text(log, "transactionHash")?;
    if text(&n.block(height).await?, "hash")? != block_hash {
        return Err("noncanonical log".into());
    }
    let r = n.rpc("eth_getTransactionReceipt", json!([hash])).await?;
    if rpc::number(&r["status"])? != 1
        || rpc::number(&r["blockNumber"])? != height
        || text(&r, "blockHash")? != block_hash
        || text(&r, "transactionHash")? != hash
        || Address::from_str(&text(&r, "to")?)? != c
    {
        return Err("receipt mismatch".into());
    }
    if !r["logs"]
        .as_array()
        .ok_or("receipt logs missing")?
        .iter()
        .any(|l| {
            l["logIndex"] == log["logIndex"]
                && l["address"] == log["address"]
                && l["topics"] == log["topics"]
                && l["data"] == log["data"]
        })
    {
        return Err("event absent from receipt".into());
    }
    let t = n.rpc("eth_getTransactionByHash", json!([hash])).await?;
    if rpc::number(&t["chainId"])? != n.chain_id
        || Address::from_str(&text(&t, "to")?)? != c
        || text(&t, "hash")? != hash
        || text(&t, "blockHash")? != block_hash
    {
        return Err("transaction mismatch".into());
    }
    let (status, fee, kind, event) = match decode(n, &d, log, &t) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error=%e,intent_id=%d.id,"merchant evidence conflicts with checkout");
            let invalid:Payment=sqlx::query_as("UPDATE pay_merchant_intents SET status='verification_required',verification_error='quote_evidence_mismatch',revision=revision+1,updated_at=now() WHERE id=$1 RETURNING *").bind(&d.id).fetch_one(&mut *tx).await?;
            emit(tx, &invalid, "payment.verification_required")
                .await
                .map_err(|e| format!("event storage: {e:?}"))?;
            return Ok(());
        }
    };
    let index = rpc::number(&log["logIndex"])? as i64;
    let already:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pay_merchant_chain_events WHERE chain_id=$1 AND contract_address=$2 AND block_hash=$3 AND tx_hash=$4 AND log_index=$5 AND canonical)").bind(n.chain_id as i64).bind(c.to_string().to_ascii_lowercase()).bind(&block_hash).bind(&hash).bind(index).fetch_one(&mut *tx).await?;
    if already {
        return Ok(());
    }
    sqlx::query("INSERT INTO pay_merchant_chain_events(chain_id,contract_address,block_number,block_hash,tx_hash,log_index,intent_id,status,fee_amount,payload) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT(chain_id,contract_address,block_hash,tx_hash,log_index) DO UPDATE SET canonical=true")
        .bind(n.chain_id as i64).bind(c.to_string().to_ascii_lowercase()).bind(height as i64).bind(&block_hash).bind(&hash).bind(index).bind(&d.id).bind(status).bind(&fee).bind(log).execute(&mut *tx).await?;
    let updated:Payment=sqlx::query_as("UPDATE pay_merchant_intents SET status=$2,fee_amount=$3,tx_hash=$4,verified_block=$5,verified_block_hash=$6,revision=revision+1,updated_at=now() WHERE id=$1 RETURNING *").bind(&d.id).bind(status).bind(fee).bind(&hash).bind(height as i64).bind(&block_hash).fetch_one(&mut *tx).await?;
    sqlx::query("UPDATE pay_merchant_operations SET status='confirmed',tx_hash=$4,verified_block=$5,verified_block_hash=$6 WHERE intent_id=$1 AND kind=$2 AND actor=$3 AND (tx_hash=$4 OR tx_hash IS NULL)").bind(&d.id).bind(kind).bind(text(&t,"from")?).bind(&hash).bind(height as i64).bind(&block_hash).execute(&mut *tx).await?;
    emit(tx, &updated, event)
        .await
        .map_err(|e| format!("event storage: {e:?}"))?;
    let _ = p;
    Ok(())
}
pub async fn tick(p: &Platform, n: &Network, mode: &str) -> std::result::Result<(), Error> {
    n.validate(mode).await?;
    let c = n.contract(mode);
    let scope = c.address.to_string().to_ascii_lowercase();
    let mut tx = p.db.begin().await?;
    if !sqlx::query_scalar::<_, bool>("SELECT pg_try_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("merchant-scanner:{}:{scope}", n.chain_id))
        .fetch_one(&mut *tx)
        .await?
    {
        return Ok(());
    }
    sqlx::query("INSERT INTO pay_merchant_checkpoints(chain_id,contract_address,next_block) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(n.chain_id as i64).bind(&scope).bind(c.deployment_block as i64).execute(&mut *tx).await?;
    let checkpoint=sqlx::query("SELECT * FROM pay_merchant_checkpoints WHERE chain_id=$1 AND contract_address=$2 FOR UPDATE").bind(n.chain_id as i64).bind(&scope).fetch_one(&mut *tx).await?;
    let next = checkpoint.get::<i64, _>("next_block") as u64;
    let head = rpc::number(&n.rpc("eth_blockNumber", json!([])).await?)?;
    if let Some(last) = checkpoint.get::<Option<String>, _>("last_block_hash") {
        let changed = head < next.saturating_sub(1)
            || text(&n.block(next.saturating_sub(1)).await?, "hash")? != last;
        if changed {
            let invalid:Vec<Payment>=sqlx::query_as("UPDATE pay_merchant_intents SET status='verification_required',revision=revision+1,verified_block=NULL,verified_block_hash=NULL,verification_error=CASE WHEN deposit_address IS NOT NULL THEN NULL ELSE verification_error END,fee_amount='0',updated_at=now() WHERE chain_id=$1 AND contract_address=$2 AND (verified_block IS NOT NULL OR status='expired' OR deposit_address IS NOT NULL AND status='verification_required') RETURNING *").bind(n.chain_id as i64).bind(&scope).fetch_all(&mut *tx).await?;
            for d in invalid {
                emit(&mut tx, &d, "payment.verification_required")
                    .await
                    .map_err(|e| format!("event storage: {e:?}"))?;
            }
            sqlx::query("UPDATE pay_merchant_chain_events SET canonical=false WHERE chain_id=$1 AND contract_address=$2").bind(n.chain_id as i64).bind(&scope).execute(&mut *tx).await?;
            sqlx::query("UPDATE pay_merchant_operations SET status='verification_required',verified_block=NULL,verified_block_hash=NULL WHERE status='confirmed' AND intent_id IN(SELECT id FROM pay_merchant_intents WHERE chain_id=$1 AND contract_address=$2)").bind(n.chain_id as i64).bind(&scope).execute(&mut *tx).await?;
            sqlx::query("UPDATE pay_merchant_checkpoints SET next_block=$3,last_block_hash=NULL,last_block_time=NULL,healthy=false,checked_at=now() WHERE chain_id=$1 AND contract_address=$2").bind(n.chain_id as i64).bind(&scope).bind(c.deployment_block as i64).execute(&mut *tx).await?;
            tx.commit().await?;
            return Ok(());
        }
    }
    if head + 1 < n.confirmations {
        return Err("waiting for confirmations".into());
    }
    let finalized = head + 1 - n.confirmations;
    let end = if next <= finalized {
        next.saturating_add(rpc::LOG_SCAN_BLOCKS - 1).min(finalized)
    } else {
        finalized
    };
    let block = n.block(end).await?;
    let end_hash = text(&block, "hash")?;
    if next <= end {
        let logs = if mode == "qr" {
            json!(super::qr::logs(n, next, end).await?)
        } else {
            n.rpc("eth_getLogs",json!([{"address":c.address,"fromBlock":format!("0x{next:x}"),"toBlock":format!("0x{end:x}")}])).await?
        };
        let mut logs = logs.as_array().ok_or("logs missing")?.clone();
        logs.sort_by_key(|l| {
            (
                rpc::number(&l["blockNumber"]).unwrap_or(u64::MAX),
                rpc::number(&l["logIndex"]).unwrap_or(u64::MAX),
            )
        });
        for log in logs {
            let height = rpc::number(&log["blockNumber"])?;
            if height < next || height > end {
                return Err("log out of range".into());
            }
            if mode == "qr" {
                super::qr::apply(p, n, &mut tx, &log).await?;
            } else {
                apply(p, n, mode, &mut tx, &log).await?;
            }
        }
    }
    if text(&n.block(end).await?, "hash")? != end_hash {
        return Err("reorg during merchant scan".into());
    }
    let time = chrono::DateTime::from_timestamp(rpc::number(&block["timestamp"])? as i64, 0)
        .ok_or("invalid block timestamp")?;
    sqlx::query("UPDATE pay_merchant_checkpoints SET next_block=$3,last_block_hash=$4,last_block_time=$5,healthy=$6,checked_at=now() WHERE chain_id=$1 AND contract_address=$2").bind(n.chain_id as i64).bind(&scope).bind((end+1) as i64).bind(end_hash).bind(time).bind(end==finalized).execute(&mut *tx).await?;
    if end == finalized {
        sqlx::query("UPDATE pay_merchant_intents SET status='awaiting_payment',tx_hash=NULL WHERE chain_id=$1 AND contract_address=$2 AND status='verification_required' AND verified_block IS NULL AND verification_error IS NULL").bind(n.chain_id as i64).bind(&scope).execute(&mut *tx).await?;
        let expired:Vec<Payment>=sqlx::query_as("UPDATE pay_merchant_intents SET status='expired',revision=revision+1,updated_at=now() WHERE chain_id=$1 AND contract_address=$2 AND status='awaiting_payment' AND expires_at<$3 RETURNING *").bind(n.chain_id as i64).bind(&scope).bind(time).fetch_all(&mut *tx).await?;
        for d in expired {
            emit(&mut tx, &d, "checkout.expired")
                .await
                .map_err(|e| format!("event storage: {e:?}"))?;
        }
    }
    tx.commit().await?;
    if end == finalized {
        let ops=sqlx::query("SELECT o.id,o.tx_hash FROM pay_merchant_operations o JOIN pay_merchant_intents d ON d.id=o.intent_id WHERE d.chain_id=$1 AND d.contract_address=$2 AND o.status IN ('pending','verification_required') AND o.tx_hash IS NOT NULL LIMIT 100").bind(n.chain_id as i64).bind(&scope).fetch_all(&p.db).await?;
        for op in ops {
            let r = n
                .rpc(
                    "eth_getTransactionReceipt",
                    json!([op.get::<String, _>("tx_hash")]),
                )
                .await?;
            if r.is_null() {
                continue;
            }
            let height = rpc::number(&r["blockNumber"])?;
            if height <= end && text(&n.block(height).await?, "hash")? == text(&r, "blockHash")? {
                sqlx::query("UPDATE pay_merchant_operations SET status='failed' WHERE id=$1 AND status IN ('pending','verification_required')").bind(op.get::<String,_>("id")).execute(&p.db).await?;
            }
        }
    }
    Ok(())
}
pub async fn checked_tick(p: &Platform, n: &Network, mode: &str) -> std::result::Result<(), Error> {
    let result = async {
        tick(p, n, mode).await?;
        if mode == "qr" {
            Ok(())
        } else {
            super::controls::reconcile(p, n, mode).await
        }
    }
    .await;
    if let Err(error) = &result {
        tracing::error!(%error,environment=%n.environment,mode,"merchant chain reconciliation unavailable");
        sqlx::query("UPDATE pay_merchant_checkpoints SET healthy=false,checked_at=now() WHERE chain_id=$1 AND contract_address=$2").bind(n.chain_id as i64).bind(n.contract(mode).address.to_string().to_ascii_lowercase()).execute(&p.db).await?;
    }
    result
}
pub fn spawn(p: Platform) {
    for n in p.chains.iter() {
        for mode in ["direct", "escrow"]
            .into_iter()
            .chain(n.qr.is_some().then_some("qr"))
        {
            let p = p.clone();
            let n = n.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(3));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    interval.tick().await;
                    let _ = checked_tick(&p, &n, mode).await;
                }
            });
        }
    }
}
