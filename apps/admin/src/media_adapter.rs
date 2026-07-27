//! Strict read-only compatibility adapter for the legacy admin media inventory.
//!
//! Only the public `news` and `public` buckets may cross this boundary. Private
//! chat/notification objects, provider URLs, upload/delete controls, and storage
//! errors are deliberately excluded from the SSR projection.

use epsx_dioxus_ui::pages::admin_pages::media::{
    decode_admin_media_mutation, AdminMediaList, AdminMediaMutationProjection, AdminMediaObject,
};
use serde::Deserialize;
use std::collections::HashSet;

const ADMIN_MEDIA_LIMIT: usize = 100;
const MAX_MEDIA_KEY_CHARS: usize = 1_024;
const MAX_MEDIA_KEY_BYTES: usize = 1_024;
const MAX_MEDIA_URL_CHARS: usize = 4_096;
const MAX_TIMESTAMP_CHARS: usize = 64;
pub(crate) const MAX_ADMIN_MEDIA_RESPONSE_BYTES: usize = 256 * 1024;
const MAX_UPLOAD_BYTES: usize = 25 * 1024 * 1024;
const MAX_FILENAME_CHARS: usize = 255;
const MAX_IDEMPOTENCY_KEY_CHARS: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminMediaMutationError {
    Invalid,
    Forbidden,
    Conflict,
    Unavailable,
    Malformed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MutationEnvelope {
    success: bool,
    data: Option<LegacyMediaMutation>,
    error: Option<serde_json::Value>,
    meta: Option<LegacyResponseMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyMediaMutation {
    bucket: String,
    key: String,
    url: Option<String>,
    thumb_url: Option<String>,
    mime: Option<String>,
    size: Option<u64>,
    deleted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AdminMediaQuery {
    pub(crate) bucket: &'static str,
}

impl AdminMediaQuery {
    pub(crate) fn from_raw(raw_query: &str) -> Result<Self, ()> {
        if !raw_query.is_empty() && raw_query.split('&').any(str::is_empty) {
            return Err(());
        }

        let mut bucket = "news";
        let mut bucket_seen = false;
        let mut url = reqwest::Url::parse("http://admin.invalid/")
            .expect("the fixed admin-media query base URL is valid");
        url.set_query((!raw_query.is_empty()).then_some(raw_query));

        for (key, value) in url.query_pairs() {
            if key != "bucket" || bucket_seen {
                return Err(());
            }
            bucket_seen = true;
            bucket = match value.as_ref() {
                "news" => "news",
                "public" => "public",
                _ => return Err(()),
            };
        }

        Ok(Self { bucket })
    }

    pub(crate) fn upstream_path(&self) -> String {
        format!("/api/admin/media/{}?limit={ADMIN_MEDIA_LIMIT}", self.bucket)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminMediaLoad {
    Ready(AdminMediaList),
    Empty(AdminMediaList),
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_media(
    client: &epsx_client::ServiceClient,
    query: &AdminMediaQuery,
    ctx: &epsx_client::RequestContext,
) -> AdminMediaLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminMediaLoad::Unavailable;
    };

    let http_client = match reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => return AdminMediaLoad::Unavailable,
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
        Err(_) => return AdminMediaLoad::Unavailable,
    };

    if response.status() != reqwest::StatusCode::OK {
        return match response.status() {
            reqwest::StatusCode::BAD_REQUEST => AdminMediaLoad::Malformed,
            reqwest::StatusCode::FORBIDDEN => AdminMediaLoad::Forbidden,
            _ => AdminMediaLoad::Unavailable,
        };
    }

    let body = match read_response_body_limited(response, MAX_ADMIN_MEDIA_RESPONSE_BYTES).await {
        Ok(body) => body,
        Err(()) => return AdminMediaLoad::Unavailable,
    };
    let payload = match serde_json::from_slice::<LegacyMediaEnvelope>(&body) {
        Ok(payload) => payload,
        Err(_) => return AdminMediaLoad::Malformed,
    };
    classify_payload(payload)
}

pub(crate) async fn upload_admin_public_file(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    filename: &str,
    bytes: Vec<u8>,
    idempotency_key: &str,
) -> Result<AdminMediaMutationProjection, AdminMediaMutationError> {
    validate_filename(filename)?;
    validate_idempotency_key(idempotency_key)?;
    if bytes.is_empty() || bytes.len() > MAX_UPLOAD_BYTES {
        return Err(AdminMediaMutationError::Invalid);
    }
    let token = bearer(ctx)?;
    let http_client = mutation_client(client)?;
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(bytes).file_name(filename.to_string()),
    );
    let response = http_client
        .post(format!(
            "{}/api/admin/files/upload",
            client.base_url().trim_end_matches('/')
        ))
        .header("x-request-id", ctx.request_id.to_string())
        .header("idempotency-key", idempotency_key)
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await
        .map_err(|_| AdminMediaMutationError::Unavailable)?;
    decode_mutation_response(response, "public").await
}

pub(crate) async fn delete_admin_media(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    bucket: &str,
    key: &str,
    idempotency_key: &str,
) -> Result<AdminMediaMutationProjection, AdminMediaMutationError> {
    validate_bucket_name(bucket)?;
    validate_object_key(key)?;
    validate_idempotency_key(idempotency_key)?;
    let token = bearer(ctx)?;
    let http_client = mutation_client(client)?;
    let url = media_mutation_url(client, bucket, key)?;
    let response = http_client
        .delete(url)
        .header("x-request-id", ctx.request_id.to_string())
        .header("idempotency-key", idempotency_key)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| AdminMediaMutationError::Unavailable)?;
    decode_mutation_response(response, bucket).await
}

fn bearer(ctx: &epsx_client::RequestContext) -> Result<&str, AdminMediaMutationError> {
    ctx.auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
        .ok_or(AdminMediaMutationError::Unavailable)
}

fn mutation_client(
    client: &epsx_client::ServiceClient,
) -> Result<reqwest::Client, AdminMediaMutationError> {
    reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| AdminMediaMutationError::Unavailable)
}

