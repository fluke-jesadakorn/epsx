//! Versioned Pay storage and transaction preparation. Only the chain worker finalizes funds.
use crate::{
    native_chain::{self as chain, Chain, Error},
    AppState,
};
use alloy::{
    primitives::{keccak256, Address, B256},
    sol_types::SolCall,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use epsx_service_auth::VerifiedPrincipal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, PgConnection, Row};
use std::str::FromStr;
#[derive(Debug)]
pub struct ApiError(StatusCode, &'static str);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error=%e,"Pay storage operation failed");
        Self(StatusCode::SERVICE_UNAVAILABLE, "storage_unavailable")
    }
}
impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        tracing::warn!(error=%e,"Pay evidence rejected");
        Self(StatusCode::BAD_REQUEST, "invalid_transaction_parameters")
    }
}
type Result<T> = std::result::Result<T, ApiError>;
fn forbidden() -> ApiError {
    ApiError(StatusCode::NOT_FOUND, "not_found")
}
fn conflict() -> ApiError {
    ApiError(StatusCode::CONFLICT, "operation_conflict")
}
fn owner(p: &VerifiedPrincipal) -> String {
    p.wallet_address.to_ascii_lowercase()
}
fn configured(s: &AppState) -> Result<&Chain> {
    s.native_chain.as_deref().ok_or(ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "escrow_not_configured",
    ))
}
fn key(headers: &HeaderMap) -> Result<&str> {
    headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
        })
        .ok_or(ApiError(
            StatusCode::BAD_REQUEST,
            "idempotency_key_required",
        ))
}
#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Deal {
    pub id: String,
    pub chain_id: i64,
    pub contract_address: String,
    pub contract_version: i32,
    pub salt: String,
    pub on_chain_id: String,
    pub payer: String,
    pub payee: String,
    pub token_address: String,
    pub amount: String,
    pub description: Option<String>,
    pub status: String,
    pub fee_amount: String,
    pub tx_hash: Option<String>,
    pub verified_block: Option<i64>,
    pub verified_block_hash: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Serialize, Deserialize, FromRow)]
pub struct Link {
    pub slug: String,
    pub owner: String,
    pub chain_id: i64,
    pub contract_address: String,
    pub payee: String,
    pub token_address: String,
    pub amount: String,
    pub description: Option<String>,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub disabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Serialize, Deserialize, FromRow)]
pub struct Operation {
    pub id: String,
    pub deal_id: String,
    pub actor: String,
    pub kind: String,
    pub transaction_parameters: Value,
    pub tx_hash: Option<String>,
    pub status: String,
    pub confirmed_block: Option<i64>,
    pub confirmed_block_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Deserialize, Serialize)]
