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
