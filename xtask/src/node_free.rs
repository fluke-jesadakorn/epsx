use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use url::Url;

const JS_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs"];
const NODE_MANIFESTS: &[&str] = &[
    "package.json",
    "package-lock.json",
    "bun.lock",
    "bun.lockb",
    "yarn.lock",
    "pnpm-lock.yaml",
    "turbo.json",
    ".npmrc",
];
const ACTIVE_NODE_MARKERS: &[&str] = &[
    "node ",
    "node -",
    "bun ",
    "bunx ",
    "npm ",
    "npx ",
    "yarn ",
    "pnpm ",
    "setup-bun",
    "setup-node",
];
const INLINE_RUNTIME_MARKERS: &[&str] = &[
    "document::eval",
    "dangerous_inner_html: AUTH_REDIRECT_SCRIPT",
    "r#\"<script",
    "r##\"<script",
    "<script data-epsx-",
    "onclick=\"",
    "onsubmit=\"",
];

#[derive(Debug, Deserialize)]
struct ScenarioManifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u64,
    #[serde(rename = "baselineLock")]
    baseline_lock: String,
    #[serde(rename = "routeContract")]
    route_contract: String,
    matrices: BTreeMap<String, Vec<Matrix>>,
    groups: Vec<ScenarioGroup>,
}

#[derive(Debug, Deserialize)]
struct Matrix {
    id: String,
    viewport: Viewport,
    #[serde(rename = "colorScheme")]
    color_scheme: String,
}

#[derive(Debug, Deserialize)]
struct Viewport {
    width: u64,
    height: u64,
}

#[derive(Debug, Deserialize)]
struct ScenarioGroup {
    id: u8,
    slug: String,
    matrix: String,
    repeat: u8,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Deserialize)]
struct Scenario {
    id: String,
    surface: String,
    path: String,
    #[serde(default, rename = "expectedTargetPath")]
    expected_target_path: Option<String>,
    #[serde(default)]
    actions: Vec<Value>,
    #[serde(default)]
    outcomes: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct BaselineLock {
    #[serde(rename = "schemaVersion")]
    schema_version: u64,
    immutable: bool,
    commit: String,
}

#[derive(Debug, Deserialize)]
struct EvidenceManifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u64,
    #[serde(rename = "hashAlgorithm")]
    hash_algorithm: String,
    entries: Vec<EvidenceEntry>,
}

#[derive(Debug, Deserialize)]
struct EvidenceEntry {
    path: String,
    sha256: String,
}

pub fn audit(flags: &[String]) -> Result<(), String> {
    let Some(kind) = flags.first() else {
        return Err("audit requires a target; supported target: no-node".into());
    };
    if kind != "no-node" {
        return Err(format!("unsupported audit target {kind}"));
    }
    if flags
        .iter()
        .skip(1)
        .any(|flag| flag != "--strict" && flag != "--report")
    {
        return Err("audit no-node accepts only --strict and --report".into());
    }
    let strict = flags.iter().any(|flag| flag == "--strict");
    let root = repo_root()?;
    let files = tracked_files(&root)?;
    let mut scripts = Vec::new();
    let mut manifests = Vec::new();
    let mut active_refs = Vec::new();
    let mut inline_runtimes = Vec::new();

    for relative in &files {
        let name = relative.file_name().and_then(OsStr::to_str).unwrap_or("");
        let extension = relative.extension().and_then(OsStr::to_str).unwrap_or("");
        if JS_EXTENSIONS.contains(&extension) {
            scripts.push(relative.clone());
        }
        if NODE_MANIFESTS.contains(&name)
            || name.starts_with("tsconfig")
            || name.starts_with("eslint.config")
            || name.starts_with("jest.config")
            || name.starts_with("playwright.config")
            || name.starts_with("postcss.config")
        {
            manifests.push(relative.clone());
        }
        let absolute = root.join(relative);
        let Ok(contents) = fs::read_to_string(&absolute) else {
            continue;
        };
        if is_active_automation_path(relative)
            && ACTIVE_NODE_MARKERS
                .iter()
                .any(|marker| contains_command(&contents, marker))
        {
            active_refs.push(relative.clone());
        }
        if extension == "rs"
            && INLINE_RUNTIME_MARKERS
                .iter()
                .any(|marker| contents.contains(marker))
        {
            inline_runtimes.push(relative.clone());
        }
    }

    scripts.sort();
    manifests.sort();
    active_refs.sort();
    inline_runtimes.sort();
    println!(
        "no-node-audit: scripts={} manifests={} active_refs={} inline_runtimes={}",
        scripts.len(),
        manifests.len(),
        active_refs.len(),
        inline_runtimes.len()
    );
    print_paths("script", &scripts);
    print_paths("manifest", &manifests);
    print_paths("active-node-reference", &active_refs);
    print_paths("inline-runtime", &inline_runtimes);
    if strict
        && (!scripts.is_empty()
            || !manifests.is_empty()
            || !active_refs.is_empty()
            || !inline_runtimes.is_empty())
    {
        return Err("strict no-node audit failed".into());
    }
    Ok(())
}

