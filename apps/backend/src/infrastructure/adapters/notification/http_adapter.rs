//! Remote notification publisher adapter.
//!
//! The adapter is opt-in through `NOTIFICATION_ADAPTER=remote`. It forwards
//! publisher events to the Rust notification service using the dedicated
//! publisher audience token and never logs message bodies or recipient data.

use async_trait::async_trait;
use epsx_contracts::errors::{AppError, AppResult};
use epsx_contracts::notification_port::{
    BroadcastNotificationRequest, NotificationPort, SendNotificationRequest,
};
use reqwest::{Client, StatusCode, Url};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

const PUBLISH_RESPONSE_MAX_BYTES: usize = 8 * 1024;

#[derive(Clone)]
pub struct HttpNotificationAdapter {
    client: Client,
    endpoint: Url,
    bearer: String,
}

#[derive(Serialize)]
struct PublishRequest<'a> {
    event_id: &'a str,
    event_type: &'a str,
    aggregate_id: &'a str,
    idempotency_key: &'a str,
    recipient_wallet_address: &'a str,
    notification_type: &'a str,
    priority: &'a str,
    title: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: &'a Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_url: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: &'a Option<chrono::DateTime<chrono::Utc>>,
}

impl HttpNotificationAdapter {
    pub fn from_env() -> AppResult<Self> {
        let raw_endpoint = std::env::var("NOTIFICATION_SERVICE_URL")
            .map_err(|_| AppError::configuration_error("NOTIFICATION_SERVICE_URL is required"))?;
        let bearer = std::env::var("NOTIFICATION_SERVICE_TOKEN")
            .map_err(|_| AppError::configuration_error("NOTIFICATION_SERVICE_TOKEN is required"))?;
        if bearer.trim().is_empty() || bearer.trim() != bearer {
            return Err(AppError::configuration_error(
                "NOTIFICATION_SERVICE_TOKEN must be non-empty and trimmed",
            ));
        }
        let endpoint = Url::parse(&raw_endpoint)
            .map_err(|_| AppError::configuration_error("NOTIFICATION_SERVICE_URL is invalid"))?;
        if !matches!(endpoint.scheme(), "http" | "https")
            || endpoint.host_str().is_none()
            || endpoint.username() != ""
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err(AppError::configuration_error(
                "NOTIFICATION_SERVICE_URL must be an origin or path without credentials/query",
            ));
        }
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("epsx-backend-notification-publisher/1")
            .build()
            .map_err(|error| {
                AppError::configuration_error(format!("notification HTTP client: {error}"))
            })?;
        Ok(Self {
            client,
            endpoint: endpoint.join("/api/v1/notification/publish").map_err(|_| {
                AppError::configuration_error("NOTIFICATION_SERVICE_URL cannot build publish path")
            })?,
            bearer,
        })
    }

    async fn publish(&self, request: PublishRequest<'_>) -> AppResult<String> {
        let response = self
            .client
            .post(self.endpoint.clone())
            .bearer_auth(&self.bearer)
            .header("x-request-id", request.event_id)
            .json(&request)
            .send()
            .await
            .map_err(|error| {
                AppError::network_error(format!("notification publisher request: {error}"))
            })?;
        let status = response.status();
        if status != StatusCode::ACCEPTED && status != StatusCode::OK {
            let kind = if status == StatusCode::CONFLICT {
                AppError::conflict("notification publisher idempotency conflict")
            } else if status.is_client_error() {
                AppError::validation_error("notification publisher rejected the event")
            } else {
                AppError::external_service_error("notification publisher unavailable")
            };
            return Err(kind);
        }
        if response
            .content_length()
            .is_some_and(|length| length > PUBLISH_RESPONSE_MAX_BYTES as u64)
        {
            return Err(AppError::external_service_error(
                "notification publisher response is too large",
            ));
        }
        let bytes = response.bytes().await.map_err(|error| {
            AppError::external_service_error(format!("notification publisher response: {error}"))
        })?;
        if bytes.len() > PUBLISH_RESPONSE_MAX_BYTES {
            return Err(AppError::external_service_error(
                "notification publisher response is too large",
            ));
        }
        let body: Value = serde_json::from_slice(&bytes).map_err(|error| {
            AppError::external_service_error(format!("notification publisher response: {error}"))
        })?;
        body.get("event_id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                AppError::external_service_error("notification publisher response malformed")
            })
    }

    fn validate_event_id(event_id: &str) -> AppResult<()> {
        if event_id.trim().is_empty()
            || event_id.len() > 128
            || event_id
                .chars()
                .any(|character| character.is_control() || character.is_whitespace())
        {
            return Err(AppError::validation_error(
                "notification event identity must be non-empty, bounded, and whitespace-free",
            ));
        }
        Ok(())
    }

    async fn send_request_with_event_id(
        &self,
        event_id: &str,
        request: SendNotificationRequest,
    ) -> AppResult<String> {
        Self::validate_event_id(event_id)?;
        let wallet = request.recipient_wallet_address.to_ascii_lowercase();
        self.publish(PublishRequest {
            event_id,
            event_type: "notification.send",
            aggregate_id: &wallet,
            idempotency_key: event_id,
            recipient_wallet_address: &wallet,
            notification_type: &request.notification_type,
            priority: &request.priority,
            title: &request.title,
            message: &request.message,
            data: &request.data,
            action_url: &request.action_url,
            expires_at: &request.expires_at,
        })
        .await
    }

    async fn broadcast_request_with_event_id(
        &self,
        event_id: &str,
        request: BroadcastNotificationRequest,
    ) -> AppResult<()> {
        Self::validate_event_id(event_id)?;
        self.publish(PublishRequest {
            event_id,
            event_type: "notification.broadcast",
            aggregate_id: "all",
            idempotency_key: event_id,
            recipient_wallet_address: "all",
            notification_type: &request.notification_type,
            priority: &request.priority,
            title: &request.title,
            message: &request.message,
            data: &request.data,
            action_url: &None,
            expires_at: &request.expires_at,
        })
        .await
        .map(|_| ())
    }
}

