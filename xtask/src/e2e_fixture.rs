use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};
use url::Url;

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const DEFAULT_BIND: &str = "127.0.0.1:48080";
const DEFAULT_CONTROL_TOKEN: &str = "epsx-e2e-local-reset-token";
const SIGNING_KEY_ID: &str = "epsx-e2e-rs256-v1";
const FIXTURE_TIMESTAMP: &str = "2026-01-01T00:00:00.000Z";
const FIXTURE_WALLET: &str = "0xea6400000000000000000000000000000000e3df";
const FIXTURE_PLAN_ID: &str = "00000000-0000-0000-0000-000000000001";
const FIXTURE_MERCHANT_ID: &str = "00000000-0000-0000-0000-000000000002";
const FIXTURE_NEWS_ID: &str = "00000000-0000-0000-0000-000000000006";
const FIXTURE_NOTIFICATION_ID: &str = "idem_notification_e2e_1";
const FIXTURE_CONVERSATION_ID: &str = "550e8400-e29b-41d4-a716-446655440000";
const FIXTURE_TOPIC_ID: &str = "550e8400-e29b-41d4-a716-446655440001";
const FIXTURE_MESSAGE_ID: &str = "550e8400-e29b-41d4-a716-446655440002";
const FIXTURE_API_KEY_ID: &str = "550e8400-e29b-41d4-a716-446655440003";
const FIXTURE_MODULE_ID: &str = "550e8400-e29b-41d4-a716-446655440004";
const FIXTURE_PAYMENT_INTENT_ID: &str = "intent_e2e_0001";
const FIXTURE_PAYMENT_LINK_ID: &str = "550e8400-e29b-41d4-a716-446655440005";
const SUPPORTED_GROUPS: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
const SUPPORTED_MODES: &[&str] = &[
    "healthy",
    "dependency-unavailable",
    "forbidden",
    "conflict",
    "analytics-empty",
    "analytics-limited",
    "analytics-malformed",
    "analytics-stale",
    "analytics-unavailable",
    "admin-dashboard-forbidden",
    "admin-dashboard-malformed",
    "admin-dashboard-unavailable",
    "admin-analytics-empty",
    "admin-analytics-forbidden",
    "admin-analytics-malformed",
    "admin-analytics-unavailable",
    "content-empty",
    "content-forbidden",
    "content-malformed",
    "content-unavailable",
    "notification-empty",
    "notification-forbidden",
    "notification-malformed",
    "notification-send-conflict",
    "notification-unavailable",
    "chat-empty",
    "chat-forbidden",
    "chat-malformed",
    "chat-mutation-conflict",
    "developer-conflict",
    "developer-empty",
    "developer-forbidden",
    "developer-malformed",
    "developer-unavailable",
    "payment-conflict",
    "payment-empty",
    "payment-forbidden",
    "payment-malformed",
    "payment-unavailable",
];

const SIGNING_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQC3Zucb7soDltXU
G5e/am1A1dC6zZyXA6TBse5ktX70zTTfIEsro7LoYF44UgWmM3iyrNAK5kVijIr4
hURnmaiPfxf6KO1XmRq4J4zav27yV7+LkHHX9EmFSokpZkAikhCjV2fW3acpkWBD
Yei+v5wWiSrXJdcccQr0BQieC+fP1a35jErN95VVdQ3sT+KvmBm0djGqMan4gGeW
6Zd1wXVJM2a/hPf+AcPtKfGN4MVU3l38nupiPnmcN5FN7A/f75IyXFLdd4sA51FR
QKdnnNXj3jJFuaE7k7O9eRdfFZsgOMN5lykz5aDYoBm+ju0a1RVXAyNm1DpURoi/
c3GrJlJ3AgMBAAECggEANnjIwaIdvA0ru1DqtD6e7nfTA/iXvr6lS6ZWYPELIRhl
0LOdv/th4uTkdyPda6yz95WeQO59wzRs/j1OwNqBlwUvkOxg+fiOWA3fJwVepXns
eT5Qocx7nawyquokuF/bszf9rnKs+IqmJb1JzIXKjWL2J2qkxlzI3Qs1sQNmOXMD
hUJZy0IEbTXs5Ix5r7dRWA2qUPLrHnWT8vm7oaGNYhJRaFTqTaauGRLVJ403zSoI
KXpVtU6k8MX4LlQlTpQC3ej0UnMqZewFf0aHDW1fv2cqab+el2V3I/EekMyDx68z
9EsZdult/wIZOP8BBCzWIyQE56OY+A1hvFL7w00qSQKBgQDsJ6FPkMYdAqLkEyjn
brNZqprkqpOXHluUZuwO5vOH9ragtIIIQoJKv5cxlwmY9dD/KPBjC+1MBFkzk3CM
ATNLgBqxA4/ZHwFCtPZr002IX3QtoZjM6pHUn8CN24Jp6QeBz+5Xw6c7YoMvclTb
GRhvhpTexzpWyeGNXobUDcPP2wKBgQDG0Gg5s78DkmfDktMfVuw7lOx8PrDC788R
3JlSYXe62bs9CDS1LFB8OCXxIj/vnjj6P4888SzPYZ5bW3F1cjc2o5TQe3E9DcEi
aclkrekmf649LpBTcQ66Gf7XDuC9qUIfMs5Kcre5FoY7XUFlTtkQjfj6x9AQ+Lhr
ebFdmeaIlQKBgQDIkkgxebaqAQk0SQmetqjhaUMxH6dG3GPPwTKQ3ZrNSb+G8ojW
VxauQdc6KRvfrDgb3zt8BC9BNxhD89/NKV/VqjIBUhMkx26cp3H71nWtc9UKxIsw
z7GYMy6pzVwQc/kKSf4W0HgCugLNk396ru/QGS/rnq5v8/r7xOMiy6YZrQKBgQCp
uk/QOwhuNzXIe/crAR0JvJirdSWYNfw0RnzKHJWHecvkTbYZmWxYr+KMWm301cHU
uiBBqa9UmAUF/yn8VvaV+c7YsRm6QpzIEUGyZtntWQFaD/98jL9C12B9HqF0qSPe
2JPOcOMx6u3LjlB++XJMNLgC+ERDyOJANpLZ0sJBhQKBgQCgpBL4H+ycCK4npGMG
SCZiOro0+J52I9plnzcnpT92bg3GrH5Wa72cMrfOTYg4T1KTQ2NbnCWIsvzNFV+W
v8c5kgY8YwO5hfBbV1VfoIMo3nu2rasMHbUzX9xnBxUB7PZD3bfbs3uBn29vdkmn
Wjh7jLLxLl6Tu7Awh6UNeNJ29w==
-----END PRIVATE KEY-----"#;

const SIGNING_MODULUS: &str = "t2bnG-7KA5bV1BuXv2ptQNXQus2clwOkwbHuZLV-9M003yBLK6Oy6GBeOFIFpjN4sqzQCuZFYoyK-IVEZ5moj38X-ijtV5kauCeM2r9u8le_i5Bx1_RJhUqJKWZAIpIQo1dn1t2nKZFgQ2Hovr-cFokq1yXXHHEK9AUIngvnz9Wt-YxKzfeVVXUN7E_ir5gZtHYxqjGp-IBnlumXdcF1STNmv4T3_gHD7SnxjeDFVN5d_J7qYj55nDeRTewP3--SMlxS3XeLAOdRUUCnZ5zV494yRbmhO5OzvXkXXxWbIDjDeZcpM-Wg2KAZvo7tGtUVVwMjZtQ6VEaIv3NxqyZSdw";

#[derive(Debug)]
struct FixtureState {
    mode: String,
    sequence: u64,
    requests: Vec<Value>,
    mutations: Vec<Value>,
}

