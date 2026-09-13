//! Explicit opt-in integration against isolated PostgreSQL and Anvil. No production bypasses.
use super::*;
use alloy::{
    primitives::{Address, U256},
    sol_types::SolValue,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
};
use epsx_service_auth::{VerifiedPrincipal, VerifyError};
use serde_json::json;
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
const PAYER: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
const MERCHANT: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";
const ADMIN: &str = "0x3c44cdddb6a900fa2b585dd299e03d12fa4293bc";
async fn qr_cases(p: &Platform, n: &chain::Network) {
    n.rpc("anvil_mine", json!(["0x4"])).await.unwrap();
    drain(p, n, "direct").await;
    drain(p, n, "qr").await;
    let mut previous = String::new();
    for (symbol, case) in [
        ("USDT", "paid"),
        ("USDC", "paid"),
        ("USDT", "wrong"),
        ("USDT", "late"),
        ("USDT", "reorg"),
    ] {
        let key = id("qr-test");
        let checkout=request(p,"POST","intents",Some("merchant"),json!({"mode":"direct","payment_method":"transfer","token":symbol,"amount":"5000000000000000000","expires_in":60}),&key,None).await.unwrap();
        let pi = checkout["intent"]["id"].as_str().unwrap();
        let receiver = checkout["intent"]["deposit_address"].as_str().unwrap();
        assert_ne!(receiver, previous);
        previous = receiver.into();
        let cs = checkout["checkout_id"].as_str().unwrap();
        let guest = request(
            p,
            "GET",
            &format!("checkout-sessions/{cs}"),
            None,
            json!({}),
            "",
            Some(cs),
        )
        .await
        .unwrap();
        assert_eq!(guest["available_actions"], json!([]));
        let route = format!("checkout-sessions/{cs}/prepare-transfer");
        assert!(
            request(p, "POST", &route, None, json!({"payer":PAYER}), "", None)
                .await
                .is_err()
        );
        let prepared = request(
            p,
            "POST",
            &route,
            None,
            json!({"payer":PAYER,"amount":"1","deposit_address":ADMIN,"chain_id":56}),
            "",
            Some(cs),
        )
        .await
        .unwrap();
        assert_eq!(prepared["transaction_parameters"]["chainId"], "0x7a69");
        assert_eq!(
            prepared["transaction_parameters"]["to"]
                .as_str()
                .unwrap()
                .to_lowercase(),
            n.tokens[symbol].address.to_lowercase()
        );
        assert_eq!(prepared["payment"]["amount"], "5000000000000000000");
        assert_eq!(prepared["payment"]["deposit_address"], receiver);
        assert!(prepared["payment"]["payer"].is_null());
        assert!(prepared.get("approval_transaction").is_none());
        let duplicate = request(
            p,
            "POST",
            &route,
            None,
            json!({"payer":PAYER}),
            "",
            Some(cs),
        )
        .await
        .unwrap();
        assert_eq!(
            prepared["transaction_parameters"],
            duplicate["transaction_parameters"]
        );
        let empty = "0x0000000000000000000000000000000000000011";
        let native = n
            .rpc("eth_getBalance", json!([PAYER, "latest"]))
            .await
            .unwrap();
        n.rpc("anvil_setBalance", json!([PAYER, "0x0"]))
            .await
            .unwrap();
        assert_eq!(
            request(
                p,
                "POST",
                &route,
                None,
                json!({"payer":PAYER}),
                "",
                Some(cs)
            )
            .await
            .unwrap_err()
            .1,
            "insufficient_gas_balance"
        );
        n.rpc("anvil_setBalance", json!([PAYER, native]))
            .await
            .unwrap();
        assert_eq!(
            request(
                p,
                "POST",
                &route,
                None,
                json!({"payer":empty}),
                "",
                Some(cs)
            )
            .await
            .unwrap_err()
            .1,
            "insufficient_token_balance"
        );
        sqlx::query(
            "UPDATE pay_merchant_intents SET expires_at=now()-interval '1 second' WHERE id=$1",
        )
        .bind(pi)
        .execute(&p.db)
        .await
        .unwrap();
        assert_eq!(
            request(
                p,
                "POST",
                &route,
                None,
                json!({"payer":PAYER}),
                "",
                Some(cs)
            )
            .await
            .unwrap_err()
            .1,
            "checkout_not_payable"
        );
        sqlx::query("UPDATE pay_merchant_intents SET expires_at=$2 WHERE id=$1")
            .bind(pi)
            .bind(
                chrono::DateTime::parse_from_rfc3339(guest["expires_at"].as_str().unwrap())
                    .unwrap(),
            )
            .execute(&p.db)
            .await
            .unwrap();
        assert!(checkout["intent"]["payment_uri"]
            .as_str()
            .unwrap()
            .contains("@31337/transfer?address="));
        let snapshot = n.rpc("evm_snapshot", json!([])).await.unwrap();
        if case == "late" {
            n.rpc("evm_increaseTime", json!([120])).await.unwrap();
        }
        let amount = if case == "wrong" {
            U256::from(4) * U256::from(10).pow(U256::from(18))
        } else {
            U256::from(5) * U256::from(10).pow(U256::from(18))
        };
        let data = [
            &alloy::primitives::keccak256("transfer(address,uint256)")[..4],
            &(Address::from_str(receiver).unwrap(), amount).abi_encode(),
        ]
        .concat();
        if case == "paid" {
            assert_eq!(
                prepared["transaction_parameters"]["data"],
                format!("0x{}", hex::encode(&data))
            );
        }
        let hash=n.rpc("eth_sendTransaction",json!([{"from":PAYER,"to":n.tokens[symbol].address,"data":format!("0x{}",hex::encode(data))}])).await.unwrap();
        receipt(n, &hash).await;
        // Do not grant before the required confirmations.
        reconcile::checked_tick(p, n, "qr").await.unwrap();
        let before = request(
            p,
            "GET",
            &format!("intents/{pi}"),
            Some("merchant"),
            json!({}),
            "",
            None,
        )
        .await
        .unwrap();
        assert_ne!(before["status"], "succeeded");
        n.rpc("anvil_mine", json!(["0x4"])).await.unwrap();
        drain(p, n, "qr").await;
        let state = request(
            p,
            "GET",
            &format!("intents/{pi}"),
            Some("merchant"),
            json!({}),
            "",
            None,
        )
        .await
        .unwrap();
        assert_eq!(
            state["status"],
            if ["wrong", "late"].contains(&case) {
                "verification_required"
            } else {
                "succeeded"
            },
            "{state}"
        );
        let revision = state["revision"].clone();
        drain(p, n, "qr").await;
        assert_eq!(
            request(
                p,
                "GET",
                &format!("intents/{pi}"),
                Some("merchant"),
                json!({}),
                "",
                None
            )
            .await
            .unwrap()["revision"],
            revision
        );
        if case == "paid" {
            let op = request(
                p,
                "POST",
                &format!("intents/{pi}/collect"),
                Some("merchant"),
                json!({}),
                &id("collect"),
                None,
            )
            .await
            .unwrap();
            transact(p, n, &op, Some("merchant"), None, "qr").await;
            let op = request(
                p,
                "POST",
                &format!("intents/{pi}/refund"),
                Some("merchant"),
                json!({}),
                &id("refund"),
                None,
            )
            .await
            .unwrap();
            transact(p, n, &op, Some("merchant"), None, "qr").await;
            assert_eq!(
                request(
                    p,
                    "GET",
                    &format!("intents/{pi}"),
                    Some("merchant"),
                    json!({}),
                    "",
                    None
                )
                .await
                .unwrap()["status"],
                "refunded"
            );
        }
        if case == "reorg" {
            assert_eq!(n.rpc("evm_revert", json!([snapshot])).await.unwrap(), true);
            drain(p, n, "qr").await;
            let state = request(
                p,
                "GET",
                &format!("intents/{pi}"),
                Some("merchant"),
                json!({}),
                "",
                None,
            )
            .await
            .unwrap();
            assert_ne!(state["status"], "succeeded");
            assert!(state["verified_block"].is_null());
        }
        // Keep subsequent expiry fixtures aligned to the real clock.
        n.rpc("evm_setTime", json!([chrono::Utc::now().timestamp()]))
            .await
            .unwrap();
        n.rpc("anvil_mine", json!(["0x4"])).await.unwrap();
        drain(p, n, "direct").await;
        drain(p, n, "qr").await;
    }
}
struct Verifier;
#[async_trait::async_trait]
impl AccessTokenVerifier for Verifier {
    async fn verify(&self, token: &str) -> std::result::Result<VerifiedPrincipal, VerifyError> {
        let wallet = match token {
            "merchant" => MERCHANT,
            "other" => PAYER,
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
    p: &Platform,
    method: &str,
    path: &str,
    actor: Option<&str>,
    body: Value,
    key: &str,
    cs: Option<&str>,
) -> std::result::Result<Value, Error> {
    let mut r = Request::builder()
        .uri(format!("/api/v1/pay/{path}"))
        .method(method)
        .header("x-pay-api-version", "2026-09-08")
        .header("idempotency-key", key)
        .header(
            "x-pay-guest-id",
            "test-guest-012345678901234567890123456789",
        );
    if let Some(a) = actor {
        r = r.header("authorization", format!("Bearer {a}"));
    }
    if let Some(cs) = cs {
        r = r.header("x-pay-checkout-token", capability(p, cs));
    }
    api::handle(p, r.body(Body::from(body.to_string())).unwrap()).await
}
async fn deploy(n: &chain::Network, file: &str, name: &str, args: Vec<u8>) -> Address {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/contracts/out")
        .join(file)
        .join(format!("{name}.json"));
    let artifact: Value = serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
    let code = artifact["bytecode"]["object"].as_str().unwrap();
    let data = format!("0x{}{}", code.trim_start_matches("0x"), hex::encode(args));
    let hash = n
        .rpc(
            "eth_sendTransaction",
            json!([{"from":PAYER,"data":data,"gas":"0x989680"}]),
        )
        .await
        .unwrap();
    let receipt = receipt(n, &hash).await;
    assert_eq!(receipt["status"], "0x1", "{receipt}");
    Address::from_str(receipt["contractAddress"].as_str().unwrap()).unwrap()
}
async fn receipt(n: &chain::Network, hash: &Value) -> Value {
    for _ in 0..100 {
        let r = n
            .rpc("eth_getTransactionReceipt", json!([hash]))
            .await
            .unwrap();
        if !r.is_null() {
            return r;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("Anvil receipt timeout")
}
async fn drain(p: &Platform, n: &chain::Network, mode: &str) {
    for _ in 0..20 {
        reconcile::checked_tick(p, n, mode).await.unwrap();
        let mut conn = p.db.acquire().await.unwrap();
        let ready:bool=sqlx::query_scalar("SELECT healthy FROM pay_merchant_checkpoints WHERE chain_id=$1 AND contract_address=$2").bind(n.chain_id as i64).bind(n.contract(mode).address.to_string().to_ascii_lowercase()).fetch_one(&mut *conn).await.unwrap();
        if ready {
            return;
        }
    }
    panic!("scanner did not catch up")
}
async fn transact(
    p: &Platform,
    n: &chain::Network,
    op: &Value,
    actor: Option<&str>,
    cs: Option<&str>,
    mode: &str,
) {
    if op["approval_transaction"].is_object() {
        let hash = n
            .rpc("eth_sendTransaction", json!([op["approval_transaction"]]))
            .await
            .unwrap();
        assert_eq!(receipt(n, &hash).await["status"], "0x1");
    }
    let hash = n
        .rpc("eth_sendTransaction", json!([op["transaction_parameters"]]))
        .await
        .unwrap();
    assert_eq!(receipt(n, &hash).await["status"], "0x1");
    let ack = request(
        p,
        "POST",
        &format!("operations/{}/confirm", op["id"].as_str().unwrap()),
        actor,
        json!({"tx_hash":hash}),
        "ack",
        cs,
    )
    .await
    .unwrap();
    assert_eq!(ack["status"], "pending");
    reconcile::checked_tick(p, n, mode).await.unwrap();
    let before = request(
        p,
        "GET",
        &format!("operations/{}", op["id"].as_str().unwrap()),
        actor,
        json!({}),
        "",
        cs,
    )
    .await
    .unwrap();
    assert_ne!(before["status"], "confirmed");
    n.rpc("anvil_mine", json!([2])).await.unwrap();
    drain(p, n, mode).await;
    let after = request(
        p,
        "GET",
        &format!("operations/{}", op["id"].as_str().unwrap()),
        actor,
        json!({}),
        "",
        cs,
    )
    .await
    .unwrap();
    assert_eq!(after["status"], "confirmed", "{after}");
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires EPSX_MERCHANT_TEST_DATABASE and isolated Anvil at EPSX_MERCHANT_TEST_RPC"]
async fn merchant_payments_guest_isolation_webhooks_reorg_and_recovery() {
    let database = std::env::var("EPSX_MERCHANT_TEST_DATABASE").unwrap();
    assert!(reqwest::Url::parse(&database)
        .unwrap()
        .path()
        .starts_with("/epsx_merchant_check_"));
    let db = sqlx::PgPool::connect(&database).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../../migrations/20260908000000_create_merchant_platform.sql"
    ))
    .execute(&db)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../../migrations/20260908000001_add_merchant_contract_controls.sql"
    ))
    .execute(&db)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../../migrations/20260909000000_add_qr_checkout.sql"
    ))
    .execute(&db)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../../migrations/20260909010000_add_merchant_catalog.sql"
    ))
    .execute(&db)
    .await
    .unwrap();

    let mut n = chain::Network {
        qr: None,
        environment: "test".into(),
        chain_id: 31337,
        rpc_url: std::env::var("EPSX_MERCHANT_TEST_RPC").unwrap(),
        archive_rpc_url: None,
        scan_blocks: std::env::var("EPSX_MERCHANT_TEST_SCAN_BLOCKS")
            .ok()
            .map(|value| value.parse().unwrap())
            .unwrap_or(crate::native_chain::LOG_SCAN_BLOCKS),
        admin: Address::from_str(ADMIN).unwrap(),
        treasury: Address::from_str(ADMIN).unwrap(),
        direct: chain::Contract {
            address: Address::ZERO,
            deployment_block: 0,
        },
        escrow: chain::Contract {
            address: Address::ZERO,
            deployment_block: 0,
        },
        tokens: Default::default(),
        confirmations: 3,
    };
    crate::native_chain::validate_scan_blocks(n.scan_blocks).unwrap();
    assert_eq!(
        crate::native_chain::number(&n.rpc("eth_chainId", json!([])).await.unwrap()).unwrap(),
        31337
    );
    let usdt = deploy(&n, "DealEscrow.t.sol", "TestCoin", vec![]).await;
    let usdc = deploy(&n, "DealEscrow.t.sol", "TestCoin", vec![]).await;
    let tokens = vec![usdt, usdc];
    let args = (n.admin, n.treasury, tokens).abi_encode_params();
    n.direct.address = deploy(&n, "MerchantPayments.sol", "DirectPayments", args.clone()).await;
    n.escrow.address = deploy(&n, "MerchantPayments.sol", "MerchantEscrow", args).await;
    n.direct.deployment_block =
        crate::native_chain::number(&n.rpc("eth_blockNumber", json!([])).await.unwrap()).unwrap()
            - 1;
    n.escrow.deployment_block = n.direct.deployment_block + 1;
    n.qr = Some(chain::Contract {
        address: deploy(
            &n,
            "QRCheckout.sol",
            "QRCheckout",
            (n.treasury,).abi_encode_params(),
        )
        .await,
        deployment_block: n.escrow.deployment_block + 1,
    });
    n.tokens.insert(
        "BNB".into(),
        crate::native_chain::Token {
            address: Address::ZERO.to_string(),
            decimals: 18,
        },
    );
    for (name, address) in [("USDT", usdt), ("USDC", usdc)] {
        n.tokens.insert(
            name.into(),
            crate::native_chain::Token {
                address: address.to_string(),
                decimals: 18,
            },
        );
        for wallet in [PAYER, MERCHANT] {
            let data = [
                &alloy::primitives::keccak256("mint(address,uint256)")[..4],
                &(
                    Address::from_str(wallet).unwrap(),
                    U256::from(10).pow(U256::from(24)),
                )
                    .abi_encode(),
            ]
            .concat();
            n.rpc(
                "eth_sendTransaction",
                json!([{"from":PAYER,"to":address,"data":format!("0x{}",hex::encode(data))}]),
            )
            .await
            .unwrap();
        }
    }
    let mut wrong_decimals = n.clone();
    wrong_decimals.tokens.get_mut("USDT").unwrap().decimals = 6;
    for mode in ["direct", "escrow", "qr"] {
        assert!(wrong_decimals
            .validate(mode)
            .await
            .unwrap_err()
            .to_string()
            .contains("decimals"));
    }
    let failed = Arc::new(AtomicBool::new(true));
    let seen = Arc::new(Mutex::new(Vec::<(String, Vec<u8>)>::new()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/hook", listener.local_addr().unwrap());
    let f = failed.clone();
    let received = seen.clone();
    let app = Router::new().route(
        "/hook",
        post(
            move |headers: axum::http::HeaderMap, body: axum::body::Bytes| {
                let f = f.clone();
                let received = received.clone();
                async move {
                    received.lock().unwrap().push((
                        headers["epsx-pay-signature"].to_str().unwrap().into(),
                        body.to_vec(),
                    ));
                    if f.load(Ordering::SeqCst) {
                        StatusCode::SERVICE_UNAVAILABLE
                    } else {
                        StatusCode::NO_CONTENT
                    }
                }
            },
        ),
    );
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let p = Platform {
        db: db.clone(),
        verifier: Arc::new(Verifier),
        chains: Arc::new(vec![n.clone()]),
        secret: Arc::new("isolated-merchant-integration-master-0000000000".into()),
        webhook_test_url: Some(url),
    };
    let prefix = id("test");
    for actor in ["merchant", "other", "admin"] {
        request(
            &p,
            "POST",
            "merchants",
            Some(actor),
            json!({"name":actor}),
            "",
            None,
        )
        .await
        .unwrap();
    }
    catalog_cases(&p, &n).await;
    qr_cases(&p, &n).await;
    let merchant = request(
        &p,
        "GET",
        "merchants/me",
        Some("merchant"),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    let mid = merchant["merchant_id"].as_str().unwrap();
    let key = request(
        &p,
        "POST",
        "api-keys",
        Some("merchant"),
        json!({"name":prefix}),
        "",
        None,
    )
    .await
    .unwrap();
    let api_key = key["key"].as_str().unwrap();
    assert!(request(
        &p,
        "GET",
        "merchants/me",
        Some(api_key),
        json!({}),
        "",
        None
    )
    .await
    .is_ok());
    assert!(request(
        &p,
        "POST",
        "api-keys",
        Some(api_key),
        json!({"name":"forbidden"}),
        "",
        None
    )
    .await
    .is_err());
    let endpoint = id("we");
    sqlx::query("INSERT INTO pay_merchant_endpoints(id,merchant_id,environment,url) VALUES($1,$2,'test','https://hooks.example.test/pay')").bind(&endpoint).bind(mid).execute(&db).await.unwrap();
    drain(&p, &n, "direct").await;
    drain(&p, &n, "escrow").await;
    let link=request(&p,"POST","links",Some(api_key),json!({"mode":"direct","token":"BNB","amount":"10000000000000000","max_uses":1,"expires_in":3600}),&format!("{prefix}-link"),None).await.unwrap();
    let lid = link["id"].as_str().unwrap();
    let redeem = format!("links/{lid}/checkouts");
    let (a, b) = tokio::join!(
        request(&p, "POST", &redeem, None, json!({}), "same-attempt", None),
        request(&p, "POST", &redeem, None, json!({}), "same-attempt", None)
    );
    let a = a.unwrap();
    assert_eq!(a, b.unwrap());
    assert!(request(
        &p,
        "POST",
        &redeem,
        None,
        json!({}),
        "different-attempt",
        None
    )
    .await
    .is_err());
    let cs = a["checkout_id"].as_str().unwrap();
    let pi = a["intent"]["id"].as_str().unwrap();
    assert!(request(
        &p,
        "GET",
        &format!("checkout-sessions/{cs}"),
        None,
        json!({}),
        "",
        None
    )
    .await
    .is_err());
    assert!(request(
        &p,
        "GET",
        &format!("intents/{pi}"),
        Some("other"),
        json!({}),
        "",
        None
    )
    .await
    .is_err());
    let op = request(
        &p,
        "POST",
        &format!("checkout-sessions/{cs}/pay"),
        None,
        json!({"payer":PAYER}),
        "pay-once",
        Some(cs),
    )
    .await
    .unwrap();
    // Model reconciliation holding the payment while a buyer polls its operation.
    // Polling must wait without locking the operation first, or confirmation deadlocks.
    let mut confirmation = db.begin().await.unwrap();
    sqlx::query("SELECT id FROM pay_merchant_intents WHERE id=$1 FOR UPDATE")
        .bind(pi)
        .fetch_one(&mut *confirmation)
        .await
        .unwrap();
    let polling_platform = p.clone();
    let polling_cs = cs.to_owned();
    let operation_path = format!("operations/{}", op["id"].as_str().unwrap());
    let polling = tokio::spawn(async move {
        request(
            &polling_platform,
            "GET",
            &operation_path,
            None,
            json!({}),
            "",
            Some(&polling_cs),
        )
        .await
    });
    let mut waiting = false;
    for _ in 0..100 {
        waiting = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE datname=current_database() AND wait_event_type='Lock' AND query LIKE 'SELECT * FROM pay_merchant_intents WHERE id=%')")
            .fetch_one(&db).await.unwrap();
        if waiting {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(waiting, "operation poll did not reach the payment lock");
    sqlx::query("SET LOCAL lock_timeout='250ms'")
        .execute(&mut *confirmation)
        .await
        .unwrap();
    let operation_lock =
        sqlx::query("SELECT id FROM pay_merchant_operations WHERE id=$1 FOR UPDATE")
            .bind(op["id"].as_str().unwrap())
            .fetch_one(&mut *confirmation)
            .await;
    confirmation.rollback().await.unwrap();
    assert!(
        operation_lock.is_ok(),
        "poll locked operation before payment: {operation_lock:?}"
    );
    polling.await.unwrap().unwrap();
    let mismatch = request(
        &p,
        "POST",
        &format!("checkout-sessions/{cs}/pay"),
        None,
        json!({"payer":ADMIN}),
        "wrong-payer",
        Some(cs),
    )
    .await;
    assert!(mismatch.is_err());
    transact(&p, &n, &op, None, Some(cs), "direct").await;
    let paid = request(
        &p,
        "GET",
        &format!("intents/{pi}"),
        Some(api_key),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    assert_eq!(paid["status"], "succeeded");
    assert_eq!(paid["fee_amount"], "50000000000000");
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM pay_merchant_events WHERE intent_id=$1")
            .bind(pi)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(count, 1);
    drain(&p, &n, "direct").await;
    assert_eq!(
        count,
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM pay_merchant_events WHERE intent_id=$1")
            .bind(pi)
            .fetch_one(&db)
            .await
            .unwrap()
    );
    assert!(webhooks::tick(&p).await.unwrap());
    assert!(!seen.lock().unwrap().is_empty());
    let delivery:String=sqlx::query_scalar("SELECT d.id FROM pay_merchant_deliveries d JOIN pay_merchant_events e ON e.id=d.event_id WHERE e.intent_id=$1 LIMIT 1").bind(pi).fetch_one(&db).await.unwrap();
    assert!(request(
        &p,
        "GET",
        &format!("deliveries/{delivery}"),
        Some("other"),
        json!({}),
        "",
        None
    )
    .await
    .is_err());
    failed.store(false, Ordering::SeqCst);
    sqlx::query("UPDATE pay_merchant_deliveries SET next_attempt_at=now() WHERE id=$1")
        .bind(&delivery)
        .execute(&db)
        .await
        .unwrap();
    assert!(webhooks::tick(&p).await.unwrap());
    let status: String =
        sqlx::query_scalar("SELECT status FROM pay_merchant_deliveries WHERE id=$1")
            .bind(&delivery)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(status, "delivered");
    // Rotation signs with both current and previous secrets; replay keeps the event ID.
    let rotated = request(
        &p,
        "POST",
        &format!("webhook-endpoints/{endpoint}/rotate"),
        Some("merchant"),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    request(
        &p,
        "POST",
        &format!("deliveries/{delivery}/replay"),
        Some("merchant"),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    assert!(webhooks::tick(&p).await.unwrap());
    {
        let seen = seen.lock().unwrap();
        let (header, body) = seen.last().unwrap();
        assert_eq!(header.matches("v1=").count(), 2);
        let timestamp = header
            .split(',')
            .next()
            .unwrap()
            .trim_start_matches("t=")
            .parse::<i64>()
            .unwrap();
        assert!(header.contains(&webhooks::signature(
            rotated["signing_secret"].as_str().unwrap(),
            timestamp,
            body
        )));
        assert_eq!(
            serde_json::from_slice::<Value>(body).unwrap()["id"],
            serde_json::from_slice::<Value>(&seen[0].1).unwrap()["id"]
        );
    }
    // A process crash after leasing allows a later worker to resume. Expired retry windows do not send.
    sqlx::query("UPDATE pay_merchant_deliveries SET status='pending',leased_until=now()-interval '1 second',next_attempt_at=now(),retry_until=now()-interval '1 second' WHERE id=$1").bind(&delivery).execute(&db).await.unwrap();
    let before = seen.lock().unwrap().len();
    assert!(!webhooks::tick(&p).await.unwrap());
    assert_eq!(before, seen.lock().unwrap().len());
    request(
        &p,
        "POST",
        &format!("deliveries/{delivery}/replay"),
        Some("merchant"),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    sqlx::query(
        "UPDATE pay_merchant_deliveries SET leased_until=now()-interval '1 second' WHERE id=$1",
    )
    .bind(&delivery)
    .execute(&db)
    .await
    .unwrap();
    assert!(webhooks::tick(&p).await.unwrap());
    assert_eq!(
        op,
        request(
            &p,
            "POST",
            &format!("checkout-sessions/{cs}/pay"),
            None,
            json!({"payer":PAYER}),
            "pay-once",
            Some(cs)
        )
        .await
        .unwrap()
    );
    let snap = n.rpc("evm_snapshot", json!([])).await.unwrap();
    let refund = request(
        &p,
        "POST",
        &format!("intents/{pi}/refunds"),
        Some(api_key),
        json!({}),
        "refund-once",
        None,
    )
    .await
    .unwrap();
    transact(&p, &n, &refund, Some(api_key), None, "direct").await;
    let refunded = request(
        &p,
        "GET",
        &format!("intents/{pi}"),
        Some(api_key),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    assert_eq!(refunded["status"], "refunded");
    assert_eq!(refunded["fee_amount"], "50000000000000");
    n.rpc("evm_revert", json!([snap])).await.unwrap();
    reconcile::checked_tick(&p, &n, "direct").await.unwrap();
    let invalid = request(
        &p,
        "GET",
        &format!("intents/{pi}"),
        Some(api_key),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    assert_eq!(invalid["status"], "verification_required");
    drain(&p, &n, "direct").await;
    assert_eq!(
        request(
            &p,
            "GET",
            &format!("intents/{pi}"),
            Some(api_key),
            json!({}),
            "",
            None
        )
        .await
        .unwrap()["status"],
        "succeeded"
    );
    for token in ["USDT", "USDC"] {
        for mode in ["direct", "escrow"] {
            drain(&p, &n, mode).await;
            let order=request(&p,"POST","intents",Some(api_key),json!({"mode":mode,"token":token,"amount":"10000","order_reference":format!("{prefix}-{mode}-{token}")}),&format!("{prefix}-{mode}-{token}"),None).await.unwrap();
            let cs = order["checkout_id"].as_str().unwrap();
            let pi = order["intent"]["id"].as_str().unwrap();
            let op = request(
                &p,
                "POST",
                &format!(
                    "checkout-sessions/{cs}/{}",
                    if mode == "direct" { "pay" } else { "deposit" }
                ),
                None,
                json!({"payer":PAYER}),
                "fund",
                Some(cs),
            )
            .await
            .unwrap();
            transact(&p, &n, &op, None, Some(cs), mode).await;
            if mode == "escrow" {
                let dispute = request(
                    &p,
                    "POST",
                    &format!("checkout-sessions/{cs}/dispute"),
                    None,
                    json!({"payer":PAYER}),
                    "dispute",
                    Some(cs),
                )
                .await
                .unwrap();
                transact(&p, &n, &dispute, None, Some(cs), mode).await;
                let resolve = request(
                    &p,
                    "POST",
                    &format!("escrows/{pi}/resolve-refund"),
                    Some("admin"),
                    json!({}),
                    "resolve",
                    None,
                )
                .await
                .unwrap();
                transact(&p, &n, &resolve, Some("admin"), None, mode).await;
            } else {
                let refund = request(
                    &p,
                    "POST",
                    &format!("intents/{pi}/refunds"),
                    Some(api_key),
                    json!({}),
                    "refund",
                    None,
                )
                .await
                .unwrap();
                transact(&p, &n, &refund, Some(api_key), None, mode).await;
            }
            let result = request(
                &p,
                "GET",
                &format!("intents/{pi}"),
                Some(api_key),
                json!({}),
                "",
                None,
            )
            .await
            .unwrap();
            assert_eq!(result["status"], "refunded");
        }
    }
    // Each supported token settles escrow at exactly 1%; funded never means settled.
    for token in ["BNB", "USDT", "USDC"] {
        drain(&p, &n, "escrow").await;
        let order = request(
            &p,
            "POST",
            "intents",
            Some(api_key),
            json!({"mode":"escrow","token":token,"amount":"10000"}),
            &format!("release-{token}"),
            None,
        )
        .await
        .unwrap();
        let cs = order["checkout_id"].as_str().unwrap();
        let pi = order["intent"]["id"].as_str().unwrap();
        let op = request(
            &p,
            "POST",
            &format!("checkout-sessions/{cs}/deposit"),
            None,
            json!({"payer":PAYER}),
            "deposit",
            Some(cs),
        )
        .await
        .unwrap();
        transact(&p, &n, &op, None, Some(cs), "escrow").await;
        let funded = request(
            &p,
            "GET",
            &format!("intents/{pi}"),
            Some(api_key),
            json!({}),
            "",
            None,
        )
        .await
        .unwrap();
        assert_eq!(funded["status"], "funded");
        assert_eq!(funded["fee_amount"], "0");
        let op = request(
            &p,
            "POST",
            &format!("checkout-sessions/{cs}/release"),
            None,
            json!({"payer":PAYER}),
            "release",
            Some(cs),
        )
        .await
        .unwrap();
        transact(&p, &n, &op, None, Some(cs), "escrow").await;
        let settled = request(
            &p,
            "GET",
            &format!("intents/{pi}"),
            Some(api_key),
            json!({}),
            "",
            None,
        )
        .await
        .unwrap();
        assert_eq!(settled["status"], "succeeded");
        assert_eq!(settled["fee_amount"], "100");
    }
    for mode in ["direct", "escrow"] {
        drain(&p, &n, mode).await;
        assert!(request(
            &p,
            "POST",
            &format!("contracts/{mode}/pause"),
            Some(api_key),
            json!({"paused":true}),
            "unauthorized-control",
            None
        )
        .await
        .is_err());
        for paused in [true, false] {
            let op = request(
                &p,
                "POST",
                &format!("contracts/{mode}/pause"),
                Some("admin"),
                json!({"paused":paused}),
                &format!("{mode}:{paused}"),
                None,
            )
            .await
            .unwrap();
            let hash = n
                .rpc("eth_sendTransaction", json!([op["transaction_parameters"]]))
                .await
                .unwrap();
            assert_eq!(receipt(&n, &hash).await["status"], "0x1");
            request(
                &p,
                "POST",
                &format!("contract-controls/{}/confirm", op["id"].as_str().unwrap()),
                Some("admin"),
                json!({"tx_hash":hash}),
                "",
                None,
            )
            .await
            .unwrap();
            n.rpc("anvil_mine", json!([2])).await.unwrap();
            drain(&p, &n, mode).await;
            let result = request(
                &p,
                "GET",
                &format!("contract-controls/{}", op["id"].as_str().unwrap()),
                Some("admin"),
                json!({}),
                "",
                None,
            )
            .await
            .unwrap();
            assert_eq!(result["status"], "confirmed");
        }
    }
    // Capacity remains reserved during an RPC outage, then becomes reusable only after canonical expiry.
    let link = request(
        &p,
        "POST",
        "links",
        Some(api_key),
        json!({"mode":"direct","token":"BNB","amount":"100","max_uses":1,"expires_in":3600}),
        "expiring-link",
        None,
    )
    .await
    .unwrap();
    let path = format!("links/{}/checkouts", link["id"].as_str().unwrap());
    let order = request(
        &p,
        "POST",
        &path,
        None,
        json!({"expires_in":60}),
        "reservation",
        None,
    )
    .await
    .unwrap();
    let expires =
        chrono::DateTime::parse_from_rfc3339(order["intent"]["expires_at"].as_str().unwrap())
            .unwrap()
            .timestamp();
    n.rpc("evm_setNextBlockTimestamp", json!([expires + 1]))
        .await
        .unwrap();
    n.rpc("anvil_mine", json!([3])).await.unwrap();
    assert!(request(
        &p,
        "POST",
        &path,
        None,
        json!({}),
        "second-reservation",
        None
    )
    .await
    .is_err());
    drain(&p, &n, "direct").await;
    let cs = order["checkout_id"].as_str().unwrap();
    let expired = request(
        &p,
        "GET",
        &format!("checkout-sessions/{cs}"),
        None,
        json!({}),
        "",
        Some(cs),
    )
    .await
    .unwrap();
    assert_eq!(expired["status"], "expired");
    assert!(request(
        &p,
        "POST",
        &path,
        None,
        json!({}),
        "second-reservation",
        None
    )
    .await
    .is_ok());
    sqlx::query("UPDATE pay_merchant_intents SET status='succeeded' WHERE id=(SELECT id FROM pay_merchant_intents WHERE merchant_id=$1 AND status='refunded' LIMIT 1)").bind(mid).execute(&db).await.unwrap();
    let bad_id:String=sqlx::query_scalar("SELECT id FROM pay_merchant_intents WHERE merchant_id=$1 AND status='succeeded' AND token<>'BNB' AND EXISTS(SELECT 1 FROM pay_merchant_chain_events e WHERE e.intent_id=pay_merchant_intents.id AND e.status='refunded' AND e.canonical) LIMIT 1").bind(mid).fetch_one(&db).await.unwrap();
    assert_eq!(
        request(
            &p,
            "GET",
            &format!("intents/{bad_id}"),
            Some(api_key),
            json!({}),
            "",
            None
        )
        .await
        .unwrap()["status"],
        "verification_required"
    );
    let mut outage = n.clone();
    outage.rpc_url = "http://127.0.0.1:1".into();
    assert!(reconcile::checked_tick(&p, &outage, "direct")
        .await
        .is_err());
    assert!(api::healthy(&p, &n, "direct").await.is_err());
    drain(&p, &n, "direct").await;
    request(
        &p,
        "POST",
        &format!("api-keys/{}/revoke", key["id"].as_str().unwrap()),
        Some("merchant"),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    assert!(request(
        &p,
        "GET",
        "merchants/me",
        Some(api_key),
        json!({}),
        "",
        None
    )
    .await
    .is_err());
    let old_deliveries: Vec<String> =
        sqlx::query_scalar("SELECT id FROM pay_merchant_deliveries WHERE endpoint_id=$1")
            .bind(&endpoint)
            .fetch_all(&db)
            .await
            .unwrap();
    assert!(!old_deliveries.is_empty());
    for actor in ["other", api_key] {
        assert!(request(
            &p,
            "GET",
            "webhook-endpoints",
            Some(actor),
            json!({}),
            "",
            None
        )
        .await
        .map(|v| v["items"].as_array().is_some_and(|a| a.is_empty()))
        .unwrap_or(true));
        assert!(request(
            &p,
            "POST",
            &format!("webhook-endpoints/{endpoint}/replace"),
            Some(actor),
            json!({"url":"https://hooks.example.test/pay"}),
            "",
            None
        )
        .await
        .is_err());
    }
    let replaced = request(
        &p,
        "POST",
        &format!("webhook-endpoints/{endpoint}/replace"),
        Some("merchant"),
        json!({"url":"https://hooks.example.test/updated"}),
        "",
        None,
    )
    .await
    .unwrap();
    assert_ne!(replaced["id"], endpoint);
    assert_eq!(replaced["replaces_id"], endpoint);
    assert!(request(
        &p,
        "POST",
        &format!("webhook-endpoints/{endpoint}/enable"),
        Some("merchant"),
        json!({}),
        "",
        None
    )
    .await
    .is_err());
    for delivery in old_deliveries {
        let still: String =
            sqlx::query_scalar("SELECT endpoint_id FROM pay_merchant_deliveries WHERE id=$1")
                .bind(delivery)
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(still, endpoint);
    }
    let evidence =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/merchant-validation");
    std::fs::create_dir_all(&evidence).unwrap();
    std::fs::write(
        evidence.join("network.json"),
        serde_json::to_string_pretty(&vec![n]).unwrap(),
    )
    .unwrap();
    server.abort();
}

async fn catalog_cases(p: &Platform, n: &chain::Network) {
    n.rpc("anvil_mine", json!([4])).await.unwrap();
    drain(p, n, "direct").await;
    drain(p, n, "qr").await;
    let body = json!({"name":"Research pass","description":"Daily insights","prices":{"USDT":"5","USDC":"6.25"},"duration_days":30});
    let product = request(
        p,
        "POST",
        "products",
        Some("merchant"),
        body.clone(),
        "catalog-create",
        None,
    )
    .await
    .unwrap();
    let product2 = request(
        p,
        "POST",
        "products",
        Some("merchant"),
        body.clone(),
        "catalog-create",
        None,
    )
    .await
    .unwrap();
    assert_eq!(product, product2);
    let pid = product["id"].as_str().unwrap();
    let mid = product["merchant_id"].as_str().unwrap();
    for (method, path, b) in [
        ("GET", format!("products/{pid}"), json!({})),
        ("POST", format!("products/{pid}/update"), body.clone()),
    ] {
        assert!(
            request(p, method, &path, Some("other"), b, "cross-shop", None)
                .await
                .is_err()
        );
    }
    let catalog = request(
        p,
        "GET",
        &format!("catalog/{mid}"),
        None,
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    assert_eq!(catalog["items"].as_array().unwrap().len(), 1);
    let route = format!("products/{pid}/checkouts");
    let checkout = request(
        p,
        "POST",
        &route,
        None,
        json!({"token":"USDT","amount":"1","payee":ADMIN}),
        "catalog-order",
        None,
    )
    .await
    .unwrap();
    let snap = &checkout["intent"]["checkout_snapshot"];
    assert_eq!(snap["amount"], "5000000000000000000");
    assert_eq!(snap["payee"], MERCHANT);
    assert_eq!(snap["duration_days"], 30);
    assert_eq!(snap["item_name"], "Research pass");
    assert_eq!(snap["kind"], "merchant");
    let usdc = request(
        p,
        "POST",
        &route,
        None,
        json!({"token":"USDC"}),
        "catalog-usdc",
        None,
    )
    .await
    .unwrap();
    assert_eq!(usdc["intent"]["amount"], "6250000000000000000");
    let mut edited = body.clone();
    edited["name"] = json!("New name");
    edited["prices"]["USDT"] = json!("99");
    edited["duration_days"] = json!(1);
    edited["enabled"] = json!(false);
    request(
        p,
        "POST",
        &format!("products/{pid}/update"),
        Some("merchant"),
        edited,
        "edit-catalog",
        None,
    )
    .await
    .unwrap();
    assert!(request(
        p,
        "POST",
        &route,
        None,
        json!({"token":"USDT"}),
        "new-disabled",
        None
    )
    .await
    .is_err());
    let retry = request(
        p,
        "POST",
        &route,
        None,
        json!({"token":"USDT","amount":"1","payee":ADMIN}),
        "catalog-order",
        None,
    )
    .await
    .unwrap();
    assert_eq!(retry, checkout);
    let cs = checkout["checkout_id"].as_str().unwrap();
    let pi = checkout["intent"]["id"].as_str().unwrap();
    assert_eq!(
        request(
            p,
            "POST",
            &format!("intents/{pi}/collect"),
            Some("merchant"),
            json!({}),
            "empty-collect",
            None
        )
        .await
        .unwrap_err()
        .1,
        "no_funds_to_collect"
    );
    let prepare = request(
        p,
        "POST",
        &format!("checkout-sessions/{cs}/prepare-transfer"),
        None,
        json!({"payer":PAYER}),
        "",
        Some(cs),
    )
    .await
    .unwrap();
    let hash = n
        .rpc(
            "eth_sendTransaction",
            json!([prepare["transaction_parameters"]]),
        )
        .await
        .unwrap();
    assert_eq!(receipt(n, &hash).await["status"], "0x1");
    n.rpc("anvil_mine", json!([4])).await.unwrap();
    drain(p, n, "qr").await;
    let paid = request(
        p,
        "GET",
        &format!("intents/{pi}"),
        Some("merchant"),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    assert_eq!(paid["status"], "succeeded");
    assert_eq!(paid["settlement_status"], "ready");
    assert_eq!(paid["checkout_snapshot"], *snap);
    let events: Vec<Value> =
        sqlx::query_scalar("SELECT payload FROM pay_merchant_events WHERE intent_id=$1")
            .bind(pi)
            .fetch_all(&p.db)
            .await
            .unwrap();
    assert!(!events.is_empty());
    assert!(events
        .iter()
        .any(|v| v.to_string().contains("Research pass")));
    let collect = request(
        p,
        "POST",
        &format!("intents/{pi}/collect"),
        Some("merchant"),
        json!({}),
        "catalog-collect",
        None,
    )
    .await
    .unwrap();
    transact(p, n, &collect, Some("merchant"), None, "qr").await;
    let settled = request(
        p,
        "GET",
        &format!("intents/{pi}"),
        Some("merchant"),
        json!({}),
        "",
        None,
    )
    .await
    .unwrap();
    assert_eq!(settled["settlement_status"], "settled");
    assert!(settled["settlement_tx_hash"].is_string());
    let overview = request(p, "GET", "overview", Some("merchant"), json!({}), "", None)
        .await
        .unwrap();
    assert_eq!(overview["items"][0]["ready"], "0");
    let link=request(p,"POST","links",Some("merchant"),json!({"mode":"direct","payment_method":"transfer","token":"USDT","amount":"3000000000000000000","description":"Custom item"}),"qr-link",None).await.unwrap();
    let link_checkout = request(
        p,
        "POST",
        &format!("links/{}/checkouts", link["id"].as_str().unwrap()),
        None,
        json!({}),
        "qr-link-order",
        None,
    )
    .await
    .unwrap();
    assert_eq!(link_checkout["intent"]["payment_method"], "transfer");
    assert_eq!(
        link_checkout["intent"]["checkout_snapshot"]["item_name"],
        "Custom item"
    );
}
