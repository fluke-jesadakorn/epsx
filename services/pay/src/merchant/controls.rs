//! Wallet-signed emergency controls. Pausing never prevents settlement of existing escrows.
use super::{api, auth::Actor, bad, chain, conflict, missing, Platform, Result};
use alloy::{
    primitives::B256,
    sol,
    sol_types::{SolCall, SolValue},
};
use axum::http::HeaderMap;
use serde_json::{json, Value};
use sqlx::Row;
use std::str::FromStr;
sol! { function setPaused(bool value); }

pub async fn handle(
    p: &Platform,
    a: &Actor,
    h: &HeaderMap,
    method: &str,
    path: &[&str],
    b: &Value,
) -> Result<Value> {
    let n = api::network(p, &a.environment)?;
    if !a.admin || chain::address(&a.wallet)? != n.admin {
        return Err(missing());
    }
    match (method, path) {
        ("POST", ["contracts", mode, "pause"]) if ["direct", "escrow"].contains(mode) => {
            api::healthy(p, n, mode).await?;
            let paused = b["paused"]
                .as_bool()
                .ok_or_else(|| bad("paused_boolean_required"))?;
            let key = api::key(h)?;
            let scope = format!("control:{}:{}:{mode}", a.environment, a.wallet);
            let request = json!([n.chain_id, n.contract(mode).address, paused]);
            let mut tx = p.db.begin().await?;
            if let Some(v) = api::start(&mut tx, &scope, &key, &request).await? {
                return Ok(v);
            }
            let params = json!({"chainId":format!("0x{:x}",n.chain_id),"from":a.wallet,"to":n.contract(mode).address,"data":format!("0x{}",hex::encode(setPausedCall{value:paused}.abi_encode())),"value":"0x0"});
            let id = super::id("mctl");
            sqlx::query("INSERT INTO pay_merchant_contract_controls(id,environment,mode,chain_id,contract_address,actor,paused,transaction_parameters) VALUES($1,$2,$3,$4,$5,$6,$7,$8)").bind(&id).bind(&a.environment).bind(mode).bind(n.chain_id as i64).bind(n.contract(mode).address.to_string().to_ascii_lowercase()).bind(&a.wallet).bind(paused).bind(&params).execute(&mut *tx).await?;
            let v = json!({"id":id,"status":"awaiting_signature","transaction_parameters":params});
            api::end(&mut tx, &scope, &key, &request, &v).await?;
            tx.commit().await?;
            Ok(v)
        }
        ("GET", ["contracts"]) => {
            let mut items = vec![];
            for mode in ["direct", "escrow"] {
                let paused = bool::abi_decode(
                    &n.call(n.contract(mode).address, chain::pausedCall {}.abi_encode())
                        .await?,
                )
                .map_err(|_| bad("invalid_pause_response"))?;
                items.push(json!({"mode":mode,"paused":paused,"address":n.contract(mode).address,"chain_id":n.chain_id}));
            }
            Ok(json!({"items":items}))
        }
        ("GET", ["contract-controls", id]) | ("POST", ["contract-controls", id, "confirm"]) => {
            let mut tx = p.db.begin().await?;
            let row = sqlx::query("SELECT * FROM pay_merchant_contract_controls WHERE id=$1 AND environment=$2 AND actor=$3 FOR UPDATE").bind(id).bind(&a.environment).bind(&a.wallet).fetch_optional(&mut *tx).await?.ok_or_else(missing)?;
            if method == "POST" {
                let hash = b["tx_hash"]
                    .as_str()
                    .ok_or_else(|| bad("transaction_hash_required"))?;
                B256::from_str(hash).map_err(|_| bad("invalid_transaction_hash"))?;
                if row
                    .get::<Option<String>, _>("tx_hash")
                    .is_some_and(|old| !old.eq_ignore_ascii_case(hash))
                {
                    return Err(conflict());
                }
                sqlx::query("UPDATE pay_merchant_contract_controls SET tx_hash=$2,status=CASE WHEN status='awaiting_signature' THEN 'pending' ELSE status END WHERE id=$1").bind(id).bind(hash.to_ascii_lowercase()).execute(&mut *tx).await?;
            }
            if row.get::<String, _>("status") == "confirmed" {
                api::healthy(p, n, &row.get::<String, _>("mode")).await?;
            }
            let v: Value = sqlx::query_scalar(
                "SELECT to_jsonb(c) FROM pay_merchant_contract_controls c WHERE id=$1",
            )
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(v)
        }
        _ => Err(missing()),
    }
}

pub async fn reconcile(
    p: &Platform,
    n: &chain::Network,
    mode: &str,
) -> std::result::Result<(), crate::native_chain::Error> {
    use crate::native_chain::{bytes, number};
    let rows = sqlx::query("SELECT * FROM pay_merchant_contract_controls WHERE chain_id=$1 AND contract_address=$2 AND tx_hash IS NOT NULL AND status IN ('pending','verification_required','confirmed')").bind(n.chain_id as i64).bind(n.contract(mode).address.to_string().to_ascii_lowercase()).fetch_all(&p.db).await?;
    let head = number(&n.rpc("eth_blockNumber", json!([])).await?)?;
    for row in rows {
        let id: String = row.get("id");
        let hash: String = row.get("tx_hash");
        let r = n.rpc("eth_getTransactionReceipt", json!([hash])).await?;
        if r.is_null() {
            sqlx::query("UPDATE pay_merchant_contract_controls SET status='verification_required',verified_block=NULL,verified_block_hash=NULL WHERE id=$1 AND status='confirmed'").bind(&id).execute(&p.db).await?;
            continue;
        }
        let height = number(&r["blockNumber"])?;
        if head.saturating_add(1) < height + n.confirmations {
            continue;
        }
        let block = n.block(height).await?;
        let t = n.rpc("eth_getTransactionByHash", json!([hash])).await?;
        let params: Value = row.get("transaction_parameters");
        let canonical = r["blockHash"] == block["hash"] && t["blockHash"] == block["hash"];
        let valid = canonical
            && number(&r["status"])? == 1
            && number(&t["chainId"])? == n.chain_id
            && t["to"]
                .as_str()
                .is_some_and(|v| v.eq_ignore_ascii_case(params["to"].as_str().unwrap_or("")))
            && t["from"]
                .as_str()
                .is_some_and(|v| v.eq_ignore_ascii_case(params["from"].as_str().unwrap_or("")))
            && number(&t["value"])? == 0
            && bytes(&t["input"])? == bytes(&params["data"])?
            && r["transactionHash"]
                .as_str()
                .is_some_and(|v| v.eq_ignore_ascii_case(&hash));
        let status = if !canonical {
            "verification_required"
        } else if valid {
            "confirmed"
        } else {
            "failed"
        };
        sqlx::query("UPDATE pay_merchant_contract_controls SET status=$2,verified_block=$3,verified_block_hash=$4 WHERE id=$1").bind(id).bind(status).bind(valid.then_some(height as i64)).bind(if valid {block["hash"].as_str()}else{None}).execute(&p.db).await?;
    }
    Ok(())
}
