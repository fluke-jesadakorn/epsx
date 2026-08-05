use reqwest::{blocking::Client, Method};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;

pub fn assets(flags: &[String]) -> Result<(), String> {
    if flags != ["verify"] {
        return Err("assets accepts only: assets verify".into());
    }
    let root = repo_root()?;
    let required = [
        "apps/frontend/public/dist/tailwind.css",
        "apps/admin/public/dist/tailwind.css",
        "apps/frontend/public/logos/epsx-icon.svg",
        "apps/admin/public/logos/epsx-icon.svg",
        "shared/rust/templates/assets/auth.css",
    ];
    let mut bytes = 0_u64;
    let mut digest = Sha256::new();
    for relative in required {
        let path = root.join(relative);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("required asset {} is missing: {error}", path.display()))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() == 0 {
            return Err(format!(
                "required asset {} is not a regular file",
                path.display()
            ));
        }
        let content = fs::read(&path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        if relative.ends_with("tailwind.css") {
            let css = std::str::from_utf8(&content)
                .map_err(|_| format!("{} is not UTF-8", path.display()))?;
            if css.len() <= 100_000 || !css.contains("--tw-") || !css.contains(".dark") {
                return Err(format!(
                    "frozen stylesheet {} is incomplete",
                    path.display()
                ));
            }
        }
        digest.update(relative.as_bytes());
        digest.update([0]);
        digest.update(&content);
        bytes += metadata.len();
    }
    println!(
        "assets verify: PASS — required={}, bytes={}, sha256={:x}",
        required.len(),
        bytes,
        digest.finalize()
    );
    Ok(())
}

pub fn fixtures(flags: &[String]) -> Result<(), String> {
    if flags.first().map(String::as_str) != Some("serve") {
        return Err(
            "fixtures accepts only: fixtures serve [--root PATH] [--bind LOOPBACK_ADDR]".into(),
        );
    }
    let options = parse_options(&flags[1..], &["--root", "--bind"], &[])?;
    let root = repo_root()?;
    let relative = options
        .value("--root")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("e2e/fixtures"));
    if !is_safe_relative(&relative) {
        return Err("fixture root must be a repository-relative path".into());
    }
    let fixture_root = root.join(relative);
    if !fixture_root.is_dir() {
        return Err(format!(
            "fixture root {} does not exist",
            fixture_root.display()
        ));
    }
    let bind = options.value("--bind").unwrap_or("127.0.0.1:4300");
    let address = parse_loopback(bind, "fixture bind address")?;
    let listener = TcpListener::bind(address)
        .map_err(|error| format!("could not bind fixture server to {address}: {error}"))?;
    println!(
        "fixtures serve: listening on http://{address}, root={}",
        fixture_root.display()
    );
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = serve_fixture(&mut stream, &fixture_root) {
                    eprintln!("fixture request failed: {error}");
                }
            }
            Err(error) => eprintln!("fixture connection failed: {error}"),
        }
    }
    Ok(())
}

pub fn anvil_proxy(flags: &[String]) -> Result<(), String> {
    let options = parse_options(flags, &["--listen", "--upstream"], &["--no-spawn"])?;
    let listen = options.value("--listen").unwrap_or("127.0.0.1:8545");
    let upstream = options.value("--upstream").unwrap_or("127.0.0.1:8546");
    let listen_address: SocketAddr = listen
        .parse()
        .map_err(|error| format!("invalid listen address {listen}: {error}"))?;
    let upstream_address = parse_loopback(upstream, "Anvil upstream")?;
    if listen_address.port() == upstream_address.port() {
        return Err("proxy and Anvil upstream must use different ports".into());
    }

    let mut anvil = if options.has("--no-spawn") {
        None
    } else {
        Some(spawn_anvil(upstream_address.port())?)
    };
    let listener = TcpListener::bind(listen_address)
        .map_err(|error| format!("could not bind Anvil proxy to {listen_address}: {error}"))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("could not create proxy client: {error}"))?;
    println!(
        "anvil-proxy: listening on http://{listen_address}, upstream=http://{upstream_address}"
    );
    for stream in listener.incoming() {
        if let Some(child) = anvil.as_mut() {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| format!("could not inspect Anvil: {error}"))?
            {
                return Err(format!("Anvil exited unexpectedly with {status}"));
            }
        }
        match stream {
            Ok(mut stream) => {
                if let Err(error) = serve_proxy(&mut stream, upstream_address, &client) {
                    let _ = write_response(
                        &mut stream,
                        502,
                        "text/plain; charset=utf-8",
                        format!("Anvil proxy error: {error}").as_bytes(),
                        &[],
                    );
                }
            }
            Err(error) => eprintln!("proxy connection failed: {error}"),
        }
    }
    Ok(())
}