impl Default for FixtureState {
    fn default() -> Self {
        Self {
            mode: "healthy".into(),
            sequence: 0,
            requests: Vec::new(),
            mutations: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
}

impl HttpResponse {
    fn json(value: Value) -> Self {
        Self::json_status(value, 200)
    }

    fn json_status(value: Value, status: u16) -> Self {
        Self {
            status,
            content_type: "application/json; charset=utf-8",
            body: serde_json::to_vec(&value).expect("fixture JSON is serializable"),
        }
    }

    fn event_stream(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "text/event-stream; charset=utf-8",
            body: body.as_bytes().to_vec(),
        }
    }
}

pub fn serve(flags: &[String]) -> Result<(), String> {
    let mut bind = env::var("E2E_FIXTURE_BIND").unwrap_or_else(|_| DEFAULT_BIND.into());
    let mut control_token =
        env::var("E2E_FIXTURE_TOKEN").unwrap_or_else(|_| DEFAULT_CONTROL_TOKEN.into());
    let mut index = 0;
    while index < flags.len() {
        let destination = match flags[index].as_str() {
            "--bind" => &mut bind,
            "--token" => &mut control_token,
            unknown => return Err(format!("e2e fixture-serve does not accept {unknown}")),
        };
        index += 1;
        *destination = flags
            .get(index)
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(|| format!("{} requires a value", flags[index - 1]))?
            .clone();
        index += 1;
    }
    let address: SocketAddr = bind
        .parse()
        .map_err(|error| format!("invalid fixture bind address {bind}: {error}"))?;
    if !address.ip().is_loopback() || address.port() < 1024 {
        return Err("fixture server must bind a non-privileged loopback address".into());
    }
    if control_token.len() < 16 || control_token.len() > 256 {
        return Err("fixture control token must contain 16 through 256 bytes".into());
    }
    let listener = TcpListener::bind(address)
        .map_err(|error| format!("could not bind fixture server to {address}: {error}"))?;
    let issuer = format!("http://{address}");
    let encoding_key = EncodingKey::from_rsa_pem(SIGNING_PRIVATE_KEY.as_bytes())
        .map_err(|error| format!("invalid fixture signing key: {error}"))?;
    let mut state = FixtureState::default();
    println!("rust e2e fixture: listening on {issuer}");
    for connection in listener.incoming() {
        match connection {
            Ok(mut stream) => {
                if let Err(error) = serve_connection(
                    &mut stream,
                    &issuer,
                    &control_token,
                    &encoding_key,
                    &mut state,
                ) {
                    eprintln!("rust e2e fixture request failed: {error}");
                }
            }
            Err(error) => eprintln!("rust e2e fixture connection failed: {error}"),
        }
    }
    Ok(())
}

fn serve_connection(
    stream: &mut TcpStream,
    issuer: &str,
    control_token: &str,
    encoding_key: &EncodingKey,
    state: &mut FixtureState,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .map_err(|error| format!("could not set fixture read timeout: {error}"))?;
    let request = read_request(stream)?;
    let method = request.method.clone();
    let origin = request.headers.get("origin").cloned();
    let response = route_request(request, issuer, control_token, encoding_key, state);
    write_response(stream, &method, origin.as_deref(), response)
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 8192];
    let header_end = loop {
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err("fixture request exceeds one MiB".into());
        }
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
        let count = stream
            .read(&mut chunk)
            .map_err(|error| format!("could not read fixture request: {error}"))?;
        if count == 0 {
            return Err("fixture request ended before its headers".into());
        }
        bytes.extend_from_slice(&chunk[..count]);
    };
    let headers_text = std::str::from_utf8(&bytes[..header_end])
        .map_err(|_| "fixture request headers are not UTF-8".to_string())?;
    let mut lines = headers_text.split("\r\n");
    let mut request_line = lines
        .next()
        .ok_or_else(|| "fixture request line is missing".to_string())?
        .split_whitespace();
    let method = request_line
        .next()
        .ok_or_else(|| "fixture request method is missing".to_string())?
        .to_ascii_uppercase();
    let target = request_line
        .next()
        .ok_or_else(|| "fixture request target is missing".to_string())?
        .to_string();
    if request_line.next() != Some("HTTP/1.1") || request_line.next().is_some() {
        return Err("fixture server requires an HTTP/1.1 request".into());
    }
    let mut headers = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| "fixture request contains a malformed header".to_string())?;
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
    }
    let content_length = headers
        .get("content-length")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| "fixture request has an invalid content-length".to_string())?
        .unwrap_or(0);
    if content_length > MAX_REQUEST_BYTES || header_end + content_length > MAX_REQUEST_BYTES {
        return Err("fixture request exceeds one MiB".into());
    }
    while bytes.len() < header_end + content_length {
        let count = stream
            .read(&mut chunk)
            .map_err(|error| format!("could not read fixture request body: {error}"))?;
        if count == 0 {
            return Err("fixture request body ended early".into());
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    Ok(HttpRequest {
        method,
        target,
        headers,
        body: bytes[header_end..header_end + content_length].to_vec(),
    })
}

fn write_response(
    stream: &mut TcpStream,
    method: &str,
    origin: Option<&str>,
    response: HttpResponse,
) -> Result<(), String> {
    let reason = match response.status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        413 => "Content Too Large",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let mut headers = format!(
        "HTTP/1.1 {} {}\r\ncontent-type: {}\r\ncontent-length: {}\r\ncache-control: no-store\r\nx-epsx-e2e-fixture: 1\r\naccess-control-allow-headers: authorization,content-type,x-api-version,x-access-level,x-request-id,x-epsx-e2e-token\r\naccess-control-allow-methods: GET,HEAD,POST,PUT,PATCH,DELETE,OPTIONS\r\nconnection: close\r\n",
        response.status,
        reason,
        response.content_type,
        response.body.len()
    );
    if let Some(origin) = origin.filter(|origin| !origin.is_empty()) {
        headers.push_str(&format!(
            "access-control-allow-origin: {origin}\r\naccess-control-allow-credentials: true\r\nvary: origin\r\n"
        ));
    } else {
        headers.push_str("access-control-allow-origin: *\r\n");
    }
    headers.push_str("\r\n");
    stream
        .write_all(headers.as_bytes())
        .and_then(|_| {
            if method == "HEAD" {
                Ok(())
            } else {
                stream.write_all(&response.body)
            }
        })
        .map_err(|error| format!("could not write fixture response: {error}"))
}

fn route_request(
    request: HttpRequest,
    issuer: &str,
    control_token: &str,
    encoding_key: &EncodingKey,
    state: &mut FixtureState,
) -> HttpResponse {
    let parsed = match Url::parse(&format!("{issuer}{}", request.target)) {
        Ok(url) if url.origin().ascii_serialization() == issuer => url,
        _ => return error("invalid_request_target", 400),
    };
    let path = parsed.path();
    if request.method == "OPTIONS" {
        return HttpResponse::json(json!({}));
    }
    if path.starts_with("/__e2e/") {
        return control_request(request, parsed, issuer, control_token, encoding_key, state);
    }

    state.sequence += 1;
    let query = parsed
        .query_pairs()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    let body_sha256 = if request.body.is_empty() {
        None
    } else {
        Some(format!("{:x}", Sha256::digest(&request.body)))
    };
    let entry = json!({
        "sequence": state.sequence,
        "method": request.method,
        "path": path,
        "query": query,
        "bodySha256": body_sha256,
    });
    state.requests.push(entry.clone());
    if !matches!(request.method.as_str(), "GET" | "HEAD") {
        state.mutations.push(entry);
    }

    dependency_request(&request, &parsed, state)
}

fn authorized(request: &HttpRequest, token: &str) -> bool {
    request.headers.get("x-epsx-e2e-token").map(String::as_str) == Some(token)
}

fn control_request(
    request: HttpRequest,
    url: Url,
    issuer: &str,
    control_token: &str,
    encoding_key: &EncodingKey,
    state: &mut FixtureState,
) -> HttpResponse {
    if !authorized(&request, control_token) {
        return error("forbidden", 403);
    }
    match (request.method.as_str(), url.path()) {
        ("GET", "/__e2e/capabilities") => HttpResponse::json(json!({
            "schemaVersion": 1,
            "authority": "epsx-rust-e2e-fixture",
            "sessionAlgorithm": "RS256",
            "keyId": SIGNING_KEY_ID,
            "supportedGroups": SUPPORTED_GROUPS,
            "supportedModes": SUPPORTED_MODES,
            "resetProof": true
        })),
        ("POST", "/__e2e/reset") => {
            *state = FixtureState::default();
            HttpResponse::json(json!({
                "schemaVersion": 1,
                "reset": true,
                "mode": state.mode,
                "requestCount": 0,
                "mutationCount": 0
            }))
        }
        ("GET", "/__e2e/state") => HttpResponse::json(json!({
            "schemaVersion": 1,
            "requestCount": state.requests.len(),
            "requests": state.requests,
            "mutations": state.mutations,
            "mode": state.mode,
        })),
        ("PUT", "/__e2e/mode") => {
            let mode = serde_json::from_slice::<Value>(&request.body)
                .ok()
                .and_then(|body| body.get("mode").and_then(Value::as_str).map(str::to_owned));
            match mode.filter(|mode| SUPPORTED_MODES.contains(&mode.as_str())) {
                Some(mode) => {
                    state.mode = mode;
                    HttpResponse::json(json!({"schemaVersion":1,"mode":state.mode}))
                }
                None => error("unsupported_mode", 400),
            }
        }
        ("GET", "/__e2e/session") => {
            let audience = url
                .query_pairs()
                .find(|(key, _)| key == "audience")
                .map(|(_, value)| value.into_owned());
            let permissions = url
                .query_pairs()
                .find(|(key, _)| key == "permissions")
                .map(|(_, value)| value.into_owned())
                .unwrap_or_default();
            let key_id = url
                .query_pairs()
                .find(|(key, _)| key == "key_id")
                .map(|(_, value)| value.into_owned())
                .unwrap_or_else(|| SIGNING_KEY_ID.into());
            let Some(audience) =
                audience.filter(|value| matches!(value.as_str(), "epsx-frontend" | "epsx-admin"))
            else {
                return error("invalid_audience", 400);
            };
            if !key_id.starts_with("epsx-e2e-rs256-")
                || key_id.len() > 80
                || !key_id.chars().all(|character| {
                    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
                })
            {
                return error("invalid_key_id", 400);
            }
            match fixture_access_token(issuer, &audience, &permissions, &key_id, encoding_key) {
                Ok(token) => HttpResponse::json(json!({
                    "schemaVersion": 1,
                    "accessToken": token,
                    "audience": audience,
                    "keyId": key_id
                })),
                Err(error) => HttpResponse::json_status(json!({"error":error}), 503),
            }
        }
        _ => error("control_route_not_found", 404),
    }
}

fn fixture_access_token(
    issuer: &str,
    audience: &str,
    permissions: &str,
    key_id: &str,
    encoding_key: &EncodingKey,
) -> Result<String, String> {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(key_id.into());
    encode(
        &header,
        &json!({
            "iss": issuer,
            "sub": FIXTURE_WALLET,
            "aud": [audience],
            "exp": 2524608000_i64,
            "iat": 1785283200_i64,
            "nbf": 1785283170_i64,
            "jti": format!("epsx-e2e-{audience}"),
            "scope": permissions,
            "wallet_address": FIXTURE_WALLET,
            "auth_method": "web3_siwe",
            "auth_time": 1785283200_i64,
        }),
        encoding_key,
    )
    .map_err(|error| format!("could not sign fixture session: {error}"))
}

fn dependency_request(request: &HttpRequest, url: &Url, state: &FixtureState) -> HttpResponse {
    let path = url.path();
    let mutation = !matches!(request.method.as_str(), "GET" | "HEAD");
    let admin_content_path = path.starts_with("/api/admin/news")
        || path.starts_with("/api/admin/media")
        || path.starts_with("/api/admin/files");
    let public_content_path = path == "/api/v1/content/news"
        || path.starts_with("/api/v1/content/news/")
        || path == "/api/public/news"
        || (path.starts_with("/api/public/news/") && path != "/api/public/news/featured");
    if state.mode == "dependency-unavailable" && path.contains("jwks") {
        return error("dependency_unavailable", 503);
    }
    if path.contains("jwks") {
        return HttpResponse::json(json!({"keys":[{
            "kty":"RSA",
            "use":"sig",
            "alg":"RS256",
            "kid":SIGNING_KEY_ID,
            "n":SIGNING_MODULUS,
            "e":"AQAB"
        }]}));
    }
    if let Some(required) = required_admin_permission(path, &request.method) {
        let authorized = fixture_principal(request).is_some_and(|principal| {
            principal.audience == "epsx-admin"
                && permission_allows(&principal.permissions, required)
        });
        if !authorized {
            return error("forbidden", 403);
        }
    }
    if state.mode == "content-forbidden" && admin_content_path {
        return error("forbidden", 403);
    }
    if state.mode == "content-unavailable" && (admin_content_path || public_content_path) {
        return error("dependency_unavailable", 503);
    }
    if state.mode == "content-malformed" && (admin_content_path || public_content_path) {
        return HttpResponse::json(json!({"malformed":true}));
    }
    if state.mode == "forbidden"
        && (path.starts_with("/api/v1/admin/") || path.starts_with("/api/v1/analytics/admin/"))
    {
        return error("forbidden", 403);
    }
    if state.mode == "conflict" && mutation {
        return error("optimistic_conflict", 409);
    }
    if is_notification_path(path) {
        if let Some(response) = notification_response(request, url, &state.mode) {
            return response;
        }
    }
    if path.starts_with("/api/admin/chat/") {
        if let Some(response) = chat_response(request, url, &state.mode) {
            return response;
        }
    }
    if path.starts_with("/api/admin/developer-portal/") {
        if let Some(response) = developer_response(request, url, &state.mode) {
            return response;
        }
    }
    if path.starts_with("/api/v1/admin/pay/") {
        if let Some(response) = payment_response(request, url, &state.mode) {
            return response;
        }
    }
    match path {
        "/health" | "/api/health" => HttpResponse::json(json!({
            "status":"ok",
            "service":"epsx-rust-e2e-fixture"
        })),
        "/api/analytics/rankings" | "/api/public/analytics/rankings" => {
            rankings_response(url, &state.mode)
        }
        "/api/analytics/filters" | "/api/public/analytics/filters" => HttpResponse::json(json!({
            "success":true,
            "data":{
                "countries":[{"value":"america","label":"United States"}],
                "sectors":["Technology"],
                "exchanges":["NASDAQ"],
                "stock_types":["common"]
            }
        })),
        "/api/v1/content/news" => content_list(&state.mode),
        "/api/public/news/featured" => HttpResponse::json(json!({
            "success":true,
            "data":[public_news_article()]
        })),
        "/api/auth/web3/challenge" => HttpResponse::json(json!({
            "success":true,
            "nonce":"epsx-e2e-nonce",
            "message":"epsx.io wants you to sign in with your Ethereum account",
            "expires_at":2524608000_i64,
            "wallet_address":FIXTURE_WALLET
        })),
        "/api/auth/web3/logout" => HttpResponse::json(json!({"success":true,"revoked":true})),
        "/api/users/profile" | "/api/admin/me" => profile_response(request),
        "/api/admin/settings" => HttpResponse::json(json!({
            "data":{
                "general":{"systemName":"EPSX Admin","adminEmail":"admin@epsx.io","maintenanceMode":false},
                "notifications":{"emailNotifications":true,"pushNotifications":false,"smsNotifications":true,"securityAlerts":true},
                "security":{"sessionTimeout":30},
                "appearance":{"theme":"auto","primaryColor":"#3b82f6"}
            }
        })),
        "/api/admin/news" => admin_news_response(request, url, &state.mode),
        "/api/admin/news/upload-image" => admin_content_envelope(
            json!({
                "url":"https://assets.epsx.invalid/news/cover.png",
                "thumb_url":null,
                "filename":"migration-cover.png",
                "mime":"image/png",
                "size":68
            }),
            200,
        ),
        "/api/admin/files/upload" => admin_content_envelope(
            json!({
                "bucket":"public",
                "key":"uploads/migration-proof.txt",
                "url":format!("{}/__e2e/media/public/migration-proof.txt",url.origin().ascii_serialization()),
                "thumb_url":null,
                "mime":"text/plain",
                "size":21,
                "deleted":false
            }),
            200,
        ),
        "/api/notifications/preferences" => HttpResponse::json(json!({
            "success":true,
            "data":{"preferences":{"analytics":true,"security":true,"account":true,"system":false,"marketing":false}}
        })),
        "/api/users/permissions/status" => permission_status(request),
        "/api/permissions/definitions" => permission_definitions(request),
        "/api/users/access-overview" => {
            let Some(principal) = fixture_principal(request) else {
                return error("authentication_required", 401);
            };
            HttpResponse::json(json!({
                "success":true,
                "data":{"plan":"migration-e2e","permissions":principal.permissions,"expires_at":null}
            }))
        }
        "/api/users/watchlist" => HttpResponse::json(json!({
            "success":true,"data":{"symbols":["NVDA"]}
        })),
        "/api/users/portfolio/overview" => HttpResponse::json(json!({
            "success":true,
            "data":{
                "watchlist":["NVDA"],
                "rankings":[
                    ranking(1,"NVDA","NVIDIA Corporation",4.12,184.25,42.5),
                    ranking(2,"MSFT","Microsoft Corporation",3.88,512.4,24.1)
                ]
            }
        })),
        "/api/payments/plans/my-plan-access" => HttpResponse::json(json!({
            "success":true,
            "data":{
                "wallet_address":FIXTURE_WALLET,
                "plan_name":"Migration Professional",
                "plan_id":FIXTURE_PLAN_ID,
                "plan_expires_at":null,
                "days_remaining":30,
                "status":"active",
                "ranking_offset":1,
                "can_upgrade":false,
                "tier_level":2,
                "proration_credit":null,
                "current_plan_price":"29"
            }
        })),
        "/api/admin/dashboard/user-status" => admin_dashboard_response(&state.mode),
        "/api/admin/analytics/dashboard" => admin_analytics_response(&state.mode),
        "/api/admin/dashboard/summary" => HttpResponse::json(json!({
            "success":true,
            "data":{
                "wallet_stats":{"total":12,"active":10,"today_connections":2},
                "permission_stats":{"total":8,"pending_notifications":0},
                "system_health":null
            }
        })),
        "/api/admin/web3/recent-wallets" => HttpResponse::json(json!({
            "success":true,
            "data":{
                "recent_wallets":[{
                    "wallet_address":FIXTURE_WALLET,
                    "metadata":{},
                    "created_at":FIXTURE_TIMESTAMP,
                    "last_auth_at":FIXTURE_TIMESTAMP,
                    "is_active":true,
                    "active_permissions_count":2,
                    "connection_info":{"is_new":false,"last_seen":1788278400_i64}
                }],
                "analytics":{
                    "total_in_period":1,
                    "daily_breakdown":[{"date":"2026-07-01","connections":1}],
                    "period_days":30,
                    "avg_daily":0.03
                },
                "metadata":{"limit":10,"total_count":1,"has_more":false,"generated_at":FIXTURE_TIMESTAMP}
            }
        })),
        "/api/payments/credits/balance" => HttpResponse::json(json!({
            "success":true,
            "data":{
                "wallet_address":FIXTURE_WALLET,
                "balance":120,
                "pending_balance":0,
                "available_balance":120,
                "lifetime_earned":160,
                "lifetime_spent":40,
                "last_transaction_at":FIXTURE_TIMESTAMP
            }
        })),
        "/api/payments/credits/history" => HttpResponse::json(json!({
            "success":true,
            "data":{"success":true,"data":[{
                "id":"credit-e2e-1",
                "wallet_address":FIXTURE_WALLET,
                "amount":120,
                "balance_after":120,
                "tx_type":"grant",
                "reference_id":null,
                "reference_type":null,
                "reason":"Migration baseline",
                "granted_by":FIXTURE_WALLET,
                "expires_at":null,
                "created_at":FIXTURE_TIMESTAMP
            }],"count":1}
        })),
        "/api/payments/history" => HttpResponse::json(json!({
            "success":true,
            "data":{"payments":[],"pagination":{"page":1,"per_page":10,"total":0,"total_pages":1}}
        })),
        "/api/v1/admin/wallets/stats" => HttpResponse::json(json!({
            "total":1,"active":1,"disabled":0,"new_30_days":1,"correlation_id":"e2e-wallet-stats"
        })),
        "/api/v1/admin/wallets" => HttpResponse::json(json!({
            "items":[fixture_wallet("active",3)],
            "total":1,"limit":100,"offset":0,"correlation_id":"e2e-wallet-list"
        })),
        "/api/v1/admin/credits" => HttpResponse::json(json!({
            "outstanding_minor":12000,
            "granted_today_minor":2000,
            "revoked_today_minor":500,
            "active_accounts":1,
            "correlation_id":"e2e-credit-stats"
        })),
        "/api/v1/admin/subscription/access" => HttpResponse::json(json!({
            "items":[{
                "wallet_address":FIXTURE_WALLET,
                "plan_id":FIXTURE_PLAN_ID,
                "plan_name":"Migration Professional",
                "permission":"epsx:analytics:read",
                "expires_at":null,
                "version":2,
                "assigned_by":FIXTURE_WALLET,
                "updated_at":FIXTURE_TIMESTAMP
            }],
            "correlation_id":"e2e-access-list"
        })),
        "/api/v1/admin/subscription/plans" => HttpResponse::json(json!({
            "items":[fixture_plan()],
            "total":1,"limit":100,"offset":0,"correlation_id":"e2e-plan-list"
        })),
        "/api/v1/analytics/admin/audit-log" => {
            let disable_path = format!("/api/v1/admin/wallets/{FIXTURE_WALLET}/disable");
            let wallet_disabled = state.mutations.iter().any(|mutation| {
                mutation.get("path").and_then(Value::as_str) == Some(disable_path.as_str())
            });
            HttpResponse::json(json!({
                "items":[{
                    "id":"00000000-0000-0000-0000-000000000005",
                    "category":"wallet",
                    "action":if wallet_disabled { "wallet.disabled" } else { "wallet.reviewed" },
                    "resource_type":"wallet",
                    "effect":"success",
                    "occurred_at":FIXTURE_TIMESTAMP
                }],
                "next_cursor":null,
                "has_more":false
            }))
        }
        _ => dependency_dynamic_path(request, path, &state.mode),
    }
}

fn dependency_dynamic_path(request: &HttpRequest, path: &str, mode: &str) -> HttpResponse {
    let wallet_path = format!("/api/v1/admin/wallets/{FIXTURE_WALLET}");
    let plan_path = format!("/api/v1/admin/subscription/plans/{FIXTURE_PLAN_ID}");
    if path == wallet_path {
        return HttpResponse::json(fixture_wallet("active", 3));
    }
    if path == format!("{wallet_path}/disable") {
        return HttpResponse::json(json!({
            "wallet":fixture_wallet("disabled",4),
            "evidence":{
                "operation_id":"00000000-0000-0000-0000-000000000003",
                "version":4,
                "observed_at":FIXTURE_TIMESTAMP
            },
            "correlation_id":"e2e-wallet-disable"
        }));
    }
    if path.starts_with("/api/v1/admin/credits/") {
        return HttpResponse::json(json!({
            "transaction_id":"00000000-0000-0000-0000-000000000004",
            "version":4,
            "correlation_id":"e2e-credit-mutation"
        }));
    }
    if path.starts_with("/api/v1/admin/subscription/access/") {
        return HttpResponse::json(json!({"success":true,"correlation_id":"e2e-access-mutation"}));
    }
    if path == plan_path {
        return if matches!(request.method.as_str(), "GET" | "HEAD") {
            HttpResponse::json(fixture_plan())
        } else {
            HttpResponse::json(json!({"success":true,"id":FIXTURE_PLAN_ID,"version":8}))
        };
    }
    if let Some(slug) = path.strip_prefix("/api/v1/content/news/") {
        return if slug == "deterministic-market-brief" {
            HttpResponse::json(json!({"success":true,"data":public_news_article(),"error":null}))
        } else {
            error("not_found", 404)
        };
    }
    if let Some(tail) = path.strip_prefix("/api/admin/news/") {
        let mut segments = tail.split('/');
        let id = segments.next().unwrap_or_default();
        let operation = segments.next();
        if id != FIXTURE_NEWS_ID || segments.next().is_some() {
            return error("not_found", 404);
        }
        if request.method == "DELETE" && operation.is_none() {
            return admin_content_envelope(json!({"id":FIXTURE_NEWS_ID,"deleted":true}), 200);
        }
        return admin_content_envelope(
            admin_news_article(
                if operation == Some("unpublish") {
                    "draft"
                } else {
                    "published"
                },
                operation == Some("pin"),
            ),
            200,
        );
    }
    if let Some(tail) = path.strip_prefix("/api/admin/media/") {
        let mut segments = tail.split('/');
        let bucket = segments.next().unwrap_or_default();
        if !matches!(bucket, "news" | "public") {
            return error("invalid_bucket", 400);
        }
        let key = segments.collect::<Vec<_>>().join("/");
        if request.method == "DELETE" {
            return admin_content_envelope(
                json!({
                    "bucket":bucket,"key":key,"url":null,"thumb_url":null,
                    "mime":null,"size":null,"deleted":true
                }),
                200,
            );
        }
        let items = if mode == "content-empty" {
            Vec::new()
        } else {
            vec![json!({
                "key":if bucket == "public" { "guides/getting-started.pdf" } else { "news/release-notes.pdf" },
                "url":format!("http://127.0.0.1:48080/__e2e/media/{bucket}/fixture.pdf"),
                "size":4096,
                "last_modified":FIXTURE_TIMESTAMP
            })]
        };
        return admin_content_envelope(Value::Array(items), 200);
    }
    error_with_path("fixture_route_not_implemented", path, 404)
}

fn is_notification_path(path: &str) -> bool {
    path.starts_with("/api/v1/notification/")
        || path.starts_with("/api/v1/notifications/")
        || matches!(
            path,
            "/api/notifications/stream" | "/api/admin/notifications"
        )
}

fn notification_response(request: &HttpRequest, url: &Url, mode: &str) -> Option<HttpResponse> {
    let path = url.path();
    let mutation = !matches!(request.method.as_str(), "GET" | "HEAD");
    let admin_path =
        path.starts_with("/api/v1/notification/admin") || path == "/api/v1/notification/send";
    if mode == "notification-forbidden" && admin_path {
        return Some(error("forbidden", 403));
    }
    if mode == "notification-unavailable"
        && (path.starts_with("/api/v1/notification/") || path.starts_with("/api/v1/notifications/"))
    {
        return Some(error("dependency_unavailable", 503));
    }
    if mode == "notification-malformed"
        && (path.starts_with("/api/v1/notification/") || path.starts_with("/api/v1/notifications/"))
    {
        return Some(HttpResponse::json(json!({"malformed":true})));
    }
    if mode == "notification-send-conflict" && path == "/api/v1/notification/send" {
        return Some(error("idempotency_conflict", 409));
    }

    let response = match path {
        "/api/v1/notification/preferences" => {
            let required = if mutation {
                "epsx:notifications:update"
            } else {
                "epsx:notifications:read"
            };
            let allowed = fixture_principal(request).is_some_and(|principal| {
                principal.audience == "epsx-frontend"
                    && permission_allows(&principal.permissions, required)
            });
            if !allowed {
                error("forbidden", 403)
            } else {
                HttpResponse::json(json!({
                    "channels":{"email":true,"in_app":true,"push":false},
                    "quiet_hours":{"enabled":true,"start":"22:00","end":"07:00"},
                    "timezone":"Asia/Bangkok",
                    "updated_at":FIXTURE_TIMESTAMP
                }))
            }
        }
        "/api/v1/notification/push"
        | "/api/v1/notification/push/unsubscribe"
        | "/api/v1/notifications/push"
        | "/api/v1/notifications/push/unsubscribe" => HttpResponse::json(json!({
            "enabled":false,"subscribed":false,"public_key":null
        })),
        "/api/notifications/stream"
        | "/api/v1/notification/stream"
        | "/api/v1/notifications/stream" => {
            HttpResponse::event_stream(": deterministic fixture stream connected\n\n")
        }
        "/api/v1/notification/stream/ack" | "/api/v1/notifications/stream/ack" => {
            HttpResponse::json(json!({"acknowledged":true}))
        }
        "/api/admin/notifications" => HttpResponse::json(json!({
            "success":true,
            "data":{"notifications":[],"pagination":{"page":1,"limit":5,"total":0,"total_pages":1}}
        })),
        "/api/v1/notification/list" => {
            let items = if mode == "notification-empty" {
                Vec::new()
            } else {
                vec![json!({
                    "id":FIXTURE_NOTIFICATION_ID,"user_id":FIXTURE_WALLET,
                    "channel":"in_app","recipient":FIXTURE_WALLET,"template_id":null,
                    "subject":"Security notice",
                    "body":"Your deterministic migration notification is ready.",
                    "data":null,"status":"sent","error":null,"sent_at":FIXTURE_TIMESTAMP,
                    "created_at":FIXTURE_TIMESTAMP,"read_at":null,"clicked_at":null,
                    "title":"Migration notification","notification_type":"security",
                    "priority":"high","action_url":null,"expires_at":null
                })]
            };
            HttpResponse::json(json!({"total":items.len(),"items":items}))
        }
        "/api/v1/notification/unread-count" => {
            let can_read = fixture_principal(request).is_some_and(|principal| {
                principal.audience == "epsx-frontend"
                    && permission_allows(&principal.permissions, "epsx:notifications:read")
            });
            HttpResponse::json(json!({
                "count":if can_read && mode != "notification-empty" { 1 } else { 0 }
            }))
        }
        "/api/v1/notification/mark-all-read" | "/api/v1/notification/clear-all" => {
            HttpResponse::json(json!({"success":true,"updated_count":1,"deleted_count":1}))
        }
        "/api/v1/notification/send" => HttpResponse::json(json!({
            "id":"idem_notification_send_e2e","status":"sent","delivered":true,
            "request_id":"epsx-e2e-notification-send"
        })),
        "/api/v1/notification/admin/list" => {
            let items = if mode == "notification-empty" {
                Vec::new()
            } else {
                vec![json!({
                    "id":FIXTURE_NOTIFICATION_ID,"title":"Migration notification",
                    "subject":"Security notice","channel":"in_app","status":"sent",
                    "notification_type":"security","priority":"high",
                    "sent_at":FIXTURE_TIMESTAMP,"created_at":FIXTURE_TIMESTAMP
                })]
            };
            let offset = url
                .query_pairs()
                .find(|(key, _)| key == "offset")
                .and_then(|(_, value)| value.parse::<u64>().ok())
                .unwrap_or(0);
            HttpResponse::json(json!({
                "total":items.len(),"items":items,"limit":20,"offset":offset
            }))
        }
        "/api/v1/notification/admin/metrics" => HttpResponse::json(json!({
            "queue_depth":1,"queue_age_seconds":5,"suppressed":1,"retry_wait":1,
            "terminal_failed":1,"dead_lettered":1,"provider_accepted":3,"attempting":1,
            "channel_outcomes":{"email":1,"in_app":2,"push":0},"provider_events":4,
            "delivery_attempts":5,"replay_cursors":1,"replay_cursor_age_seconds":2,
            "active_streams":1,"stream_connections_total":3,"stream_reconnects_total":1,
            "stream_replayed_events_total":2,"stream_lag_seconds":1,
            "stream_query_failures_total":0
        })),
        _ if is_notification_item_mutation(path) => {
            HttpResponse::json(json!({"success":true,"updated_count":1,"deleted_count":1}))
        }
        _ => return None,
    };
    Some(response)
}

fn is_notification_item_mutation(path: &str) -> bool {
    let Some(tail) = path.strip_prefix("/api/v1/notification/") else {
        return false;
    };
    if tail.starts_with("admin/") || tail.is_empty() {
        return false;
    }
    let segments = tail.split('/').collect::<Vec<_>>();
    (segments.len() == 1 || (segments.len() == 2 && segments[1] == "read"))
        && segments.first().is_some_and(|id| {
            id.bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        })
}

fn chat_response(request: &HttpRequest, url: &Url, mode: &str) -> Option<HttpResponse> {
    let path = url.path();
    let mutation = !matches!(request.method.as_str(), "GET" | "HEAD");
    if mode == "chat-forbidden" {
        return Some(error("forbidden", 403));
    }
    if mode == "chat-malformed" && !mutation {
        return Some(HttpResponse::json(
            json!({"success":true,"data":{"malformed":true},"error":null}),
        ));
    }
    if mode == "chat-mutation-conflict" && mutation {
        return Some(error("idempotency_conflict", 409));
    }
    let conversation = json!({
        "id":FIXTURE_CONVERSATION_ID,"topic_id":FIXTURE_TOPIC_ID,
        "wallet_address":FIXTURE_WALLET,"subject":"Migration support conversation",
        "status":"open","assigned_agent":FIXTURE_WALLET,"last_message_at":FIXTURE_TIMESTAMP,
        "unread_user":0,"unread_agent":1,"created_at":FIXTURE_TIMESTAMP,
        "updated_at":FIXTURE_TIMESTAMP
    });
    if path == "/api/admin/chat/conversations" {
        let items = if mode == "chat-empty" {
            Vec::new()
        } else {
            vec![conversation]
        };
        let page = url
            .query_pairs()
            .find(|(key, _)| key == "page")
            .and_then(|(_, value)| value.parse::<u64>().ok())
            .unwrap_or(1);
        let limit = url
            .query_pairs()
            .find(|(key, _)| key == "limit")
            .and_then(|(_, value)| value.parse::<u64>().ok())
            .unwrap_or(20);
        return Some(HttpResponse::json(json!({
            "success":true,
            "data":{"total":items.len(),"items":items,"page":page,"limit":limit,"has_next":false},
            "error":null,"meta":{"timestamp":FIXTURE_TIMESTAMP,"request_id":"epsx-e2e-chat-list"}
        })));
    }
    let base = format!("/api/admin/chat/conversations/{FIXTURE_CONVERSATION_ID}");
    if path == base {
        return Some(HttpResponse::json(json!({
            "success":true,"data":conversation,"error":null,
            "meta":{"timestamp":FIXTURE_TIMESTAMP,"request_id":"epsx-e2e-chat-detail"}
        })));
    }
    if path == format!("{base}/messages") {
        return Some(if mutation {
            HttpResponse::json(json!({"success":true}))
        } else {
            HttpResponse::json(json!({
                "success":true,"data":[{
                    "id":FIXTURE_MESSAGE_ID,"conversation_id":FIXTURE_CONVERSATION_ID,
                    "sender_type":"user","sender_address":FIXTURE_WALLET,
                    "content":"Please verify my migration notification.","is_read":false,
                    "created_at":FIXTURE_TIMESTAMP
                }],"error":null,
                "meta":{"timestamp":FIXTURE_TIMESTAMP,"request_id":"epsx-e2e-chat-messages"}
            }))
        });
    }
    if ["status", "assign", "read"]
        .iter()
        .any(|operation| path == format!("{base}/{operation}"))
    {
        return Some(HttpResponse::json(json!({"success":true})));
    }
    None
}

fn developer_response(request: &HttpRequest, url: &Url, mode: &str) -> Option<HttpResponse> {
    let path = url.path();
    let mutation = !matches!(request.method.as_str(), "GET" | "HEAD");
    if mode == "developer-forbidden" {
        return Some(error("forbidden", 403));
    }
    if mode == "developer-unavailable" {
        return Some(error("dependency_unavailable", 503));
    }
    if mode == "developer-malformed" && !mutation {
        return Some(HttpResponse::json(
            json!({"success":true,"data":{"full_key":"must-not-project"}}),
        ));
    }
    if mode == "developer-conflict" && mutation {
        return Some(error("idempotency_conflict", 409));
    }
    let key = developer_key();
    let response = match path {
        "/api/admin/developer-portal/api-keys" if mutation => HttpResponse::json_status(
            json!({
                "success":true,
                "data":{"api_key":key,"secret":"epsx_live_e2e_secret_once_7f4a"},
                "error":null,"meta":fixture_response_meta()
            }),
            201,
        ),
        "/api/admin/developer-portal/api-keys" => {
            let api_keys = if mode == "developer-empty" {
                Vec::new()
            } else {
                vec![key]
            };
            HttpResponse::json(json!({
                "success":true,"data":{"total":api_keys.len(),"api_keys":api_keys},
                "error":null,"meta":fixture_response_meta()
            }))
        }
        "/api/admin/developer-portal/stats" => {
            let empty = mode == "developer-empty";
            HttpResponse::json(json!({
                "success":true,
                "data":{
                    "total_api_keys":if empty {0} else {1},
                    "active_api_keys":if empty {0} else {1},
                    "revoked_api_keys":0,"expired_api_keys":0,
                    "total_modules":if empty {0} else {1},
                    "active_modules":if empty {0} else {1},
                    "total_requests_today":if empty {0} else {7},
                    "total_requests_this_month":if empty {0} else {42},
                    "top_modules_by_usage":if empty { Vec::<Value>::new() } else { vec![json!({
                        "module_id":FIXTURE_MODULE_ID,"module_name":"Market analytics",
                        "request_count":42,"unique_api_keys":1
                    })] }
                },
                "error":null,"meta":fixture_response_meta()
            }))
        }
        _ if path
            == format!("/api/admin/developer-portal/api-keys/{FIXTURE_API_KEY_ID}/revoke")
            || path
                == format!(
                    "/api/admin/developer-portal/api-keys/{FIXTURE_API_KEY_ID}/expiration"
                ) =>
        {
            HttpResponse::json(json!({
                "success":true,"data":key,"error":null,"meta":fixture_response_meta()
            }))
        }
        _ => return None,
    };
    Some(response)
}

fn developer_key() -> Value {
    json!({
        "id":FIXTURE_API_KEY_ID,"key_prefix":"epsx_e2e1234",
        "client_name":"Migration integration","client_description":null,
        "client_contact_email":null,"wallet_address":FIXTURE_WALLET,
        "status":"active","total_requests":42,"ip_restrictions":[],
        "rate_limits":{"per_minute":60,"per_day":10_000},"allowed_modules":[],
        "permission_plans":[],"selected_permissions":["epsx:analytics:read"],
        "expires_at":null,"last_used_at":FIXTURE_TIMESTAMP,"revoked_at":null,
        "revoked_by":null,"revocation_reason":null,"created_at":FIXTURE_TIMESTAMP,
        "created_by":FIXTURE_WALLET,"updated_at":FIXTURE_TIMESTAMP
    })
}

fn fixture_response_meta() -> Value {
    json!({"timestamp":FIXTURE_TIMESTAMP,"version":"v1"})
}

fn payment_response(request: &HttpRequest, url: &Url, mode: &str) -> Option<HttpResponse> {
    let path = url.path();
    let mutation = !matches!(request.method.as_str(), "GET" | "HEAD");
    if mode == "payment-forbidden" {
        return Some(error("forbidden", 403));
    }
    if mode == "payment-unavailable" {
        return Some(error("dependency_unavailable", 503));
    }
    if mode == "payment-malformed" && !mutation {
        return Some(HttpResponse::json(
            json!({"items":[{"id":"unverified"}],"total":1}),
        ));
    }
    if mode == "payment-conflict" && mutation {
        return Some(error("stale_version", 409));
    }
    if path == "/api/v1/admin/pay/intents" {
        let items = if mode == "payment-empty" {
            Vec::new()
        } else {
            vec![json!({
                "id":FIXTURE_PAYMENT_INTENT_ID,"chain_id":"31337","payer":FIXTURE_WALLET,
                "payee":"0x1111111111111111111111111111111111111111","amount":"29000000",
                "token_address":"0x2222222222222222222222222222222222222222",
                "status":"pending","escrow_id":null,"tx_hash":null,
                "description":"Migration plan checkout","expires_at":"2027-01-02T00:00:00.000Z",
                "created_at":FIXTURE_TIMESTAMP,"updated_at":FIXTURE_TIMESTAMP
            })]
        };
        return Some(HttpResponse::json(
            json!({"total":items.len(),"items":items}),
        ));
    }
    if path == format!("/api/v1/admin/pay/intents/{FIXTURE_PAYMENT_INTENT_ID}/cancel") {
        return Some(HttpResponse::json(json!({
            "id":FIXTURE_PAYMENT_INTENT_ID,"status":"cancelled",
            "evidence":{"operation_id":"550e8400-e29b-41d4-a716-446655440005","financial_finality":"not_applicable"}
        })));
    }
    if path == "/api/v1/admin/pay/links" {
        if mutation {
            return Some(HttpResponse::json(json!({
                "link":payment_link("active",0),
                "evidence":{"operation_id":"550e8400-e29b-41d4-a716-446655440006","financial_finality":"not_applicable"}
            })));
        }
        let items = if mode == "payment-empty" {
            Vec::new()
        } else {
            vec![payment_link("active", 0)]
        };
        return Some(HttpResponse::json(json!({
            "total":items.len(),"items":items,"limit":100,"offset":0,
            "correlation_id":"epsx-e2e-payment-links"
        })));
    }
    if path == format!("/api/v1/admin/pay/links/{FIXTURE_PAYMENT_LINK_ID}/disable") {
        return Some(HttpResponse::json(json!({
            "id":FIXTURE_PAYMENT_LINK_ID,"status":"disabled","version":1,
            "evidence":{"operation_id":"550e8400-e29b-41d4-a716-446655440007","financial_finality":"not_applicable"}
        })));
    }
    None
}

fn payment_link(status: &str, version: u64) -> Value {
    json!({
        "id":FIXTURE_PAYMENT_LINK_ID,"slug":"migration-checkout",
        "intent_id":FIXTURE_PAYMENT_INTENT_ID,"max_uses":1,"current_uses":0,
        "expires_at":"2027-01-02T00:00:00.000Z","created_at":FIXTURE_TIMESTAMP,
        "status":status,"version":version
    })
}

fn rankings_response(url: &Url, mode: &str) -> HttpResponse {
    if mode == "analytics-unavailable" {
        return error("dependency_unavailable", 503);
    }
    let limit = url
        .query_pairs()
        .find(|(key, _)| key == "limit")
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(3)
        .clamp(1, 10);
    let page = url
        .query_pairs()
        .find(|(key, _)| key == "page")
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(1)
        .clamp(1, 1_000_000);
    let mut rankings = vec![
        ranking(1, "NVDA", "NVIDIA Corporation", 4.12, 184.25, 42.5),
        ranking(2, "MSFT", "Microsoft Corporation", 3.88, 512.4, 24.1),
        ranking(3, "AAPL", "Apple Inc.", 2.34, 228.7, 16.8),
    ];
    if mode == "analytics-limited" {
        rankings.remove(0);
    } else if mode == "analytics-empty" {
        rankings.clear();
    }
    let start = page.saturating_sub(1).saturating_mul(limit);
    let data = rankings
        .iter()
        .skip(start)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let total_pages = rankings.len().div_ceil(limit);
    let timestamp = if mode == "analytics-malformed" {
        "not-an-rfc3339-timestamp"
    } else if mode == "analytics-stale" {
        "2020-01-01T00:00:00Z"
    } else {
        "2026-07-28T00:00:00Z"
    };
    HttpResponse::json(json!({
        "success":true,
        "data":data,
        "pagination":{
            "page":page,"limit":limit,"total":rankings.len(),"totalPages":total_pages,
            "hasNext":page < total_pages,"hasPrev":page > 1 && total_pages > 0
        },
        "metadata":{
            "available_countries":["United States"],
            "available_sectors":["Technology"],
            "request_timestamp":timestamp,
            "data_source":if mode == "analytics-stale" { "stale-cache" } else { "epsx-rust-e2e-fixture-v1" }
        },
        "access_info":{
            "min_accessible_rank":if mode == "analytics-limited" { 2 } else { 1 },
            "locked_ranks_count":if mode == "analytics-limited" { 1 } else { 0 }
        },
        "message":null,
        "processing_time_ms":1
    }))
}

fn admin_dashboard_response(mode: &str) -> HttpResponse {
    match mode {
        "admin-dashboard-forbidden" => error("forbidden", 403),
        "admin-dashboard-unavailable" => error("dependency_unavailable", 503),
        "admin-dashboard-malformed" => HttpResponse::json(json!({
            "success":true,
            "data":{"observed_at":"not-an-rfc3339-timestamp","total_users":12,"active_users":10},
            "error":null,
            "message":"Dashboard user status retrieved successfully",
            "timestamp":FIXTURE_TIMESTAMP,
            "admin_meta":{"operation":"get_dashboard_user_status","performed_by":FIXTURE_WALLET}
        })),
        _ => HttpResponse::json(json!({
            "success":true,
            "data":{"observed_at":FIXTURE_TIMESTAMP,"total_users":12,"active_users":10},
            "error":null,
            "message":"Dashboard user status retrieved successfully",
            "timestamp":FIXTURE_TIMESTAMP,
            "admin_meta":{"operation":"get_dashboard_user_status","performed_by":FIXTURE_WALLET}
        })),
    }
}

fn admin_analytics_response(mode: &str) -> HttpResponse {
    if mode == "admin-analytics-forbidden" {
        return error("forbidden", 403);
    }
    if mode == "admin-analytics-unavailable" {
        return error("dependency_unavailable", 503);
    }
    let empty = mode == "admin-analytics-empty";
    let mut data = json!({
        "user_stats":if empty { Value::Null } else { json!({"total":12,"active":10,"today_connections":2,"total_users":12,"active_users":10}) },
        "permission_analytics":if empty { Value::Null } else { json!({"total":8,"total_plans":2,"total_permissions":8,"active_permissions":7}) },
        "plan_stats":if empty { Value::Null } else { json!({"total_plans":2,"active_plans":2,"total_memberships":6,"active_memberships":5,"recent_assignments":1}) },
        "system_metrics":null,
        "developer_portal":if empty { Value::Null } else { json!({"total_api_keys":3,"active_api_keys":2}) }
    });
    if mode == "admin-analytics-malformed" {
        data["observed_at"] = Value::String(FIXTURE_TIMESTAMP.into());
    }
    HttpResponse::json(json!({
        "success":true,
        "data":data,
        "error":null,
        "message":"Analytics dashboard retrieved",
        "timestamp":FIXTURE_TIMESTAMP,
        "admin_meta":{"operation":"get_admin_analytics_dashboard","performed_by":FIXTURE_WALLET}
    }))
}

fn ranking(rank: u64, symbol: &str, company: &str, eps: f64, price: f64, growth: f64) -> Value {
    json!({
        "rank":rank,
        "symbol":symbol,
        "company_name":company,
        "latest_date":"2026-06-30",
        "value":eps,
        "active_status":"active",
        "quarterly_performance":[{
            "quarter":"Q2 2026",
            "date":"2026-06-30",
            "price":price,
            "eps":eps,
            "eps_growth":growth,
            "price_growth":11.4,
            "announcement_date":"2026-07-21",
            "announcement_timestamp":1784592000_i64,
            "is_estimated":false
        }],
        "next_quarter_estimate":null,
        "next_earnings_date":null,
        "last_earnings_date":1784592000_i64,
        "next_earnings_date_formatted":null,
        "days_until_next_earnings":null,
        "progress_percentage":null,
        "current_eps":eps,
        "growth_factor":1.241,
        "price_current":price
    })
}

fn public_news_article() -> Value {
    json!({
        "id":FIXTURE_NEWS_ID,
        "slug":"deterministic-market-brief",
        "title":"Deterministic Market Brief",
        "summary":"A fixed local article used by the migration evidence harness.",
        "content":"This fixture is intentionally stable across repeated E2E runs.\n\nIt verifies the published content boundary.",
        "cover_image_url":null,
        "author":"EPSX Research",
        "status":"published",
        "published_at":"2026-07-01T00:00:00Z",
        "tags":["Research","engineering"],
        "featured":true
    })
}

fn content_list(mode: &str) -> HttpResponse {
    let articles = if mode == "content-empty" {
        Vec::new()
    } else {
        vec![public_news_article()]
    };
    HttpResponse::json(json!({
        "success":true,
        "data":{"articles":articles,"total":articles.len(),"page":1,"limit":100},
        "error":null
    }))
}

fn admin_news_article(status: &str, pinned: bool) -> Value {
    json!({
        "id":FIXTURE_NEWS_ID,
        "title":"Deterministic Market Brief",
        "slug":"deterministic-market-brief",
        "summary":"A fixed local article used by the migration evidence harness.",
        "content":"This fixture is intentionally stable across repeated E2E runs.",
        "cover_image_url":null,
        "author_wallet":FIXTURE_WALLET,
        "status":status,
        "tags":["Research"],
        "published_at":if status == "published" { Some("2026-07-01T00:00:00Z") } else { None },
        "created_at":"2026-06-30T00:00:00Z",
        "updated_at":FIXTURE_TIMESTAMP,
        "is_pinned":pinned,
        "pinned_at":if pinned { Some(FIXTURE_TIMESTAMP) } else { None }
    })
}

fn admin_content_envelope(data: Value, status: u16) -> HttpResponse {
    HttpResponse::json_status(
        json!({
            "success":true,
            "data":data,
            "error":null,
            "meta":{"timestamp":FIXTURE_TIMESTAMP,"version":"v1"}
        }),
        status,
    )
}

fn admin_news_response(request: &HttpRequest, url: &Url, mode: &str) -> HttpResponse {
    let status = url
        .query_pairs()
        .find(|(key, _)| key == "status")
        .map(|(_, value)| value.into_owned())
        .filter(|value| value == "draft")
        .unwrap_or_else(|| "published".into());
    let article = admin_news_article(&status, false);
    if matches!(request.method.as_str(), "GET" | "HEAD") {
        let articles = if mode == "content-empty" {
            Vec::new()
        } else {
            vec![article]
        };
        admin_content_envelope(
            json!({
                "articles":articles,
                "total":articles.len(),
                "page":url.query_pairs().find(|(key,_)| key == "page").and_then(|(_,value)| value.parse::<u64>().ok()).unwrap_or(1),
                "limit":20
            }),
            200,
        )
    } else {
        admin_content_envelope(article, 201)
    }
}

fn fixture_wallet(status: &str, version: u64) -> Value {
    json!({
        "address":FIXTURE_WALLET,
        "chain_id":"31337",
        "label":"Migration owner",
        "role":"user",
        "status":status,
        "metadata":{},
        "version":version,
        "created_at":FIXTURE_TIMESTAMP
    })
}

fn fixture_plan() -> Value {
    json!({
        "id":FIXTURE_PLAN_ID,
        "merchant_id":FIXTURE_MERCHANT_ID,
        "name":"Migration Professional",
        "description":"Deterministic backend-authoritative access plan.",
        "amount":"2900",
        "currency":"USD",
        "chain_id":"31337",
        "interval":30,
        "active":true,
        "created_at":FIXTURE_TIMESTAMP,
        "version":7
    })
}

#[derive(Debug)]
struct FixturePrincipal {
    subject: String,
    audience: String,
    permissions: Vec<String>,
}

fn fixture_principal(request: &HttpRequest) -> Option<FixturePrincipal> {
    let token = request
        .headers
        .get("authorization")?
        .strip_prefix("Bearer ")?;
    let payload = token.split('.').nth(1)?;
    let claims: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).ok()?).ok()?;
    let subject = claims.get("sub")?.as_str()?.to_string();
    if claims.get("wallet_address")?.as_str()? != subject {
        return None;
    }
    let audiences = claims.get("aud")?.as_array()?;
    if audiences.len() != 1 {
        return None;
    }
    let audience = audiences[0].as_str()?.to_string();
    let permissions = claims
        .get("scope")?
        .as_str()?
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    Some(FixturePrincipal {
        subject,
        audience,
        permissions,
    })
}

