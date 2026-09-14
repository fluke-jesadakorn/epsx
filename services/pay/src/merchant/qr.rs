//! Receipt-backed ordinary ERC-20 transfers to unique, merchant-owned receivers.
use super::{
    api::Payment,
    chain::{self, Network},
    emit, Platform,
};
use crate::native_chain::{self as rpc, Error};
use alloy::{
    primitives::{keccak256, Address, U256},
    sol_types::SolValue,
};
use serde_json::{json, Value};
use std::str::FromStr;

fn address_topic(log: &Value, index: usize) -> std::result::Result<Address, Error> {
    let raw = rpc::bytes(&log["topics"][index])?;
    if raw.len() != 32 || raw[..12].iter().any(|b| *b != 0) {
        return Err("invalid address topic".into());
    }
    Ok(Address::from_slice(&raw[12..]))
}
fn text<'a>(v: &'a Value, key: &str) -> std::result::Result<&'a str, Error> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("missing {key}").into())
}
pub async fn logs(n: &Network, start: u64, end: u64) -> std::result::Result<Vec<Value>, Error> {
    let tokens: Vec<_> = n
        .tokens
        .values()
        .filter(|t| chain::address(&t.address).is_ok_and(|a| !a.is_zero()))
        .map(|t| &t.address)
        .collect();
    let mut logs = n.rpc("eth_getLogs",json!([{"address":tokens,"fromBlock":format!("0x{start:x}"),"toBlock":format!("0x{end:x}"),"topics":[keccak256("Transfer(address,address,uint256)")]}])).await?
        .as_array().ok_or("invalid token logs")?.clone();
    logs.extend(n.rpc("eth_getLogs",json!([{"address":n.contract("qr").address,"fromBlock":format!("0x{start:x}"),"toBlock":format!("0x{end:x}"),"topics":[[keccak256("QRRefunded(address,address,address,uint256)"),keccak256("QRCollected(address)")]]}])).await?
        .as_array().ok_or("invalid refund logs")?.clone());
    Ok(logs)
}
pub async fn apply(
    p: &Platform,
    n: &Network,
    tx: &mut sqlx::PgConnection,
    log: &Value,
) -> std::result::Result<(), Error> {
    let transfer = log["topics"][0] == json!(keccak256("Transfer(address,address,uint256)"));
    let receiver = address_topic(log, if transfer { 2 } else { 1 })?
        .to_string()
        .to_ascii_lowercase();
    let scope = n.contract("qr").address.to_string().to_ascii_lowercase();
    let d:Option<Payment>=sqlx::query_as("SELECT * FROM pay_merchant_intents WHERE chain_id=$1 AND contract_address=$2 AND deposit_address=$3 FOR UPDATE")
        .bind(n.chain_id as i64).bind(&scope).bind(receiver).fetch_optional(&mut *tx).await?;
    let Some(d) = d else { return Ok(()) };
    let emitter = Address::from_str(text(log, "address")?)?;
    if transfer && emitter != Address::from_str(&d.token_address)? {
        return Ok(());
    }
    if !transfer && emitter != n.contract("qr").address {
        return Err("foreign refund".into());
    }
    if log["removed"] == true {
        return Err("removed QR log".into());
    }
    let height = rpc::number(&log["blockNumber"])?;
    let block = n.block(height).await?;
    if block["hash"] != log["blockHash"] {
        return Err("noncanonical QR block".into());
    }
    let hash = text(log, "transactionHash")?;
    let receipt = n.rpc("eth_getTransactionReceipt", json!([hash])).await?;
    if rpc::number(&receipt["status"])? != 1
        || receipt["blockHash"] != block["hash"]
        || receipt["blockNumber"] != log["blockNumber"]
        || receipt["transactionHash"] != hash
        || !receipt["logs"]
            .as_array()
            .ok_or("missing receipt logs")?
            .iter()
            .any(|l| {
                l["address"] == log["address"]
                    && l["logIndex"] == log["logIndex"]
                    && l["topics"] == log["topics"]
                    && l["data"] == log["data"]
            })
    {
        return Err("QR receipt mismatch".into());
    }
    if log["topics"][0] == json!(keccak256("QRCollected(address)")) {
        // Anyone can emit a collection for an empty receiver. Only a receipt
        // with a token transfer to this invoice's merchant proves settlement.
        let moved = receipt["logs"]
            .as_array()
            .ok_or("missing logs")?
            .iter()
            .any(|l| {
                l["topics"][0] == json!(keccak256("Transfer(address,address,uint256)"))
                    && l["address"]
                        .as_str()
                        .is_some_and(|a| a.eq_ignore_ascii_case(&d.token_address))
                    && address_topic(l, 1).ok()
                        == d.deposit_address
                            .as_deref()
                            .and_then(|a| Address::from_str(a).ok())
                    && address_topic(l, 2).ok() == Address::from_str(&d.payee).ok()
                    && l["data"]
                        .as_str()
                        .and_then(|v| U256::from_str(v).ok())
                        .is_some_and(|v| !v.is_zero())
            });
        if !moved {
            return Ok(());
        }
        sqlx::query("INSERT INTO pay_merchant_chain_events(chain_id,contract_address,block_number,block_hash,tx_hash,log_index,intent_id,status,fee_amount,payload) VALUES($1,$2,$3,$4,$5,$6,$7,'collected','0',$8) ON CONFLICT(chain_id,contract_address,block_hash,tx_hash,log_index) DO UPDATE SET canonical=true")
            .bind(n.chain_id as i64).bind(&scope).bind(height as i64).bind(text(log,"blockHash")?).bind(hash).bind(rpc::number(&log["logIndex"])? as i64).bind(&d.id).bind(log).execute(&mut *tx).await?;
        sqlx::query("UPDATE pay_merchant_operations SET status='confirmed',tx_hash=$2,verified_block=$3,verified_block_hash=$4 WHERE intent_id=$1 AND kind='collect' AND tx_hash=$2")
            .bind(&d.id).bind(hash).bind(height as i64).bind(text(log,"blockHash")?).execute(&mut *tx).await?;
        return Ok(());
    }
    let index = rpc::number(&log["logIndex"])? as i64;
    let already:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pay_merchant_chain_events WHERE chain_id=$1 AND contract_address=$2 AND block_hash=$3 AND tx_hash=$4 AND log_index=$5 AND canonical)")
        .bind(n.chain_id as i64).bind(&scope).bind(text(log,"blockHash")?).bind(hash).bind(index).fetch_one(&mut *tx).await?;
    if already {
        return Ok(());
    }
    let payer;
    let status;
    let error;
    if transfer {
        if d.verified_block.is_some() {
            return Ok(());
        }
        payer = address_topic(log, 1)?.to_string().to_ascii_lowercase();
        if chain::address(&payer).map_err(|_| "bad payer")?.is_zero() || payer == d.payee {
            return Ok(());
        }
        let amount = U256::abi_decode(&rpc::bytes(&log["data"])?)?;
        let at = rpc::number(&block["timestamp"])? as i64;
        error = if at > d.expires_at.timestamp() {
            Some("qr_payment_late")
        } else if amount != rpc::amount(&d.amount)? {
            Some("qr_amount_mismatch")
        } else {
            None
        };
        status = if error.is_some() {
            "verification_required"
        } else {
            "succeeded"
        };
    } else {
        if d.status != "succeeded" {
            return Ok(());
        }
        payer = address_topic(log, 2)?.to_string().to_ascii_lowercase();
        let (token, amount) = <(Address, U256)>::abi_decode(&rpc::bytes(&log["data"])?)?;
        let transaction = n.rpc("eth_getTransactionByHash", json!([hash])).await?;
        if d.payer.as_deref() != Some(&payer)
            || token != Address::from_str(&d.token_address)?
            || amount != rpc::amount(&d.amount)?
            || Address::from_str(text(&transaction, "from")?)? != Address::from_str(&d.payee)?
            || Address::from_str(text(&transaction, "to")?)? != n.contract("qr").address
            || rpc::bytes(&transaction["input"])?
                != chain::input(&d, "refund").map_err(|_| "invalid refund input")?
        {
            tracing::warn!(intent_id=%d.id,"Ignoring refund to a recipient other than this invoice's payer");
            return Ok(());
        }
        error = None;
        status = "refunded";
    }
    // Non-matching deposits are retained as review evidence, never as paid proof.
    let fee = chain::fee(rpc::amount(&d.amount)?, 50).to_string();
    sqlx::query("INSERT INTO pay_merchant_chain_events(chain_id,contract_address,block_number,block_hash,tx_hash,log_index,intent_id,status,fee_amount,payload) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT(chain_id,contract_address,block_hash,tx_hash,log_index) DO UPDATE SET canonical=true")
        .bind(n.chain_id as i64).bind(&scope).bind(height as i64).bind(text(log,"blockHash")?).bind(hash).bind(index).bind(&d.id).bind(status).bind(&fee).bind(log).execute(&mut *tx).await?;
    let updated:Payment=sqlx::query_as("UPDATE pay_merchant_intents SET status=$2,payer=$3,tx_hash=$4,verified_block=$5,verified_block_hash=$6,verification_error=$7,fee_amount=$8,revision=revision+1,updated_at=now() WHERE id=$1 RETURNING *")
        .bind(&d.id).bind(status).bind(&payer).bind(hash).bind(error.is_none().then_some(height as i64)).bind(error.is_none().then_some(text(log,"blockHash")?)).bind(error).bind(if error.is_none(){&fee}else{"0"}).fetch_one(&mut *tx).await?;
    if !transfer {
        sqlx::query("UPDATE pay_merchant_operations SET status='confirmed',tx_hash=$2,verified_block=$3,verified_block_hash=$4 WHERE intent_id=$1 AND kind='refund'")
            .bind(&d.id).bind(hash).bind(height as i64).bind(text(log,"blockHash")?).execute(&mut *tx).await?;
    }
    emit(
        tx,
        &updated,
        if error.is_some() {
            "payment.verification_required"
        } else if transfer {
            "payment.succeeded"
        } else {
            "refund.succeeded"
        },
    )
    .await
    .map_err(|e| format!("QR event: {e:?}"))?;
    let _ = p;
    Ok(())
}