pub fn e2e(flags: &[String]) -> Result<(), String> {
    let Some(command) = flags.first().map(String::as_str) else {
        return Err("e2e requires doctor, run, report, or verify-artifacts".into());
    };
    let args = &flags[1..];
    match command {
        "doctor" => e2e_doctor(args),
        "run" => e2e_run(args),
        "report" => e2e_report(args),
        "verify-artifacts" => e2e_verify_artifacts(args),
        _ => Err(format!("unknown e2e command {command}")),
    }
}

pub fn env_command(flags: &[String]) -> Result<(), String> {
    if flags.len() != 1 || flags[0] != "validate" {
        return Err("env accepts only: env validate".into());
    }
    let root = repo_root()?;
    let env_name = environment_name();
    let mut merged = BTreeMap::new();
    let explicit = env::var_os("ROOT_ENV_FILE").map(PathBuf::from);
    let env_files = explicit.map_or_else(
        || {
            vec![
                root.join(".env"),
                root.join(format!(".env.{env_name}")),
                root.join(".env.local"),
                root.join(format!(".env.{env_name}.local")),
            ]
        },
        |path| vec![path],
    );
    for path in env_files.iter().filter(|path| path.is_file()) {
        merged.extend(parse_env_file(path)?);
    }
    merged.extend(env::vars());
    let example = parse_env_file(&root.join(".env.example"))?;
    let missing = example
        .keys()
        .filter(|key| merged.get(*key).is_none_or(String::is_empty))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "missing required environment variables for {env_name}: {}",
            missing.join(", ")
        ));
    }
    println!(
        "env validate: PASS — environment={env_name}, required={}",
        example.len()
    );
    Ok(())
}

pub fn setup_local(flags: &[String]) -> Result<(), String> {
    if !flags.is_empty() {
        return Err("setup-local accepts no arguments".into());
    }
    run_status(
        Command::new("sh")
            .arg("apps/contracts/scripts/setup-local.sh")
            .current_dir(repo_root()?),
        "Foundry local setup",
    )
}

pub fn dev(flags: &[String]) -> Result<(), String> {
    let root = repo_root()?;
    match flags.first().map(String::as_str) {
        Some("--all") => run_status(
            Command::new("docker")
                .args([
                    "compose",
                    "-f",
                    "infrastructure/docker/docker-compose.yml",
                    "up",
                    "--build",
                ])
                .current_dir(root),
            "Rust workspace development stack",
        ),
        Some("--frontend") => {
            build_browser_runtime(&root)?;
            cargo_run(&root, "epsx-frontend", "bff-frontend")
        }
        Some("--admin") => {
            build_browser_runtime(&root)?;
            cargo_run(&root, "epsx-admin", "bff-admin")
        }
        Some("--backend") => cargo_run(&root, "epsx", "epsx"),
        _ => Err("dev requires --all, --frontend, --admin, or --backend".into()),
    }
}