pub struct NewDeal {
    payer: Option<String>,
    payee: String,
    amount: String,
    token: String,
    description: Option<String>,
    expires_in: Option<i64>,
}
#[derive(Deserialize, Serialize)]
pub struct NewLink {
    amount: String,
    token: String,
    description: Option<String>,
    max_uses: Option<i32>,
    expires_in: Option<i64>,
}
#[derive(Deserialize, Serialize)]
pub struct Redeem {
    payer: Option<String>,
}
#[derive(Deserialize, Serialize)]
pub struct Confirmation {
    pub tx_hash: String,
}
#[derive(Deserialize, Serialize)]
pub struct Resolution {
    to_payee: bool,
}
fn expiration(seconds: Option<i64>) -> Result<chrono::DateTime<chrono::Utc>> {
    let seconds = seconds.unwrap_or(3600);
    if !(60..=2592000).contains(&seconds) {
        return Err(ApiError(StatusCode::BAD_REQUEST, "invalid_expiration"));
    }
    Ok(chrono::Utc::now() + chrono::Duration::seconds(seconds))
}
fn token(chain: &Chain, symbol: &str) -> Result<String> {
    chain
        .tokens
        .get(symbol)
        .map(|t| t.address.to_ascii_lowercase())
        .ok_or(ApiError(StatusCode::BAD_REQUEST, "unsupported_token"))
}
async fn healthy(s: &AppState, c: &Chain) -> Result<()> {
    let valid=sqlx::query_scalar::<_,bool>("SELECT healthy AND last_checked_at > now()-interval '60 seconds' FROM pay_v1_chain_checkpoints WHERE chain_id=$1 AND contract_address=$2").bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase()).fetch_optional(&s.db).await?.unwrap_or(false);
    if !valid {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "chain_verification_unavailable",
        ));
    }
    Ok(())
}
async fn request_start(
    conn: &mut PgConnection,
    actor: &str,
    key: &str,
    request: &Value,
) -> Result<Option<Value>> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("pay:{actor}:{key}"))
        .execute(&mut *conn)
        .await?;
    let hash = format!("{:#x}", keccak256(serde_json::to_vec(request).unwrap()));
    if let Some(row) = sqlx::query(
        "SELECT request_hash,response FROM pay_v1_requests WHERE actor=$1 AND idempotency_key=$2",
    )
    .bind(actor)
    .bind(key)
    .fetch_optional(&mut *conn)
    .await?
    {
        if row.get::<String, _>("request_hash") != hash {
            return Err(conflict());
        }
        return Ok(Some(row.get("response")));
    }
    Ok(None)
}
async fn request_end(
    conn: &mut PgConnection,
    actor: &str,
    key: &str,
    request: &Value,
    response: &Value,
) -> Result<()> {
    sqlx::query("INSERT INTO pay_v1_requests(actor,idempotency_key,request_hash,response) VALUES($1,$2,$3,$4)").bind(actor).bind(key).bind(format!("{:#x}",keccak256(serde_json::to_vec(request).unwrap()))).bind(response).execute(conn).await?;
    Ok(())
}
// Keep the immutable contract terms explicit at the persistence boundary.
#[allow(clippy::too_many_arguments)]
async fn insert_deal(
    conn: &mut PgConnection,
    c: &Chain,
    payer: &str,
    payee: &str,
    amount: &str,
    token: &str,
    description: Option<&str>,
    expires: chrono::DateTime<chrono::Utc>,
) -> Result<Deal> {
    chain::amount(amount)?;
    let payer_address = chain::address(payer)?;
    let payee_address = chain::address(payee)?;
    if payer_address.is_zero()
        || payee_address.is_zero()
        || payer_address == payee_address
        || payee_address == c.contract
        || description.is_some_and(|s| s.len() > 2000)
    {
        return Err(ApiError(StatusCode::BAD_REQUEST, "invalid_deal"));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let salt = keccak256(id.as_bytes());
    let on_chain_id = c.id(payer_address, salt);
    Ok(sqlx::query_as("INSERT INTO pay_v1_deals(id,chain_id,contract_address,contract_version,salt,on_chain_id,payer,payee,token_address,amount,description,expires_at) VALUES($1,$2,$3,1,$4,$5,$6,$7,$8,$9,$10,$11) RETURNING *").bind(id).bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase()).bind(format!("{salt:#x}")).bind(format!("{on_chain_id:#x}")).bind(payer.to_ascii_lowercase()).bind(payee.to_ascii_lowercase()).bind(token).bind(amount).bind(description).bind(expires).fetch_one(conn).await?)
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
pub async fn config(State(s): State<AppState>) -> Result<Json<Value>> {
    let c = configured(&s)?;
    Ok(Json(
        json!({"chain_id":c.chain_id,"contract_address":c.contract,"contract_version":1,"tokens":c.tokens,"confirmations":c.confirmations,"fee_bps":30}),
    ))
}
pub async fn create_intent(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Json(body): Json<NewDeal>,
) -> Result<Json<Value>> {
    let c = configured(&s)?;
    healthy(&s, c).await?;
    let actor = owner(&p);
    if body
        .payer
        .as_ref()
        .is_some_and(|w| !w.eq_ignore_ascii_case(&actor))
    {
        return Err(forbidden());
    }
    let key = key(&headers)?;
    let request = json!(["create_intent", body]);
    let mut tx = s.db.begin().await?;
    if let Some(response) = request_start(&mut tx, &actor, key, &request).await? {
        return Ok(Json(response));
    }
    let deal = insert_deal(
        &mut tx,
        c,
        &actor,
        &body.payee,
        &body.amount,
        &token(c, &body.token)?,
        body.description.as_deref(),
        expiration(body.expires_in)?,
    )
    .await?;
    let response = json!({"intent":deal,"pay_url":public_url(&format!("/checkout/{}",deal.id))});
    request_end(&mut tx, &actor, key, &request, &response).await?;
    tx.commit().await?;
    Ok(Json(response))
}
async fn load(conn: &mut PgConnection, id: &str) -> Result<Deal> {
    let mut deal: Deal = sqlx::query_as("SELECT * FROM pay_v1_deals WHERE id=$1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(forbidden)?;
    verify_projections(conn, std::slice::from_mut(&mut deal)).await?;
    Ok(deal)
}

fn event_signature(status: &str) -> Option<String> {
    let event = match status {
        "active" | "deposit" => "Deposited(bytes32,address,address,address,uint256)",
        "released" | "release" | "resolve-release" => {
            "Released(bytes32,address,address,address,uint256,uint256)"
        }
        "refunded" | "refund" | "resolve-refund" => {
            "Refunded(bytes32,address,address,address,uint256)"
        }
        "disputed" | "dispute" => "Disputed(bytes32,address)",
        _ => return None,
    };
    Some(format!("{:#x}", keccak256(event)))
}

/// A status column alone is never payment evidence. Only the scanner writes
/// canonical events; bind the presented state to that exact event and block.
async fn verify_projections(conn: &mut PgConnection, deals: &mut [Deal]) -> Result<()> {
    let ids: Vec<_> = deals.iter().map(|d| d.id.clone()).collect();
    let proofs = sqlx::query("SELECT d.id,e.payload FROM pay_v1_deals d JOIN pay_v1_chain_events e ON e.chain_id=d.chain_id AND e.contract_address=d.contract_address AND e.tx_hash=d.tx_hash AND e.block_number=d.verified_block AND e.block_hash=d.verified_block_hash WHERE d.id=ANY($1) AND e.payload->'topics'->>1=d.on_chain_id")
        .bind(ids).fetch_all(conn).await?;
    for deal in deals {
        let Some(signature) = event_signature(&deal.status) else {
            continue;
        };
        let fee = if deal.status == "released" {
            chain::amount(&deal.amount)
                .ok()
                .map(|amount| chain::fee(amount).to_string())
        } else {
            Some("0".into())
        };
        let valid = deal.contract_version == 1
            && fee.as_ref() == Some(&deal.fee_amount)
            && proofs.iter().any(|row| {
                row.get::<String, _>("id") == deal.id
                    && row.get::<Value, _>("payload")["topics"][0].as_str()
                        == Some(signature.as_str())
            });
        if !valid {
            deal.status = "verification_required".into();
        }
    }
    Ok(())
}
fn participant(p: &VerifiedPrincipal, d: &Deal) -> bool {
    d.payer.eq_ignore_ascii_case(&p.wallet_address)
        || d.payee.eq_ignore_ascii_case(&p.wallet_address)
}
pub async fn get_intent(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let mut conn = s.db.acquire().await?;
    let d = load(&mut conn, &id).await?;
    if !participant(&p, &d) {
        return Err(forbidden());
    }
    let actor = owner(&p);
    let mut actions = Vec::new();
    if d.status == "pending" && actor == d.payer && d.expires_at > chrono::Utc::now() {
        actions.push("deposit")
    }
    if d.status == "active" {
        if actor == d.payer {
            actions.push("release")
        }
        if actor == d.payee {
            actions.push("refund")
        }
        actions.push("dispute")
    }
    if d.status == "disputed" && actor == d.payee {
        actions.push("refund")
    }
    let mut value = json!(d);
    value["available_actions"] = json!(actions);
    if let Some(c) = s.native_chain.as_deref() {
        if let Some((symbol, token)) = c
            .tokens
            .iter()
            .find(|(_, t)| t.address.eq_ignore_ascii_case(&d.token_address))
        {
            value["token_symbol"] = json!(symbol);
            value["token_decimals"] = json!(token.decimals)
        }
    }
    Ok(Json(value))
}
pub async fn list_intents(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
) -> Result<Json<Value>> {
    let mut conn = s.db.acquire().await?;
    let mut deals: Vec<Deal> = sqlx::query_as(
        "SELECT * FROM pay_v1_deals WHERE payer=$1 OR payee=$1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(owner(&p))
    .fetch_all(&mut *conn)
    .await?;
    verify_projections(&mut conn, &mut deals).await?;
    Ok(Json(json!({"total":deals.len(),"items":deals})))
}
pub async fn create_link(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Json(body): Json<NewLink>,
) -> Result<Json<Value>> {
    let c = configured(&s)?;
    healthy(&s, c).await?;
    chain::amount(&body.amount)?;
    let actor = owner(&p);
    let key = key(&headers)?;
    let max = body.max_uses.unwrap_or(1);
    if !(1..=100000).contains(&max) || body.description.as_ref().is_some_and(|d| d.len() > 2000) {
        return Err(ApiError(StatusCode::BAD_REQUEST, "invalid_link"));
    }
    let request = json!(["create_link", body]);
    let mut tx = s.db.begin().await?;
    if let Some(r) = request_start(&mut tx, &actor, key, &request).await? {
        return Ok(Json(r));
    }
    let slug = format!("epsx-{}", uuid::Uuid::new_v4().simple());
    let link:Link=sqlx::query_as("INSERT INTO pay_v1_links(slug,owner,chain_id,contract_address,payee,token_address,amount,description,max_uses,expires_at) VALUES($1,$2,$3,$4,$2,$5,$6,$7,$8,$9) RETURNING *").bind(&slug).bind(&actor).bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase()).bind(token(c,&body.token)?).bind(&body.amount).bind(&body.description).bind(max).bind(expiration(body.expires_in)?).fetch_one(&mut *tx).await?;
    let response = json!({"link":link,"url":public_url(&format!("/r/{slug}"))});
    request_end(&mut tx, &actor, key, &request, &response).await?;
    tx.commit().await?;
    Ok(Json(response))
}
pub async fn get_link(State(s): State<AppState>, Path(slug): Path<String>) -> Result<Json<Value>> {
    let link:Link=sqlx::query_as("SELECT * FROM pay_v1_links WHERE slug=$1 AND NOT disabled AND expires_at>now() AND current_uses<max_uses").bind(slug).fetch_optional(&s.db).await?.ok_or_else(forbidden)?;
    Ok(Json(json!({"link":link})))
}
pub async fn redeem_link(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Redeem>,
) -> Result<Json<Value>> {
    let c = configured(&s)?;
    healthy(&s, c).await?;
    let actor = owner(&p);
    if body
        .payer
        .as_ref()
        .is_some_and(|w| !w.eq_ignore_ascii_case(&actor))
    {
        return Err(forbidden());
    }
    let key = key(&headers)?;
    let request = json!(["redeem", slug, actor]);
    let mut tx = s.db.begin().await?;
    if let Some(r) = request_start(&mut tx, &actor, key, &request).await? {
        return Ok(Json(r));
    }
    let link: Link = sqlx::query_as("SELECT * FROM pay_v1_links WHERE slug=$1 FOR UPDATE")
        .bind(&slug)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(forbidden)?;
    if link.disabled
        || link.expires_at <= chrono::Utc::now()
        || link.current_uses >= link.max_uses
        || link.chain_id != c.chain_id as i64
        || link.contract_address != c.contract.to_string().to_ascii_lowercase()
    {
        return Err(conflict());
    }
    let d = insert_deal(
        &mut tx,
        c,
        &actor,
        &link.payee,
        &link.amount,
        &link.token_address,
        link.description.as_deref(),
        link.expires_at
            .min(chrono::Utc::now() + chrono::Duration::hours(1)),
    )
    .await?;
    sqlx::query("INSERT INTO pay_v1_link_checkouts(slug,payer,idempotency_key,intent_id) VALUES($1,$2,$3,$4)").bind(&slug).bind(&actor).bind(key).bind(&d.id).execute(&mut *tx).await?;
    sqlx::query("UPDATE pay_v1_links SET current_uses=current_uses+1 WHERE slug=$1")
        .bind(&slug)
        .execute(&mut *tx)
        .await?;
    let response = json!({"intent":d,"pay_url":public_url(&format!("/checkout/{}",d.id))});
    request_end(&mut tx, &actor, key, &request, &response).await?;
    tx.commit().await?;
    Ok(Json(response))
}
async fn prepare(
    s: AppState,
    p: VerifiedPrincipal,
    id: String,
    headers: HeaderMap,
    kind: &str,
) -> Result<Json<Value>> {
    let c = configured(&s)?;
    healthy(&s, c).await?;
    if kind == "deposit" && c.deposits_paused().await? {
        return Err(ApiError(StatusCode::CONFLICT, "deposits_paused"));
    }
    let actor = owner(&p);
    let key = key(&headers)?;
    let request = json!([kind, id]);
    let mut tx = s.db.begin().await?;
    if let Some(r) = request_start(&mut tx, &actor, key, &request).await? {
        return Ok(Json(r));
    }
    let d = load(&mut tx, &id).await?;
    if d.chain_id != c.chain_id as i64
        || d.contract_address != c.contract.to_string().to_ascii_lowercase()
    {
        return Err(conflict());
    }
    let admin = p.audience == "epsx-admin"
        && p.has_permission("admin:payments:manage")
        && chain::address(&actor)? == c.admin;
    let allowed = match kind {
        "deposit" => actor == d.payer && d.status == "pending" && d.expires_at > chrono::Utc::now(),
        "release" => actor == d.payer && d.status == "active",
        "refund" => actor == d.payee && ["active", "disputed"].contains(&d.status.as_str()),
        "dispute" => participant(&p, &d) && d.status == "active",
        "resolve-release" | "resolve-refund" => admin && d.status == "disputed",
        _ => false,
    };
    if !allowed {
        return Err(forbidden());
    }
    let escrow = B256::from_str(&d.on_chain_id).map_err(|_| conflict())?;
    let raw_amount = chain::amount(&d.amount)?;
    let token = chain::address(&d.token_address)?;
    let data = match kind {
        "deposit" => chain::depositCall {
            salt: B256::from_str(&d.salt).map_err(|_| conflict())?,
            payee: chain::address(&d.payee)?,
            token,
            amount: raw_amount,
        }
        .abi_encode(),
        "release" => chain::releaseCall { id: escrow }.abi_encode(),
        "refund" => chain::refundCall { id: escrow }.abi_encode(),
        "dispute" => chain::disputeCall { id: escrow }.abi_encode(),
        _ => chain::resolveCall {
            id: escrow,
            toPayee: kind == "resolve-release",
        }
        .abi_encode(),
    };
    let params = json!({"chainId":format!("0x{:x}",c.chain_id),"from":actor,"to":c.contract,"data":format!("0x{}",hex::encode(data)),"value":if kind=="deposit"&&token==Address::ZERO{format!("{raw_amount:#x}")}else{"0x0".into()}});
    let operation:Operation=sqlx::query_as("INSERT INTO pay_v1_operations(id,deal_id,actor,kind,transaction_parameters) VALUES($1,$2,$3,$4,$5) RETURNING *").bind(uuid::Uuid::new_v4().to_string()).bind(&id).bind(&actor).bind(kind).bind(&params).fetch_one(&mut *tx).await?;
    let approval = if kind == "deposit" && token != Address::ZERO {
        json!({"chainId":format!("0x{:x}",c.chain_id),"from":actor,"to":token,"data":format!("0x{}",hex::encode(chain::approveCall{spender:c.contract,amount:raw_amount}.abi_encode())),"value":"0x0"})
    } else {
        Value::Null
    };
    let response = json!({"operation":operation,"transaction":params,"approval_transaction":approval,"status":"awaiting_signature"});
    request_end(&mut tx, &actor, key, &request, &response).await?;
    tx.commit().await?;
    Ok(Json(response))
}
macro_rules! action {
    ($name:ident,$kind:literal) => {
        pub async fn $name(
            State(s): State<AppState>,
            Extension(p): Extension<VerifiedPrincipal>,
            Path(id): Path<String>,
            headers: HeaderMap,
        ) -> Result<Json<Value>> {
            prepare(s, p, id, headers, $kind).await
        }
    };
}
action!(prepare_deposit, "deposit");
action!(release, "release");
action!(refund, "refund");
action!(dispute, "dispute");
pub async fn resolve(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Resolution>,
) -> Result<Json<Value>> {
    prepare(
        s,
        p,
        id,
        headers,
        if body.to_payee {
            "resolve-release"
        } else {
            "resolve-refund"
        },
    )
    .await
}
pub async fn operation(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let mut op: Operation =
        sqlx::query_as("SELECT * FROM pay_v1_operations WHERE id=$1 AND actor=$2")
            .bind(id)
            .bind(owner(&p))
            .fetch_optional(&s.db)
            .await?
            .ok_or_else(forbidden)?;
    if op.status == "confirmed" {
        let verified: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pay_v1_deals d JOIN pay_v1_chain_events e ON e.chain_id=d.chain_id AND e.contract_address=d.contract_address WHERE d.id=$1 AND e.tx_hash=$2 AND e.block_number=$3 AND e.block_hash=$4 AND e.payload->'topics'->>1=d.on_chain_id AND e.payload->'topics'->>0=$5)")
            .bind(&op.deal_id).bind(&op.tx_hash).bind(op.confirmed_block).bind(&op.confirmed_block_hash).bind(event_signature(&op.kind)).fetch_one(&s.db).await?;
        if !verified {
            op.status = "verification_required".into();
        }
    }
    Ok(Json(json!(op)))
}
pub async fn confirm_operation(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    Path(id): Path<String>,
    Json(body): Json<Confirmation>,
) -> Result<(StatusCode, Json<Value>)> {
    let hash = B256::from_str(&body.tx_hash)
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "invalid_transaction_hash"))?;
    let mut tx = s.db.begin().await?;
    let op: Operation =
        sqlx::query_as("SELECT * FROM pay_v1_operations WHERE id=$1 AND actor=$2 FOR UPDATE")
            .bind(&id)
            .bind(owner(&p))
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(forbidden)?;
    let hash = format!("{hash:#x}");
    if op.tx_hash.as_ref().is_some_and(|old| old != &hash) {
        return Err(conflict());
    }
    sqlx::query("UPDATE pay_v1_operations SET tx_hash=$2,status=CASE WHEN status='awaiting_signature' THEN 'pending' ELSE status END,updated_at=now() WHERE id=$1").bind(&id).bind(hash).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"operation_id":id,"status":"pending"})),
    ))
}