fn required_admin_permission(path: &str, method: &str) -> Option<&'static str> {
    let mutation = !matches!(method, "GET" | "HEAD");
    if path.starts_with("/api/v1/admin/wallets") {
        return Some(if mutation {
            "admin:wallets:manage"
        } else {
            "admin:wallets:read"
        });
    }
    if path.starts_with("/api/v1/admin/credits") {
        return Some(if mutation {
            "admin:credits:manage"
        } else {
            "admin:credits:read"
        });
    }
    if path.starts_with("/api/v1/admin/subscription/access") {
        return Some(if mutation {
            "admin:access:manage"
        } else {
            "admin:access:read"
        });
    }
    if path.starts_with("/api/v1/admin/subscription/plans") {
        return Some(if mutation {
            "admin:plans:manage"
        } else {
            "admin:plans:read"
        });
    }
    if path.starts_with("/api/admin/dashboard/")
        || path.starts_with("/api/admin/web3/recent-wallets")
    {
        return Some("admin:dashboard:view");
    }
    if path.starts_with("/api/admin/analytics/") {
        return Some("admin:analytics:view");
    }
    if path.starts_with("/api/admin/news")
        || path.starts_with("/api/admin/media")
        || path.starts_with("/api/admin/files")
    {
        return Some("admin:content:manage");
    }
    if path == "/api/v1/notification/send" {
        return Some("admin:notifications:create");
    }
    if path.starts_with("/api/v1/notification/admin") {
        return Some(if mutation {
            "admin:notifications:manage"
        } else {
            "admin:notifications:read"
        });
    }
    if path.starts_with("/api/admin/chat/conversations") {
        return Some(if mutation {
            "admin:chat:send"
        } else {
            "admin:chat:read"
        });
    }
    if path.starts_with("/api/admin/developer-portal") {
        return Some(if !mutation {
            "admin:developer:read"
        } else if path == "/api/admin/developer-portal/api-keys" {
            "admin:developer:create"
        } else {
            "admin:developer:manage"
        });
    }
    if path.starts_with("/api/v1/admin/pay/links") {
        return Some(if mutation {
            "admin:payment-links:manage"
        } else {
            "admin:payment-links:view"
        });
    }
    if path.starts_with("/api/v1/admin/pay") {
        return Some(if mutation {
            "admin:payments:manage"
        } else {
            "admin:payments:view"
        });
    }
    path.starts_with("/api/v1/analytics/admin/audit-log")
        .then_some("admin:audit:read")
}