fn spawn_anvil(port: u16) -> Result<Child, String> {
    Command::new("anvil")
        .args([
            "--port",
            &port.to_string(),
            "--chain-id",
            "31337",
            "--host",
            "127.0.0.1",
            "--gas-price",
            "3000000000",
            "--block-time",
            "1",
            "--accounts",
            "10",
            "--balance",
            "10000",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| {
            format!("could not start Anvil; install Foundry and ensure anvil is in PATH: {error}")
        })
}

fn serve_fixture(stream: &mut TcpStream, root: &Path) -> Result<(), String> {
    let request = read_request(stream)?;
    if !matches!(request.method.as_str(), "GET" | "HEAD") {
        return write_response(stream, 405, "text/plain", b"method not allowed", &[]);
    }
    if request.target == "/health" {
        return write_response(
            stream,
            200,
            "application/json",
            if request.method == "HEAD" {
                b""
            } else {
                br#"{"status":"ok"}"#
            },
            &[],
        );
    }
    let raw_path = request.target.split('?').next().unwrap_or("/");
    if raw_path.contains('%') {
        return write_response(
            stream,
            400,
            "text/plain",
            b"encoded paths are not supported",
            &[],
        );
    }
    let relative = PathBuf::from(raw_path.trim_start_matches('/'));
    if !is_safe_relative(&relative) {
        return write_response(stream, 400, "text/plain", b"unsafe fixture path", &[]);
    }
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path).ok();
    if metadata
        .as_ref()
        .is_none_or(|item| !item.is_file() || item.file_type().is_symlink())
    {
        return write_response(stream, 404, "text/plain", b"fixture not found", &[]);
    }
    let content = fs::read(&path).map_err(|error| format!("could not read fixture: {error}"))?;
    let body: &[u8] = if request.method == "HEAD" {
        &[]
    } else {
        &content
    };
    write_response(stream, 200, content_type(&path), body, &[])
}

fn serve_proxy(
    stream: &mut TcpStream,
    upstream: SocketAddr,
    client: &Client,
) -> Result<(), String> {
    let request = read_request(stream)?;
    if request.method == "GET" && request.target.starts_with("/tx/") {
        let hash = request
            .target
            .split('?')
            .next()
            .unwrap_or("")
            .trim_start_matches("/tx/")
            .trim_end_matches('/');
        let html = render_transaction(client, upstream, hash)?;
        return write_response(
            stream,
            200,
            "text/html; charset=utf-8",
            html.as_bytes(),
            &[(
                "Content-Security-Policy",
                "default-src 'none'; style-src 'unsafe-inline'",
            )],
        );
    }

    let method = Method::from_bytes(request.method.as_bytes())
        .map_err(|error| format!("unsupported HTTP method: {error}"))?;
    let mut forwarded = client.request(method, format!("http://{upstream}{}", request.target));
    for (name, value) in &request.headers {
        if !matches!(name.as_str(), "host" | "content-length" | "connection") {
            forwarded = forwarded.header(name, value);
        }
    }
    let response = forwarded
        .body(request.body)
        .send()
        .map_err(|error| format!("Anvil request failed: {error}"))?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let body = response
        .bytes()
        .map_err(|error| format!("could not read Anvil response: {error}"))?;
    write_response(stream, status, &content_type, &body, &[])
}

fn render_transaction(client: &Client, upstream: SocketAddr, hash: &str) -> Result<String, String> {
    let safe_hash = escape_html(hash);
    if hash.len() != 66
        || !hash.starts_with("0x")
        || !hash[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Ok(transaction_page(
            &safe_hash,
            "Invalid transaction hash",
            &[],
        ));
    }
    let transaction = rpc_result(client, upstream, "eth_getTransactionByHash", json!([hash]))?;
    if transaction.is_null() {
        return Ok(transaction_page(&safe_hash, "Transaction not found", &[]));
    }
    let receipt = rpc_result(client, upstream, "eth_getTransactionReceipt", json!([hash]))?;
    let status = match receipt.get("status").and_then(Value::as_str) {
        Some("0x1") => "Success",
        Some(_) => "Failed",
        None => "Pending",
    };
    let fields = [
        ("Status", status.to_string()),
        ("Block", json_text(transaction.get("blockNumber"))),
        ("From", json_text(transaction.get("from"))),
        ("To", json_text(transaction.get("to"))),
        ("Value", json_text(transaction.get("value"))),
        ("Gas used", json_text(receipt.get("gasUsed"))),
        ("Gas price", json_text(transaction.get("gasPrice"))),
        ("Nonce", json_text(transaction.get("nonce"))),
    ];
    Ok(transaction_page(&safe_hash, "Transaction details", &fields))
}