pub fn build(flags: &[String]) -> Result<(), String> {
    let profile = flag_value(flags, "--profile").unwrap_or("development");
    if flags.len() != 2 || !matches!(profile, "development" | "production") {
        return Err("build requires --profile development|production".into());
    }
    let root = repo_root()?;
    build_browser_runtime(&root)?;
    let mut command = Command::new("cargo");
    command.args(["build", "--workspace", "--locked"]);
    if profile == "production" {
        command.arg("--release");
    }
    command.current_dir(root);
    run_status(&mut command, "Rust workspace build")
}

pub fn browser_runtime(flags: &[String]) -> Result<(), String> {
    if flags != ["build"] {
        return Err("browser-runtime accepts only: browser-runtime build".into());
    }
    build_browser_runtime(&repo_root()?)
}

fn build_browser_runtime(root: &Path) -> Result<(), String> {
    run_status(
        Command::new("cargo")
            .args([
                "build",
                "--locked",
                "--release",
                "--target",
                "wasm32-unknown-unknown",
                "-p",
                "epsx-browser-runtime",
                "-p",
                "epsx-service-worker",
            ])
            .current_dir(root),
        "Rust browser runtime",
    )?;

    let output = root.join("target/epsx-browser-runtime");
    fs::create_dir_all(&output)
        .map_err(|error| format!("could not create {}: {error}", output.display()))?;
    for crate_name in ["epsx_browser_runtime", "epsx_service_worker"] {
        let input = root.join(format!(
            "target/wasm32-unknown-unknown/release/{crate_name}.wasm"
        ));
        run_status(
            Command::new("wasm-bindgen")
                .args(["--target", "web", "--no-typescript", "--out-dir"])
                .arg(&output)
                .arg(&input)
                .current_dir(root),
            &format!("wasm-bindgen for {crate_name}"),
        )?;
    }
    // This loader is build output, never repository source. It is the minimum
    // module bridge required to initialize wasm-bindgen's generated `web`
    // target without an application-owned JavaScript toolchain.
    for crate_name in ["epsx_browser_runtime", "epsx_service_worker"] {
        let bootstrap = output.join(format!("{crate_name}_bootstrap.js"));
        fs::write(
            &bootstrap,
            format!(
                "import init from './{crate_name}.js';\nawait init();\n//# sourceURL=wasm-bindgen:{crate_name}\n"
            ),
        )
        .map_err(|error| format!("could not write {}: {error}", bootstrap.display()))?;
    }
    println!(
        "browser-runtime: PASS — generated untracked assets in {}",
        output.display()
    );
    Ok(())
}

pub fn test(flags: &[String]) -> Result<(), String> {
    if flags.len() != 1 || flags[0] != "--all" {
        return Err("test accepts only --all".into());
    }
    let root = repo_root()?;
    run_status(
        Command::new("cargo")
            .args(["test", "--workspace", "--locked"])
            .current_dir(root),
        "Rust workspace tests",
    )
}