fn permission_allows(held: &[String], required: &str) -> bool {
    let required = required.split(':').collect::<Vec<_>>();
    if required.len() != 3 || required.iter().any(|part| part.is_empty() || *part == "*") {
        return false;
    }
    held.iter().any(|permission| {
        if matches!(permission.as_str(), "*:*" | "*:*:*") {
            return true;
        }
        let parts = permission.split(':').collect::<Vec<_>>();
        parts.len() == 3
            && parts.iter().all(|part| !part.is_empty())
            && parts[0] != "*"
            && parts[0] == required[0]
            && ((parts[1] == "*" && parts[2] == "*")
                || (parts[1] == required[1] && (parts[2] == "*" || parts[2] == required[2])))
    })
}

fn profile_response(request: &HttpRequest) -> HttpResponse {
    let Some(principal) = fixture_principal(request) else {
        return error("authentication_required", 401);
    };
    HttpResponse::json(json!({
        "success":true,
        "data":{
            "id":principal.subject,
            "subject":principal.subject,
            "wallet_address":principal.subject,
            "permissions":principal.permissions,
            "capabilities":["migration-e2e"],
            "auth_method":"web3_siwe",
            "tier":"migration-e2e",
            "status":"active",
            "created_at":FIXTURE_TIMESTAMP,
            "last_login":FIXTURE_TIMESTAMP
        }
    }))
}

