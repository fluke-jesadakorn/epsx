//! Strict read-only adapter for the backend-owned dashboard user-status snapshot.
//!
//! The admin root needs only two bounded counts and the database observation
//! time. Identity fields, permissions, plans, activity, health, and mutation
//! affordances must not cross this boundary.

use epsx_dioxus_ui::pages::admin_pages::dashboard::AdminDashboardUserStatus;
use serde::Deserialize;

const DASHBOARD_USER_STATUS_PATH: &str = "/api/admin/dashboard/user-status";
const DASHBOARD_USER_STATUS_OPERATION: &str = "get_dashboard_user_status";
const MAX_DASHBOARD_USER_STATUS_RESPONSE_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct AdminDashboardUserStatusQuery;

impl AdminDashboardUserStatusQuery {
    pub(crate) fn from_raw(raw_query: Option<&str>) -> Result<Self, ()> {
        if raw_query.is_none() {
            Ok(Self)
        } else {
            Err(())
        }
    }

    pub(crate) fn upstream_path(self) -> &'static str {
        DASHBOARD_USER_STATUS_PATH
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminDashboardUserStatusLoad {
    Ready(AdminDashboardUserStatus),
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_dashboard_user_status(
    client: &epsx_client::ServiceClient,
    query: AdminDashboardUserStatusQuery,
    ctx: &epsx_client::RequestContext,
) -> AdminDashboardUserStatusLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminDashboardUserStatusLoad::Unavailable;
    };