pub async fn admin_list(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
) -> Result<Json<Value>> {
    if p.audience != "epsx-admin" || !p.has_permission("admin:payments:view") {
        return Err(forbidden());
    }
    let mut conn = s.db.acquire().await?;
    let mut items: Vec<Deal> =
        sqlx::query_as("SELECT * FROM pay_v1_deals ORDER BY created_at DESC LIMIT 100")
            .fetch_all(&mut *conn)
            .await?;
    verify_projections(&mut conn, &mut items).await?;
    Ok(Json(json!({"total":items.len(),"items":items})))
}
pub async fn admin_get(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    if p.audience != "epsx-admin" || !p.has_permission("admin:payments:view") {
        return Err(forbidden());
    }
    let mut conn = s.db.acquire().await?;
    let d = load(&mut conn, &id).await?;
    let can_resolve = d.status == "disputed"
        && p.has_permission("admin:payments:manage")
        && s.native_chain
            .as_ref()
            .is_some_and(|c| c.admin.to_string().eq_ignore_ascii_case(&p.wallet_address));
    let mut v = json!(d);
    v["available_actions"] = if can_resolve {
        json!(["resolve-release", "resolve-refund"])
    } else {
        json!([])
    };
    if let Some(c) = s.native_chain.as_deref() {
        if let Some((symbol, t)) = c
            .tokens
            .iter()
            .find(|(_, t)| t.address.eq_ignore_ascii_case(&d.token_address))
        {
            v["token_symbol"] = json!(symbol);
            v["token_decimals"] = json!(t.decimals)
        }
    }
    Ok(Json(v))
}