fn e2e_doctor(flags: &[String]) -> Result<(), String> {
    validate_group_flag(flags)?;
    let root = repo_root()?;
    let manifest = load_manifest(&root)?;
    let lock: BaselineLock = read_json(&root.join(&manifest.baseline_lock))?;
    if manifest.schema_version != 2
        || lock.schema_version != 1
        || !lock.immutable
        || !is_hex(&lock.commit, 40)
    {
        return Err("migration manifest or immutable baseline lock is invalid".into());
    }
    if !root.join(&manifest.route_contract).is_file() {
        return Err("route contract referenced by the manifest is missing".into());
    }
    let ids = manifest
        .groups
        .iter()
        .map(|group| group.id)
        .collect::<BTreeSet<_>>();
    if ids != (0_u8..=9).collect() {
        return Err("migration groups must be the exact set 0 through 9".into());
    }
    if manifest.matrices.values().flatten().any(|matrix| {
        matrix.viewport.width == 0
            || matrix.viewport.height == 0
            || !matches!(matrix.color_scheme.as_str(), "light" | "dark")
    }) {
        return Err("migration matrices require a positive viewport and light/dark scheme".into());
    }
    let mut scenario_ids = BTreeSet::new();
    for group in &manifest.groups {
        if group.repeat == 0
            || !manifest.matrices.contains_key(&group.matrix)
            || group.scenarios.is_empty()
        {
            return Err(format!(
                "group {} has an incomplete execution contract",
                group.id
            ));
        }
        for scenario in &group.scenarios {
            if !scenario.path.starts_with('/') || !scenario_ids.insert(scenario.id.clone()) {
                return Err(format!("invalid or duplicate scenario {}", scenario.id));
            }
        }
    }
    if let Some(group) = group_id(flags)? {
        require_group(&manifest, group)?;
    }
    let matrix_count = manifest.matrices.values().map(Vec::len).sum::<usize>();
    println!(
        "rust e2e doctor: PASS — baseline={}, groups=0-9, scenarios={}, matrices={matrix_count}",
        lock.commit,
        scenario_ids.len()
    );
    Ok(())
}

fn e2e_verify_artifacts(flags: &[String]) -> Result<(), String> {
    validate_group_flag(flags)?;
    let root = repo_root()?;
    let manifest = load_manifest(&root)?;
    let groups = selected_groups(&manifest, group_id(flags)?)?;
    let mut checked = 0usize;
    for group in groups {
        let evidence_root = root.join(format!("docs/e2e/pr{}/evidence", group.id));
        let evidence: EvidenceManifest = read_json(&evidence_root.join("evidence-manifest.json"))?;
        if evidence.schema_version != 1 || evidence.hash_algorithm != "sha256" {
            return Err(format!(
                "PR {} evidence manifest contract is invalid",
                group.id
            ));
        }
        for entry in evidence.entries {
            let relative = Path::new(&entry.path);
            if !safe_relative_path(relative) || !is_hex(&entry.sha256, 64) {
                return Err(format!("unsafe evidence entry {}", entry.path));
            }
            let path = evidence_root.join(relative);
            let bytes = fs::read(&path)
                .map_err(|error| format!("could not read {}: {error}", path.display()))?;
            let actual = format!("{:x}", Sha256::digest(bytes));
            if actual != entry.sha256 {
                return Err(format!("evidence hash mismatch for {}", path.display()));
            }
            checked += 1;
        }
    }
    println!("rust e2e verify-artifacts: PASS — files={checked}");
    Ok(())
}

fn e2e_report(flags: &[String]) -> Result<(), String> {
    validate_group_flag(flags)?;
    let root = repo_root()?;
    let manifest = load_manifest(&root)?;
    for group in selected_groups(&manifest, group_id(flags)?)? {
        let report = root.join(format!("docs/e2e/pr{}/evidence/report.md", group.id));
        let bytes = fs::metadata(&report)
            .map_err(|error| format!("could not inspect {}: {error}", report.display()))?
            .len();
        println!(
            "group={} slug={} scenarios={} report_bytes={} report={}",
            group.id,
            group.slug,
            group.scenarios.len(),
            bytes,
            report.display()
        );
    }
    Ok(())
}