fn rpc_result(
    client: &Client,
    upstream: SocketAddr,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    let response: Value = client
        .post(format!("http://{upstream}/"))
        .json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
        .send()
        .map_err(|error| format!("RPC {method} failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("RPC {method} returned an error: {error}"))?
        .json()
        .map_err(|error| format!("RPC {method} returned invalid JSON: {error}"))?;
    if let Some(error) = response.get("error") {
        return Err(format!("RPC {method} failed: {error}"));
    }
    Ok(response.get("result").cloned().unwrap_or(Value::Null))
}

fn transaction_page(hash: &str, title: &str, fields: &[(&str, String)]) -> String {
    let rows = fields
        .iter()
        .map(|(label, value)| {
            format!(
                "<div class=\"row\"><strong>{}</strong><span>{}</span></div>",
                escape_html(label),
                escape_html(value)
            )
        })
        .collect::<String>();
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title} | Local Anvil</title><style>body{{background:#0f111a;color:#e2e8f0;font:16px system-ui;max-width:800px;margin:auto;padding:2rem}}main{{background:#1e212b;border-radius:12px;padding:2rem}}.row{{display:grid;grid-template-columns:150px 1fr;gap:1rem;padding:.75rem 0;border-bottom:1px solid #334155}}span,code{{overflow-wrap:anywhere}}</style></head><body><main><h1>{title}</h1>{rows}<p><strong>Hash</strong><br><code>{hash}</code></p></main></body></html>"
    )
}

struct HttpRequest {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|error| format!("could not set request timeout: {error}"))?;
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    let header_end = loop {
        let count = stream
            .read(&mut buffer)
            .map_err(|error| format!("could not read request: {error}"))?;
        if count == 0 {
            return Err("connection closed before request headers".into());
        }
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err("request exceeds the 16 MiB limit".into());
        }
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let header_text =
        std::str::from_utf8(&bytes[..header_end]).map_err(|_| "request headers are not UTF-8")?;
    let mut lines = header_text.split("\r\n");
    let mut start = lines.next().unwrap_or("").split_whitespace();
    let method = start.next().ok_or("request omitted method")?.to_string();
    let target = start.next().ok_or("request omitted target")?.to_string();
    let mut headers = Vec::new();
    let mut content_length = 0_usize;
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').ok_or("invalid request header")?;
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        if name == "content-length" {
            content_length = value
                .parse()
                .map_err(|_| "invalid request content length")?;
        }
        headers.push((name, value));
    }
    if header_end + content_length > MAX_REQUEST_BYTES {
        return Err("request exceeds the 16 MiB limit".into());
    }
    while bytes.len() < header_end + content_length {
        let count = stream
            .read(&mut buffer)
            .map_err(|error| format!("could not read request body: {error}"))?;
        if count == 0 {
            return Err("connection closed before request body".into());
        }
        bytes.extend_from_slice(&buffer[..count]);
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
    status: u16,
    content_type: &str,
    body: &[u8],
    headers: &[(&str, &str)],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        502 => "Bad Gateway",
        _ => "Response",
    };
    let mut head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in headers {
        head.push_str(name);
        head.push_str(": ");
        head.push_str(value);
        head.push_str("\r\n");
    }
    head.push_str("\r\n");
    stream
        .write_all(head.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|error| format!("could not write response: {error}"))
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|value| value.to_str()) {
        Some("json") => "application/json",
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("txt" | "md") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn json_text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Null) | None => "—".into(),
        Some(value) => value.to_string(),
    }
}

fn escape_html(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn repo_root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|error| format!("could not run git: {error}"))?;
    if !output.status.success() {
        return Err("git rev-parse failed".into());
    }
    String::from_utf8(output.stdout)
        .map(|value| PathBuf::from(value.trim()))
        .map_err(|_| "repository path is not UTF-8".into())
}

fn parse_loopback(value: &str, label: &str) -> Result<SocketAddr, String> {
    let address: SocketAddr = value
        .parse()
        .map_err(|error| format!("invalid {label} {value}: {error}"))?;
    if !address.ip().is_loopback() {
        return Err(format!("{label} must be loopback-only"));
    }
    Ok(address)
}

fn is_safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

struct Options<'a> {
    values: Vec<(&'a str, &'a str)>,
    switches: Vec<&'a str>,
}

impl Options<'_> {
    fn value(&self, name: &str) -> Option<&str> {
        self.values
            .iter()
            .find_map(|(key, value)| (*key == name).then_some(*value))
    }

    fn has(&self, name: &str) -> bool {
        self.switches.contains(&name)
    }
}

fn parse_options<'a>(
    flags: &'a [String],
    values: &[&str],
    switches: &[&str],
) -> Result<Options<'a>, String> {
    let mut parsed = Options {
        values: Vec::new(),
        switches: Vec::new(),
    };
    let mut index = 0;
    while index < flags.len() {
        let flag = flags[index].as_str();
        if switches.contains(&flag) {
            parsed.switches.push(flag);
            index += 1;
        } else if values.contains(&flag) {
            let value = flags
                .get(index + 1)
                .ok_or_else(|| format!("{flag} requires a value"))?;
            parsed.values.push((flag, value));
            index += 2;
        } else {
            return Err(format!("unsupported flag {flag}"));
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::{escape_html, is_safe_relative, parse_loopback};
    use std::path::Path;

    #[test]
    fn fixture_paths_stay_within_the_repository() {
        assert!(is_safe_relative(Path::new("e2e/fixtures")));
        assert!(!is_safe_relative(Path::new("../fixtures")));
        assert!(!is_safe_relative(Path::new("/tmp/fixtures")));
    }

    #[test]
    fn local_servers_are_loopback_only() {
        assert!(parse_loopback("127.0.0.1:4300", "test").is_ok());
        assert!(parse_loopback("0.0.0.0:4300", "test").is_err());
    }

    #[test]
    fn transaction_fields_are_html_escaped() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
    }
}