#[derive(Deserialize, Serialize)]
pub struct PauseRequest {
    paused: bool,
}
pub async fn prepare_pause(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Json(body): Json<PauseRequest>,
) -> Result<Json<Value>> {
    let c = configured(&s)?;
    healthy(&s, c).await?;
    let actor = owner(&p);
    if p.audience != "epsx-admin"
        || !p.has_permission("admin:payments:manage")
        || chain::address(&actor)? != c.admin
    {
        return Err(forbidden());
    }
    let key = key(&headers)?;
    let request = json!(["pause", body.paused, c.chain_id, c.contract]);
    let mut tx = s.db.begin().await?;
    if let Some(r) = request_start(&mut tx, &actor, key, &request).await? {
        return Ok(Json(r));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let params = json!({"chainId":format!("0x{:x}",c.chain_id),"from":actor,"to":c.contract,"data":format!("0x{}",hex::encode(chain::setPausedCall{paused_:body.paused}.abi_encode())),"value":"0x0"});
    sqlx::query("INSERT INTO pay_v1_contract_operations(id,chain_id,contract_address,actor,paused,transaction_parameters) VALUES($1,$2,$3,$4,$5,$6)").bind(&id).bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase()).bind(&actor).bind(body.paused).bind(&params).execute(&mut *tx).await?;
    let response = json!({"operation":{"id":id,"status":"awaiting_signature"},"transaction":params,"approval_transaction":null});
    request_end(&mut tx, &actor, key, &request, &response).await?;
    tx.commit().await?;
    Ok(Json(response))
}
pub async fn contract_operation(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let result: Option<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(o) FROM pay_v1_contract_operations o WHERE id=$1 AND actor=$2",
    )
    .bind(id)
    .bind(owner(&p))
    .fetch_optional(&s.db)
    .await?;
    Ok(Json(result.ok_or_else(forbidden)?))
}
pub async fn confirm_contract_operation(
    State(s): State<AppState>,
    Extension(p): Extension<VerifiedPrincipal>,
    Path(id): Path<String>,
    Json(body): Json<Confirmation>,
) -> Result<(StatusCode, Json<Value>)> {
    let hash = B256::from_str(&body.tx_hash)
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "invalid_transaction_hash"))?;
    let changed=sqlx::query("UPDATE pay_v1_contract_operations SET tx_hash=$3,status=CASE WHEN status='awaiting_signature' THEN 'pending' ELSE status END WHERE id=$1 AND actor=$2 AND (tx_hash IS NULL OR tx_hash=$3)").bind(id).bind(owner(&p)).bind(format!("{hash:#x}")).execute(&s.db).await?;
    if changed.rows_affected() != 1 {
        return Err(conflict());
    }
    Ok((StatusCode::ACCEPTED, Json(json!({"status":"pending"}))))
}
