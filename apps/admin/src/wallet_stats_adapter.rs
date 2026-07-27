//! Strict read-only adapter for the backend-owned wallet inventory summary.
//!
//! Only aggregate counts cross this boundary. Wallet addresses, metadata,
//! plans, permissions, activity, and correlation evidence are parsed only to
//! prove the exact service DTO and are never projected into SSR state.

use epsx_dioxus_ui::pages::admin_pages::wallet_wallets::AdminWalletStatsSummary;
use serde::Deserialize;

// Keep the commerce loaders behind the already-owned route adapter module so
// no central app module or SSR registry change is required for this slice.
#[path = "commerce_adapter.rs"]
mod commerce_adapter;
pub(crate) use commerce_adapter::{
    access_mutation_path, credit_mutation_path, decode_admin_envelope, load_access,
    load_credit_stats, load_payment_links, load_plan_detail, load_plans, load_wallet_detail,
    load_wallet_list, payment_intent_cancel_path, payment_link_mutation_path, plan_detail_path,
    plan_mutation_path, send_admin_json, wallet_detail_path, wallet_metadata_mutation_path,
    wallet_status_mutation_path, AdminCommerceLoad, AdminCommerceMutationLoad, CreditCommand,
    ExpectedVersionCommand, WalletMetadataCommand,
};

const WALLET_STATS_PATH: &str = "/api/v1/admin/wallets/stats";
const MAX_ADMIN_WALLET_STATS_RESPONSE_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct AdminWalletStatsQuery;

impl AdminWalletStatsQuery {
    pub(crate) fn from_raw(raw_query: &str) -> Result<Self, ()> {
        if raw_query.is_empty() {
            Ok(Self)
        } else {
            Err(())
        }
    }

    pub(crate) fn upstream_path(self) -> &'static str {
        WALLET_STATS_PATH
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminWalletStatsLoad {
    Ready(AdminWalletStatsSummary),
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_wallet_stats(
    client: &epsx_client::ServiceClient,
    query: AdminWalletStatsQuery,
    ctx: &epsx_client::RequestContext,
) -> AdminWalletStatsLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminWalletStatsLoad::Unavailable;
    };

    let http_client = match reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => return AdminWalletStatsLoad::Unavailable,
    };
    let url = format!(
        "{}{}",
        client.base_url().trim_end_matches('/'),
        query.upstream_path()
    );
    let response = match http_client
        .get(url)
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return AdminWalletStatsLoad::Unavailable,
    };

    if response.status() != reqwest::StatusCode::OK {
        return if response.status() == reqwest::StatusCode::FORBIDDEN {
            AdminWalletStatsLoad::Forbidden
        } else {
            AdminWalletStatsLoad::Unavailable
        };
    }

    let body =
        match read_response_body_limited(response, MAX_ADMIN_WALLET_STATS_RESPONSE_BYTES).await {
            Ok(body) => body,
            Err(()) => return AdminWalletStatsLoad::Unavailable,
        };
    let payload = match decode_admin_envelope::<BackendWalletStatsResponse>(&body) {
        Ok(payload) => payload,
        Err(_) => return AdminWalletStatsLoad::Malformed,
    };

    classify_payload(payload)
}

