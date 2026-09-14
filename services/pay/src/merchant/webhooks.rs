use super::{
    auth::{self, Actor},
    bad, hash, id, missing, Platform, Result,
};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use sqlx::Row;
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

pub fn derive_secret(master: &str, context: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(master.as_bytes()).expect("HMAC accepts any key size");
    mac.update(context.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
fn endpoint_secret(p: &Platform, id: &str, version: i32) -> String {
    format!(
        "whsec_{}",
        derive_secret(&p.secret, &format!("webhook:{id}:{version}"))
    )
}
pub fn signature(secret: &str, timestamp: i64, body: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(format!("{timestamp}.").as_bytes());
    mac.update(body);
    hex::encode(mac.finalize().into_bytes())
}
pub fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(a) => {
            let [x, y, z, _] = a.octets();
            !a.is_private()
                && !a.is_loopback()
                && !a.is_link_local()
                && !a.is_multicast()
                && !a.is_unspecified()
                && x != 0
                && x < 224
                && !(x == 100 && (64..=127).contains(&y))
                && !(x == 192 && y == 0)
                && !(x == 198 && (y == 18 || y == 19 || y == 51 && z == 100))
                && !(x == 203 && y == 0 && z == 113)
        }
        IpAddr::V6(a) => {
            if let Some(v4) = a.to_ipv4_mapped() {
                return public_ip(IpAddr::V4(v4));
            }
            let s = a.segments();
            (s[0] & 0xe000) == 0x2000
                && !(s[0] == 0x2001 && (s[1] < 0x200 || s[1] == 0xdb8))
                && s[0] != 0x2002
        }
    }
}
fn endpoint_url(raw: &str) -> std::result::Result<reqwest::Url, &'static str> {
    let u = reqwest::Url::parse(raw).map_err(|_| "invalid_webhook_url")?;
    if u.scheme() != "https"
        || u.port_or_known_default() != Some(443)
        || !u.username().is_empty()
        || u.password().is_some()
        || u.fragment().is_some()
        || u.host_str().is_none()
    {
        return Err("public_https_webhook_required");
    }
    Ok(u)
}
pub async fn destination(
    raw: &str,
) -> std::result::Result<(reqwest::Url, Vec<SocketAddr>), &'static str> {
    let u = endpoint_url(raw)?;
    let host = u.host_str().unwrap().trim_matches(['[', ']']);
    let addresses: Vec<_> =
        tokio::time::timeout(Duration::from_secs(5), tokio::net::lookup_host((host, 443)))
            .await
            .map_err(|_| "dns_timeout")?
            .map_err(|_| "dns_failed")?
            .collect();
    if addresses.is_empty() || addresses.iter().any(|a| !public_ip(a.ip())) {
        return Err("private_webhook_destination_rejected");
    }
    Ok((u, addresses))
}
async fn validate_endpoint(p: &Platform, url: &str) -> Result<()> {
    #[cfg(test)]
    if p.webhook_test_url.is_some()
        && [
            "https://hooks.example.test/pay",
            "https://hooks.example.test/updated",
        ]
        .contains(&url)
    {
        endpoint_url(url).map_err(bad)?;
        return Ok(());
    }
    let _ = p;
    destination(url).await.map_err(bad)?;
    Ok(())
}
pub async fn management(
    p: &Platform,
    a: &Actor,
    method: &str,
    path: &[&str],
    b: &Value,
) -> Result<Value> {
    if matches!(path.first(), Some(&"webhook-endpoints" | &"deliveries")) {
        auth::require_owner(a)?;
    }
    match (method, path) {
        ("POST", ["api-keys"]) => {
            auth::require_owner(a)?;
            let name = b
                .get("name")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty() && s.len() <= 100)
                .ok_or_else(|| bad("invalid_key_name"))?;
            let bytes: [u8; 32] = rand::random();
            let key = format!("epsxpay_{}_{}", a.environment, hex::encode(bytes));
            let key_id = id("key");
            sqlx::query("INSERT INTO pay_merchant_keys(id,merchant_id,environment,name,key_hash,prefix) VALUES($1,$2,$3,$4,$5,$6)").bind(&key_id).bind(&a.merchant_id).bind(&a.environment).bind(name).bind(hash(key.as_bytes())).bind(&key[..24]).execute(&p.db).await?;
            Ok(json!({"id":key_id,"key":key,"shown_once":true}))
        }
        ("GET", ["api-keys"]) => {
            auth::require_owner(a)?;
            let rows=sqlx::query("SELECT id,name,prefix,created_at,revoked_at FROM pay_merchant_keys WHERE merchant_id=$1 AND environment=$2 ORDER BY created_at DESC LIMIT 100").bind(&a.merchant_id).bind(&a.environment).fetch_all(&p.db).await?;
            Ok(
                json!({"items":rows.iter().map(|r|json!({"id":r.get::<String,_>("id"),"name":r.get::<String,_>("name"),"prefix":r.get::<String,_>("prefix"),"revoked":r.get::<Option<chrono::DateTime<chrono::Utc>>,_>("revoked_at").is_some()})).collect::<Vec<_>>()}),
            )
        }
        ("POST", ["api-keys", key, "revoke"]) => {
            auth::require_owner(a)?;
            let rows=sqlx::query("UPDATE pay_merchant_keys SET revoked_at=coalesce(revoked_at,now()) WHERE id=$1 AND merchant_id=$2 AND environment=$3").bind(key).bind(&a.merchant_id).bind(&a.environment).execute(&p.db).await?.rows_affected();
            if rows == 0 {
                return Err(missing());
            }
            Ok(json!({"revoked":true}))
        }
        ("GET", ["links"]) => {
            let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(l) || jsonb_build_object('successful_uses',(SELECT count(*) FROM pay_merchant_intents p WHERE p.link_id=l.id AND p.status IN ('funded','disputed','succeeded','refunded')),'reserved_uses',(SELECT count(*) FROM pay_merchant_intents p WHERE p.link_id=l.id AND p.status IN ('awaiting_payment','verification_required'))) FROM pay_merchant_links l WHERE merchant_id=$1 AND environment=$2 ORDER BY created_at DESC LIMIT 100").bind(&a.merchant_id).bind(&a.environment).fetch_all(&p.db).await?;
            Ok(json!({"items":rows}))
        }
        ("POST", ["links", link, "disable"]) => {
            let rows=sqlx::query("UPDATE pay_merchant_links SET disabled=true WHERE id=$1 AND merchant_id=$2 AND environment=$3").bind(link).bind(&a.merchant_id).bind(&a.environment).execute(&p.db).await?.rows_affected();
            if rows == 0 {
                return Err(missing());
            }
            Ok(json!({"disabled":true}))
        }
        ("POST", ["webhook-endpoints"]) => {
            auth::require_owner(a)?;
            let url = b
                .get("url")
                .and_then(Value::as_str)
                .filter(|s| s.len() <= 2048)
                .ok_or_else(|| bad("invalid_webhook_url"))?;
            validate_endpoint(p, url).await?;
            let endpoint = id("we");
            sqlx::query("INSERT INTO pay_merchant_endpoints(id,merchant_id,environment,url) VALUES($1,$2,$3,$4)").bind(&endpoint).bind(&a.merchant_id).bind(&a.environment).bind(url).execute(&p.db).await?;
            Ok(
                json!({"id":endpoint,"url":url,"signing_secret":endpoint_secret(p,&endpoint,1),"shown_once":true}),
            )
        }
        ("GET", ["webhook-endpoints"]) => {
            let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(e) FROM pay_merchant_endpoints e WHERE merchant_id=$1 AND environment=$2 ORDER BY created_at DESC LIMIT 100").bind(&a.merchant_id).bind(&a.environment).fetch_all(&p.db).await?;
            Ok(json!({"items":rows}))
        }
        ("POST", ["webhook-endpoints", endpoint, "replace"]) => {
            auth::require_owner(a)?;
            let url = super::api::text(b, "url", 2048)?;
            validate_endpoint(p, url).await?;
            let mut tx = p.db.begin().await?;
            let old:Option<String>=sqlx::query_scalar("SELECT id FROM pay_merchant_endpoints WHERE id=$1 AND merchant_id=$2 AND environment=$3 FOR UPDATE").bind(endpoint).bind(&a.merchant_id).bind(&a.environment).fetch_optional(&mut *tx).await?;
            old.ok_or_else(missing)?;
            let replaced: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pay_merchant_endpoints WHERE replaces_id=$1)",
            )
            .bind(endpoint)
            .fetch_one(&mut *tx)
            .await?;
            if replaced {
                return Err(bad("endpoint_already_replaced"));
            }
            let next = id("we");
            sqlx::query("INSERT INTO pay_merchant_endpoints(id,merchant_id,environment,url,replaces_id) VALUES($1,$2,$3,$4,$5)").bind(&next).bind(&a.merchant_id).bind(&a.environment).bind(url).bind(endpoint).execute(&mut *tx).await?;
            sqlx::query("UPDATE pay_merchant_endpoints SET enabled=false WHERE id=$1")
                .bind(endpoint)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(
                json!({"id":next,"url":url,"replaces_id":endpoint,"signing_secret":endpoint_secret(p,&next,1),"shown_once":true}),
            )
        }
        ("POST", ["webhook-endpoints", endpoint, "enable"]) => {
            if sqlx::query("UPDATE pay_merchant_endpoints SET enabled=true WHERE id=$1 AND merchant_id=$2 AND environment=$3 AND NOT EXISTS(SELECT 1 FROM pay_merchant_endpoints n WHERE n.replaces_id=$1)").bind(endpoint).bind(&a.merchant_id).bind(&a.environment).execute(&p.db).await?.rows_affected()==0{return Err(missing());}
            Ok(json!({"enabled":true}))
        }
        ("POST", ["webhook-endpoints", endpoint, "rotate"]) => {
            auth::require_owner(a)?;
            let version:i32=sqlx::query_scalar("UPDATE pay_merchant_endpoints SET previous_version=secret_version,previous_until=now()+interval '24 hours',secret_version=secret_version+1 WHERE id=$1 AND merchant_id=$2 AND environment=$3 RETURNING secret_version").bind(endpoint).bind(&a.merchant_id).bind(&a.environment).fetch_optional(&p.db).await?.ok_or_else(missing)?;
            Ok(
                json!({"id":endpoint,"signing_secret":endpoint_secret(p,endpoint,version),"previous_secret_valid_for_hours":24}),
            )
        }
        ("POST", ["webhook-endpoints", endpoint, "disable"]) => {
            auth::require_owner(a)?;
            if sqlx::query("UPDATE pay_merchant_endpoints SET enabled=false WHERE id=$1 AND merchant_id=$2 AND environment=$3").bind(endpoint).bind(&a.merchant_id).bind(&a.environment).execute(&p.db).await?.rows_affected()==0{return Err(missing())}
            Ok(json!({"disabled":true}))
        }
        ("GET", ["events"]) => {
            let rows:Vec<Value>=sqlx::query_scalar("SELECT payload FROM pay_merchant_events WHERE merchant_id=$1 AND environment=$2 ORDER BY created_at DESC LIMIT 100").bind(&a.merchant_id).bind(&a.environment).fetch_all(&p.db).await?;
            Ok(json!({"items":rows}))
        }
        ("GET", ["events", event]) => {
            let v:Value=sqlx::query_scalar("SELECT payload FROM pay_merchant_events WHERE id=$1 AND merchant_id=$2 AND environment=$3").bind(event).bind(&a.merchant_id).bind(&a.environment).fetch_optional(&p.db).await?.ok_or_else(missing)?;
            Ok(v)
        }
        ("GET", ["deliveries"]) => {
            let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(d) FROM pay_merchant_deliveries d JOIN pay_merchant_events e ON e.id=d.event_id WHERE e.merchant_id=$1 AND e.environment=$2 AND d.created_at>now()-interval '30 days' ORDER BY d.created_at DESC LIMIT 100").bind(&a.merchant_id).bind(&a.environment).fetch_all(&p.db).await?;
            Ok(json!({"items":rows}))
        }
        ("GET", ["deliveries", delivery]) => {
            let row:Value=sqlx::query_scalar("SELECT to_jsonb(d) FROM pay_merchant_deliveries d JOIN pay_merchant_events e ON e.id=d.event_id WHERE d.id=$1 AND e.merchant_id=$2 AND e.environment=$3").bind(delivery).bind(&a.merchant_id).bind(&a.environment).fetch_optional(&p.db).await?.ok_or_else(missing)?;
            let attempts:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(a) FROM pay_merchant_delivery_attempts a WHERE delivery_id=$1 ORDER BY attempt DESC LIMIT 100").bind(delivery).fetch_all(&p.db).await?;
            Ok(json!({"delivery":row,"attempts":attempts}))
        }
        ("POST", ["deliveries", delivery, "replay"]) => {
            let affected=sqlx::query("UPDATE pay_merchant_deliveries d SET status='pending',next_attempt_at=now(),retry_until=now()+interval '72 hours' FROM pay_merchant_events e,pay_merchant_endpoints w WHERE d.id=$1 AND d.event_id=e.id AND d.endpoint_id=w.id AND e.merchant_id=$2 AND e.environment=$3 AND e.created_at>now()-interval '30 days' AND w.enabled AND (d.leased_until IS NULL OR d.leased_until<now())").bind(delivery).bind(&a.merchant_id).bind(&a.environment).execute(&p.db).await?.rows_affected();
            if affected == 0 {
                return Err(missing());
            }
            Ok(json!({"status":"pending"}))
        }
        _ => Err(missing()),
    }
}
async fn send(p: &Platform, row: &sqlx::postgres::PgRow) -> std::result::Result<u16, &'static str> {
    let raw: String = row.get("url");
    #[cfg(test)]
    let target = if raw == "https://hooks.example.test/pay" {
        p.webhook_test_url.as_deref()
    } else {
        None
    };
    #[cfg(not(test))]
    let target: Option<&str> = None;
    let (url, addresses) = if let Some(target) = target {
        let u = reqwest::Url::parse(target).map_err(|_| "test_url_invalid")?;
        if u.host_str() != Some("127.0.0.1") {
            return Err("test_url_must_be_loopback");
        }
        (u, vec![])
    } else {
        destination(&raw).await?
    };
    let mut builder = reqwest::Client::builder()
        .user_agent("EPSX-Pay-Webhooks/1.0")
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10));
    if !addresses.is_empty() {
        builder = builder.resolve_to_addrs(url.host_str().unwrap(), &addresses)
    }
    let client = builder.build().map_err(|_| "http_client_failed")?;
    let endpoint: String = row.get("endpoint_id");
    let timestamp = chrono::Utc::now().timestamp();
    let body = row.get::<Value, _>("payload").to_string();
    let secret = endpoint_secret(p, &endpoint, row.get("secret_version"));
    let mut sig = format!(
        "t={timestamp},v1={}",
        signature(&secret, timestamp, body.as_bytes())
    );
    if row
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("previous_until")
        .is_some_and(|t| t > chrono::Utc::now())
    {
        if let Some(version) = row.get::<Option<i32>, _>("previous_version") {
            sig.push_str(&format!(
                ",v1={}",
                signature(
                    &endpoint_secret(p, &endpoint, version),
                    timestamp,
                    body.as_bytes()
                )
            ));
        }
    }
    let response = client
        .post(url)
        .header("content-type", "application/json")
        .header("epsx-pay-signature", sig)
        .header("epsx-pay-event-id", row.get::<String, _>("event_id"))
        .body(body)
        .send()
        .await
        .map_err(|_| "connection_failed")?;
    Ok(response.status().as_u16())
}
pub async fn tick(p: &Platform) -> Result<bool> {
    let mut tx = p.db.begin().await?;
    sqlx::query("UPDATE pay_merchant_deliveries SET status='exhausted' WHERE status='pending' AND retry_until<=now() AND (leased_until IS NULL OR leased_until<now())").execute(&mut *tx).await?;
    let row=sqlx::query("SELECT d.id,d.attempts,d.event_id,d.endpoint_id,w.url,w.enabled,w.secret_version,w.previous_version,w.previous_until,e.payload FROM pay_merchant_deliveries d JOIN pay_merchant_endpoints w ON w.id=d.endpoint_id JOIN pay_merchant_events e ON e.id=d.event_id WHERE d.status='pending' AND d.retry_until>now() AND d.next_attempt_at<=now() AND (d.leased_until IS NULL OR d.leased_until<now()) AND (e.event_type='payment.verification_required' OR EXISTS(SELECT 1 FROM pay_merchant_intents i JOIN pay_merchant_checkpoints c ON c.chain_id=i.chain_id AND c.contract_address=i.contract_address WHERE i.id=e.intent_id AND c.healthy AND c.checked_at>now()-interval '60 seconds')) ORDER BY d.next_attempt_at LIMIT 1 FOR UPDATE OF d SKIP LOCKED").fetch_optional(&mut *tx).await?;
    let Some(row) = row else {
        tx.commit().await?;
        return Ok(false);
    };
    let delivery: String = row.get("id");
    if !row.get::<bool, _>("enabled") {
        sqlx::query("UPDATE pay_merchant_deliveries SET status='disabled' WHERE id=$1")
            .bind(&delivery)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(true);
    }
    let attempt = row.get::<i32, _>("attempts") + 1;
    sqlx::query("UPDATE pay_merchant_deliveries SET leased_until=now()+interval '30 seconds',attempts=$2 WHERE id=$1").bind(&delivery).bind(attempt).execute(&mut *tx).await?;
    tx.commit().await?;
    let result = send(p, &row).await;
    let code = result.as_ref().ok().copied().map(i32::from);
    let ok = code.is_some_and(|c| (200..300).contains(&c));
    let error = if ok {
        None
    } else {
        Some(result.err().unwrap_or("non_success_status"))
    };
    let delay = (15_i64.saturating_mul(2_i64.pow((attempt - 1).min(8) as u32))).min(3600);
    let mut tx = p.db.begin().await?;
    let current: i32 =
        sqlx::query_scalar("SELECT attempts FROM pay_merchant_deliveries WHERE id=$1 FOR UPDATE")
            .bind(&delivery)
            .fetch_one(&mut *tx)
            .await?;
    if current != attempt {
        tx.commit().await?;
        return Ok(true);
    }
    sqlx::query("INSERT INTO pay_merchant_delivery_attempts(delivery_id,attempt,http_status,error) VALUES($1,$2,$3,$4)").bind(&delivery).bind(attempt).bind(code).bind(error).execute(&mut *tx).await?;
    sqlx::query("UPDATE pay_merchant_deliveries SET leased_until=NULL,last_status=$3,last_error=$4,status=CASE WHEN $5 THEN 'delivered' WHEN retry_until<=now() THEN 'exhausted' ELSE 'pending' END,delivered_at=CASE WHEN $5 THEN now() ELSE delivered_at END,next_attempt_at=now()+make_interval(secs=>$6) WHERE id=$1 AND attempts=$2").bind(delivery).bind(attempt).bind(code).bind(error).bind(ok).bind(delay as f64).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(true)
}
pub fn spawn(p: Platform) {
    tokio::spawn(async move {
        loop {
            match tick(&p).await {
                Ok(true) => {}
                Ok(false) => tokio::time::sleep(Duration::from_secs(2)).await,
                Err(e) => {
                    tracing::error!(error=?e,"merchant webhook worker unavailable");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_internal_and_special_networks() {
        for s in [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.1.1",
            "192.168.1.2",
            "169.254.169.254",
            "100.64.1.1",
            "0.0.0.0",
            "224.1.1.1",
            "198.18.1.1",
            "192.0.2.1",
            "::1",
            "fc00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
            "2001:db8::1",
        ] {
            assert!(!public_ip(s.parse().unwrap()), "{s}")
        }
        for s in ["1.1.1.1", "8.8.8.8", "2606:4700:4700::1111"] {
            assert!(public_ip(s.parse().unwrap()), "{s}")
        }
    }
    #[test]
    fn signatures_bind_time_body_and_endpoint() {
        assert_ne!(signature("secret", 1, b"a"), signature("secret", 2, b"a"));
        assert_ne!(signature("secret", 1, b"a"), signature("secret", 1, b"b"));
        assert_ne!(
            derive_secret("master", "endpoint1"),
            derive_secret("master", "endpoint2")
        );
    }
    #[test]
    fn only_https_without_userinfo_or_redirect_targets() {
        for u in [
            "http://example.com/hook",
            "https://a:b@example.com/hook",
            "https://example.com:8080/hook",
            "file:///tmp/test",
            "https://example.com/#secret",
        ] {
            assert!(endpoint_url(u).is_err())
        }
        assert!(endpoint_url("https://example.com/hook").is_ok())
    }
}
