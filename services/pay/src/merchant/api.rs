use super::{
    auth::{self, Actor},
    bad, capability, chain, conflict, hash, id, missing, Error, Platform, Result,
};
use alloy::{
    primitives::B256,
    sol_types::{SolCall, SolValue},
};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, PgConnection, Row};
use std::str::FromStr;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: String,
    pub merchant_id: String,
    pub environment: String,
    pub order_reference: String,
    pub link_id: Option<String>,
    pub mode: String,
    pub chain_id: i64,
    pub contract_address: String,
    pub contract_version: i32,
    pub payer: Option<String>,
    pub payee: String,
    pub token: String,
    pub token_address: String,
    pub token_decimals: i32,
    pub amount: String,
    pub fee_bps: i32,
    pub fee_amount: String,
    pub description: Option<String>,
    pub metadata: Value,
    pub salt: String,
    pub on_chain_id: Option<String>,
    pub checkout_id: String,
    pub capability_hash: String,
    pub status: String,
    pub revision: i64,
    pub tx_hash: Option<String>,
    pub verified_block: Option<i64>,
    pub verified_block_hash: Option<String>,
    pub verification_error: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deposit_address: Option<String>,
    pub checkout_snapshot: Value,
}
pub fn public_payment(d: &Payment, guest: bool) -> Value {
    let mut v = json!({"id":d.id,"merchant_id":d.merchant_id,"environment":d.environment,"order_reference":d.order_reference,
        "mode":d.mode,"chain_id":d.chain_id,"contract_address":d.contract_address,"contract_version":d.contract_version,
        "payer":d.payer,"payee":d.payee,"token":d.token,"token_address":d.token_address,"token_decimals":d.token_decimals,
        "amount":d.amount,"fee_bps":d.fee_bps,"fee_amount":d.fee_amount,"description":d.description,
        "status":d.status,"revision":d.revision,"tx_hash":d.tx_hash,"verified_block":d.verified_block,"verified_block_hash":d.verified_block_hash,
        "expires_at":d.expires_at,"created_at":d.created_at,"checkout_id":d.checkout_id,"on_chain_id":d.on_chain_id});
    v["checkout_snapshot"] = d.checkout_snapshot.clone();
    let actions: Vec<&str> = if guest {
        match (d.mode.as_str(), d.status.as_str()) {
            ("direct", "awaiting_payment")
                if d.expires_at > Utc::now() && d.deposit_address.is_none() =>
            {
                vec!["pay"]
            }
            ("escrow", "awaiting_payment") if d.expires_at > Utc::now() => vec!["deposit"],
            ("escrow", "funded") => vec!["release", "dispute"],
            _ => vec![],
        }
    } else {
        match (d.mode.as_str(), d.status.as_str()) {
            ("direct", "succeeded") => vec!["refund"],
            ("escrow", "funded") => vec!["refund", "dispute"],
            ("escrow", "disputed") => vec!["refund"],
            _ => vec![],
        }
    };
    let mut actions = actions;
    if !guest && d.deposit_address.is_some() {
        actions.push("collect");
    }
    v["available_actions"] = json!(actions);
    if let Some(address) = &d.deposit_address {
        let uri = format!(
            "ethereum:{}@{}/transfer?address={address}&uint256={}",
            d.token_address, d.chain_id, d.amount
        );
        v["payment_method"] = json!("transfer");
        v["deposit_address"] = json!(address);
        v["payment_uri"] = json!(uri);
        if let Ok(code) = qrcode::QrCode::new(uri.as_bytes()) {
            v["qr_svg"] = json!(code
                .render::<qrcode::render::svg::Color>()
                .min_dimensions(240, 240)
                .build());
        }
    }
    if !guest {
        v["metadata"] = d.metadata.clone();
    } else {
        v.as_object_mut().unwrap().remove("order_reference");
    }
    v
}
pub fn network<'a>(p: &'a Platform, environment: &str) -> Result<&'a chain::Network> {
    p.chains
        .iter()
        .find(|n| n.environment == environment)
        .ok_or(Error(
            StatusCode::SERVICE_UNAVAILABLE,
            "environment_not_configured",
        ))
}
pub async fn healthy(p: &Platform, n: &chain::Network, mode: &str) -> Result<()> {
    let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pay_merchant_checkpoints WHERE chain_id=$1 AND contract_address=$2 AND healthy AND checked_at>now()-interval '60 seconds')")
        .bind(n.chain_id as i64).bind(n.contract(mode).address.to_string().to_ascii_lowercase()).fetch_one(&p.db).await?;
    if !valid {
        return Err(Error(
            StatusCode::SERVICE_UNAVAILABLE,
            "chain_verification_unavailable",
        ));
    }
    Ok(())
}
pub fn key(h: &HeaderMap) -> Result<String> {
    h.get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
        })
        .map(str::to_owned)
        .ok_or_else(|| bad("idempotency_key_required"))
}
pub async fn start(
    tx: &mut PgConnection,
    scope: &str,
    key: &str,
    request: &Value,
) -> Result<Option<Value>> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("merchant:{scope}:{key}"))
        .execute(&mut *tx)
        .await?;
    let row = sqlx::query(
        "SELECT request_hash,response FROM pay_merchant_requests WHERE scope=$1 AND key=$2",
    )
    .bind(scope)
    .bind(key)
    .fetch_optional(tx)
    .await?;
    if let Some(row) = row {
        if row.get::<String, _>("request_hash") != hash(request.to_string().as_bytes()) {
            return Err(conflict());
        }
        Ok(Some(row.get("response")))
    } else {
        Ok(None)
    }
}
pub async fn end(
    tx: &mut PgConnection,
    scope: &str,
    key: &str,
    request: &Value,
    response: &Value,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO pay_merchant_requests(scope,key,request_hash,response) VALUES($1,$2,$3,$4)",
    )
    .bind(scope)
    .bind(key)
    .bind(hash(request.to_string().as_bytes()))
    .bind(response)
    .execute(tx)
    .await?;
    Ok(())
}
pub async fn load(tx: &mut PgConnection, id: &str) -> Result<Payment> {
    let mut d: Payment = sqlx::query_as(
        "SELECT * FROM pay_merchant_intents WHERE id=$1 OR checkout_id=$1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(missing)?;
    verify_projection(tx, &mut d).await?;
    Ok(d)
}
pub async fn verify_projection(tx: &mut PgConnection, d: &mut Payment) -> Result<()> {
    if !["succeeded", "funded", "disputed", "refunded"].contains(&d.status.as_str()) {
        return Ok(());
    }
    let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pay_merchant_chain_events e WHERE e.intent_id=$1 AND e.canonical AND e.tx_hash=$2 AND e.block_number=$3 AND e.block_hash=$4 AND e.status=$5 AND e.fee_amount=$6)")
        .bind(&d.id).bind(&d.tx_hash).bind(d.verified_block).bind(&d.verified_block_hash).bind(&d.status).bind(&d.fee_amount).fetch_one(&mut *tx).await?;
    if !valid {
        d.status = "verification_required".into();
    } else {
        let healthy:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pay_merchant_checkpoints WHERE chain_id=$1 AND contract_address=$2 AND healthy AND checked_at>now()-interval '60 seconds')").bind(d.chain_id).bind(&d.contract_address).fetch_one(&mut *tx).await?;
        if !healthy {
            return Err(Error(
                StatusCode::SERVICE_UNAVAILABLE,
                "chain_verification_unavailable",
            ));
        }
    }
    Ok(())
}
fn owned(a: &Actor, d: &Payment) -> Result<()> {
    if a.merchant_id == d.merchant_id && a.environment == d.environment {
        Ok(())
    } else {
        Err(missing())
    }
}
fn guest(h: &HeaderMap, d: &Payment) -> Result<()> {
    let cap = h
        .get("x-pay-checkout-token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(missing)?;
    use subtle::ConstantTimeEq;
    if bool::from(
        hash(cap.as_bytes())
            .as_bytes()
            .ct_eq(d.capability_hash.as_bytes()),
    ) {
        Ok(())
    } else {
        Err(missing())
    }
}
fn public_url(path: &str) -> String {
    format!(
        "{}{}",
        std::env::var("PAY_FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3002".into())
            .trim_end_matches('/'),
        path
    )
}
pub(super) fn checkout_response(p: &Platform, mut value: Value) -> Value {
    if let Some(cs) = value
        .get("checkout_id")
        .and_then(Value::as_str)
        .map(str::to_owned)
    {
        value["pay_url"] = json!(public_url(&format!(
            "/checkout/{cs}#token={}",
            capability(p, &cs)
        )));
    }
    value
}
pub(super) fn text<'a>(b: &'a Value, key: &str, max: usize) -> Result<&'a str> {
    b.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && s.len() <= max)
        .ok_or_else(|| bad("invalid_request"))
}
fn terms(b: &Value) -> Result<(&str, &str, &str)> {
    let mode = text(b, "mode", 16)?;
    if !["direct", "escrow"].contains(&mode) {
        return Err(bad("invalid_mode"));
    }
    let token = text(b, "token", 8)?;
    let amount = text(b, "amount", 78)?;
    chain::amount(amount)?;
    Ok((mode, token, amount))
}
pub(super) fn expiration(b: &Value, default: i64) -> Result<DateTime<Utc>> {
    let seconds = match b.get("expires_in") {
        None => default,
        Some(v) => v.as_i64().ok_or_else(|| bad("invalid_expiration"))?,
    };
    if !(60..=2592000).contains(&seconds) {
        return Err(bad("invalid_expiration"));
    }
    Ok(Utc::now() + chrono::Duration::seconds(seconds))
}
pub(super) async fn insert(
    p: &Platform,
    tx: &mut PgConnection,
    a: &Actor,
    b: &Value,
    link: Option<&str>,
    expires: DateTime<Utc>,
) -> Result<Payment> {
    let (mode, symbol, amount) = terms(b)?;
    let n = network(p, &a.environment)?;
    let token = n
        .tokens
        .get(symbol)
        .ok_or_else(|| bad("unsupported_token"))?;
    let payee = chain::address(&a.wallet)?;
    if payee.is_zero() || payee == n.contract(mode).address {
        return Err(bad("invalid_recipient"));
    }
    let description = b.get("description").and_then(Value::as_str);
    if description.is_some_and(|v| v.len() > 2000) {
        return Err(bad("description_too_long"));
    }
    let metadata = b.get("metadata").cloned().unwrap_or(json!({}));
    if !metadata.is_object() || metadata.to_string().len() > 4096 {
        return Err(bad("invalid_metadata"));
    }
    let order = b
        .get("order_reference")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| id("order"));
    if order.is_empty() || order.len() > 128 {
        return Err(bad("invalid_order_reference"));
    }
    let cs = id("cs");
    let salt = format!("{:#x}", alloy::primitives::keccak256(id("salt")));
    let mut d:Payment=sqlx::query_as("INSERT INTO pay_merchant_intents(id,merchant_id,environment,order_reference,link_id,mode,chain_id,contract_address,contract_version,payee,token,token_address,token_decimals,amount,fee_bps,description,metadata,salt,checkout_id,capability_hash,expires_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21) RETURNING *")
        .bind(id("pi")).bind(&a.merchant_id).bind(&a.environment).bind(order).bind(link).bind(mode).bind(n.chain_id as i64).bind(n.contract(mode).address.to_string().to_ascii_lowercase()).bind(if mode=="escrow"{2}else{1}).bind(&a.wallet).bind(symbol).bind(token.address.to_ascii_lowercase()).bind(token.decimals as i32).bind(amount).bind(if mode=="escrow"{100}else{50}).bind(description).bind(metadata).bind(salt).bind(&cs).bind(hash(capability(p,&cs).as_bytes())).bind(expires).fetch_one(&mut *tx).await?;
    if b["payment_method"] == "transfer" {
        if mode != "direct" || symbol == "BNB" {
            return Err(bad("qr_requires_stablecoin_direct_payment"));
        }
        let factory = n.qr.as_ref().ok_or_else(|| bad("qr_not_configured"))?;
        healthy(p, n, "qr").await?;
        let address = alloy::primitives::Address::abi_decode(
            &n.call(
                factory.address,
                chain::receiverCall {
                    merchant: payee,
                    token: chain::address(&d.token_address)?,
                    amount: chain::amount(&d.amount)?,
                    salt: B256::from_str(&d.salt).map_err(|_| bad("invalid_salt"))?,
                }
                .abi_encode(),
            )
            .await?,
        )
        .map_err(|_| bad("invalid_receiver"))?
        .to_string()
        .to_ascii_lowercase();
        d = sqlx::query_as("UPDATE pay_merchant_intents SET deposit_address=$2,contract_address=$3 WHERE id=$1 RETURNING *")
            .bind(&d.id).bind(address).bind(factory.address.to_string().to_ascii_lowercase()).fetch_one(&mut *tx).await?;
    }
    let name: String = sqlx::query_scalar("SELECT name FROM pay_merchants WHERE id=$1")
        .bind(&a.merchant_id)
        .fetch_one(&mut *tx)
        .await?;
    let epsx = std::env::var("PAY_EPSX_MERCHANT_ID").ok().as_deref()
        == Some(a.merchant_id.as_str())
        && d.metadata["epsx_order_id"].is_string();
    let mut snapshot = json!({"merchant_name":name,"item_name":d.description.as_deref().unwrap_or("Payment"),"kind":if epsx {"epsx_plan"} else {"merchant"},"payee":d.payee,"token":d.token,"amount":d.amount,"duration_days":null,"product_id":null});
    if epsx && d.metadata["pricing"].is_object() {
        snapshot["pricing"] = d.metadata["pricing"].clone();
    }
    d = sqlx::query_as(
        "UPDATE pay_merchant_intents SET checkout_snapshot=$2 WHERE id=$1 RETURNING *",
    )
    .bind(&d.id)
    .bind(snapshot)
    .fetch_one(&mut *tx)
    .await?;
    if let Some(payer) = b.get("payer").and_then(Value::as_str) {
        bind_payer(tx, &mut d, payer).await?;
    }
    Ok(d)
}
async fn bind_payer(tx: &mut PgConnection, d: &mut Payment, payer: &str) -> Result<()> {
    let payer = chain::address(payer)?;
    if payer.is_zero() || payer == chain::address(&d.payee)? {
        return Err(bad("invalid_payer"));
    }
    let payer = payer.to_string().to_ascii_lowercase();
    if d.payer.as_ref().is_some_and(|p| p != &payer) {
        return Err(Error(StatusCode::CONFLICT, "checkout_wallet_already_bound"));
    }
    let on_chain_id = format!("{:#x}", chain::payment_id(d, chain::address(&payer)?)?);
    sqlx::query("UPDATE pay_merchant_intents SET payer=$2,on_chain_id=$3 WHERE id=$1")
        .bind(&d.id)
        .bind(&payer)
        .bind(&on_chain_id)
        .execute(tx)
        .await?;
    d.payer = Some(payer);
    d.on_chain_id = Some(on_chain_id);
    Ok(())
}
async fn create_intent(p: &Platform, a: &Actor, h: &HeaderMap, b: &Value) -> Result<Value> {
    let (mode, _, _) = terms(b)?;
    healthy(p, network(p, &a.environment)?, mode).await?;
    let key = key(h)?;
    let scope = format!("{}:{}:intents", a.merchant_id, a.environment);
    let mut tx = p.db.begin().await?;
    if let Some(v) = start(&mut tx, &scope, &key, b).await? {
        return Ok(checkout_response(p, v));
    }
    if let Some(order) = b.get("order_reference").and_then(Value::as_str) {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("order:{scope}:{order}"))
            .execute(&mut *tx)
            .await?;
        let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pay_merchant_intents WHERE merchant_id=$1 AND environment=$2 AND order_reference=$3)").bind(&a.merchant_id).bind(&a.environment).bind(order).fetch_one(&mut *tx).await?;
        if exists {
            return Err(Error(
                StatusCode::CONFLICT,
                "order_reference_already_exists",
            ));
        }
    }
    let d = insert(p, &mut tx, a, b, None, expiration(b, 1800)?).await?;
    let response = json!({"intent":public_payment(&d,false),"checkout_id":d.checkout_id});
    end(&mut tx, &scope, &key, b, &response).await?;
    tx.commit().await?;
    Ok(checkout_response(p, response))
}
async fn create_link(p: &Platform, a: &Actor, h: &HeaderMap, b: &Value) -> Result<Value> {
    let (mode, token, amount) = terms(b)?;
    let n = network(p, &a.environment)?;
    healthy(p, n, mode).await?;
    if !n.tokens.contains_key(token) {
        return Err(bad("unsupported_token"));
    }
    let max = match b.get("max_uses") {
        None | Some(Value::Null) => None,
        Some(v) => Some(
            v.as_i64()
                .filter(|v| (1..=100000).contains(v))
                .ok_or_else(|| bad("invalid_max_uses"))? as i32,
        ),
    };
    let description = b.get("description").and_then(Value::as_str);
    if description.is_some_and(|v| v.len() > 2000) {
        return Err(bad("invalid_description"));
    }
    let key = key(h)?;
    let scope = format!("{}:{}:links", a.merchant_id, a.environment);
    let mut tx = p.db.begin().await?;
    if let Some(v) = start(&mut tx, &scope, &key, b).await? {
        return Ok(v);
    }
    let method = if b["payment_method"] == "transfer" {
        "transfer"
    } else {
        "contract"
    };
    if method == "transfer" && (mode != "direct" || token == "BNB" || n.qr.is_none()) {
        return Err(bad("qr_requires_stablecoin_direct_payment"));
    }
    let link = id("plink");
    sqlx::query("INSERT INTO pay_merchant_links(id,merchant_id,environment,mode,payee,token,amount,description,max_uses,expires_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
        .bind(&link).bind(&a.merchant_id).bind(&a.environment).bind(mode).bind(&a.wallet).bind(token).bind(amount).bind(description).bind(max).bind(expiration(b,2592000)?).execute(&mut *tx).await?;
    sqlx::query("UPDATE pay_merchant_links SET payment_method=$2 WHERE id=$1")
        .bind(&link)
        .bind(method)
        .execute(&mut *tx)
        .await?;
    let v = json!({"id":link,"url":public_url(&format!("/r/{link}"))});
    end(&mut tx, &scope, &key, b, &v).await?;
    tx.commit().await?;
    Ok(v)
}
async fn redeem(p: &Platform, link: &str, h: &HeaderMap, b: &Value) -> Result<Value> {
    let guest_id = h
        .get("x-pay-guest-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| {
            (32..=128).contains(&s.len())
                && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
        .ok_or_else(|| bad("guest_id_required"))?;
    let key = key(h)?;
    let scope = format!("link:{link}:{}", hash(guest_id.as_bytes()));
    let mut tx = p.db.begin().await?;
    if let Some(v) = start(&mut tx, &scope, &key, b).await? {
        return Ok(checkout_response(p, v));
    }
    let row = sqlx::query("SELECT * FROM pay_merchant_links WHERE id=$1 FOR UPDATE")
        .bind(link)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(missing)?;
    if row.get::<bool, _>("disabled") || row.get::<DateTime<Utc>, _>("expires_at") <= Utc::now() {
        return Err(conflict());
    }
    let n = network(p, &row.get::<String, _>("environment"))?;
    let mode: String = row.get("mode");
    healthy(p, n, &mode).await?;
    // Pending reservations only become reusable after the scanner marks them expired.
    let reserved: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pay_merchant_intents WHERE link_id=$1 AND status<>'expired'",
    )
    .bind(link)
    .fetch_one(&mut *tx)
    .await?;
    if row
        .get::<Option<i32>, _>("max_uses")
        .is_some_and(|max| reserved >= max as i64)
    {
        return Err(Error(StatusCode::CONFLICT, "link_capacity_reserved"));
    }
    let actor = Actor {
        merchant_id: row.get("merchant_id"),
        wallet: row.get("payee"),
        environment: row.get("environment"),
        owner_session: false,
        admin: false,
    };
    let body = json!({"mode":mode,"token":row.get::<String,_>("token"),"amount":row.get::<String,_>("amount"),"description":row.get::<Option<String>,_>("description"),"payment_method":row.get::<String,_>("payment_method"),"payer":b.get("payer")});
    let expires = expiration(b, 1800)?
        .min(Utc::now() + chrono::Duration::minutes(30))
        .min(row.get("expires_at"));
    let d = insert(p, &mut tx, &actor, &body, Some(link), expires).await?;
    let v = json!({"intent":public_payment(&d,true),"checkout_id":d.checkout_id});
    end(&mut tx, &scope, &key, b, &v).await?;
    tx.commit().await?;
    Ok(checkout_response(p, v))
}
// Preparing a QR transfer never binds the payer or changes the payment projection.
// Only canonical token logs may establish who paid this invoice.
async fn prepare_transfer(p: &Platform, h: &HeaderMap, cs: &str, b: &Value) -> Result<Value> {
    use alloy::sol_types::SolCall;
    let mut tx = p.db.begin().await?;
    let d = load(&mut tx, cs).await?;
    guest(h, &d)?;
    if d.status != "awaiting_payment" || d.expires_at <= Utc::now() {
        return Err(Error(StatusCode::CONFLICT, "checkout_not_payable"));
    }
    let recipient = chain::address(
        d.deposit_address
            .as_deref()
            .ok_or_else(|| bad("not_transfer_checkout"))?,
    )?;
    let payer = chain::address(text(b, "payer", 42)?)?;
    if payer.is_zero() || payer == recipient {
        return Err(bad("invalid_payer"));
    }
    let n = network(p, &d.environment)?;
    healthy(p, n, "qr").await?;
    let token = chain::address(&d.token_address)?;
    let amount = d
        .amount
        .parse::<alloy::primitives::U256>()
        .map_err(|_| bad("invalid_amount"))?;
    let balance = chain::balanceOfCall::abi_decode_returns(
        &n.call(token, chain::balanceOfCall { owner: payer }.abi_encode())
            .await?,
    )
    .map_err(|_| bad("invalid_token_balance"))?;
    if balance < amount {
        return Err(bad("insufficient_token_balance"));
    }
    let params = json!({"from":payer.to_string(),"to":token.to_string(),"value":"0x0",
        "chainId":format!("0x{:x}", n.chain_id),"data":format!("0x{}",hex::encode(chain::transferCall { to:recipient, amount }.abi_encode()))});
    let native = n
        .rpc("eth_getBalance", json!([payer.to_string(), "pending"]))
        .await?;
    let quantity = |v: &Value| {
        alloy::primitives::U256::from_str_radix(
            v.as_str().unwrap_or("").trim_start_matches("0x"),
            16,
        )
        .map_err(|_| bad("invalid_rpc_quantity"))
    };
    if quantity(&native)?.is_zero() {
        return Err(bad("insufficient_gas_balance"));
    }
    let gas = n.rpc("eth_estimateGas", json!([params])).await?;
    let price = n.rpc("eth_gasPrice", json!([])).await?;
    if quantity(&native)? < quantity(&gas)?.saturating_mul(quantity(&price)?) {
        return Err(bad("insufficient_gas_balance"));
    }
    if d.expires_at <= Utc::now() {
        return Err(Error(StatusCode::CONFLICT, "checkout_not_payable"));
    }
    Ok(json!({"transaction_parameters":params,"payment":public_payment(&d,true)}))
}

async fn prepare(
    p: &Platform,
    h: &HeaderMap,
    id: &str,
    kind: &str,
    b: &Value,
    guest_access: bool,
) -> Result<Value> {
    let mut tx = p.db.begin().await?;
    let mut d = load(&mut tx, id).await?;
    let n = network(p, &d.environment)?;
    healthy(
        p,
        n,
        if d.deposit_address.is_some() {
            "qr"
        } else {
            &d.mode
        },
    )
    .await?;
    if d.deposit_address.is_some() && !["refund", "collect"].contains(&kind) {
        return Err(bad("use_token_transfer"));
    }
    let actor = if guest_access {
        guest(h, &d)?;
        text(b, "payer", 42)?.to_ascii_lowercase()
    } else {
        let a = auth::actor(p, h).await?;
        if kind.starts_with("resolve-") {
            if !a.admin || a.environment != d.environment || chain::address(&a.wallet)? != n.admin {
                return Err(missing());
            }
        } else {
            owned(&a, &d)?;
        }
        a.wallet
    };
    let key = key(h)?;
    let scope = format!("operation:{}:{actor}", d.id);
    let request = json!([kind, b]);
    if let Some(v) = start(&mut tx, &scope, &key, &request).await? {
        return Ok(v);
    }
    let funding = matches!(kind, "pay" | "deposit");
    if funding {
        if d.status != "awaiting_payment"
            || d.expires_at <= Utc::now()
            || kind != if d.mode == "escrow" { "deposit" } else { "pay" }
        {
            return Err(conflict());
        }
        let paused = bool::abi_decode(
            &n.call(
                n.contract(&d.mode).address,
                chain::pausedCall {}.abi_encode(),
            )
            .await?,
        )
        .map_err(|_| bad("invalid_pause_response"))?;
        if paused {
            return Err(Error(StatusCode::CONFLICT, "payments_paused"));
        }
        bind_payer(&mut tx, &mut d, &actor).await?;
    }
    let payer = d.payer.as_deref() == Some(actor.as_str());
    let payee = d.payee == actor;
    let allowed = match kind {
        "collect" => payee && d.deposit_address.is_some(),
        "pay" | "deposit" => payer,
        "release" => d.mode == "escrow" && payer && d.status == "funded",
        "dispute" => d.mode == "escrow" && (payer || payee) && d.status == "funded",
        "refund" => {
            payee
                && if d.mode == "direct" {
                    d.status == "succeeded"
                } else {
                    ["funded", "disputed"].contains(&d.status.as_str())
                }
        }
        "resolve-release" | "resolve-refund" => {
            !guest_access
                && d.mode == "escrow"
                && d.status == "disputed"
                && chain::address(&actor)? == n.admin
        }
        _ => false,
    };
    if !allowed {
        return Err(missing());
    }
    if kind == "collect" {
        let balance = chain::balanceOfCall::abi_decode_returns(
            &n.call(
                chain::address(&d.token_address)?,
                chain::balanceOfCall {
                    owner: chain::address(d.deposit_address.as_deref().ok_or_else(missing)?)?,
                }
                .abi_encode(),
            )
            .await?,
        )
        .map_err(|_| bad("invalid_balance_response"))?;
        if balance.is_zero() {
            return Err(bad("no_funds_to_collect"));
        }
    }
    let (params, mut approval) = chain::parameters(&d, &actor, kind)?;
    if approval.is_object() {
        let allowance = alloy::primitives::U256::abi_decode(
            &n.call(
                chain::address(&d.token_address)?,
                chain::allowanceCall {
                    owner: chain::address(&actor)?,
                    spender: chain::address(&d.contract_address)?,
                }
                .abi_encode(),
            )
            .await?,
        )
        .map_err(|_| bad("invalid_allowance_response"))?;
        if allowance >= chain::amount(&d.amount)? {
            approval = Value::Null;
        }
    }
    let op = super::id("mop");
    sqlx::query("INSERT INTO pay_merchant_operations(id,intent_id,actor,kind,transaction_parameters,approval_transaction) VALUES($1,$2,$3,$4,$5,$6)").bind(&op).bind(&d.id).bind(&actor).bind(kind).bind(&params).bind(&approval).execute(&mut *tx).await?;
    let v = json!({"id":op,"intent_id":d.id,"status":"awaiting_signature","transaction_parameters":params,"approval_transaction":approval});
    end(&mut tx, &scope, &key, &request, &v).await?;
    tx.commit().await?;
    Ok(v)
}
async fn operation(p: &Platform, h: &HeaderMap, id: &str, b: Option<&Value>) -> Result<Value> {
    let mut tx = p.db.begin().await?;
    // The intent reference is immutable. Lock its payment before the operation,
    // matching the reconciler's order when it confirms or invalidates evidence.
    let intent: String =
        sqlx::query_scalar("SELECT intent_id FROM pay_merchant_operations WHERE id=$1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(missing)?;
    let d = load(&mut tx, &intent).await?;
    let row = sqlx::query("SELECT * FROM pay_merchant_operations WHERE id=$1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(missing)?;
    if h.contains_key("x-pay-checkout-token") {
        guest(h, &d)?
    } else {
        let a = auth::actor(p, h).await?;
        if !(a.admin && a.environment == d.environment && a.wallet == row.get::<String, _>("actor"))
        {
            owned(&a, &d)?
        }
    }
    if let Some(b) = b {
        let value = text(b, "tx_hash", 66)?;
        B256::from_str(value).map_err(|_| bad("invalid_transaction_hash"))?;
        let existing: Option<String> = row.get("tx_hash");
        if existing
            .as_ref()
            .is_some_and(|s| !s.eq_ignore_ascii_case(value))
        {
            return Err(conflict());
        }
        sqlx::query("UPDATE pay_merchant_operations SET tx_hash=$2,status=CASE WHEN status='awaiting_signature' THEN 'pending' ELSE status END WHERE id=$1").bind(id).bind(value.to_ascii_lowercase()).execute(&mut *tx).await?;
    }
    let op = sqlx::query("SELECT * FROM pay_merchant_operations WHERE id=$1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    let mut status: String = op.get("status");
    if status == "confirmed" {
        let proof:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pay_merchant_chain_events WHERE intent_id=$1 AND tx_hash=$2 AND block_number=$3 AND block_hash=$4 AND canonical)").bind(&d.id).bind(op.get::<Option<String>,_>("tx_hash")).bind(op.get::<Option<i64>,_>("verified_block")).bind(op.get::<Option<String>,_>("verified_block_hash")).fetch_one(&mut *tx).await?;
        if !proof {
            status = "verification_required".into();
        } else {
            healthy(
                p,
                network(p, &d.environment)?,
                if d.deposit_address.is_some() {
                    "qr"
                } else {
                    &d.mode
                },
            )
            .await?;
        }
    }
    let v = json!({"id":id,"intent_id":d.id,"kind":op.get::<String,_>("kind"),"status":status,"tx_hash":op.get::<Option<String>,_>("tx_hash"),"transaction_parameters":op.get::<Value,_>("transaction_parameters"),"approval_transaction":op.get::<Option<Value>,_>("approval_transaction")});
    tx.commit().await?;
    Ok(v)
}
pub async fn handle(p: &Platform, request: Request) -> Result<Value> {
    let (parts, body) = request.into_parts();
    let h = parts.headers;
    let path = parts.uri.path().trim_start_matches("/api/v1/pay/");
    let segments: Vec<_> = path.split('/').collect();
    let method = parts.method.as_str();
    if method == "GET" && path == "config" {
        return Ok(
            json!({"environments":p.chains.iter().map(|n|json!({"environment":n.environment,"chain_id":n.chain_id,"tokens":n.tokens,"confirmations":n.confirmations,"direct_contract":n.direct.address,"escrow_contract":n.escrow.address,"qr_contract":n.qr.as_ref().map(|c|c.address)})).collect::<Vec<_>>(),"direct_fee_bps":50,"escrow_fee_bps":100}),
        );
    }
    if p.secret.len() < 32 {
        return Err(Error(
            StatusCode::SERVICE_UNAVAILABLE,
            "merchant_platform_not_configured",
        ));
    }
    let bytes = axum::body::to_bytes(body, 64 * 1024)
        .await
        .map_err(|_| bad("request_too_large"))?;
    let b: Value = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).map_err(|_| bad("invalid_json"))?
    };
    if method == "POST" && path == "merchants" {
        let principal = auth::principal(p, &h).await?;
        let wallet = principal.wallet_address.to_ascii_lowercase();
        chain::address(&wallet)?;
        auth::rate(p, &format!("signup:{wallet}"), 5).await?;
        let name = text(&b, "name", 100)?.trim();
        if name.is_empty() {
            return Err(bad("invalid_merchant_name"));
        }
        let row=sqlx::query("INSERT INTO pay_merchants(id,owner,name) VALUES($1,$2,$3) ON CONFLICT(owner) DO UPDATE SET owner=EXCLUDED.owner RETURNING id,owner,name").bind(id("mer")).bind(wallet).bind(name).fetch_one(&p.db).await?;
        return Ok(
            json!({"id":row.get::<String,_>("id"),"owner":row.get::<String,_>("owner"),"name":row.get::<String,_>("name")}),
        );
    }
    if let Some(result) = super::catalog::public(p, &h, method, &segments, &b).await {
        return result;
    }
    match (method, segments.as_slice()) {
        ("GET", ["links", link]) => {
            let row=sqlx::query("SELECT l.*,m.name FROM pay_merchant_links l JOIN pay_merchants m ON m.id=l.merchant_id WHERE l.id=$1 AND NOT l.disabled AND l.expires_at>now()") .bind(link).fetch_optional(&p.db).await?.ok_or_else(missing)?;
            return Ok(
                json!({"id":link,"merchant_name":row.get::<String,_>("name"),"payee":row.get::<String,_>("payee"),"environment":row.get::<String,_>("environment"),"mode":row.get::<String,_>("mode"),"token":row.get::<String,_>("token"),"amount":row.get::<String,_>("amount"),"description":row.get::<Option<String>,_>("description")}),
            );
        }
        ("POST", ["links", link, "checkouts"]) => {
            auth::rate(p, &format!("redeem:{link}"), 60).await?;
            return redeem(p, link, &h, &b).await;
        }
        ("GET", ["checkout-sessions", cs]) => {
            let mut tx = p.db.begin().await?;
            let d = load(&mut tx, cs).await?;
            guest(&h, &d)?;
            return Ok(public_payment(&d, true));
        }
        ("POST", ["checkout-sessions", cs, kind]) => {
            auth::rate(p, &format!("guest:{cs}"), 30).await?;
            if *kind == "prepare-transfer" {
                return prepare_transfer(p, &h, cs, &b).await;
            }
            return prepare(p, &h, cs, kind, &b, true).await;
        }
        ("GET", ["operations", op]) => return operation(p, &h, op, None).await,
        ("POST", ["operations", op, "confirm"]) => return operation(p, &h, op, Some(&b)).await,
        _ => {}
    }
    let a = auth::actor(p, &h).await?;
    auth::rate(
        p,
        &format!(
            "{}:{}:{}",
            a.merchant_id,
            a.environment,
            if method == "GET" { "read" } else { "write" }
        ),
        if method == "GET" { 600 } else { 60 },
    )
    .await?;
    if matches!(
        segments.first(),
        Some(&"contracts") | Some(&"contract-controls")
    ) {
        return super::controls::handle(p, &a, &h, method, &segments, &b).await;
    }
    if let Some(result) = super::catalog::private(p, &a, &h, method, &segments, &b).await {
        return result;
    }
    match (method, segments.as_slice()) {
        ("GET", ["merchants", "me"]) => Ok(json!(a)),
        ("POST", ["intents"] | ["checkout-sessions"]) => create_intent(p, &a, &h, &b).await,
        ("POST", ["links"]) => create_link(p, &a, &h, &b).await,
        ("GET", ["intents"] | ["escrows"]) => {
            let mut tx = p.db.begin().await?;
            let mut rows:Vec<Payment>=sqlx::query_as("SELECT * FROM pay_merchant_intents WHERE (merchant_id=$1 OR $3) AND environment=$2 ORDER BY created_at DESC LIMIT 100").bind(&a.merchant_id).bind(&a.environment).bind(a.admin).fetch_all(&mut *tx).await?;
            let mut items = vec![];
            for d in &mut rows {
                verify_projection(&mut tx, d).await?;
                if path != "escrows" || d.mode == "escrow" {
                    items.push(super::catalog::settlement(p, &mut tx, d).await?);
                }
            }
            Ok(json!({"items":items}))
        }
        ("GET", ["intents", pi] | ["escrows", pi]) => {
            let mut tx = p.db.begin().await?;
            let d = load(&mut tx, pi).await?;
            if !a.admin {
                owned(&a, &d)?;
            } else if a.environment != d.environment {
                return Err(missing());
            }
            let mut v = super::catalog::settlement(p, &mut tx, &d).await?;
            if a.admin {
                v["available_actions"] = json!(if d.mode == "escrow"
                    && d.status == "disputed"
                    && chain::address(&a.wallet)? == network(p, &d.environment)?.admin
                {
                    vec!["resolve-release", "resolve-refund"]
                } else {
                    vec![]
                });
            }
            Ok(v)
        }
        ("POST", ["intents", pi, kind] | ["escrows", pi, kind]) => {
            prepare(
                p,
                &h,
                pi,
                if *kind == "refunds" { "refund" } else { kind },
                &b,
                false,
            )
            .await
        }
        _ => super::webhooks::management(p, &a, method, &segments, &b).await,
    }
}