async fn read_response_body_limited(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, ()> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(());
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
        let next_len = body.len().checked_add(chunk.len()).ok_or(())?;
        if next_len > limit {
            return Err(());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendWalletStatsResponse {
    total_users: i64,
    active_users: i64,
    inactive_users: i64,
    users_by_tier: serde_json::Value,
    new_users_30_days: i64,
    active_users_30_days: i64,
    growth_rate: f64,
}

fn classify_payload(payload: BackendWalletStatsResponse) -> AdminWalletStatsLoad {
    let BackendWalletStatsResponse {
        total_users,
        active_users,
        inactive_users,
        users_by_tier,
        new_users_30_days,
        active_users_30_days,
        growth_rate,
    } = payload;
    if !users_by_tier.is_object()
        || !growth_rate.is_finite()
        || [
            total_users,
            active_users,
            inactive_users,
            new_users_30_days,
            active_users_30_days,
        ]
        .into_iter()
        .any(|count| count < 0)
        || active_users
            .checked_add(inactive_users)
            .is_none_or(|known_total| known_total != total_users)
        || new_users_30_days > total_users
        || active_users_30_days > active_users
    {
        return AdminWalletStatsLoad::Malformed;
    }
    AdminWalletStatsLoad::Ready(AdminWalletStatsSummary {
        total_users,
        active_users,
        inactive_users,
        new_users_30_days,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn read_request(stream: &mut tokio::net::TcpStream) -> String {
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut chunk).await.unwrap();
            assert!(read > 0);
            request.extend_from_slice(&chunk[..read]);
            assert!(request.len() <= 16 * 1024);
        }
        String::from_utf8(request).unwrap()
    }

    fn loopback_client(address: std::net::SocketAddr) -> epsx_client::ServiceClient {
        epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: Duration::from_secs(2),
        })
    }

    fn verified_context() -> epsx_client::RequestContext {
        epsx_client::RequestContext {
            request_id: uuid::Uuid::parse_str("e44f180b-3d2b-41ec-badf-cb4332f05fb2").unwrap(),
            auth_token: Some("verified-wallet-stats-token".to_string()),
            user_id: Some(uuid::Uuid::nil()),
            address: Some("0xmust-not-cross-the-boundary".to_string()),
        }
    }

    fn valid_payload() -> Value {
        json!({
            "success": true,
            "data": {
                "total_users": 11,
                "active_users": 8,
                "inactive_users": 3,
                "users_by_tier": {},
                "new_users_30_days": 2,
                "active_users_30_days": 6,
                "growth_rate": 1.5
            },
            "error": null,
            "message": "Wallet statistics retrieved",
            "timestamp": "2026-07-27T00:00:00Z",
            "admin_meta": null
        })
    }

    fn classify_value(value: Value) -> AdminWalletStatsLoad {
        match decode_admin_envelope::<BackendWalletStatsResponse>(value.to_string().as_bytes()) {
            Ok(payload) => classify_payload(payload),
            Err(_) => AdminWalletStatsLoad::Malformed,
        }
    }

    async fn serve_once(
        status: &str,
        body: Vec<u8>,
    ) -> (std::net::SocketAddr, tokio::task::JoinHandle<String>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let status = status.to_string();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            let headers = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(headers.as_bytes()).await.unwrap();
            stream.write_all(&body).await.unwrap();
            request
        });
        (address, server)
    }

    #[test]
    fn query_accepts_only_the_absent_query_grammar() {
        assert_eq!(
            AdminWalletStatsQuery::from_raw("").unwrap().upstream_path(),
            WALLET_STATS_PATH
        );
        for raw in ["page=1", "status=active", "unknown=", "%20", " "] {
            assert!(
                AdminWalletStatsQuery::from_raw(raw).is_err(),
                "accepted query: {raw:?}"
            );
        }
    }

    #[test]
    fn valid_payload_projects_only_four_authoritative_counts() {
        assert_eq!(
            classify_value(valid_payload()),
            AdminWalletStatsLoad::Ready(AdminWalletStatsSummary {
                total_users: 11,
                active_users: 8,
                inactive_users: 3,
                new_users_30_days: 2,
            })
        );
    }

    #[test]
    fn zero_counts_are_authoritative_ready_data() {
        let mut value = valid_payload();
        value["data"]["total_users"] = json!(0);
        value["data"]["active_users"] = json!(0);
        value["data"]["inactive_users"] = json!(0);
        value["data"]["new_users_30_days"] = json!(0);
        value["data"]["active_users_30_days"] = json!(0);
        assert!(matches!(
            classify_value(value),
            AdminWalletStatsLoad::Ready(AdminWalletStatsSummary { total_users: 0, .. })
        ));
    }

    #[test]
    fn invalid_dto_and_count_semantics_are_malformed() {
        let mut cases = Vec::new();

        let mut value = valid_payload();
        value["unknown"] = json!(true);
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["active_users"] = json!(-1);
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["total_users"] = json!(12);
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["new_users_30_days"] = json!(12);
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["active_users_30_days"] = json!(12);
        cases.push(value);

        for case in cases {
            assert!(matches!(
                classify_value(case),
                AdminWalletStatsLoad::Malformed
            ));
        }
    }

    #[test]
    fn raw_dto_and_unsuccessful_or_unknown_envelopes_are_rejected() {
        let raw = json!({
            "total": 11,
            "active": 8,
            "disabled": 3,
            "new_30_days": 2,
            "correlation_id": "e44f180b-3d2b-41ec-badf-cb4332f05fb2"
        });
        assert!(matches!(
            classify_value(raw),
            AdminWalletStatsLoad::Malformed
        ));

        let mut failed = valid_payload();
        failed["success"] = json!(false);
        failed["data"] = Value::Null;
        failed["error"] = json!("permission denied");
        assert!(matches!(
            classify_value(failed),
            AdminWalletStatsLoad::Malformed
        ));

        let mut unknown = valid_payload();
        unknown["unexpected"] = json!(true);
        assert!(matches!(
            classify_value(unknown),
            AdminWalletStatsLoad::Malformed
        ));
    }

    #[tokio::test]
    async fn loader_sends_only_exact_authenticated_get_and_request_id() {
        let body = serde_json::to_vec(&valid_payload()).unwrap();
        let (address, server) = serve_once("200 OK", body).await;
        let load = load_admin_wallet_stats(
            &loopback_client(address),
            AdminWalletStatsQuery,
            &verified_context(),
        )
        .await;
        let request = server.await.unwrap();

        assert!(matches!(load, AdminWalletStatsLoad::Ready(_)));
        assert!(request.starts_with("GET /api/v1/admin/wallets/stats HTTP/1.1\r\n"));
        let lowered = request.to_ascii_lowercase();
        assert!(lowered.contains("authorization: bearer verified-wallet-stats-token\r\n"));
        assert!(lowered.contains("x-request-id: e44f180b-3d2b-41ec-badf-cb4332f05fb2\r\n"));
        assert!(!lowered.contains("x-user-id:"));
        assert!(!lowered.contains("x-user-address:"));
    }

    #[tokio::test]
    async fn loader_maps_only_403_to_forbidden() {
        for (status, expected_forbidden) in [
            ("403 Forbidden", true),
            ("201 Created", false),
            ("400 Bad Request", false),
            ("401 Unauthorized", false),
            ("404 Not Found", false),
            ("500 Internal Server Error", false),
        ] {
            let (address, server) = serve_once(status, Vec::new()).await;
            let load = load_admin_wallet_stats(
                &loopback_client(address),
                AdminWalletStatsQuery,
                &verified_context(),
            )
            .await;
            let _ = server.await.unwrap();
            if expected_forbidden {
                assert!(matches!(load, AdminWalletStatsLoad::Forbidden));
            } else {
                assert!(matches!(load, AdminWalletStatsLoad::Unavailable));
            }
        }
    }

    #[tokio::test]
    async fn loader_requires_a_verified_bearer_before_network_io() {
        let client = epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: "http://127.0.0.1:9".to_string(),
            timeout: Duration::from_millis(50),
        });
        let mut context = verified_context();
        context.auth_token = Some("  ".to_string());
        assert!(matches!(
            load_admin_wallet_stats(&client, AdminWalletStatsQuery, &context).await,
            AdminWalletStatsLoad::Unavailable
        ));
    }

    #[tokio::test]
    async fn loader_never_follows_redirects_with_the_admin_bearer() {
        let redirect_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let redirect_address = redirect_listener.local_addr().unwrap();
        let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_address = target_listener.local_addr().unwrap();
        let redirect_server = tokio::spawn(async move {
            let (mut stream, _) = redirect_listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            assert!(request.starts_with("GET /api/v1/admin/wallets/stats HTTP/1.1\r\n"));
            let response = format!(
                "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/bearer-leak\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let load = load_admin_wallet_stats(
            &loopback_client(redirect_address),
            AdminWalletStatsQuery,
            &verified_context(),
        )
        .await;
        redirect_server.await.unwrap();
        assert!(matches!(load, AdminWalletStatsLoad::Unavailable));
        assert!(
            tokio::time::timeout(Duration::from_millis(100), target_listener.accept())
                .await
                .is_err(),
            "the wallet-stats bearer followed an upstream redirect"
        );
    }

    #[tokio::test]
    async fn loader_caps_declared_and_streamed_response_bodies() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let declared_server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = read_request(&mut stream).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                MAX_ADMIN_WALLET_STATS_RESPONSE_BYTES + 1
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        assert!(matches!(
            load_admin_wallet_stats(
                &loopback_client(address),
                AdminWalletStatsQuery,
                &verified_context()
            )
            .await,
            AdminWalletStatsLoad::Unavailable
        ));
        declared_server.await.unwrap();

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let streamed_server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = read_request(&mut stream).await;
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
            let body = vec![b'x'; MAX_ADMIN_WALLET_STATS_RESPONSE_BYTES + 1];
            let header = format!("{:X}\r\n", body.len());
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(&body).await.unwrap();
            stream.write_all(b"\r\n0\r\n\r\n").await.unwrap();
        });
        assert!(matches!(
            load_admin_wallet_stats(
                &loopback_client(address),
                AdminWalletStatsQuery,
                &verified_context()
            )
            .await,
            AdminWalletStatsLoad::Unavailable
        ));
        streamed_server.await.unwrap();
    }
}