    let http_client = match reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => return AdminDashboardUserStatusLoad::Unavailable,
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
        Err(_) => return AdminDashboardUserStatusLoad::Unavailable,
    };

    if response.status() != reqwest::StatusCode::OK {
        return if response.status() == reqwest::StatusCode::FORBIDDEN {
            AdminDashboardUserStatusLoad::Forbidden
        } else {
            AdminDashboardUserStatusLoad::Unavailable
        };
    }

    let body = match read_response_body_limited(response, MAX_DASHBOARD_USER_STATUS_RESPONSE_BYTES)
        .await
    {
        Ok(body) => body,
        Err(()) => return AdminDashboardUserStatusLoad::Unavailable,
    };
    let payload = match serde_json::from_slice::<BackendDashboardUserStatusEnvelope>(&body) {
        Ok(payload) => payload,
        Err(_) => return AdminDashboardUserStatusLoad::Malformed,
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
struct BackendDashboardUserStatusEnvelope {
    success: bool,
    data: Option<AdminDashboardUserStatus>,
    error: Option<String>,
    message: String,
    timestamp: String,
    admin_meta: Option<BackendAdminMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendAdminMetadata {
    operation: String,
    performed_by: Option<String>,
    pagination: Option<BackendPaginationInfo>,
    permissions: Option<BackendPermissionContext>,
    metadata: Option<BackendEmptyObject>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPaginationInfo {
    page: u32,
    limit: u32,
    total_count: u64,
    total_pages: u32,
    has_next: bool,
    has_prev: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendPermissionContext {
    admin_plan: String,
    available_actions: Vec<String>,
    restricted_actions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendEmptyObject {}

fn classify_payload(payload: BackendDashboardUserStatusEnvelope) -> AdminDashboardUserStatusLoad {
    let BackendDashboardUserStatusEnvelope {
        success,
        data,
        error,
        message,
        timestamp,
        admin_meta,
    } = payload;

    let Some(data) = data else {
        return AdminDashboardUserStatusLoad::Malformed;
    };
    let Some(admin_meta) = admin_meta else {
        return AdminDashboardUserStatusLoad::Malformed;
    };
    let Ok(response_timestamp) = chrono::DateTime::parse_from_rfc3339(&timestamp) else {
        return AdminDashboardUserStatusLoad::Malformed;
    };
    let Ok(observed_at) = chrono::DateTime::parse_from_rfc3339(&data.observed_at) else {
        return AdminDashboardUserStatusLoad::Malformed;
    };
    if !success
        || error.is_some()
        || admin_meta.operation != DASHBOARD_USER_STATUS_OPERATION
        || admin_meta.pagination.is_some()
        || admin_meta.permissions.is_some()
        || admin_meta.metadata.is_some()
        || observed_at > response_timestamp
        || data.total_users < 0
        || data.active_users < 0
        || data.active_users > data.total_users
    {
        return AdminDashboardUserStatusLoad::Malformed;
    }

    // Human-facing envelope text and actor metadata are deliberately not
    // projected into the page. The UI receives only the backend snapshot.
    let _ = (message, timestamp, admin_meta.performed_by);
    AdminDashboardUserStatusLoad::Ready(data)
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
            request_id: uuid::Uuid::parse_str("9fb33b37-9f3c-4b8c-92d5-0fc4265bdeda").unwrap(),
            auth_token: Some("verified-dashboard-token".to_string()),
            user_id: Some(uuid::Uuid::nil()),
            address: Some("0xmust-not-cross-the-boundary".to_string()),
        }
    }

    fn valid_payload() -> Value {
        json!({
            "success": true,
            "data": {
                "observed_at": "2026-07-23T03:04:04Z",
                "total_users": 11,
                "active_users": 8
            },
            "message": "Dashboard user status retrieved successfully",
            "timestamp": "2026-07-23T03:04:05Z",
            "admin_meta": {
                "operation": "get_dashboard_user_status",
                "performed_by": "admin"
            }
        })
    }

    fn classify_value(value: Value) -> AdminDashboardUserStatusLoad {
        match serde_json::from_value::<BackendDashboardUserStatusEnvelope>(value) {
            Ok(payload) => classify_payload(payload),
            Err(_) => AdminDashboardUserStatusLoad::Malformed,
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
            AdminDashboardUserStatusQuery::from_raw(None)
                .unwrap()
                .upstream_path(),
            DASHBOARD_USER_STATUS_PATH
        );
        for raw in ["", "page=1", "status=active", "unknown=", "%20", " "] {
            assert!(
                AdminDashboardUserStatusQuery::from_raw(Some(raw)).is_err(),
                "accepted query: {raw:?}"
            );
        }
    }

    #[test]
    fn valid_payload_projects_only_the_two_counts_and_observation_time() {
        assert_eq!(
            classify_value(valid_payload()),
            AdminDashboardUserStatusLoad::Ready(AdminDashboardUserStatus {
                observed_at: "2026-07-23T03:04:04Z".to_string(),
                total_users: 11,
                active_users: 8,
            })
        );
    }

    #[test]
    fn zero_counts_are_authoritative_ready_data() {
        let mut value = valid_payload();
        value["data"]["total_users"] = json!(0);
        value["data"]["active_users"] = json!(0);
        assert!(matches!(
            classify_value(value),
            AdminDashboardUserStatusLoad::Ready(AdminDashboardUserStatus {
                total_users: 0,
                active_users: 0,
                ..
            })
        ));
    }

    #[test]
    fn envelope_count_and_time_drift_are_malformed() {
        let mut cases = Vec::new();

        let mut value = valid_payload();
        value["unknown"] = json!(true);
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["unknown"] = json!(true);
        cases.push(value);

        let mut value = valid_payload();
        value["admin_meta"]["unknown"] = json!(true);
        cases.push(value);

        let mut value = valid_payload();
        value["success"] = json!(false);
        cases.push(value);

        let mut value = valid_payload();
        value["error"] = json!("unexpected");
        cases.push(value);

        let mut value = valid_payload();
        value["data"] = Value::Null;
        cases.push(value);

        let mut value = valid_payload();
        value["admin_meta"] = Value::Null;
        cases.push(value);

        let mut value = valid_payload();
        value["admin_meta"]["operation"] = json!("get_dashboard_summary");
        cases.push(value);

        let mut value = valid_payload();
        value["admin_meta"]["metadata"] = json!({});
        cases.push(value);

        let mut value = valid_payload();
        value["timestamp"] = json!("2026-02-30T03:04:05Z");
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["observed_at"] = json!("not-a-time");
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["observed_at"] = json!("2026-07-23T03:04:06Z");
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["total_users"] = json!(-1);
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["active_users"] = json!(-1);
        cases.push(value);

        let mut value = valid_payload();
        value["data"]["active_users"] = json!(12);
        cases.push(value);

        for case in cases {
            assert!(matches!(
                classify_value(case),
                AdminDashboardUserStatusLoad::Malformed
            ));
        }
    }

    #[tokio::test]
    async fn loader_sends_only_exact_authenticated_get_and_request_id() {
        let body = serde_json::to_vec(&valid_payload()).unwrap();
        let (address, server) = serve_once("200 OK", body).await;
        let load = load_admin_dashboard_user_status(
            &loopback_client(address),
            AdminDashboardUserStatusQuery,
            &verified_context(),
        )
        .await;
        let request = server.await.unwrap();

        assert!(matches!(load, AdminDashboardUserStatusLoad::Ready(_)));
        assert!(request.starts_with("GET /api/admin/dashboard/user-status HTTP/1.1\r\n"));
        let lowered = request.to_ascii_lowercase();
        assert!(lowered.contains("authorization: bearer verified-dashboard-token\r\n"));
        assert!(lowered.contains("x-request-id: 9fb33b37-9f3c-4b8c-92d5-0fc4265bdeda\r\n"));
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
            let load = load_admin_dashboard_user_status(
                &loopback_client(address),
                AdminDashboardUserStatusQuery,
                &verified_context(),
            )
            .await;
            let _ = server.await.unwrap();
            if expected_forbidden {
                assert!(matches!(load, AdminDashboardUserStatusLoad::Forbidden));
            } else {
                assert!(matches!(load, AdminDashboardUserStatusLoad::Unavailable));
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
            load_admin_dashboard_user_status(&client, AdminDashboardUserStatusQuery, &context)
                .await,
            AdminDashboardUserStatusLoad::Unavailable
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
            assert!(request.starts_with("GET /api/admin/dashboard/user-status HTTP/1.1\r\n"));
            let response = format!(
                "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/bearer-leak\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let load = load_admin_dashboard_user_status(
            &loopback_client(redirect_address),
            AdminDashboardUserStatusQuery,
            &verified_context(),
        )
        .await;
        redirect_server.await.unwrap();
        assert!(matches!(load, AdminDashboardUserStatusLoad::Unavailable));
        assert!(
            tokio::time::timeout(Duration::from_millis(100), target_listener.accept())
                .await
                .is_err(),
            "the dashboard bearer followed an upstream redirect"
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
                MAX_DASHBOARD_USER_STATUS_RESPONSE_BYTES + 1
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        assert!(matches!(
            load_admin_dashboard_user_status(
                &loopback_client(address),
                AdminDashboardUserStatusQuery,
                &verified_context()
            )
            .await,
            AdminDashboardUserStatusLoad::Unavailable
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
            let body = vec![b'x'; MAX_DASHBOARD_USER_STATUS_RESPONSE_BYTES + 1];
            let header = format!("{:X}\r\n", body.len());
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(&body).await.unwrap();
            stream.write_all(b"\r\n0\r\n\r\n").await.unwrap();
        });
        assert!(matches!(
            load_admin_dashboard_user_status(
                &loopback_client(address),
                AdminDashboardUserStatusQuery,
                &verified_context()
            )
            .await,
            AdminDashboardUserStatusLoad::Unavailable
        ));
        streamed_server.await.unwrap();
    }
}
