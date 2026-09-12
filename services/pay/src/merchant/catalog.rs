//! Wallet-owned catalog. Checkout terms are copied under the product row lock.
use super::{
    api,
    auth::{self, Actor},
    bad, id, missing, Platform, Result,
};
use alloy::{primitives::U256, sol_types::SolCall};
use axum::http::HeaderMap;
use serde_json::{json, Value};
use sqlx::{PgConnection, Row};

fn units(input: &str, decimals: u32) -> Result<String> {
    let mut parts = input.split('.');
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next().unwrap_or_default();
    if whole.is_empty()
        || parts.next().is_some()
        || fraction.len() > decimals as usize
        || !whole
            .bytes()
            .chain(fraction.bytes())
            .all(|b| b.is_ascii_digit())
        || input.len() > 80
    {
        return Err(bad("invalid_token_price"));
    }
    let raw = format!(
        "{whole}{fraction}{}",
        "0".repeat(decimals as usize - fraction.len())
    );
    let raw = raw.trim_start_matches('0');
    super::chain::amount(raw)?;
    Ok(raw.into())
}
fn terms(p: &Platform, a: &Actor, b: &Value) -> Result<Value> {
    let name = api::text(b, "name", 100)?.trim();
    if name.is_empty() {
        return Err(bad("invalid_product_name"));
    }
    let description = b.get("description").and_then(Value::as_str).unwrap_or("");
    if description.len() > 2000 {
        return Err(bad("description_too_long"));
    }
    let duration = match b.get("duration_days") {
        None | Some(Value::Null) => None,
        Some(v) => Some(
            v.as_i64()
                .filter(|v| (1..=365000).contains(v))
                .ok_or_else(|| bad("invalid_duration"))?,
        ),
    };
    let input = b["prices"]
        .as_object()
        .filter(|m| !m.is_empty())
        .ok_or_else(|| bad("prices_required"))?;
    let n = api::network(p, &a.environment)?;
    let mut prices = json!({});
    for (symbol, v) in input {
        if !["USDT", "USDC"].contains(&symbol.as_str()) {
            return Err(bad("unsupported_token"));
        }
        let token = n
            .tokens
            .get(symbol)
            .ok_or_else(|| bad("unsupported_token"))?;
        prices[symbol] = json!(units(
            v.as_str().ok_or_else(|| bad("invalid_token_price"))?,
            token.decimals.into()
        )?);
    }
    Ok(
        json!({"name":name,"description":description,"duration_days":duration,"prices":prices,"enabled":b.get("enabled").and_then(Value::as_bool).unwrap_or(true)}),
    )
}
pub async fn public(
    p: &Platform,
    h: &HeaderMap,
    method: &str,
    path: &[&str],
    b: &Value,
) -> Option<Result<Value>> {
    match(method,path) {
        ("GET",["catalog",merchant])=>Some(async {
            let env=auth::environment(h)?;
            let m:Value=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'name',name,'owner',owner) FROM pay_merchants WHERE id=$1").bind(merchant).fetch_optional(&p.db).await?.ok_or_else(missing)?;
            let items:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(p) FROM pay_merchant_products p WHERE merchant_id=$1 AND environment=$2 AND enabled ORDER BY created_at DESC LIMIT 100").bind(merchant).bind(&env).fetch_all(&p.db).await?;
            Ok(json!({"merchant":m,"environment":env,"items":items}))
        }.await),
        ("GET",["catalog","products",product])=>Some(async {
            let v:Value=sqlx::query_scalar("SELECT to_jsonb(p)||jsonb_build_object('merchant_name',m.name) FROM pay_merchant_products p JOIN pay_merchants m ON m.id=p.merchant_id WHERE p.id=$1 AND p.enabled AND p.environment=$2").bind(product).bind(auth::environment(h)?).fetch_optional(&p.db).await?.ok_or_else(missing)?;
            Ok(v)
        }.await),
        ("POST",["products",product,"checkouts"])=>Some(checkout(p,h,product,b).await),
        _=>None,
    }
}
async fn checkout(p: &Platform, h: &HeaderMap, product: &str, b: &Value) -> Result<Value> {
    let guest = h
        .get("x-pay-guest-id")
        .and_then(|s| s.to_str().ok())
        .filter(|s| {
            (32..=128).contains(&s.len())
                && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
        .ok_or_else(|| bad("guest_id_required"))?;
    auth::rate(p, &format!("product:{product}"), 60).await?;
    let env = auth::environment(h)?;
    let scope = format!("product:{product}:{env}:{}", super::hash(guest.as_bytes()));
    let key = api::key(h)?;
    let mut tx = p.db.begin().await?;
    if let Some(v) = api::start(&mut tx, &scope, &key, b).await? {
        return Ok(api::checkout_response(p, v));
    }
    let row=sqlx::query("SELECT p.*,m.owner FROM pay_merchant_products p JOIN pay_merchants m ON m.id=p.merchant_id WHERE p.id=$1 AND p.environment=$2 FOR UPDATE OF p").bind(product).bind(&env).fetch_optional(&mut *tx).await?.ok_or_else(missing)?;
    if !row.get::<bool, _>("enabled") {
        return Err(bad("package_not_for_sale"));
    }
    let token = api::text(b, "token", 8)?;
    let prices: Value = row.get("prices");
    let amount = prices[token]
        .as_str()
        .ok_or_else(|| bad("unsupported_token"))?;
    let actor = Actor {
        merchant_id: row.get("merchant_id"),
        wallet: row.get("owner"),
        environment: env,
        owner_session: false,
        admin: false,
    };
    let body = json!({"mode":"direct","payment_method":"transfer","token":token,"amount":amount,"description":row.get::<String,_>("name")});
    let mut d = api::insert(
        p,
        &mut tx,
        &actor,
        &body,
        None,
        chrono::Utc::now() + chrono::Duration::minutes(30),
    )
    .await?;
    d.checkout_snapshot["product_id"] = json!(product);
    d.checkout_snapshot["description"] = json!(row.get::<String, _>("description"));
    d.checkout_snapshot["duration_days"] = json!(row.get::<Option<i32>, _>("duration_days"));
    d.checkout_snapshot["product_revision"] = json!(row.get::<i64, _>("revision"));
    sqlx::query("UPDATE pay_merchant_intents SET checkout_snapshot=$2 WHERE id=$1")
        .bind(&d.id)
        .bind(&d.checkout_snapshot)
        .execute(&mut *tx)
        .await?;
    let v = json!({"intent":api::public_payment(&d,true),"checkout_id":d.checkout_id});
    api::end(&mut tx, &scope, &key, b, &v).await?;
    tx.commit().await?;
    Ok(api::checkout_response(p, v))
}
pub async fn private(
    p: &Platform,
    a: &Actor,
    h: &HeaderMap,
    method: &str,
    path: &[&str],
    b: &Value,
) -> Option<Result<Value>> {
    if !matches!(path.first(), Some(&"products" | &"overview"))
        && path != ["merchants", "me", "update"]
        && path != ["merchants", "me"]
    {
        return None;
    }
    Some(private_inner(p, a, h, method, path, b).await)
}
async fn private_inner(
    p: &Platform,
    a: &Actor,
    h: &HeaderMap,
    method: &str,
    path: &[&str],
    b: &Value,
) -> Result<Value> {
    if !(method == "GET" && path == ["merchants", "me"]) {
        auth::require_owner(a)?;
    }
    match(method,path){
        ("GET",["merchants","me"])=>{
            let m:Value=sqlx::query_scalar("SELECT jsonb_build_object('merchant_id',id,'name',name,'wallet',owner) FROM pay_merchants WHERE id=$1").bind(&a.merchant_id).fetch_optional(&p.db).await?.ok_or_else(missing)?;
            let mut profile=json!(a);profile["name"]=m["name"].clone();Ok(profile)
        }
        ("POST",["merchants","me","update"])=>{
            let name=api::text(b,"name",100)?.trim();
            if name.is_empty(){return Err(bad("invalid_merchant_name"));}
            sqlx::query("UPDATE pay_merchants SET name=$2 WHERE id=$1 AND owner=$3").bind(&a.merchant_id).bind(name).bind(&a.wallet).execute(&p.db).await?;
            Ok(json!({"name":name}))
        }
        ("GET",["products"])=>{
            let items:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(p) FROM pay_merchant_products p WHERE merchant_id=$1 AND environment=$2 ORDER BY created_at DESC LIMIT 100").bind(&a.merchant_id).bind(&a.environment).fetch_all(&p.db).await?;
            Ok(json!({"items":items}))
        }
        ("GET",["products",product])=>sqlx::query_scalar("SELECT to_jsonb(p) FROM pay_merchant_products p WHERE id=$1 AND merchant_id=$2 AND environment=$3").bind(product).bind(&a.merchant_id).bind(&a.environment).fetch_optional(&p.db).await?.ok_or_else(missing),
        ("POST",["products"])|("POST",["products",_,"update"])=>{
            let v=terms(p,a,b)?;
            let key=api::key(h)?;
            let scope=format!("catalog:{}:{}:{}",a.merchant_id,a.environment,path.join("/"));
            let mut tx=p.db.begin().await?;
            if let Some(v)=api::start(&mut tx,&scope,&key,b).await?{return Ok(v);}
            let pid=if path.len()==1 {id("pkg")} else {path[1].to_string()};
            let row:Value=if path.len()==1 {
                sqlx::query_scalar("INSERT INTO pay_merchant_products(id,merchant_id,environment,name,description,prices,duration_days,enabled) VALUES($1,$2,$3,$4,$5,$6,$7,$8) RETURNING to_jsonb(pay_merchant_products)").bind(&pid).bind(&a.merchant_id).bind(&a.environment).bind(v["name"].as_str()).bind(v["description"].as_str()).bind(&v["prices"]).bind(v["duration_days"].as_i64().map(|n|n as i32)).bind(v["enabled"].as_bool()).fetch_one(&mut *tx).await?
            }else{
                sqlx::query_scalar("UPDATE pay_merchant_products SET name=$4,description=$5,prices=$6,duration_days=$7,enabled=$8,revision=revision+1,updated_at=now() WHERE id=$1 AND merchant_id=$2 AND environment=$3 RETURNING to_jsonb(pay_merchant_products)").bind(&pid).bind(&a.merchant_id).bind(&a.environment).bind(v["name"].as_str()).bind(v["description"].as_str()).bind(&v["prices"]).bind(v["duration_days"].as_i64().map(|n|n as i32)).bind(v["enabled"].as_bool()).fetch_optional(&mut *tx).await?.ok_or_else(missing)?
            };
            api::end(&mut tx,&scope,&key,b,&row).await?;tx.commit().await?;Ok(row)
        }
        ("GET",["overview"])=>overview(p,a).await,
        _=>Err(missing())
    }
}
async fn overview(p: &Platform, a: &Actor) -> Result<Value> {
    let mut tx = p.db.begin().await?;
    let mut invoices: Vec<api::Payment> = sqlx::query_as(
        "SELECT * FROM pay_merchant_intents WHERE merchant_id=$1 AND environment=$2",
    )
    .bind(&a.merchant_id)
    .bind(&a.environment)
    .fetch_all(&mut *tx)
    .await?;
    let mut totals = std::collections::BTreeMap::<(String, i32), (U256, U256, U256, u64)>::new();
    for invoice in &mut invoices {
        api::verify_projection(&mut tx, invoice).await?;
        let total = totals
            .entry((invoice.token.clone(), invoice.token_decimals))
            .or_default();
        if invoice.status == "succeeded" {
            total.0 = total
                .0
                .checked_add(super::chain::amount(&invoice.amount)?)
                .ok_or_else(|| bad("amount_overflow"))?;
            total.3 += 1;
        }
        if ["succeeded", "refunded"].contains(&invoice.status.as_str()) {
            let fee = invoice
                .fee_amount
                .parse::<U256>()
                .map_err(|_| bad("invalid_fee"))?;
            total.1 = total
                .1
                .checked_add(fee)
                .ok_or_else(|| bad("amount_overflow"))?;
        }
        if invoice.deposit_address.is_some() {
            let value = settlement(p, &mut tx, invoice).await?;
            let amount = value["collectable_amount"]
                .as_str()
                .unwrap_or("0")
                .parse::<U256>()
                .map_err(|_| bad("invalid_amount"))?;
            total.2 = total
                .2
                .checked_add(amount)
                .ok_or_else(|| bad("amount_overflow"))?;
        }
    }
    let items:Vec<_>=totals.into_iter().map(|((token,decimals),(paid,fees,ready,payments))|json!({"token":token,"decimals":decimals,"paid":paid.to_string(),"fees":fees.to_string(),"ready":ready.to_string(),"payments":payments})).collect();
    Ok(json!({"items":items,"fees_estimated_until_collection":true}))
}
pub async fn settlement(p: &Platform, tx: &mut PgConnection, d: &api::Payment) -> Result<Value> {
    let mut v = api::public_payment(d, false);
    let hash:Option<String>=sqlx::query_scalar("SELECT tx_hash FROM pay_merchant_chain_events WHERE intent_id=$1 AND canonical AND status='collected' ORDER BY block_number DESC,log_index DESC LIMIT 1").bind(&d.id).fetch_optional(tx).await?;
    let balance = if let Some(receiver) = &d.deposit_address {
        let n = api::network(p, &d.environment)?;
        api::healthy(p, n, "qr").await?;
        super::chain::balanceOfCall::abi_decode_returns(
            &n.call(
                super::chain::address(&d.token_address)?,
                super::chain::balanceOfCall {
                    owner: super::chain::address(receiver)?,
                }
                .abi_encode(),
            )
            .await?,
        )
        .map_err(|_| bad("invalid_balance_response"))?
    } else {
        U256::ZERO
    };
    let ready = balance - super::chain::fee(balance, 50);
    v["collectable_amount"] = json!(ready.to_string());
    if balance.is_zero() {
        if let Some(actions) = v["available_actions"].as_array_mut() {
            actions.retain(|a| a != "collect");
        }
    }
    v["settlement_status"] = json!(if !balance.is_zero() {
        "ready"
    } else if hash.is_some() || d.deposit_address.is_none() && d.status == "succeeded" {
        "settled"
    } else {
        "not_ready"
    });
    v["settlement_tx_hash"] = json!(hash);
    Ok(v)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_prices() {
        assert_eq!(units("5.25", 6).unwrap(), "5250000");
        assert_eq!(units("0.000001", 6).unwrap(), "1");
        for p in ["0", "-1", "1e2", "1.0000001", "1..2", "NaN"] {
            assert!(units(p, 6).is_err(), "{p}");
        }
    }
}