async fn decode_mutation_response(
    response: reqwest::Response,
    expected_bucket: &str,
) -> Result<AdminMediaMutationProjection, AdminMediaMutationError> {
    let status = response.status();
    let body = read_response_body_limited(response, MAX_ADMIN_MEDIA_RESPONSE_BYTES)
        .await
        .map_err(|_| AdminMediaMutationError::Unavailable)?;
    match status {
        reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => {}
        reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
            return Err(AdminMediaMutationError::Invalid)
        }
        reqwest::StatusCode::FORBIDDEN => return Err(AdminMediaMutationError::Forbidden),
        reqwest::StatusCode::CONFLICT => return Err(AdminMediaMutationError::Conflict),
        _ => return Err(AdminMediaMutationError::Unavailable),
    }
    let envelope: MutationEnvelope =
        serde_json::from_slice(&body).map_err(|_| AdminMediaMutationError::Malformed)?;
    if !envelope.success
        || envelope.error.is_some()
        || !default_meta_is_valid(envelope.meta.as_ref())
    {
        return Err(AdminMediaMutationError::Malformed);
    }
    let mutation = envelope.data.ok_or(AdminMediaMutationError::Malformed)?;
    if mutation.bucket != expected_bucket {
        return Err(AdminMediaMutationError::Malformed);
    }
    let size = mutation
        .size
        .map(i64::try_from)
        .transpose()
        .map_err(|_| AdminMediaMutationError::Malformed)?;
    let projection = AdminMediaMutationProjection {
        bucket: mutation.bucket,
        key: mutation.key,
        size,
        deleted: mutation.deleted,
    };
    decode_admin_media_mutation(
        serde_json::to_value(projection).map_err(|_| AdminMediaMutationError::Malformed)?,
    )
    .ok_or(AdminMediaMutationError::Malformed)
}

fn media_mutation_url(
    client: &epsx_client::ServiceClient,
    bucket: &str,
    key: &str,
) -> Result<String, AdminMediaMutationError> {
    let mut url = reqwest::Url::parse(&format!(
        "{}/api/admin/media/",
        client.base_url().trim_end_matches('/')
    ))
    .map_err(|_| AdminMediaMutationError::Malformed)?;
    url.path_segments_mut()
        .map_err(|_| AdminMediaMutationError::Malformed)?
        .push(bucket)
        .push(key);
    Ok(url.to_string())
}