#[async_trait]
impl NotificationPort for HttpNotificationAdapter {
    async fn send(&self, request: SendNotificationRequest) -> AppResult<String> {
        let event_id = Uuid::now_v7().to_string();
        self.send_request_with_event_id(&event_id, request).await
    }

    async fn send_with_event_id(
        &self,
        event_id: &str,
        request: SendNotificationRequest,
    ) -> AppResult<String> {
        self.send_request_with_event_id(event_id, request).await
    }

    async fn broadcast(&self, request: BroadcastNotificationRequest) -> AppResult<()> {
        let event_id = Uuid::now_v7().to_string();
        self.broadcast_request_with_event_id(&event_id, request)
            .await
    }

    async fn broadcast_with_event_id(
        &self,
        event_id: &str,
        request: BroadcastNotificationRequest,
    ) -> AppResult<()> {
        self.broadcast_request_with_event_id(event_id, request)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::Json,
        http::{header, HeaderMap},
        response::IntoResponse,
        routing::post,
        Router,
    };
    use epsx_contracts::notification_port::{
        BroadcastNotificationRequest, NotificationPort, SendNotificationRequest,
    };
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    #[test]
    fn publish_endpoint_is_service_scoped() {
        let endpoint = Url::parse("https://notifications.example/api/").unwrap();
        assert_eq!(
            endpoint
                .join("/api/v1/notification/publish")
                .unwrap()
                .as_str(),
            "https://notifications.example/api/v1/notification/publish"
        );
    }