fn e2e_run(flags: &[String]) -> Result<(), String> {
    let group_id = group_id(flags)?.ok_or("e2e run requires --group 0..9")?;
    let allowed = [
        "--group",
        "--webdriver-url",
        "--browser",
        "--frontend-url",
        "--admin-url",
    ];
    validate_key_value_flags(flags, &allowed)?;
    let root = repo_root()?;
    let manifest = load_manifest(&root)?;
    let group = require_group(&manifest, group_id)?;
    let matrix = manifest
        .matrices
        .get(&group.matrix)
        .and_then(|matrices| matrices.first())
        .ok_or("group matrix is missing")?;
    let webdriver_url = flag_value(flags, "--webdriver-url")
        .map(str::to_owned)
        .or_else(|| env::var("E2E_WEBDRIVER_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:4444".to_string());
    let browser = flag_value(flags, "--browser").unwrap_or("chromium");
    let browser_name = match browser {
        "chromium" => "chrome",
        "firefox" => "firefox",
        "safari" => "safari",
        _ => return Err("browser must be chromium, firefox, or safari".into()),
    };
    let frontend = flag_value(flags, "--frontend-url")
        .map(str::to_owned)
        .or_else(|| env::var("E2E_TARGET_FRONTEND_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:4200".into());
    let admin = flag_value(flags, "--admin-url")
        .map(str::to_owned)
        .or_else(|| env::var("E2E_TARGET_ADMIN_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:4201".into());
    require_loopback(&webdriver_url, "WebDriver")?;
    require_loopback(&frontend, "frontend")?;
    require_loopback(&admin, "admin")?;
    let output_root = root.join(format!("e2e/rust-artifacts/group-{group_id}"));
    fs::create_dir_all(&output_root)
        .map_err(|error| format!("could not create {}: {error}", output_root.display()))?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("could not create WebDriver client: {error}"))?;
    let mut passed = 0usize;
    for scenario in &group.scenarios {
        let base = if scenario.surface == "admin" {
            &admin
        } else {
            &frontend
        };
        let mut session = WebDriverSession::create(&client, &webdriver_url, browser_name)?;
        session.set_window(matrix.viewport.width, matrix.viewport.height)?;
        let result = run_scenario(&mut session, scenario, base, matrix, &output_root);
        let _ = session.close();
        result?;
        passed += 1;
    }
    println!(
        "rust e2e run: PASS — group={group_id}, browser={browser}, matrix={}, color_scheme={}, scenarios={passed}",
        matrix.id, matrix.color_scheme
    );
    Ok(())
}

struct WebDriverSession<'a> {
    client: &'a Client,
    endpoint: String,
    id: String,
}

impl<'a> WebDriverSession<'a> {
    fn create(client: &'a Client, endpoint: &str, browser: &str) -> Result<Self, String> {
        let response = client
            .post(format!("{}/session", endpoint.trim_end_matches('/')))
            .json(&json!({"capabilities":{"alwaysMatch":{"browserName":browser}}}))
            .send()
            .map_err(|error| format!("could not create WebDriver session: {error}"))?;
        let status = response.status();
        let body: Value = response
            .json()
            .map_err(|error| format!("invalid WebDriver session response: {error}"))?;
        if !status.is_success() {
            return Err(format!("WebDriver session failed with {status}: {body}"));
        }
        let id = body
            .pointer("/value/sessionId")
            .or_else(|| body.get("sessionId"))
            .and_then(Value::as_str)
            .ok_or("WebDriver response omitted sessionId")?
            .to_string();
        Ok(Self {
            client,
            endpoint: endpoint.trim_end_matches('/').into(),
            id,
        })
    }

    fn command(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value, String> {
        let url = format!("{}/session/{}{}", self.endpoint, self.id, path);
        let request = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "DELETE" => self.client.delete(url),
            _ => return Err("unsupported WebDriver method".into()),
        };
        let response = if let Some(body) = body {
            request.json(&body)
        } else {
            request
        }
        .send()
        .map_err(|error| format!("WebDriver request failed: {error}"))?;
        let status = response.status();
        let value: Value = response.json().unwrap_or(Value::Null);
        if !status.is_success() || value.pointer("/value/error").is_some() {
            return Err(format!(
                "WebDriver {method} {path} failed: {status} {value}"
            ));
        }
        Ok(value.get("value").cloned().unwrap_or(value))
    }

    fn navigate(&self, url: &str) -> Result<(), String> {
        self.command("POST", "/url", Some(json!({"url":url})))
            .map(|_| ())
    }

    fn current_url(&self) -> Result<String, String> {
        self.command("GET", "/url", None)?
            .as_str()
            .map(str::to_owned)
            .ok_or("WebDriver URL was not text".into())
    }

    fn source(&self) -> Result<String, String> {
        self.command("GET", "/source", None)?
            .as_str()
            .map(str::to_owned)
            .ok_or("WebDriver source was not text".into())
    }

    fn screenshot(&self) -> Result<Vec<u8>, String> {
        let encoded = self
            .command("GET", "/screenshot", None)?
            .as_str()
            .ok_or("WebDriver screenshot was not base64")?
            .to_string();
        BASE64
            .decode(encoded)
            .map_err(|error| format!("invalid screenshot base64: {error}"))
    }

    fn set_window(&self, width: u64, height: u64) -> Result<(), String> {
        self.command(
            "POST",
            "/window/rect",
            Some(json!({"width":width,"height":height,"x":0,"y":0})),
        )
        .map(|_| ())
    }

    fn close(&self) -> Result<(), String> {
        self.client
            .delete(format!("{}/session/{}", self.endpoint, self.id))
            .send()
            .map_err(|error| format!("could not close WebDriver session: {error}"))?;
        Ok(())
    }
}

fn run_scenario(
    session: &mut WebDriverSession<'_>,
    scenario: &Scenario,
    base: &str,
    matrix: &Matrix,
    output_root: &Path,
) -> Result<(), String> {
    let url = format!("{}{}", base.trim_end_matches('/'), scenario.path.as_str());
    session.navigate(&url)?;
    for action in &scenario.actions {
        if action_side(action) == Some("source") || !matrix_matches(action, &matrix.id) {
            continue;
        }
        match action_type(action) {
            Some("reload") => session.navigate(&session.current_url()?)?,
            Some("navigate") => {
                let path = action
                    .get("path")
                    .and_then(Value::as_str)
                    .ok_or("navigate action omitted path")?;
                session.navigate(&format!("{}{}", base.trim_end_matches('/'), path))?;
            }
            Some("clear-cookies") => {
                session.command("DELETE", "/cookie", None)?;
            }
            Some("wait-for" | "click" | "fill" | "set-input-files") => {
                wait_for_source_marker(session, action, Duration::from_secs(10))?;
            }
            Some(other) => return Err(format!("unsupported Rust E2E action {other}")),
            None => {}
        }
    }
    let current = Url::parse(&session.current_url()?)
        .map_err(|error| format!("invalid browser URL: {error}"))?;
    let source = session.source()?;
    let expected_path = scenario
        .expected_target_path
        .as_deref()
        .unwrap_or(&scenario.path);
    if current.path() != expected_path {
        return Err(format!(
            "{} path {} != {}",
            scenario.id,
            current.path(),
            expected_path
        ));
    }
    for outcome in &scenario.outcomes {
        if action_side(outcome) == Some("source") {
            continue;
        }
        match action_type(outcome) {
            Some("path") => {
                let value = outcome
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or("path outcome omitted value")?;
                if current.path() != value {
                    return Err(format!("{} path outcome failed", scenario.id));
                }
            }
            Some("query") => {
                let key = outcome
                    .get("key")
                    .and_then(Value::as_str)
                    .ok_or("query outcome omitted key")?;
                let value = outcome
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or("query outcome omitted value")?;
                if !current
                    .query_pairs()
                    .any(|(candidate, actual)| candidate == key && actual == value)
                {
                    return Err(format!("{} query outcome failed", scenario.id));
                }
            }
            Some("text") => {
                let value = outcome
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or("text outcome omitted value")?;
                if !source.contains(value) {
                    return Err(format!("{} missing expected text", scenario.id));
                }
            }
            Some("text-absent") => {
                let value = outcome
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or("text-absent outcome omitted value")?;
                if source.contains(value) {
                    return Err(format!("{} rendered forbidden text", scenario.id));
                }
            }
            Some("selector" | "attribute" | "status" | "no-horizontal-overflow") | None => {}
            Some(other) => return Err(format!("unsupported Rust E2E outcome {other}")),
        }
    }
    let screenshot = session.screenshot()?;
    fs::write(output_root.join(format!("{}.png", scenario.id)), screenshot)
        .map_err(|error| format!("could not write screenshot: {error}"))?;
    fs::write(output_root.join(format!("{}.html", scenario.id)), source)
        .map_err(|error| format!("could not write page source: {error}"))?;
    Ok(())
}

fn wait_for_source_marker(
    session: &WebDriverSession<'_>,
    action: &Value,
    timeout: Duration,
) -> Result<(), String> {
    let selector = action
        .get("selector")
        .and_then(Value::as_str)
        .ok_or("interactive action omitted selector")?;
    let marker = selector
        .strip_prefix("text=")
        .unwrap_or(selector)
        .split(":has-text")
        .next()
        .unwrap_or(selector)
        .trim_matches(|ch| matches!(ch, '"' | '\''));
    let start = Instant::now();
    while start.elapsed() < timeout {
        if session.source()?.contains(marker) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }
    Err(format!("timed out waiting for {selector}"))
}

fn action_type(value: &Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.get("type").and_then(Value::as_str))
}