fn permission_status(request: &HttpRequest) -> HttpResponse {
    let Some(principal) = fixture_principal(request) else {
        return error("authentication_required", 401);
    };
    let permissions = principal
        .permissions
        .iter()
        .map(|permission| {
            json!({
                "permission":permission,
                "expires_at":null,
                "source":"session",
                "granted_by":null,
                "granted_at":FIXTURE_TIMESTAMP,
                "is_active":true,
                "expires_soon":false,
                "time_until_expiry":null,
                "metadata":null
            })
        })
        .collect::<Vec<_>>();
    HttpResponse::json(json!({
        "success":true,
        "data":{
            "wallet_address":principal.subject,
            "permissions":permissions,
            "permission_version":1,
            "last_updated":FIXTURE_TIMESTAMP,
            "total_permissions":principal.permissions.len(),
            "active_permissions":principal.permissions.len(),
            "expired_permissions":0,
            "expiring_soon":0,
            "has_admin_access":principal.permissions.iter().any(|permission| permission.starts_with("admin:")),
            "platform_permissions":{
                "epsx":principal.permissions.iter().filter(|permission| permission.starts_with("epsx:")).collect::<Vec<_>>(),
                "admin":principal.permissions.iter().filter(|permission| permission.starts_with("admin:")).collect::<Vec<_>>()
            }
        }
    }))
}