    #[tokio::test]
    async fn send_and_broadcast_forward_scoped_identity_and_typed_payloads() {
        let observations = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
        let capture = observations.clone();
        let router = Router::new().route(
            "/api/v1/notification/publish",
            post(
                move |headers: HeaderMap, Json(body): Json<serde_json::Value>| {
                    let capture = capture.clone();
                    async move {
                        capture.lock().unwrap().push(json!({
                            "authorization": headers
                                .get(header::AUTHORIZATION)
                                .and_then(|value| value.to_str().ok()),
                            "request_id": headers
                                .get("x-request-id")
                                .and_then(|value| value.to_str().ok()),
                            "body": body,
                        }));
                        Json(json!({"event_id": "accepted-event"})).into_response()
                    }
                },
            ),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let adapter = HttpNotificationAdapter {
            client: Client::builder().build().unwrap(),
            endpoint: Url::parse(&format!("http://{address}/api/v1/notification/publish")).unwrap(),
            bearer: "publisher-test-token".into(),
        };
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        let event_id = adapter
            .send(SendNotificationRequest {
                recipient_wallet_address: "0xABCDEFabcdefABCDEFabcdefABCDEFabcdefABCD".into(),
                notification_type: "payment".into(),
                priority: "high".into(),
                title: "Payment complete".into(),
                message: "Your payment completed".into(),
                data: Some(json!({"amount": 12})),
                action_url: Some("/payments/12".into()),
                expires_at: Some(expires_at),
            })
            .await
            .unwrap();
        assert_eq!(event_id, "accepted-event");
        adapter
            .broadcast(BroadcastNotificationRequest {
                notification_type: "announcement".into(),
                priority: "normal".into(),
                title: "Maintenance".into(),
                message: "Scheduled maintenance".into(),
                data: None,
                expires_at: None,
            })
            .await
            .unwrap();

        let observations = observations.lock().unwrap();
        assert_eq!(observations.len(), 2);
        for observation in observations.iter() {
            assert_eq!(observation["authorization"], "Bearer publisher-test-token");
            assert!(!observation["body"]["event_id"].as_str().unwrap().is_empty());
            assert_eq!(observation["request_id"], observation["body"]["event_id"]);
            assert_eq!(
                observation["body"]["idempotency_key"],
                observation["body"]["event_id"]
            );
        }
        assert_eq!(observations[0]["body"]["event_type"], "notification.send");
        assert_eq!(
            observations[0]["body"]["recipient_wallet_address"],
            "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
        );
        assert_eq!(
            observations[0]["body"]["aggregate_id"],
            "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
        );
        assert!(observations[0]["body"]["expires_at"].is_string());
        assert_eq!(
            observations[1]["body"]["event_type"],
            "notification.broadcast"
        );
        assert_eq!(observations[1]["body"]["recipient_wallet_address"], "all");
        assert_eq!(observations[1]["body"]["aggregate_id"], "all");
    }

    #[tokio::test]
    async fn stable_event_id_is_reused_for_remote_retry_identity() {
        let observations = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
        let capture = observations.clone();
        let router = Router::new().route(
            "/api/v1/notification/publish",
            post(move |Json(body): Json<serde_json::Value>| {
                let capture = capture.clone();
                async move {
                    capture.lock().unwrap().push(body);
                    Json(json!({"event_id": "stable-payment-1"})).into_response()
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let adapter = HttpNotificationAdapter {
            client: Client::builder().build().unwrap(),
            endpoint: Url::parse(&format!("http://{address}/api/v1/notification/publish")).unwrap(),
            bearer: "publisher-test-token".into(),
        };
        let request = SendNotificationRequest {
            recipient_wallet_address: "0x1111111111111111111111111111111111111111".into(),
            notification_type: "payment".into(),
            priority: "normal".into(),
            title: "Payment complete".into(),
            message: "Your payment completed".into(),
            data: None,
            action_url: None,
            expires_at: None,
        };
        adapter
            .send_with_event_id("payment.completed:payment-1", request.clone())
            .await
            .unwrap();
        adapter
            .send_with_event_id("payment.completed:payment-1", request)
            .await
            .unwrap();

        let observations = observations.lock().unwrap();
        assert_eq!(observations.len(), 2);
        for observation in observations.iter() {
            assert_eq!(observation["event_id"], "payment.completed:payment-1");
            assert_eq!(
                observation["idempotency_key"],
                "payment.completed:payment-1"
            );
        }
    }

    #[tokio::test]
    async fn stable_event_id_rejects_unsafe_identity() {
        let adapter = HttpNotificationAdapter {
            client: Client::builder().build().unwrap(),
            endpoint: Url::parse("http://127.0.0.1:1/api/v1/notification/publish").unwrap(),
            bearer: "publisher-test-token".into(),
        };
        let request = SendNotificationRequest {
            recipient_wallet_address: "0x1111111111111111111111111111111111111111".into(),
            notification_type: "payment".into(),
            priority: "normal".into(),
            title: "Payment complete".into(),
            message: "Your payment completed".into(),
            data: None,
            action_url: None,
            expires_at: None,
        };
        assert!(adapter
            .send_with_event_id("payment completed", request)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn oversized_publish_response_fails_closed_before_event_acceptance() {
        let router = Router::new().route(
            "/api/v1/notification/publish",
            post(|| async {
                Json(json!({
                    "event_id": "accepted-event",
                    "padding": "x".repeat(PUBLISH_RESPONSE_MAX_BYTES + 1)
                }))
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let adapter = HttpNotificationAdapter {
            client: Client::builder().build().unwrap(),
            endpoint: Url::parse(&format!("http://{address}/api/v1/notification/publish")).unwrap(),
            bearer: "publisher-test-token".into(),
        };
        let request = SendNotificationRequest {
            recipient_wallet_address: "0x1111111111111111111111111111111111111111".into(),
            notification_type: "payment".into(),
            priority: "normal".into(),
            title: "Payment complete".into(),
            message: "Your payment completed".into(),
            data: None,
            action_url: None,
            expires_at: None,
        };
        assert!(adapter
            .send_with_event_id("payment.completed:large", request)
            .await
            .is_err());
    }
}