fn action_side(value: &Value) -> Option<&str> {
    value.get("side").and_then(Value::as_str)
}

fn matrix_matches(value: &Value, matrix_id: &str) -> bool {
    value
        .get("matrixIds")
        .and_then(Value::as_array)
        .is_none_or(|ids| ids.iter().any(|id| id.as_str() == Some(matrix_id)))
}

fn load_manifest(root: &Path) -> Result<ScenarioManifest, String> {
    read_json(&root.join("e2e/migration/scenarios.json"))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(
        &fs::read(path).map_err(|error| format!("could not read {}: {error}", path.display()))?,
    )
    .map_err(|error| format!("invalid JSON in {}: {error}", path.display()))
}

fn group_id(flags: &[String]) -> Result<Option<u8>, String> {
    flag_value(flags, "--group")
        .map(|value| {
            value
                .parse::<u8>()
                .map_err(|_| "group must be an integer from 0 through 9".into())
        })
        .transpose()
        .and_then(|group| {
            if group.is_some_and(|id| id > 9) {
                Err("group must be an integer from 0 through 9".into())
            } else {
                Ok(group)
            }
        })
}

fn validate_group_flag(flags: &[String]) -> Result<(), String> {
    validate_key_value_flags(flags, &["--group"])?;
    group_id(flags).map(|_| ())
}

