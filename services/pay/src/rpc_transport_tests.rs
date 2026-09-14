use super::*;
use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use std::sync::Arc;
type Calls = Arc<Mutex<Vec<(String, String, Instant)>>>;

async fn respond(
    State(calls): State<Calls>,
    uri: Uri,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let method = body["method"].as_str().unwrap_or_default().to_owned();
    calls
        .lock()
        .await
        .push((uri.path().to_owned(), method.clone(), Instant::now()));
    let value = match uri.path() {
        "/pruned" => json!({"error":{"code":-32701,"message":"history pruned"}}),
        "/limited" => return (StatusCode::TOO_MANY_REQUESTS, Json(json!({}))),
        "/rpc-limited" => json!({"error":{"code":-32005,"message":"quota"}}),
        "/wrong-chain" if method == "eth_chainId" => json!({"result":"0x61"}),
        _ if method == "eth_chainId" => json!({"result":"0x38"}),
        _ => json!({"result":[]}),
    };
    (StatusCode::OK, Json(value))
}

async fn server() -> (String, Calls, tokio::task::JoinHandle<()>) {
    let calls = Calls::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .fallback(post(respond))
        .with_state(calls.clone());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (url, calls, task)
}

fn logs() -> Value {
    json!([{"address":"0x1111111111111111111111111111111111111111","fromBlock":"0x1","toBlock":"0x32"}])
}

#[tokio::test]
async fn pruned_logs_use_verified_archive_and_obey_shared_rate_limit() {
    let (url, calls, task) = server().await;
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    for _ in 0..2 {
        let value = read(
            &client,
            &format!("{url}/pruned"),
            Some(&format!("{url}/archive")),
            56,
            "eth_getLogs",
            logs(),
        )
        .await
        .unwrap();
        assert_eq!(value, json!([]));
    }
    let calls = calls.lock().await;
    let archive: Vec<_> = calls
        .iter()
        .filter(|(path, _, _)| path == "/archive")
        .collect();
    assert_eq!(archive.len(), 4);
    assert_eq!(
        (&archive[0].1, &archive[1].1),
        (&"eth_chainId".to_owned(), &"eth_getLogs".to_owned())
    );
    // The limiter measures client send times; loopback scheduling may shift the
    // server's timestamps slightly between the first and reused connection.
    assert!(archive[2].2.duration_since(archive[0].2) >= Duration::from_millis(2900));
    task.abort();
}

#[tokio::test]
async fn archive_chain_mismatch_and_out_of_bound_requests_fail_closed() {
    let (url, calls, task) = server().await;
    let client = Client::new();
    assert!(read(
        &client,
        &format!("{url}/pruned"),
        Some(&format!("{url}/wrong-chain")),
        56,
        "eth_getLogs",
        logs()
    )
    .await
    .unwrap_err()
    .to_string()
    .contains("chain mismatch"));
    let mut oversized = logs();
    oversized[0]["toBlock"] = json!("0x33");
    assert!(read(
        &client,
        &format!("{url}/pruned"),
        Some(&format!("{url}/archive")),
        56,
        "eth_getLogs",
        oversized
    )
    .await
    .is_err());
    let calls = calls.lock().await;
    assert_eq!(
        calls
            .iter()
            .filter(|(path, _, _)| path == "/wrong-chain")
            .count(),
        1
    );
    assert!(!calls.iter().any(|(path, _, _)| path == "/archive"));
    task.abort();
}

#[tokio::test]
async fn rate_limits_and_other_methods_never_trigger_archive_requests() {
    let (url, calls, task) = server().await;
    let client = Client::new();
    for (path, method) in [
        ("/limited", "eth_getLogs"),
        ("/rpc-limited", "eth_getLogs"),
        ("/pruned", "eth_call"),
    ] {
        assert!(read(
            &client,
            &format!("{url}{path}"),
            Some(&format!("{url}/archive")),
            56,
            method,
            logs()
        )
        .await
        .is_err());
    }
    assert_eq!(calls.lock().await.len(), 3);
    task.abort();
}

#[test]
fn archive_endpoints_and_ranges_are_restricted() {
    for url in [
        "https://rpc.example/secret",
        "http://127.0.0.1:1234",
        "http://[::1]:1234",
    ] {
        assert!(validate_endpoint(url).is_ok());
    }
    for url in [
        "http://rpc.example",
        "https://user:secret@rpc.example",
        "https://rpc.example/#secret",
    ] {
        assert!(validate_endpoint(url).is_err());
    }
    assert!(bounded_logs(&logs()));
    for value in [
        json!([]),
        json!([{"fromBlock":"0x1","toBlock":"0x2"}]),
        json!([{"address":"x","fromBlock":"latest","toBlock":"latest"}]),
        json!([{"address":"x","fromBlock":"0x3","toBlock":"0x2"}]),
    ] {
        assert!(!bounded_logs(&value));
    }
}

#[tokio::test]
#[ignore = "requires private read-only BSC RPC qualification endpoints and receipt evidence"]
async fn authenticated_primary_and_archive_recover_historical_payment_logs() {
    let primary = std::env::var("EPSX_RPC_QUALIFY_PRIMARY").unwrap();
    let archive = std::env::var("EPSX_RPC_QUALIFY_ARCHIVE").unwrap();
    let receipt: Value = serde_json::from_slice(
        &std::fs::read(std::env::var("EPSX_RPC_QUALIFY_RECEIPT").unwrap()).unwrap(),
    )
    .unwrap();
    let client = Client::builder()
        .user_agent("EPSX-Pay/1.0")
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let height = crate::native_chain::number(&receipt["blockNumber"]).unwrap();
    let params = json!([{"address":receipt["to"],"fromBlock":format!("0x{:x}",height-49),"toBlock":receipt["blockNumber"]}]);
    let logs = read(&client, &primary, Some(&archive), 56, "eth_getLogs", params)
        .await
        .unwrap();
    assert!(logs
        .as_array()
        .unwrap()
        .iter()
        .any(|log| log["transactionHash"] == receipt["transactionHash"]
            && log["blockHash"] == receipt["blockHash"]));
    let block = read(
        &client,
        &primary,
        None,
        56,
        "eth_getBlockByNumber",
        json!([receipt["blockNumber"], false]),
    )
    .await
    .unwrap();
    assert_eq!(block["hash"], receipt["blockHash"]);
}