fn validate_bucket_name(bucket: &str) -> Result<(), AdminMediaMutationError> {
    if matches!(bucket, "news" | "public") {
        Ok(())
    } else {
        Err(AdminMediaMutationError::Invalid)
    }
}

fn validate_filename(filename: &str) -> Result<(), AdminMediaMutationError> {
    if filename.is_empty()
        || filename.chars().count() > MAX_FILENAME_CHARS
        || filename.trim() != filename
        || filename.chars().any(char::is_control)
        || filename.contains('/')
        || filename.contains('\\')
        || matches!(filename, "." | "..")
    {
        Err(AdminMediaMutationError::Invalid)
    } else {
        Ok(())
    }
}

fn validate_object_key(key: &str) -> Result<(), AdminMediaMutationError> {
    if key.is_empty()
        || key.chars().count() > MAX_MEDIA_KEY_CHARS
        || key.len() > MAX_MEDIA_KEY_BYTES
        || key.trim() != key
        || key.chars().any(char::is_control)
        || key.contains('\\')
        || key.split('/').any(|segment| matches!(segment, "." | ".."))
    {
        Err(AdminMediaMutationError::Invalid)
    } else {
        Ok(())
    }
}

fn validate_idempotency_key(key: &str) -> Result<(), AdminMediaMutationError> {
    if key.is_empty()
        || key.chars().count() > MAX_IDEMPOTENCY_KEY_CHARS
        || key.trim() != key
        || key.chars().any(char::is_control)
    {
        Err(AdminMediaMutationError::Invalid)
    } else {
        Ok(())
    }
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
struct LegacyMediaEnvelope {
    success: bool,
    data: Option<Vec<LegacyFileInfo>>,
    error: Option<serde_json::Value>,
    meta: Option<LegacyResponseMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyResponseMeta {
    timestamp: String,
    request_id: Option<String>,
    version: Option<String>,
    message: Option<String>,
    pagination: Option<serde_json::Value>,
    permissions: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyFileInfo {
    key: String,
    url: String,
    size: i64,
    last_modified: Option<String>,
}

fn classify_payload(payload: LegacyMediaEnvelope) -> AdminMediaLoad {
    if !payload.success {
        return AdminMediaLoad::Unavailable;
    }
    if payload.error.is_some() || !default_meta_is_valid(payload.meta.as_ref()) {
        return AdminMediaLoad::Malformed;
    }
    let Some(raw_items) = payload.data else {
        return AdminMediaLoad::Malformed;
    };
    if raw_items.len() > ADMIN_MEDIA_LIMIT {
        return AdminMediaLoad::Malformed;
    }

    let mut keys = HashSet::with_capacity(raw_items.len());
    let mut previous_key: Option<String> = None;
    let mut items = Vec::with_capacity(raw_items.len());
    for item in raw_items {
        if !bounded_control_free(&item.key, 1, MAX_MEDIA_KEY_CHARS)
            || item.key.len() > MAX_MEDIA_KEY_BYTES
            || !keys.insert(item.key.clone())
            || previous_key
                .as_ref()
                .is_some_and(|previous| previous >= &item.key)
            || !bounded_control_free(&item.url, 1, MAX_MEDIA_URL_CHARS)
            || item.size < 0
            || !valid_optional_timestamp(item.last_modified.as_deref())
        {
            return AdminMediaLoad::Malformed;
        }
        previous_key = Some(item.key.clone());
        items.push(AdminMediaObject {
            key: item.key,
            size: item.size,
            last_modified: item.last_modified,
        });
    }

    let projection = AdminMediaList { items };
    if projection.items.is_empty() {
        AdminMediaLoad::Empty(projection)
    } else {
        AdminMediaLoad::Ready(projection)
    }
}

fn default_meta_is_valid(meta: Option<&LegacyResponseMeta>) -> bool {
    let Some(meta) = meta else {
        return false;
    };
    bounded_control_free(&meta.timestamp, 1, MAX_TIMESTAMP_CHARS)
        && chrono::DateTime::parse_from_rfc3339(&meta.timestamp).is_ok()
        && meta
            .request_id
            .as_deref()
            .is_none_or(|id| uuid::Uuid::parse_str(id).is_ok())
        && meta.version.as_deref() == Some("v1")
        && meta.message.is_none()
        && meta.pagination.is_none()
        && meta.permissions.is_none()
}

fn valid_optional_timestamp(value: Option<&str>) -> bool {
    value.is_none_or(|value| {
        bounded_control_free(value, 1, MAX_TIMESTAMP_CHARS)
            && chrono::DateTime::parse_from_rfc3339(value).is_ok()
    })
}

fn bounded_control_free(value: &str, min_chars: usize, max_chars: usize) -> bool {
    let chars = value.chars().count();
    chars >= min_chars
        && chars <= max_chars
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn read_request(stream: &mut tokio::net::TcpStream) -> String {
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1_024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut chunk).await.unwrap();
            assert!(read > 0);
            request.extend_from_slice(&chunk[..read]);
            assert!(request.len() <= 16 * 1_024);
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
            request_id: uuid::Uuid::parse_str("d9dbcc48-7f46-46cb-9b87-7cda68cb3af2").unwrap(),
            auth_token: Some("verified-admin-token".to_string()),
            user_id: Some(uuid::Uuid::nil()),
            address: Some("0xspoofable-address".to_string()),
        }
    }

    fn payload(value: serde_json::Value) -> LegacyMediaEnvelope {
        serde_json::from_value(value).unwrap()
    }

    fn valid_meta() -> serde_json::Value {
        serde_json::json!({
            "timestamp": "2026-07-23T00:00:00Z",
            "version": "v1"
        })
    }

    fn valid_item(key: &str) -> serde_json::Value {
        serde_json::json!({
            "key": key,
            "url": format!("https://objects.example/news/{key}"),
            "size": 42,
            "last_modified": "2026-07-22T12:00:00Z"
        })
    }

    fn success_body(items: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "success": true,
            "data": items,
            "meta": valid_meta()
        }))
        .unwrap()
    }

    async fn serve_response(
        status: &str,
        body: Vec<u8>,
    ) -> (std::net::SocketAddr, tokio::task::JoinHandle<String>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let status = status.to_string();
        let task = tokio::spawn(async move {
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
        (address, task)
    }

    #[test]
    fn query_is_closed_unique_and_defaults_to_news() {
        let default = AdminMediaQuery::from_raw("").unwrap();
        assert_eq!(default.bucket, "news");
        assert_eq!(default.upstream_path(), "/api/admin/media/news?limit=100");
        let public = AdminMediaQuery::from_raw("bucket=public").unwrap();
        assert_eq!(public.bucket, "public");
        assert_eq!(public.upstream_path(), "/api/admin/media/public?limit=100");
        for raw in [
            "&",
            "&&",
            "&bucket=news",
            "bucket=",
            "bucket=chat",
            "bucket=notifications",
            "bucket=public&",
            "bucket=news&&",
            "bucket=news&bucket=public",
            "bucket=news&prefix=private",
            "limit=100",
            "unknown=value",
        ] {
            assert!(AdminMediaQuery::from_raw(raw).is_err(), "accepted {raw}");
        }
    }

    #[test]
    fn mutation_inputs_and_envelopes_are_bounded_and_typed() {
        assert!(validate_bucket_name("news").is_ok());
        assert!(validate_bucket_name("private").is_err());
        assert!(validate_object_key("images/launch.webp").is_ok());
        assert!(validate_object_key("../secret").is_err());
        assert!(validate_object_key("images\\secret").is_err());
        assert!(validate_filename("launch.webp").is_ok());
        assert!(validate_filename("../secret").is_err());
        assert!(validate_idempotency_key("media-upload-1").is_ok());
        assert!(validate_idempotency_key(" media-upload-1").is_err());

        let projection = decode_admin_media_mutation(serde_json::json!({
            "bucket": "public",
            "key": "launch.webp",
            "size": 42,
            "deleted": false
        }))
        .unwrap();
        assert_eq!(projection.bucket, "public");
        assert!(decode_admin_media_mutation(serde_json::json!({
            "bucket": "public",
            "key": "launch.webp",
            "size": 42,
            "deleted": false,
            "url": "https://secret.example/object"
        }))
        .is_none());
    }

    #[test]
    fn payload_is_strict_sorted_bounded_and_projects_urls_away() {
        let load = classify_payload(payload(serde_json::json!({
            "success": true,
            "data": [valid_item("a/banner.png"), valid_item("z/whitepaper.pdf")],
            "meta": valid_meta()
        })));
        let AdminMediaLoad::Ready(projection) = load else {
            panic!("expected ready projection");
        };
        let encoded = serde_json::to_value(&projection).unwrap();
        assert_eq!(encoded["items"].as_array().unwrap().len(), 2);
        assert_eq!(encoded["items"][0]["key"], "a/banner.png");
        assert!(encoded["items"][0].get("url").is_none());

        for invalid in [
            serde_json::json!({
                "success": true,
                "data": [valid_item("z.png"), valid_item("a.png")],
                "meta": valid_meta()
            }),
            serde_json::json!({
                "success": true,
                "data": [valid_item("same.png"), valid_item("same.png")],
                "meta": valid_meta()
            }),
            serde_json::json!({
                "success": true,
                "data": [{"key":"bad\nkey","url":"https://objects.example/x","size":1,"last_modified":null}],
                "meta": valid_meta()
            }),
            serde_json::json!({
                "success": true,
                "data": [{"key":"bad-size","url":"https://objects.example/x","size":-1,"last_modified":null}],
                "meta": valid_meta()
            }),
            serde_json::json!({
                "success": true,
                "data": [{"key":"bad-time","url":"https://objects.example/x","size":1,"last_modified":"yesterday"}],
                "meta": valid_meta()
            }),
            serde_json::json!({
                "success": true,
                "data": [{"key":" padded ","url":"https://objects.example/x","size":1,"last_modified":null}],
                "meta": valid_meta()
            }),
            serde_json::json!({
                "success": true,
                "data": [{"key":"é".repeat(600),"url":"https://objects.example/x","size":1,"last_modified":null}],
                "meta": valid_meta()
            }),
            serde_json::json!({
                "success": true,
                "data": [{"key":"long-time","url":"https://objects.example/x","size":1,"last_modified":format!("2026-07-22T12:00:00.{}Z", "1".repeat(80))}],
                "meta": valid_meta()
            }),
            serde_json::json!({
                "success": true,
                "data": [],
                "meta": {"timestamp":"2026-07-23T00:00:00Z","version":"v1","pagination":{}}
            }),
        ] {
            assert!(matches!(
                classify_payload(payload(invalid)),
                AdminMediaLoad::Malformed
            ));
        }

        let extra_file_field = serde_json::json!({
            "success": true,
            "data": [{"key":"a","url":"https://objects.example/a","size":1,"last_modified":null,"etag":"secret"}],
            "meta": valid_meta()
        });
        assert!(serde_json::from_value::<LegacyMediaEnvelope>(extra_file_field).is_err());

        for unknown_field in [
            serde_json::json!({
                "success": true,
                "data": [],
                "meta": valid_meta(),
                "debug": "private"
            }),
            serde_json::json!({
                "success": true,
                "data": [],
                "meta": {
                    "timestamp": "2026-07-23T00:00:00Z",
                    "version": "v1",
                    "debug": "private"
                }
            }),
        ] {
            assert!(serde_json::from_value::<LegacyMediaEnvelope>(unknown_field).is_err());
        }

        let too_many = (0..=ADMIN_MEDIA_LIMIT)
            .map(|index| valid_item(&format!("{index:03}.png")))
            .collect::<Vec<_>>();
        assert!(matches!(
            classify_payload(payload(serde_json::json!({
                "success": true,
                "data": too_many,
                "meta": valid_meta()
            }))),
            AdminMediaLoad::Malformed
        ));
    }

    #[test]
    fn success_false_is_unavailable_and_empty_is_authoritative() {
        assert!(matches!(
            classify_payload(payload(serde_json::json!({
                "success": false,
                "error": {"provider":"private detail"},
                "meta": valid_meta()
            }))),
            AdminMediaLoad::Unavailable
        ));
        assert!(matches!(
            classify_payload(payload(serde_json::json!({
                "success": true,
                "data": [],
                "meta": valid_meta()
            }))),
            AdminMediaLoad::Empty(_)
        ));
    }

    #[tokio::test]
    async fn loader_sends_only_the_exact_bearer_read_and_request_id() {
        let (address, server) = serve_response(
            "200 OK",
            success_body(serde_json::json!([valid_item("news/banner.png")])),
        )
        .await;
        let load = load_admin_media(
            &loopback_client(address),
            &AdminMediaQuery::from_raw("").unwrap(),
            &verified_context(),
        )
        .await;
        let request = server.await.unwrap();
        assert!(request.starts_with("GET /api/admin/media/news?limit=100 HTTP/1.1\r\n"));
        let lowercase = request.to_ascii_lowercase();
        assert!(lowercase.contains("authorization: bearer verified-admin-token\r\n"));
        assert!(lowercase.contains("x-request-id: d9dbcc48-7f46-46cb-9b87-7cda68cb3af2\r\n"));
        for forbidden in ["x-user-id:", "x-user-address:", "0xspoofable-address"] {
            assert!(!lowercase.contains(forbidden), "leaked {forbidden}");
        }
        assert!(matches!(load, AdminMediaLoad::Ready(_)));
    }

    #[tokio::test]
    async fn loader_requires_verified_bearer_before_network_io() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let client = loopback_client(listener.local_addr().unwrap());
        let mut context = verified_context();
        context.auth_token = Some("  ".to_string());
        assert!(matches!(
            load_admin_media(&client, &AdminMediaQuery::from_raw("").unwrap(), &context).await,
            AdminMediaLoad::Unavailable
        ));
        assert!(
            tokio::time::timeout(Duration::from_millis(100), listener.accept())
                .await
                .is_err(),
            "a blank bearer reached the media upstream"
        );
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
            assert!(request.starts_with("GET /api/admin/media/news?limit=100 HTTP/1.1\r\n"));
            let response = format!(
                "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/bearer-leak\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let load = load_admin_media(
            &loopback_client(redirect_address),
            &AdminMediaQuery::from_raw("").unwrap(),
            &verified_context(),
        )
        .await;
        redirect_server.await.unwrap();
        assert!(matches!(load, AdminMediaLoad::Unavailable));
        assert!(
            tokio::time::timeout(Duration::from_millis(100), target_listener.accept())
                .await
                .is_err(),
            "the media bearer followed an upstream redirect"
        );
    }

    #[tokio::test]
    async fn loader_maps_statuses_without_exposing_error_bodies() {
        for (status, body, expected) in [
            (
                "400 Bad Request",
                br#"{"error":"bad"}"#.to_vec(),
                "malformed",
            ),
            (
                "403 Forbidden",
                br#"{"error":"secret"}"#.to_vec(),
                "forbidden",
            ),
            (
                "500 Internal Server Error",
                br#"{"error":"db"}"#.to_vec(),
                "unavailable",
            ),
            (
                "200 OK",
                serde_json::to_vec(&serde_json::json!({
                    "success": false,
                    "error": {"message":"provider secret"},
                    "meta": valid_meta()
                }))
                .unwrap(),
                "unavailable",
            ),
        ] {
            let (address, server) = serve_response(status, body).await;
            let load = load_admin_media(
                &loopback_client(address),
                &AdminMediaQuery::from_raw("bucket=public").unwrap(),
                &verified_context(),
            )
            .await;
            server.await.unwrap();
            assert!(
                matches!(
                    (&load, expected),
                    (AdminMediaLoad::Malformed, "malformed")
                        | (AdminMediaLoad::Forbidden, "forbidden")
                        | (AdminMediaLoad::Unavailable, "unavailable")
                ),
                "unexpected classification for {status}: {load:?}"
            );
        }
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
                MAX_ADMIN_MEDIA_RESPONSE_BYTES + 1
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        assert!(matches!(
            load_admin_media(
                &loopback_client(address),
                &AdminMediaQuery::from_raw("").unwrap(),
                &verified_context()
            )
            .await,
            AdminMediaLoad::Unavailable
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
            let body = vec![b'x'; MAX_ADMIN_MEDIA_RESPONSE_BYTES + 1];
            let header = format!("{:X}\r\n", body.len());
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(&body).await.unwrap();
            stream.write_all(b"\r\n0\r\n\r\n").await.unwrap();
        });
        assert!(matches!(
            load_admin_media(
                &loopback_client(address),
                &AdminMediaQuery::from_raw("").unwrap(),
                &verified_context()
            )
            .await,
            AdminMediaLoad::Unavailable
        ));
        streamed_server.await.unwrap();
    }
}