fn permission_definitions(request: &HttpRequest) -> HttpResponse {
    let Some(principal) = fixture_principal(request) else {
        return error("authentication_required", 401);
    };
    let definitions = principal
        .permissions
        .iter()
        .enumerate()
        .map(|(index, permission)| {
            json!({
                "id":format!("fixture-permission-{}",index + 1),
                "permission_string":permission,
                "name":permission,
                "description":"Backend-issued migration contract permission",
                "platform":permission.split(':').next().unwrap_or("epsx"),
                "category":permission.split(':').nth(1),
                "is_system":true,
                "is_active":true,
                "created_at":FIXTURE_TIMESTAMP
            })
        })
        .collect::<Vec<_>>();
    HttpResponse::json(json!({"success":true,"data":definitions}))
}

fn error(code: &str, status: u16) -> HttpResponse {
    HttpResponse::json_status(json!({"success":false,"error":code}), status)
}

fn error_with_path(code: &str, path: &str, status: u16) -> HttpResponse {
    HttpResponse::json_status(json!({"success":false,"error":code,"path":path}), status)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: &str, target: &str, token: Option<&str>, body: Value) -> HttpRequest {
        let mut headers = BTreeMap::new();
        if let Some(token) = token {
            headers.insert("x-epsx-e2e-token".into(), token.into());
        }
        HttpRequest {
            method: method.into(),
            target: target.into(),
            headers,
            body: serde_json::to_vec(&body).unwrap(),
        }
    }

    #[test]
    fn control_routes_are_authenticated_and_reset_state() {
        let key = EncodingKey::from_rsa_pem(SIGNING_PRIVATE_KEY.as_bytes()).unwrap();
        let mut state = FixtureState {
            mode: "forbidden".into(),
            sequence: 1,
            requests: vec![json!({"path":"/test"})],
            mutations: Vec::new(),
        };
        let denied = route_request(
            request("POST", "/__e2e/reset", None, json!({})),
            "http://127.0.0.1:48080",
            DEFAULT_CONTROL_TOKEN,
            &key,
            &mut state,
        );
        assert_eq!(denied.status, 403);
        let reset = route_request(
            request(
                "POST",
                "/__e2e/reset",
                Some(DEFAULT_CONTROL_TOKEN),
                json!({}),
            ),
            "http://127.0.0.1:48080",
            DEFAULT_CONTROL_TOKEN,
            &key,
            &mut state,
        );
        assert_eq!(reset.status, 200);
        assert_eq!(state.mode, "healthy");
        assert!(state.requests.is_empty());
    }

    #[test]
    fn sessions_use_rs256_and_declared_audience() {
        let key = EncodingKey::from_rsa_pem(SIGNING_PRIVATE_KEY.as_bytes()).unwrap();
        let token = fixture_access_token(
            "http://127.0.0.1:48080",
            "epsx-admin",
            "admin:wallets:read",
            SIGNING_KEY_ID,
            &key,
        )
        .unwrap();
        let header = jsonwebtoken::decode_header(&token).unwrap();
        assert_eq!(header.alg, Algorithm::RS256);
        assert_eq!(header.kid.as_deref(), Some(SIGNING_KEY_ID));
        let payload = token.split('.').nth(1).unwrap();
        let claims: Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap();
        assert_eq!(claims["aud"], json!(["epsx-admin"]));
        assert_eq!(claims["scope"], "admin:wallets:read");
    }

    #[test]
    fn fixture_modes_fail_closed_for_covered_service_paths() {
        let mut state = FixtureState {
            mode: "forbidden".into(),
            ..FixtureState::default()
        };
        let response = dependency_request(
            &request("GET", "/api/v1/admin/wallets", None, json!({})),
            &Url::parse("http://127.0.0.1:48080/api/v1/admin/wallets").unwrap(),
            &state,
        );
        assert_eq!(response.status, 403);
        state.mode = "conflict".into();
        let response = dependency_request(
            &request("POST", "/api/v1/content/news", None, json!({})),
            &Url::parse("http://127.0.0.1:48080/api/v1/content/news").unwrap(),
            &state,
        );
        assert_eq!(response.status, 409);
    }

    #[test]
    fn admin_permission_grammar_is_exact_and_bounded() {
        assert!(permission_allows(
            &["admin:wallets:manage".into()],
            "admin:wallets:manage"
        ));
        assert!(permission_allows(
            &["admin:wallets:*".into()],
            "admin:wallets:manage"
        ));
        assert!(permission_allows(
            &["admin:*:*".into()],
            "admin:wallets:manage"
        ));
        assert!(!permission_allows(
            &["admin:wallets:read".into()],
            "admin:wallets:manage"
        ));
        assert!(!permission_allows(
            &["*:wallets:manage".into()],
            "admin:wallets:manage"
        ));
    }

    #[test]
    fn capabilities_cover_every_migration_group_and_declared_mode() {
        assert_eq!(SUPPORTED_GROUPS, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let manifest: Value =
            serde_json::from_str(include_str!("../../e2e/migration/scenarios.json")).unwrap();
        let groups = manifest["groups"].as_array().unwrap();
        let declared_groups = groups
            .iter()
            .filter_map(|group| group["id"].as_u64())
            .map(|group| u8::try_from(group).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(declared_groups, SUPPORTED_GROUPS);

        let supported = SUPPORTED_MODES
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(supported.len(), SUPPORTED_MODES.len());
        for scenario in groups
            .iter()
            .flat_map(|group| group["scenarios"].as_array().unwrap())
        {
            if let Some(mode) = scenario["state"]["fixtureMode"].as_str() {
                assert!(
                    supported.contains(mode),
                    "fixture mode {mode} is not advertised"
                );
            }
        }
    }
}
