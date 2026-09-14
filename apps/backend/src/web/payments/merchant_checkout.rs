//! EPSX is a merchant of Pay. The Rust backend owns quotes and entitlements.
use crate::domain::subscription_management::token_pricing;
use crate::web::{auth::AppState, middleware::OpenIDUserContext};
use axum::{
    body::Bytes,
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Row};
use std::{str::FromStr, time::Duration};
use uuid::Uuid;

#[derive(Debug)]
pub struct Error(StatusCode, &'static str);
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error=%e,"Pay purchase storage failed");
        Self(StatusCode::SERVICE_UNAVAILABLE, "storage_unavailable")
    }
}
type Result<T> = std::result::Result<T, Error>;
fn bad(s: &'static str) -> Error {
    Error(StatusCode::BAD_REQUEST, s)
}
fn unavailable() -> Error {
    Error(StatusCode::SERVICE_UNAVAILABLE, "pay_checkout_unavailable")
}
fn required(k: &str) -> Result<String> {
    std::env::var(k)
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(unavailable)
}
struct Config {
    base: String,
    key: String,
    merchant: String,
    environment: String,
    payee: String,
    frontend: String,
}
impl Config {
    fn load() -> Result<Self> {
        let base = required("PAYMENT_SERVICE_URL")?;
        let u = reqwest::Url::parse(&base).map_err(|_| unavailable())?;
        let local = u.host_str().is_some_and(|s| {
            s.trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|i| i.is_loopback())
        });
        if !(u.scheme() == "https" || u.scheme() == "http" && local)
            || !u.username().is_empty()
            || u.password().is_some()
        {
            return Err(unavailable());
        }
        let environment = required("EPSX_PAY_ENVIRONMENT")?;
        if !["test", "live"].contains(&environment.as_str()) {
            return Err(unavailable());
        }
        Ok(Self {
            base,
            key: required("EPSX_PAY_API_KEY")?,
            merchant: required("EPSX_PAY_MERCHANT_ID")?,
            environment,
            payee: required("EPSX_PAY_RECIPIENT")?.to_ascii_lowercase(),
            frontend: required("PAY_FRONTEND_URL")?,
        })
    }
    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
        key: Option<&str>,
    ) -> Result<Value> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| unavailable())?;
        let mut r = client
            .request(
                method,
                format!("{}/api/v1/pay/{path}", self.base.trim_end_matches('/')),
            )
            .bearer_auth(&self.key)
            .header("x-pay-api-version", "2026-09-08")
            .header("x-pay-environment", &self.environment);
        if let Some(body) = body {
            r = r.json(&body)
        }
        if let Some(key) = key {
            r = r.header("idempotency-key", key)
        }
        r.send()
            .await
            .map_err(|_| unavailable())?
            .error_for_status()
            .map_err(|_| unavailable())?
            .json()
            .await
            .map_err(|_| unavailable())
    }
}
#[derive(Clone, FromRow, Serialize)]
pub struct Order {
    id: Uuid,
    wallet_address: String,
    plan_id: Uuid,
    request_key: String,
    request_hash: String,
    merchant_id: String,
    environment: String,
    chain_id: i64,
    contract_address: String,
    payee: String,
    token: String,
    token_address: String,
    token_decimals: i32,
    amount: String,
    pricing_snapshot: Value,
    duration_days: Option<i64>,
    description: String,
    pay_intent_id: Option<String>,
    checkout_url: Option<String>,
    status: String,
    payment_revision: i64,
    first_paid_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}