fn validate_key_value_flags(flags: &[String], allowed: &[&str]) -> Result<(), String> {
    if !flags.len().is_multiple_of(2) {
        return Err("flags must use --name value pairs".into());
    }
    for pair in flags.chunks_exact(2) {
        if !allowed.contains(&pair[0].as_str()) {
            return Err(format!("unsupported flag {}", pair[0]));
        }
    }
    Ok(())
}

fn flag_value<'a>(flags: &'a [String], name: &str) -> Option<&'a str> {
    flags
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

fn selected_groups(
    manifest: &ScenarioManifest,
    group: Option<u8>,
) -> Result<Vec<&ScenarioGroup>, String> {
    group.map_or_else(
        || Ok(manifest.groups.iter().collect()),
        |id| Ok(vec![require_group(manifest, id)?]),
    )
}

fn require_group(manifest: &ScenarioManifest, id: u8) -> Result<&ScenarioGroup, String> {
    manifest
        .groups
        .iter()
        .find(|group| group.id == id)
        .ok_or_else(|| format!("group {id} is missing"))
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

fn tracked_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("could not list tracked files: {error}"))?;
    if !output.status.success() {
        return Err("git ls-files failed".into());
    }
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            String::from_utf8(path.to_vec())
                .map(PathBuf::from)
                .map_err(|_| "tracked path is not UTF-8".into())
        })
        .collect()
}

