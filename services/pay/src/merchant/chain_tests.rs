use super::*;
use axum::{
    extract::{ConnectInfo, State},
    http::{StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
    routing::post,
    Json, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

type Requests = Arc<Mutex<Vec<(SocketAddr, String)>>>;

async fn respond(
    State(requests): State<Requests>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    uri: Uri,
) -> Response {
    requests.lock().await.push((peer, uri.path().to_owned()));
    match uri.path() {
        "/private-token/redirect" => Redirect::temporary("/followed").into_response(),
        "/private-token/status" => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        "/private-token/invalid-json" => "not JSON".into_response(),
        _ => Json(json!({"jsonrpc":"2.0","id":1,"result":"0x7a69"})).into_response(),
    }
}

fn network(url: String) -> Network {
    Network {
        environment: "test".into(),
        chain_id: 31337,
        rpc_url: url,
        archive_rpc_url: None,
        scan_blocks: rpc::LOG_SCAN_BLOCKS,
        admin: Address::ZERO,
        treasury: Address::ZERO,
        direct: Contract {
            address: Address::ZERO,
            deployment_block: 0,
        },
        escrow: Contract {
            address: Address::ZERO,
            deployment_block: 0,
        },
        qr: None,
        tokens: BTreeMap::new(),
        confirmations: 1,
    }
}

async fn server() -> (String, Requests, tokio::task::JoinHandle<()>) {
    let requests = Requests::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .fallback(post(respond))
        .with_state(requests.clone());
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    (format!("http://{address}"), requests, task)
}

#[tokio::test]
async fn cloned_networks_reuse_connection_without_reusing_request_path() {
    let (url, requests, task) = server().await;
    let first = network(format!("{url}/first"));
    let mut second = first.clone();
    second.rpc_url = format!("{url}/second");
    for n in [&first, &second, &first] {
        assert_eq!(n.rpc("eth_chainId", json!([])).await.unwrap(), "0x7a69");
    }
    let received = requests.lock().await;
    assert_eq!(received.len(), 3);
    assert!(received.iter().all(|(peer, _)| *peer == received[0].0));
    assert_eq!(
        received
            .iter()
            .map(|(_, path)| path.as_str())
            .collect::<Vec<_>>(),
        ["/first", "/second", "/first"]
    );
    task.abort();
}

#[tokio::test]
async fn rpc_rejects_redirects_and_hides_credentials_in_transport_errors() {
    let (url, requests, task) = server().await;
    for suffix in ["redirect", "status", "invalid-json"] {
        let n = network(format!("{url}/private-token/{suffix}"));
        let error = n.rpc("eth_chainId", json!([])).await.unwrap_err();
        assert!(!error.to_string().contains("private-token"));
        assert!(!format!("{error:?}").contains("private-token"));
    }
    let received = requests.lock().await;
    assert_eq!(received.len(), 3);
    assert!(received.iter().all(|(_, path)| path != "/followed"));
    task.abort();
}

#[test]
fn existing_network_configuration_keeps_ten_block_default_and_archive_round_trips() {
    let mut value = serde_json::to_value(network("https://rpc.example".into())).unwrap();
    value.as_object_mut().unwrap().remove("scan_blocks");
    let mut parsed: Network = serde_json::from_value(value).unwrap();
    assert_eq!(parsed.scan_blocks, 10);
    assert!(parsed.archive_rpc_url.is_none());
    parsed.scan_blocks = 50;
    parsed.archive_rpc_url = Some("https://archive.example/credential".into());
    let restored: Network = serde_json::from_value(serde_json::to_value(&parsed).unwrap()).unwrap();
    assert_eq!(restored.archive_rpc_url, parsed.archive_rpc_url);
    assert_eq!(restored.scan_blocks, 50);
    for blocks in [1, 10, 50] {
        assert!(rpc::validate_scan_blocks(blocks).is_ok());
    }
    for blocks in [0, 51, u64::MAX] {
        assert!(rpc::validate_scan_blocks(blocks).is_err());
    }
}
