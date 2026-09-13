//! Real PostgreSQL + Anvil integration. Explicit opt-in; only rehearsal database names.
use super::*;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use epsx_service_auth::{AccessTokenVerifier, VerifiedPrincipal, VerifyError};
use serde_json::{json, Value};
use tower::ServiceExt;
const PAYER: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
const PAYEE: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";
const ADMIN: &str = "0x3c44cdddb6a900fa2b585dd299e03d12fa4293bc";
struct TestVerifier;
#[async_trait::async_trait]
impl AccessTokenVerifier for TestVerifier {
    async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
        let wallet = match token {
            "payer" => PAYER,
            "payee" => PAYEE,
            "admin" => ADMIN,
            _ => return Err(VerifyError::Validation),
        };
        Ok(VerifiedPrincipal {
            subject: wallet.into(),
            wallet_address: wallet.into(),
            audience: if token == "admin" {
                "epsx-admin"
            } else {
                "epsx-pay"
            }
            .into(),
            permissions: if token == "admin" {
                vec!["admin:payments:manage".into(), "admin:payments:view".into()]
            } else {
                vec![]
            },
        })
    }
}
async fn request(
    app: &Router,
    method: &str,
    path: &str,
    actor: &str,
    body: Value,
    key: &str,
) -> (StatusCode, Value) {
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("authorization", format!("Bearer {actor}"))
                .header("content-type", "application/json")
                .header("idempotency-key", key)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let b = to_bytes(r.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&b).unwrap_or(Value::Null))
}
async fn send(s: &AppState, app: &Router, response: &Value, actor: &str) {
    let c = s.native_chain.as_ref().unwrap();
    let hash = c
        .rpc("eth_sendTransaction", json!([response["transaction"]]))
        .await
        .unwrap();
    // Anvil may return the hash before mining the transaction. Wait for its
    // first receipt before mining the remaining confirmation blocks.
    let mut receipt = Value::Null;
    for _ in 0..100 {
        receipt = c
            .rpc("eth_getTransactionReceipt", json!([hash]))
            .await
            .unwrap();
        if !receipt.is_null() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(
        receipt["status"], "0x1",
        "transaction must be mined successfully"
    );
    let id = response["operation"]["id"].as_str().unwrap();
    let (status, ack) = request(
        app,
        "POST",
        &format!("/api/v1/pay/operations/{id}/confirm"),
        actor,
        json!({"tx_hash":hash}),
        "confirm",
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED, "{ack}");
    assert_eq!(ack["status"], "pending");
    if c.confirmations > 1 {
        native_reconcile::tick(s, c).await.unwrap();
        let (_, before) = request(
            app,
            "GET",
            &format!("/api/v1/pay/operations/{id}"),
            actor,
            json!({}),
            "",
        )
        .await;
        assert_ne!(
            before["status"], "confirmed",
            "one receipt must not bypass confirmations"
        );
        c.rpc("anvil_mine", json!([c.confirmations - 1]))
            .await
            .unwrap();
    }
    for _ in 0..30 {
        native_reconcile::tick(s, c).await.unwrap();
        let (_, op) = request(
            app,
            "GET",
            &format!("/api/v1/pay/operations/{id}"),
            actor,
            json!({}),
            "",
        )
        .await;
        if op["status"] == "confirmed" {
            return;
        }
        assert_ne!(op["status"], "failed", "{op}");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("operation was not confirmed");
}

async fn catch_up(s: &AppState, c: &native_chain::Chain) {
    for _ in 0..100 {
        native_reconcile::tick(s, c).await.unwrap();
        let healthy: bool = sqlx::query_scalar("SELECT healthy FROM pay_v1_chain_checkpoints WHERE chain_id=$1 AND contract_address=$2")
            .bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase())
            .fetch_one(&s.db).await.unwrap();
        if healthy {
            return;
        }
    }
    panic!("scanner must catch up in bounded 10-block batches");
}
#[tokio::test]
#[ignore = "requires EPSX_NATIVE_TEST_DATABASE_URL and a locally deployed Anvil contract"]
async fn escrow_lifecycle_retries_reorg_and_owner_boundaries() {
    let db = sqlx::PgPool::connect(
        &std::env::var("EPSX_NATIVE_TEST_DATABASE_URL").expect("explicit test DB"),
    )
    .await
    .unwrap();
    let name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(
        name.starts_with("epsx_native_check_"),
        "refusing non-rehearsal DB"
    );
    let mut chain = native_chain::Chain::from_env().unwrap().unwrap();
    chain.validate().await.unwrap();
    let mut wrong_treasury = chain.clone();
    wrong_treasury.treasury = native_chain::address(PAYEE).unwrap();
    assert!(wrong_treasury
        .validate()
        .await
        .unwrap_err()
        .to_string()
        .contains("treasury mismatch"));
    chain.confirmations = 3;
    let c = Arc::new(chain);
    assert_eq!(c.chain_id, 31337);
    let s = AppState {
        db,
        native_chain: Some(c.clone()),
        chain_id: 31337,
        provider: Arc::new(RwLock::new(None)),
        escrow_contract: c.contract.to_string(),
    };
    catch_up(&s, &c).await;
    let app = application(s.clone(), Arc::new(TestVerifier));
    let prefix = uuid::Uuid::new_v4().to_string();
    let (status, link) = request(
        &app,
        "POST",
        "/api/v1/pay/links",
        "payee",
        json!({"amount":"10000000000000000","token":"BNB","max_uses":4}),
        &format!("{prefix}-link"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{link}");
    let slug = link["link"]["slug"].as_str().unwrap();
    let path = format!("/api/v1/pay/links/{slug}/redeem");
    let key = format!("{prefix}-redeem");
    let (a, b) = tokio::join!(
        request(&app, "POST", &path, "payer", json!({}), &key),
        request(&app, "POST", &path, "payer", json!({}), &key)
    );
    assert_eq!(a.0, StatusCode::OK, "{}", a.1);
    assert_eq!(a, b);
    let id = a.1["intent"]["id"].as_str().unwrap();
    let uses: i32 = sqlx::query_scalar("SELECT current_uses FROM pay_v1_links WHERE slug=$1")
        .bind(slug)
        .fetch_one(&s.db)
        .await
        .unwrap();
    assert_eq!(uses, 1);
    let (status, _) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/intents/{id}"),
        "forged-wallet",
        json!({}),
        "",
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = request(
        &app,
        "POST",
        &format!("/api/v1/pay/intents/{id}/deposit"),
        "payee",
        json!({}),
        "foreign",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, deposit) = request(
        &app,
        "POST",
        &format!("/api/v1/pay/intents/{id}/deposit"),
        "payer",
        json!({}),
        &format!("{prefix}-deposit"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{deposit}");
    // A successful unrelated transaction is not evidence of an escrow deposit.
    let unrelated = c
        .rpc(
            "eth_sendTransaction",
            json!([{"from":PAYER,"to":PAYEE,"value":"0x0"}]),
        )
        .await
        .unwrap();
    let bad_id = deposit["operation"]["id"].as_str().unwrap();
    let (status, _) = request(
        &app,
        "POST",
        &format!("/api/v1/pay/operations/{bad_id}/confirm"),
        "payer",
        json!({"tx_hash":unrelated}),
        "",
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    c.rpc("anvil_mine", json!([3])).await.unwrap();
    native_reconcile::tick(&s, &c).await.unwrap();
    let (_, bad) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/operations/{bad_id}"),
        "payer",
        json!({}),
        "",
    )
    .await;
    assert_eq!(bad["status"], "failed");
    let (_, unpaid) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/intents/{id}"),
        "payer",
        json!({}),
        "",
    )
    .await;
    assert_eq!(unpaid["status"], "pending");
    // Simulate RPC outage, then recover using the same persisted checkpoint.
    let rpc_url = std::env::var("PAY_ESCROW_RPC_URL").unwrap();
    std::env::set_var("PAY_ESCROW_RPC_URL", "http://127.0.0.1:39888");
    let unavailable = native_chain::Chain::from_env().unwrap().unwrap();
    std::env::set_var("PAY_ESCROW_RPC_URL", rpc_url);
    assert!(native_reconcile::checked_tick(&s, &unavailable)
        .await
        .is_err());
    let (status, _) = request(
        &app,
        "POST",
        &format!("/api/v1/pay/intents/{id}/deposit"),
        "payer",
        json!({}),
        &format!("{prefix}-during-outage"),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    native_reconcile::checked_tick(&s, &c).await.unwrap();
    let (status, retry) = request(
        &app,
        "POST",
        &format!("/api/v1/pay/intents/{id}/deposit"),
        "payer",
        json!({}),
        &format!("{prefix}-valid-retry"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{retry}");
    send(&s, &app, &retry, "payer").await;
    let (_, d) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/intents/{id}"),
        "payer",
        json!({}),
        "",
    )
    .await;
    assert_eq!(d["status"], "active");
    let snapshot = c.rpc("evm_snapshot", json!([])).await.unwrap();
    let (status, release) = request(
        &app,
        "POST",
        &format!("/api/v1/pay/escrows/{id}/release"),
        "payer",
        json!({}),
        &format!("{prefix}-release"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{release}");
    send(&s, &app, &release, "payer").await;
    let (_, d) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/intents/{id}"),
        "payee",
        json!({}),
        "",
    )
    .await;
    assert_eq!(d["status"], "released");
    assert_eq!(d["fee_amount"], "30000000000000");
    // Restart/replay is idempotent; reverting the settled block invalidates the projection.
    native_reconcile::tick(&s, &c).await.unwrap();
    c.rpc("evm_revert", json!([snapshot])).await.unwrap();
    c.rpc("anvil_mine", json!([3])).await.unwrap();
    native_reconcile::tick(&s, &c).await.unwrap();
    let (_, d) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/intents/{id}"),
        "payer",
        json!({}),
        "",
    )
    .await;
    assert_eq!(d["status"], "verification_required");
    catch_up(&s, &c).await;
    let (_, d) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/intents/{id}"),
        "payer",
        json!({}),
        "",
    )
    .await;
    assert_eq!(d["status"], "active");
    assert_eq!(d["fee_amount"], "0");
    let (status, dispute) = request(
        &app,
        "POST",
        &format!("/api/v1/pay/escrows/{id}/dispute"),
        "payee",
        json!({}),
        &format!("{prefix}-dispute"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{dispute}");
    send(&s, &app, &dispute, "payee").await;
    let (status, resolution) = request(
        &app,
        "POST",
        &format!("/api/v1/pay/escrows/{id}/resolve"),
        "admin",
        json!({"to_payee":false}),
        &format!("{prefix}-resolve"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{resolution}");
    send(&s, &app, &resolution, "admin").await;
    let (_, d) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/intents/{id}"),
        "payer",
        json!({}),
        "",
    )
    .await;
    assert_eq!(d["status"], "refunded");
    assert_eq!(d["fee_amount"], "0");
    // A SQL-only status change does not become a financial success on the API.
    sqlx::query(
        "UPDATE pay_v1_deals SET status='released',fee_amount='30000000000000' WHERE id=$1",
    )
    .bind(id)
    .execute(&s.db)
    .await
    .unwrap();
    let (_, unproven) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/intents/{id}"),
        "payer",
        json!({}),
        "",
    )
    .await;
    assert_eq!(unproven["status"], "verification_required");
    assert_eq!(unproven["available_actions"], json!([]));
    sqlx::query("UPDATE pay_v1_deals SET status='refunded',fee_amount='0' WHERE id=$1")
        .bind(id)
        .execute(&s.db)
        .await
        .unwrap();
    let (status, _) = request(
        &app,
        "POST",
        &format!("/api/v1/pay/escrows/{id}/refund"),
        "payee",
        json!({}),
        &format!("{prefix}-repeat"),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    // Admin controls are also wallet transactions with receipt/event confirmation.
    assert_controls(&s, &app, &prefix, true).await;
    assert_controls(&s, &app, &prefix, false).await;
    // A valid webhook signature is only a hint; even duplicate hints do not move funds.
    std::env::set_var(
        "EPSX_PAY_WEBHOOK_SECRET",
        "native-rehearsal-only-32-byte-secret-value",
    );
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let payload=json!({"event_id":format!("{prefix}-forged-webhook"),"tx_hash":format!("0x{}","a".repeat(64)),"event_type":"released"}).to_string();
    let mut mac =
        Hmac::<Sha256>::new_from_slice(b"native-rehearsal-only-32-byte-secret-value").unwrap();
    mac.update(payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    for _ in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/pay/webhooks/on-chain")
                    .header("content-type", "application/json")
                    .header("x-pay-webhook-signature", &signature)
                    .body(Body::from(payload.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }
    let (_, unchanged) = request(
        &app,
        "GET",
        &format!("/api/v1/pay/intents/{id}"),
        "payer",
        json!({}),
        "",
    )
    .await;
    assert_eq!(unchanged["status"], "refunded");
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM pay_v1_chain_hints WHERE event_id=$1")
            .bind(format!("{prefix}-forged-webhook"))
            .fetch_one(&s.db)
            .await
            .unwrap();
    assert_eq!(count, 1);
}

async fn assert_controls(s: &AppState, app: &Router, prefix: &str, paused: bool) {
    let c = s.native_chain.as_ref().unwrap();
    let (status, operation) = request(
        app,
        "POST",
        "/api/v1/admin/pay/contract/pause",
        "admin",
        json!({"paused":paused}),
        &format!("{prefix}-pause-{paused}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{operation}");
    let hash = c
        .rpc("eth_sendTransaction", json!([operation["transaction"]]))
        .await
        .unwrap();
    let id = operation["operation"]["id"].as_str().unwrap();
    let (status, _) = request(
        app,
        "POST",
        &format!("/api/v1/admin/pay/contract/operations/{id}/confirm"),
        "admin",
        json!({"tx_hash":hash}),
        "unused",
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    c.rpc("anvil_mine", json!([c.confirmations])).await.unwrap();
    for _ in 0..30 {
        native_reconcile::tick(s, c).await.unwrap();
        let (_, state) = request(
            app,
            "GET",
            &format!("/api/v1/admin/pay/contract/operations/{id}"),
            "admin",
            json!({}),
            "",
        )
        .await;
        if state["status"] == "confirmed" {
            assert_eq!(c.deposits_paused().await.unwrap(), paused);
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("pause transaction was not confirmed")
}