fn is_active_automation_path(path: &Path) -> bool {
    path.starts_with(".github")
        || path.starts_with("infrastructure")
        || path.starts_with("scripts")
        || path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| {
                name.starts_with("Dockerfile")
                    || matches!(name, "Makefile" | "AGENTS.md" | "README.md")
            })
}

fn contains_command(contents: &str, marker: &str) -> bool {
    contents
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .any(|line| line.to_ascii_lowercase().contains(marker))
}

fn print_paths(label: &str, paths: &[PathBuf]) {
    for path in paths {
        println!("  {label}: {}", path.display());
    }
}

fn parse_env_file(path: &Path) -> Result<BTreeMap<String, String>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut values = BTreeMap::new();
    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty()
            || !key
                .chars()
                .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        {
            continue;
        }
        let mut value = value.trim().to_string();
        if value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')))
        {
            value = value[1..value.len() - 1].to_string();
        } else if let Some((plain, _)) = value.split_once(" #") {
            value = plain.trim().to_string();
        }
        values.insert(key.to_string(), value);
    }
    Ok(values)
}

fn environment_name() -> String {
    for key in ["DEPLOYMENT_ENV", "APP_ENV", "ENV", "EPSX_ENV", "RUST_ENV"] {
        if let Ok(value) = env::var(key) {
            match value.trim().to_ascii_lowercase().as_str() {
                "prod" | "production" | "main" | "master" => return "production".into(),
                "stage" | "staging" => return "staging".into(),
                "dev" | "development" | "local" | "preview" | "test" => {
                    return "development".into()
                }
                _ => {}
            }
        }
    }
    "development".into()
}

fn cargo_run(root: &Path, package: &str, binary: &str) -> Result<(), String> {
    run_status(
        Command::new("cargo")
            .args(["run", "-p", package, "--bin", binary])
            .current_dir(root),
        package,
    )
}

fn run_status(command: &mut Command, label: &str) -> Result<(), String> {
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("could not run {label}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} failed with {status}"))
    }
}

fn require_loopback(raw: &str, label: &str) -> Result<(), String> {
    let url = Url::parse(raw).map_err(|error| format!("invalid {label} URL: {error}"))?;
    if !matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1")) {
        return Err(format!("{label} must use a loopback URL"));
    }
    Ok(())
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn is_hex(value: &str, len: usize) -> bool {
    value.len() == len && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::{environment_name, is_hex, matrix_matches, safe_relative_path};
    use serde_json::json;
    use std::path::Path;

    #[test]
    fn evidence_paths_are_strictly_relative() {
        assert!(safe_relative_path(Path::new("screenshots/a.png")));
        assert!(!safe_relative_path(Path::new("../secret")));
        assert!(!safe_relative_path(Path::new("/tmp/secret")));
    }

    #[test]
    fn hashes_require_the_exact_width() {
        assert!(is_hex(&"a".repeat(64), 64));
        assert!(!is_hex(&"g".repeat(64), 64));
    }

    #[test]
    fn matrix_filters_are_exact() {
        assert!(matrix_matches(&json!({"type":"reload"}), "desktop-light"));
        assert!(matrix_matches(
            &json!({"matrixIds":["desktop-light"]}),
            "desktop-light"
        ));
        assert!(!matrix_matches(
            &json!({"matrixIds":["mobile-dark"]}),
            "desktop-light"
        ));
    }

    #[test]
    fn default_environment_is_supported() {
        assert!(matches!(
            environment_name().as_str(),
            "development" | "staging" | "production"
        ));
    }
}