#[derive(Deserialize, Serialize)]
pub struct CheckoutRequest {
    pub plan_id: Uuid,
    pub token: String,
}
fn digest(s: &[u8]) -> String {
    hex::encode(Sha256::digest(s))
}
pub(crate) fn units(value: &str, decimals: u32) -> Result<String> {
    let raw = BigDecimal::from_str(value).map_err(|_| bad("invalid_token_price"))?;
    let scale = BigDecimal::from_str(&format!("1{}", "0".repeat(decimals as usize)))
        .map_err(|_| bad("invalid_token_price"))?;
    let units = raw * scale;
    if units <= 0 || units.with_scale(0) != units {
        return Err(bad("token_price_precision_exceeded"));
    }
    let result = units.with_scale(0).to_string();
    if result.len() > 78 || ethers::types::U256::from_dec_str(&result).is_err() {
        return Err(bad("token_price_out_of_range"));
    }
    Ok(result)
}
fn duration(metadata: &Value, cycle: &str) -> Option<i64> {
    metadata["duration_days"]
        .as_i64()
        .filter(|n| (1..=3650).contains(n))
        .or(match cycle {
            "daily" => Some(1),
            "weekly" => Some(7),
            "quarterly" => Some(90),
            "yearly" | "annual" => Some(365),
            "lifetime" => None,
            _ => Some(30),
        })
}
#[derive(Deserialize)]
pub struct QuoteQuery {
    token: String,
}
pub async fn quote(
    State(state): State<AppState>,
    Path(plan): Path<Uuid>,
    Query(q): Query<QuoteQuery>,
) -> Result<Json<Value>> {
    let c = Config::load()?;
    let row=sqlx::query("SELECT price,currency,plan_metadata FROM plans WHERE id=$1 AND is_active AND is_public AND NOT is_system AND plan_type='subscription'").bind(plan).fetch_optional(state.db_pool.as_ref()).await?.ok_or(Error(StatusCode::NOT_FOUND,"plan_unavailable"))?;
    let metadata: Value = row.get("plan_metadata");
    let pricing = token_pricing::calculate(&metadata, &q.token, Utc::now()).map_err(bad)?;
    let price = &pricing.price;
    let config = c
        .request(reqwest::Method::GET, "config", None, None)
        .await?;
    let network = config["environments"]
        .as_array()
        .and_then(|ns| ns.iter().find(|n| n["environment"] == c.environment))
        .ok_or_else(unavailable)?;
    let token = &network["tokens"][&q.token];
    let decimals = token["decimals"]
        .as_u64()
        .filter(|n| *n <= 36)
        .ok_or_else(|| bad("unsupported_token"))?;
    let amount = units(price, decimals as u32)?;
    Ok(Json(
        json!({"plan_id":plan,"token":q.token,"price":price,"pricing":pricing,"amount":amount,"token_decimals":decimals,"token_address":token["address"],"chain_id":network["chain_id"],"recipient":c.payee,"environment":c.environment}),
    ))
}
pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<OpenIDUserContext>,
    h: HeaderMap,
    Json(body): Json<CheckoutRequest>,
) -> Result<Json<Value>> {
    let c = Config::load()?;
    let key = h
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
        })
        .ok_or_else(|| bad("idempotency_key_required"))?;
    let wallet = user.wallet_address.to_ascii_lowercase();
    let request_hash = digest(serde_json::to_string(&body).unwrap().as_bytes());
    let mut tx = state.db_pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("pay-order:{wallet}:{key}"))
        .execute(&mut *tx)
        .await?;
    let existing: Option<Order> = sqlx::query_as(
        "SELECT * FROM pay_purchase_orders WHERE wallet_address=$1 AND request_key=$2",
    )
    .bind(&wallet)
    .bind(key)
    .fetch_optional(&mut *tx)
    .await?;
    let order = if let Some(order) = existing {
        if order.request_hash != request_hash {
            return Err(Error(StatusCode::CONFLICT, "idempotency_conflict"));
        }
        order
    } else {
        let plan=sqlx::query("SELECT name,price,currency,plan_metadata,billing_cycle FROM plans WHERE id=$1 AND is_active AND is_public AND NOT is_system AND plan_type='subscription'").bind(body.plan_id).fetch_optional(&mut *tx).await?.ok_or(Error(StatusCode::NOT_FOUND,"plan_unavailable"))?;
        let metadata: Value = plan.get("plan_metadata");
        let pricing = token_pricing::calculate(&metadata, &body.token, Utc::now()).map_err(bad)?;
        let price = &pricing.price;
        let config = c
            .request(reqwest::Method::GET, "config", None, None)
            .await?;
        let network = config["environments"]
            .as_array()
            .and_then(|ns| ns.iter().find(|n| n["environment"] == c.environment))
            .ok_or_else(unavailable)?;
        let token = &network["tokens"][&body.token];
        let decimals = token["decimals"]
            .as_u64()
            .filter(|d| *d <= 36)
            .ok_or_else(|| bad("unsupported_token"))?;
        let token_address = token["address"].as_str().ok_or_else(unavailable)?;
        let chain = network["chain_id"].as_i64().ok_or_else(unavailable)?;
        let contract =
            network[if std::env::var("EPSX_PAY_CHECKOUT_METHOD").as_deref() == Ok("transfer") {
                "qr_contract"
            } else {
                "direct_contract"
            }]
            .as_str()
            .ok_or_else(unavailable)?;
        let amount = units(price, decimals as u32)?;
        let name: String = plan.get("name");
        let cycle: Option<String> = plan.get("billing_cycle");
        sqlx::query_as("INSERT INTO pay_purchase_orders(id,wallet_address,plan_id,request_key,request_hash,merchant_id,environment,chain_id,contract_address,payee,token,token_address,token_decimals,amount,duration_days,description,pricing_snapshot) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17) RETURNING *")
            .bind(Uuid::new_v4()).bind(&wallet).bind(body.plan_id).bind(key).bind(request_hash).bind(&c.merchant).bind(&c.environment).bind(chain).bind(contract.to_ascii_lowercase()).bind(&c.payee).bind(&body.token).bind(token_address.to_ascii_lowercase()).bind(decimals as i32).bind(amount).bind(duration(&metadata,cycle.as_deref().unwrap_or("monthly"))).bind(name).bind(serde_json::to_value(&pricing).map_err(|_| unavailable())?).fetch_one(&mut *tx).await?
    };
    tx.commit().await?;
    if let Some(url) = order.checkout_url {
        return Ok(Json(
            json!({"order_id":order.id,"pay_url":url,"status":order.status}),
        ));
    }
    let response=c.request(reqwest::Method::POST,"intents",Some(json!({"mode":"direct","payment_method":std::env::var("EPSX_PAY_CHECKOUT_METHOD").unwrap_or_else(|_|"contract".into()),"order_reference":order.id,"token":order.token,"amount":order.amount,"description":order.description,"metadata":{"epsx_order_id":order.id,"pricing":order.pricing_snapshot},"expires_in":1800})),Some(&format!("epsx-order-{}",order.id))).await?;
    let pi = response["intent"]["id"].as_str().ok_or_else(unavailable)?;
    let url = response["pay_url"].as_str().ok_or_else(unavailable)?;
    let intent = &response["intent"];
    if intent["merchant_id"] != order.merchant_id
        || intent["payee"] != order.payee
        || intent["chain_id"] != order.chain_id
        || intent["contract_address"] != order.contract_address
        || intent["token_address"] != order.token_address
        || intent["amount"] != order.amount
        || intent["order_reference"] != order.id.to_string()
    {
        return Err(unavailable());
    }
    let origin = reqwest::Url::parse(&c.frontend)
        .map_err(|_| unavailable())?
        .origin();
    let checkout = reqwest::Url::parse(url).map_err(|_| unavailable())?;
    if checkout.origin() != origin || !checkout.path().starts_with("/checkout/cs_") {
        return Err(unavailable());
    }
    sqlx::query("UPDATE pay_purchase_orders SET pay_intent_id=$2,checkout_url=$3 WHERE id=$1 AND (pay_intent_id IS NULL OR pay_intent_id=$2)").bind(order.id).bind(pi).bind(url).execute(state.db_pool.as_ref()).await?;
    Ok(Json(
        json!({"order_id":order.id,"pay_url":url,"status":"pending"}),
    ))
}
#[derive(Default, Deserialize)]
pub struct OrderQuery {
    pub wallet: Option<String>,
    pub status: Option<String>,
    pub plan_id: Option<Uuid>,
    pub offset: Option<i64>,
}

async fn order_rows(
    state: &AppState,
    owner: Option<&str>,
    id: Option<Uuid>,
    q: &OrderQuery,
) -> Result<Vec<Value>> {
    let offset = q.offset.unwrap_or(0);
    if !(0..=1_000_000).contains(&offset) {
        return Err(bad("invalid_offset"));
    }
    // Admin filtering never replaces owner binding on the customer endpoint.
    let rows = sqlx::query_scalar::<_, Value>(r#"
        SELECT jsonb_build_object(
            'order_id',o.id,'plan_id',o.plan_id,'plan_name',coalesce(p.name,o.description),
            'wallet_address',o.wallet_address,'amount',o.amount,'token',o.token,'token_decimals',o.token_decimals,
            'duration_days',o.duration_days,'status',o.status,'payment_id',o.pay_intent_id,
            'chain_id',o.chain_id,'contract_address',o.contract_address,'environment',o.environment,
            'created_at',o.created_at,'first_paid_at',o.first_paid_at,
            'fulfillment_status',CASE WHEN g.superseded THEN 'manual_override'
                WHEN g.active THEN 'granted' WHEN g.reference IS NOT NULL THEN 'revoked'
                WHEN o.status IN ('refunded','expired') THEN o.status ELSE 'pending' END)
        FROM pay_purchase_orders o LEFT JOIN plans p ON p.id=o.plan_id
        LEFT JOIN pay_purchase_grants g ON g.reference='epsx-pay:'||o.id::text
        WHERE ($1::text IS NULL OR o.wallet_address=$1)
          AND ($2::uuid IS NULL OR o.id=$2)
          AND ($3::text IS NULL OR o.wallet_address=lower($3))
          AND ($4::text IS NULL OR o.status=$4)
          AND ($5::uuid IS NULL OR o.plan_id=$5)
        ORDER BY o.created_at DESC,o.id DESC LIMIT 101 OFFSET $6
    "#).bind(owner).bind(id).bind(q.wallet.as_deref()).bind(q.status.as_deref()).bind(q.plan_id).bind(offset)
        .fetch_all(state.db_pool.as_ref()).await?;
    Ok(rows)
}
fn order_page(mut rows: Vec<Value>, offset: i64) -> Json<Value> {
    let more = rows.len() > 100;
    rows.truncate(100);
    Json(json!({"orders":rows,"next_offset":if more {Some(offset+100)} else {None}}))
}
pub async fn list_orders(
    State(state): State<AppState>,
    Extension(user): Extension<OpenIDUserContext>,
    Query(q): Query<OrderQuery>,
) -> Result<Json<Value>> {
    let wallet = user.wallet_address.to_ascii_lowercase();
    Ok(order_page(
        order_rows(&state, Some(&wallet), None, &q).await?,
        q.offset.unwrap_or(0),
    ))
}
pub async fn admin_orders(
    State(state): State<AppState>,
    Query(q): Query<OrderQuery>,
) -> Result<Json<Value>> {
    Ok(order_page(
        order_rows(&state, None, None, &q).await?,
        q.offset.unwrap_or(0),
    ))
}
async fn order_detail(state: &AppState, owner: Option<&str>, id: Uuid) -> Result<Json<Value>> {
    let mut view = order_rows(state, owner, Some(id), &OrderQuery::default())
        .await?
        .pop()
        .ok_or(Error(StatusCode::NOT_FOUND, "order_not_found"))?;
    // Chain evidence comes from Pay; reading an order never grants an entitlement.
    view["payment_status"] = Value::Null;
    view["payment_available"] = json!(false);
    if let Some(pi) = view["payment_id"].as_str().map(str::to_owned) {
        if let Ok(c) = Config::load() {
            if let Ok(payment) = c
                .request(reqwest::Method::GET, &format!("intents/{pi}"), None, None)
                .await
            {
                if payment["order_reference"] == id.to_string()
                    && payment["merchant_id"] == c.merchant
                {
                    view["payment_available"] = json!(true);
                    for (to, from) in [
                        ("payment_status", "status"),
                        ("tx_hash", "tx_hash"),
                        ("verified_block", "verified_block"),
                        ("fee_amount", "fee_amount"),
                    ] {
                        view[to] = payment[from].clone();
                    }
                }
            }
        }
    }
    Ok(Json(view))
}
pub async fn get(
    State(state): State<AppState>,
    Extension(user): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>> {
    order_detail(&state, Some(&user.wallet_address.to_ascii_lowercase()), id).await
}
pub async fn admin_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>> {
    order_detail(&state, None, id).await
}

fn verify(secret: &str, header: &str, body: &[u8], now: i64) -> bool {
    let parts: Vec<_> = header
        .split(',')
        .filter_map(|s| s.split_once('='))
        .collect();
    let times: Vec<_> = parts.iter().filter(|(k, _)| *k == "t").collect();
    if times.len() != 1 {
        return false;
    }
    let Ok(t) = times[0].1.parse::<i64>() else {
        return false;
    };
    if now.abs_diff(t) > 300 {
        return false;
    }
    parts.iter().filter(|(k, _)| *k == "v1").any(|(_, v)| {
        let Ok(sig) = hex::decode(v) else {
            return false;
        };
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
            return false;
        };
        mac.update(format!("{t}.").as_bytes());
        mac.update(body);
        mac.verify_slice(&sig).is_ok()
    })
}
pub async fn webhook(
    State(state): State<AppState>,
    h: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>> {
    let c = Config::load()?;
    let secret = required("EPSX_PAY_WEBHOOK_SECRET_OUTBOUND")?;
    consume_webhook(
        &state.db_pool,
        Some(state.cache.as_ref()),
        &c,
        &secret,
        &h,
        &body,
    )
    .await
}
async fn consume_webhook(
    pool: &sqlx::PgPool,
    cache: Option<&dyn crate::infrastructure::cache::Cache>,
    c: &Config,
    secret: &str,
    h: &HeaderMap,
    body: &[u8],
) -> Result<Json<Value>> {
    if body.len() > 64 * 1024 {
        return Err(bad("request_too_large"));
    }
    let sig = h
        .get("epsx-pay-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(Error(StatusCode::UNAUTHORIZED, "invalid_signature"))?;
    if !verify(secret, sig, body, Utc::now().timestamp()) {
        return Err(Error(StatusCode::UNAUTHORIZED, "invalid_signature"));
    }
    let event: Value = serde_json::from_slice(body).map_err(|_| bad("invalid_event"))?;
    let event_id = event["id"]
        .as_str()
        .filter(|s| s.starts_with("evt_") && s.len() < 100)
        .ok_or_else(|| bad("invalid_event"))?;
    let pi = event["data"]["id"]
        .as_str()
        .filter(|s| s.starts_with("pi_") && s.len() < 100)
        .ok_or_else(|| bad("invalid_payment_id"))?;
    if event["merchant_id"] != c.merchant || event["environment"] != c.environment {
        return Err(bad("wrong_merchant"));
    }
    let verified = c
        .request(reqwest::Method::GET, &format!("intents/{pi}"), None, None)
        .await?;
    let order_id = Uuid::parse_str(
        verified["order_reference"]
            .as_str()
            .ok_or_else(|| bad("missing_order"))?,
    )
    .map_err(|_| bad("invalid_order"))?;
    let mut tx = pool.begin().await?;
    let order: Order = sqlx::query_as("SELECT * FROM pay_purchase_orders WHERE id=$1 FOR UPDATE")
        .bind(order_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(Error(StatusCode::NOT_FOUND, "order_not_found"))?;
    if verified["id"] != pi
        || verified["merchant_id"] != order.merchant_id
        || verified["environment"] != order.environment
        || verified["mode"] != "direct"
        || verified["chain_id"] != order.chain_id
        || verified["contract_address"] != order.contract_address
        || verified["payee"] != order.payee
        || verified["token_address"] != order.token_address
        || verified["amount"] != order.amount
        || order.pay_intent_id.as_ref().is_some_and(|s| s != pi)
    {
        return Err(bad("payment_order_mismatch"));
    }
    let revision = verified["revision"]
        .as_i64()
        .ok_or_else(|| bad("invalid_revision"))?;
    let inserted=sqlx::query("INSERT INTO pay_fulfillment_inbox(event_id,order_id,payment_revision) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(event_id).bind(order.id).bind(revision).execute(&mut *tx).await?.rows_affected();
    if revision < order.payment_revision || inserted == 0 && revision == order.payment_revision {
        tx.commit().await?;
        if let Some(cache) = cache {
            crate::infrastructure::cache::redis_cache::set_perm_invalidated(
                cache,
                &order.wallet_address,
            );
        }
        return Ok(Json(json!({"received":true})));
    }
    let status = verified["status"]
        .as_str()
        .ok_or_else(|| bad("invalid_status"))?;
    if status == "succeeded"
        && (verified["verified_block"].as_i64().is_none()
            || verified["verified_block_hash"].as_str().is_none()
            || verified["tx_hash"].as_str().is_none())
    {
        return Err(bad("missing_chain_proof"));
    }
    let paid_at = order.first_paid_at.unwrap_or_else(Utc::now);
    if status == "succeeded" || order.first_paid_at.is_some() {
        apply_grant(
            &mut tx,
            &order.wallet_address,
            order.plan_id,
            &format!("epsx-pay:{}", order.id),
            paid_at,
            order.duration_days,
            status == "succeeded",
            "epsx_pay",
        )
        .await?;
    }
    sqlx::query("UPDATE pay_purchase_orders SET status=$2,payment_revision=$3,pay_intent_id=$4,first_paid_at=CASE WHEN $2='succeeded' THEN coalesce(first_paid_at,$5) ELSE first_paid_at END WHERE id=$1")
        .bind(order.id).bind(status).bind(revision).bind(pi).bind(paid_at).execute(&mut *tx).await?;
    if let Some(cache) = cache {
        crate::infrastructure::cache::redis_cache::set_perm_invalidated(
            cache,
            &order.wallet_address,
        );
    }
    tx.commit().await?;
    // Permission caches must not keep a refunded or suspended purchase effective.
    if let Some(cache) = cache {
        crate::infrastructure::cache::redis_cache::set_perm_invalidated(
            cache,
            &order.wallet_address,
        );
    }
    Ok(Json(json!({"received":true})))
}

/// One ledger entry per purchase; rebuilding one plan never revokes another plan.
#[allow(clippy::too_many_arguments)]
pub async fn apply_grant(
    tx: &mut sqlx::PgConnection,
    wallet: &str,
    plan: Uuid,
    reference: &str,
    at: DateTime<Utc>,
    days: Option<i64>,
    active: bool,
    source: &str,
) -> std::result::Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!("plan-ledger:{wallet}:{plan}"))
        .execute(&mut *tx)
        .await?;
    let old=sqlx::query("SELECT id,expires_at,is_active,assigned_at FROM wallet_plan_assignments WHERE LOWER(wallet_address)=LOWER($1) AND plan_id=$2 FOR UPDATE").bind(wallet).bind(plan).fetch_optional(&mut *tx).await?;
    let old_expiry = old
        .as_ref()
        .and_then(|r| r.get::<Option<DateTime<Utc>>, _>("expires_at"));
    let old_active = old.as_ref().is_some_and(|r| r.get::<bool, _>("is_active"));
    let old_at = old.as_ref().map(|r| r.get("assigned_at")).unwrap_or(at);
    let current_projection = if old.is_some() {
        json!({"expires_at":old_expiry,"is_active":old_active})
    } else {
        Value::Null
    };
    sqlx::query("INSERT INTO pay_assignment_baselines(wallet_address,plan_id,expires_at,was_active,assigned_at,projection) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT DO NOTHING").bind(wallet).bind(plan).bind(old_expiry).bind(old_active).bind(old_at).bind(&current_projection).execute(&mut *tx).await?;
    let drift: bool = sqlx::query_scalar("SELECT CASE WHEN projection IS NULL THEN false WHEN projection='null'::jsonb THEN $3 ELSE NOT $3 OR (projection->>'expires_at')::timestamptz IS DISTINCT FROM $4 OR (projection->>'is_active')::boolean IS DISTINCT FROM $5 END FROM pay_assignment_baselines WHERE wallet_address=$1 AND plan_id=$2").bind(wallet).bind(plan).bind(old.is_some()).bind(old_expiry).bind(old_active).fetch_one(&mut *tx).await?;
    if drift {
        // A manual absolute assignment is authoritative. Old purchase corrections must
        // not erase it or add those already-replaced grants to it a second time.
        sqlx::query("UPDATE pay_assignment_baselines SET expires_at=$3,was_active=$4,assigned_at=$5 WHERE wallet_address=$1 AND plan_id=$2").bind(wallet).bind(plan).bind(old_expiry).bind(old_active).bind(old_at).execute(&mut *tx).await?;
        sqlx::query(
            "UPDATE pay_purchase_grants SET superseded=true WHERE wallet_address=$1 AND plan_id=$2",
        )
        .bind(wallet)
        .bind(plan)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("INSERT INTO pay_purchase_grants(reference,wallet_address,plan_id,granted_at,duration_days,active,source) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(reference) DO UPDATE SET active=EXCLUDED.active").bind(reference).bind(wallet).bind(plan).bind(at).bind(days).bind(active).bind(source).execute(&mut *tx).await?;
    let baseline = sqlx::query(
        "SELECT * FROM pay_assignment_baselines WHERE wallet_address=$1 AND plan_id=$2",
    )
    .bind(wallet)
    .bind(plan)
    .fetch_one(&mut *tx)
    .await?;
    let mut enabled = baseline.get::<bool, _>("was_active");
    let mut expiry: Option<DateTime<Utc>> = if enabled {
        baseline.get("expires_at")
    } else {
        Some(at)
    };
    let mut assigned: DateTime<Utc> = baseline.get("assigned_at");
    let grants=sqlx::query("SELECT granted_at,duration_days FROM pay_purchase_grants WHERE wallet_address=$1 AND plan_id=$2 AND active AND NOT superseded ORDER BY granted_at,reference").bind(wallet).bind(plan).fetch_all(&mut *tx).await?;
    for grant in grants {
        let at: DateTime<Utc> = grant.get("granted_at");
        let days: Option<i64> = grant.get("duration_days");
        expiry = if enabled && expiry.is_none() {
            None
        } else {
            days.map(|days| expiry.filter(|e| *e > at).unwrap_or(at) + chrono::Duration::days(days))
        };
        assigned = assigned.max(at);
        enabled = true;
    }
    let expiry = expiry.and_then(|e| DateTime::from_timestamp_micros(e.timestamp_micros()));
    let enabled = enabled && expiry.is_none_or(|e| e > Utc::now());
    sqlx::query("INSERT INTO wallet_users(wallet_address,is_active,tier_level,wallet_metadata) VALUES($1,true,'Bronze','{}') ON CONFLICT(wallet_address) DO NOTHING").bind(wallet).execute(&mut *tx).await?;
    if let Some(old) = old {
        sqlx::query("UPDATE wallet_plan_assignments SET assigned_at=$2,expires_at=$3,is_active=$4,payment_reference=$5,updated_at=now() WHERE id=$1").bind(old.get::<Uuid,_>("id")).bind(assigned).bind(expiry).bind(enabled).bind(reference).execute(&mut *tx).await?;
    } else {
        sqlx::query("INSERT INTO wallet_plan_assignments(wallet_address,plan_id,assigned_at,expires_at,is_active,assignment_source,payment_reference,assignment_metadata) VALUES($1,$2,$3,$4,$5,'payment',$6,'{}') ON CONFLICT(wallet_address,plan_id) DO UPDATE SET assigned_at=$3,expires_at=$4,is_active=$5,payment_reference=$6,updated_at=now()")
        .bind(wallet).bind(plan).bind(assigned).bind(expiry).bind(enabled).bind(reference).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE wallet_users SET tier_level=CASE WHEN x.tier>=9 THEN 'Platinum' WHEN x.tier>=6 THEN 'Gold' WHEN x.tier>=3 THEN 'Silver' ELSE 'Bronze' END,updated_at=now() FROM (SELECT coalesce(max(p.tier_level),0) AS tier FROM wallet_plan_assignments a JOIN plans p ON p.id=a.plan_id WHERE LOWER(a.wallet_address)=LOWER($1) AND a.is_active AND (a.expires_at IS NULL OR a.expires_at>now())) x WHERE LOWER(wallet_users.wallet_address)=LOWER($1)").bind(wallet).execute(&mut *tx).await?;
    sqlx::query(
        "UPDATE pay_assignment_baselines SET projection=$3 WHERE wallet_address=$1 AND plan_id=$2",
    )
    .bind(wallet)
    .bind(plan)
    .bind(json!({"expires_at":expiry,"is_active":enabled}))
    .execute(&mut *tx)
    .await?;
    Ok(expiry)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_token_prices() {
        assert_eq!(units("100.50", 6).unwrap(), "100500000");
        assert!(units("0.0000001", 6).is_err());
        assert!(units("0", 18).is_err());
        assert!(units("-1", 18).is_err());
    }
    #[test]
    fn signed_webhooks_bind_payload_and_timestamp() {
        let secret = "whsec_test";
        let body = b"{}";
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(b"1000.{}");
        let sig = format!("t=1000,v1={}", hex::encode(mac.finalize().into_bytes()));
        assert!(verify(secret, &sig, body, 1001));
        assert!(!verify(secret, &sig, body, 1400));
        assert!(!verify(secret, &sig, b"{ }", 1001));
        assert!(!verify(secret, &format!("{sig},t=1000"), body, 1001));
    }
    #[tokio::test]
    #[ignore = "requires migrated isolated EPSX_MERCHANT_CORE database"]
    async fn purchase_grants_are_idempotent_and_reversible_without_losing_other_purchases() {
        let url = std::env::var("EPSX_MERCHANT_CORE").unwrap();
        assert!(reqwest::Url::parse(&url)
            .unwrap()
            .path()
            .starts_with("/epsx_merchant_check_"));
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let wallet = format!("0x{:040x}", Uuid::new_v4().as_u128());
        let plan = Uuid::new_v4();
        let other = Uuid::new_v4();
        let at = Utc::now();
        for id in [plan, other] {
            sqlx::query("INSERT INTO plans(id,name,slug,plan_type,price,currency,billing_cycle) VALUES($1,$2,$2,'subscription',100,'USD','monthly')").bind(id).bind(format!("Merchant test {id}")).execute(&pool).await.unwrap();
        }
        sqlx::query("INSERT INTO wallet_users(wallet_address) VALUES($1)")
            .bind(&wallet)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO wallet_plan_assignments(wallet_address,plan_id,expires_at,is_active) VALUES($1,$2,$3,true),($1,$4,NULL,true)").bind(&wallet).bind(plan).bind(at+chrono::Duration::days(10)).bind(other).execute(&pool).await.unwrap();
        let a = format!("purchase-a-{plan}");
        let b = format!("purchase-b-{plan}");
        let legacy = format!("legacy-{plan}");
        let mut tx = pool.begin().await.unwrap();
        let first = apply_grant(&mut tx, &wallet, plan, &a, at, Some(30), true, "epsx_pay")
            .await
            .unwrap();
        assert!(
            (first.unwrap() - (at + chrono::Duration::days(40)))
                .num_seconds()
                .abs()
                < 1
        );
        assert_eq!(
            first,
            apply_grant(&mut tx, &wallet, plan, &a, at, Some(30), true, "epsx_pay")
                .await
                .unwrap()
        );
        apply_grant(
            &mut tx,
            &wallet,
            plan,
            &b,
            at + chrono::Duration::seconds(1),
            Some(30),
            true,
            "epsx_pay",
        )
        .await
        .unwrap();
        apply_grant(
            &mut tx,
            &wallet,
            plan,
            &legacy,
            at + chrono::Duration::seconds(2),
            Some(30),
            true,
            "legacy",
        )
        .await
        .unwrap();
        let refunded = apply_grant(&mut tx, &wallet, plan, &a, at, Some(30), false, "epsx_pay")
            .await
            .unwrap()
            .unwrap();
        assert!(
            (refunded - (at + chrono::Duration::days(70)))
                .num_seconds()
                .abs()
                < 1
        );
        let restored = apply_grant(&mut tx, &wallet, plan, &a, at, Some(30), true, "epsx_pay")
            .await
            .unwrap()
            .unwrap();
        assert!(
            (restored - (at + chrono::Duration::days(100)))
                .num_seconds()
                .abs()
                < 1
        );
        let unaffected:bool=sqlx::query_scalar("SELECT is_active AND expires_at IS NULL FROM wallet_plan_assignments WHERE wallet_address=$1 AND plan_id=$2").bind(&wallet).bind(other).fetch_one(&mut *tx).await.unwrap();
        assert!(unaffected);
        let manual_expiry = at + chrono::Duration::days(200);
        sqlx::query("UPDATE wallet_plan_assignments SET expires_at=$3,assignment_reason='manual extension' WHERE wallet_address=$1 AND plan_id=$2").bind(&wallet).bind(plan).bind(manual_expiry).execute(&mut *tx).await.unwrap();
        let preserved = apply_grant(&mut tx, &wallet, plan, &a, at, Some(30), false, "epsx_pay")
            .await
            .unwrap()
            .unwrap();
        assert!((preserved - manual_expiry).num_seconds().abs() < 1);
        let future = apply_grant(
            &mut tx,
            &wallet,
            plan,
            &format!("future-{plan}"),
            at,
            Some(30),
            true,
            "epsx_pay",
        )
        .await
        .unwrap()
        .unwrap();
        assert!(
            (future - (manual_expiry + chrono::Duration::days(30)))
                .num_seconds()
                .abs()
                < 1
        );
        tx.commit().await.unwrap();
    }
    #[tokio::test]
    #[ignore = "requires migrated isolated EPSX_MERCHANT_CORE database"]
    async fn webhook_fulfillment_fetches_authenticated_pay_and_handles_reordered_corrections() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        };
        let url = std::env::var("EPSX_MERCHANT_CORE").unwrap();
        assert!(reqwest::Url::parse(&url)
            .unwrap()
            .path()
            .starts_with("/epsx_merchant_check_"));
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let id = Uuid::new_v4();
        let plan = Uuid::new_v4();
        let wallet = format!("0x{:040x}", id.as_u128());
        let pi = format!("pi_{}", id.simple());
        sqlx::query("INSERT INTO plans(id,name,slug,plan_type,price,currency,billing_cycle) VALUES($1,$2,$2,'subscription',100,'USD','monthly')").bind(plan).bind(format!("Fulfillment {id}")).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO pay_purchase_orders(id,wallet_address,plan_id,request_key,request_hash,merchant_id,environment,chain_id,contract_address,payee,token,token_address,token_decimals,amount,duration_days,description,pay_intent_id) VALUES($1,$2,$3,$4,'test','mer_epsx','test',31337,'contract','recipient','USDT','token',6,'100000000',30,'test',$5)").bind(id).bind(&wallet).bind(plan).bind(id.to_string()).bind(&pi).execute(&pool).await.unwrap();
        let payment = Arc::new(Mutex::new(
            json!({"id":pi,"merchant_id":"mer_epsx","environment":"test","mode":"direct","order_reference":id,"chain_id":31337,"contract_address":"contract","payee":"recipient","token_address":"token","amount":"100000000","revision":1,"status":"succeeded","verified_block":100,"verified_block_hash":"hash","tx_hash":"tx"}),
        ));
        let outage = Arc::new(AtomicBool::new(false));
        let data = payment.clone();
        let unavailable = outage.clone();
        let app = axum::Router::new().route(
            "/api/v1/pay/intents/{id}",
            axum::routing::get(move |h: HeaderMap| {
                let data = data.clone();
                let unavailable = unavailable.clone();
                async move {
                    assert_eq!(h.get("authorization").unwrap(), "Bearer isolated-test-key");
                    assert_eq!(h.get("x-pay-environment").unwrap(), "test");
                    if unavailable.load(Ordering::SeqCst) {
                        StatusCode::SERVICE_UNAVAILABLE.into_response()
                    } else {
                        Json(data.lock().unwrap().clone()).into_response()
                    }
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let c = Config {
            base,
            key: "isolated-test-key".into(),
            merchant: "mer_epsx".into(),
            environment: "test".into(),
            payee: "recipient".into(),
            frontend: "http://127.0.0.1:1".into(),
        };
        let secret = "test-endpoint-secret";
        let event=json!({"id":format!("evt_{}",id.simple()),"merchant_id":"mer_epsx","environment":"test","type":"payment.succeeded","data":{"id":pi}}).to_string();
        let timestamp = Utc::now().timestamp();
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(format!("{timestamp}.").as_bytes());
        mac.update(event.as_bytes());
        let mut h = HeaderMap::new();
        h.insert(
            "epsx-pay-signature",
            format!(
                "t={timestamp},v1={}",
                hex::encode(mac.finalize().into_bytes())
            )
            .parse()
            .unwrap(),
        );
        assert!(
            consume_webhook(&pool, None, &c, "wrong-secret", &h, event.as_bytes())
                .await
                .is_err()
        );
        payment.lock().unwrap()["amount"] = json!("1");
        assert!(
            consume_webhook(&pool, None, &c, secret, &h, event.as_bytes())
                .await
                .is_err()
        );
        payment.lock().unwrap()["amount"] = json!("100000000");
        for _ in 0..2 {
            let _ = consume_webhook(&pool, None, &c, secret, &h, event.as_bytes())
                .await
                .unwrap();
        }
        let grants: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pay_purchase_grants WHERE wallet_address=$1 AND active",
        )
        .bind(&wallet)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(grants, 1);
        let expiry: DateTime<Utc> = sqlx::query_scalar(
            "SELECT expires_at FROM wallet_plan_assignments WHERE wallet_address=$1 AND plan_id=$2",
        )
        .bind(&wallet)
        .bind(plan)
        .fetch_one(&pool)
        .await
        .unwrap();
        // A replayed old event retrieves the newer refund and cannot regrant access.
        payment.lock().unwrap()["revision"] = json!(2);
        payment.lock().unwrap()["status"] = json!("refunded");
        let _ = consume_webhook(&pool, None, &c, secret, &h, event.as_bytes())
            .await
            .unwrap();
        let active: bool = sqlx::query_scalar(
            "SELECT is_active FROM wallet_plan_assignments WHERE wallet_address=$1 AND plan_id=$2",
        )
        .bind(&wallet)
        .bind(plan)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!active);
        payment.lock().unwrap()["revision"] = json!(3);
        payment.lock().unwrap()["status"] = json!("verification_required");
        let _ = consume_webhook(&pool, None, &c, secret, &h, event.as_bytes())
            .await
            .unwrap();
        outage.store(true, Ordering::SeqCst);
        assert!(
            consume_webhook(&pool, None, &c, secret, &h, event.as_bytes())
                .await
                .is_err()
        );
        outage.store(false, Ordering::SeqCst);
        payment.lock().unwrap()["revision"] = json!(4);
        payment.lock().unwrap()["status"] = json!("succeeded");
        let _ = consume_webhook(&pool, None, &c, secret, &h, event.as_bytes())
            .await
            .unwrap();
        let restored:DateTime<Utc>=sqlx::query_scalar("SELECT expires_at FROM wallet_plan_assignments WHERE wallet_address=$1 AND plan_id=$2 AND is_active").bind(&wallet).bind(plan).fetch_one(&pool).await.unwrap();
        assert_eq!(expiry, restored);
        // This decision uses durable grants, including inactive ones, without Redis.
        assert!(
            crate::web::middleware::bearer_middleware::has_purchase_grants(&pool, &wallet)
                .await
                .unwrap()
        );
        sqlx::query("UPDATE pay_purchase_grants SET active=false WHERE wallet_address=$1")
            .bind(&wallet)
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            crate::web::middleware::bearer_middleware::has_purchase_grants(&pool, &wallet)
                .await
                .unwrap()
        );
        server.abort();
    }
}
