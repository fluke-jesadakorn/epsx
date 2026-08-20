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
const ACTIVE_NODE_COMMANDS: &[&str] = &["node", "bun", "bunx", "npm", "npx", "yarn", "pnpm"];
const INLINE_RUNTIME_MARKERS: &[&str] = &[
    "document::eval",
    "dangerous_inner_html: AUTH_REDIRECT_SCRIPT",
];
const W3C_ELEMENT_KEY: &str = "element-6066-11e4-a52e-4f735466cecf";

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
    #[serde(default)]
    state: Value,
    #[serde(default, rename = "fixtureRequirements")]
    fixture_requirements: Vec<String>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FixtureCapabilities {
    schema_version: u64,
    authority: String,
    session_algorithm: String,
    key_id: String,
    supported_groups: Vec<u8>,
    supported_modes: Vec<String>,
    reset_proof: bool,
}

struct StateProvisioner<'a> {
    client: &'a Client,
    endpoint: String,
    token: String,
    capabilities: FixtureCapabilities,
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
        if is_active_automation_path(relative) && contains_node_command(&contents) {
            active_refs.push(relative.clone());
        }
        if extension == "rs"
            && relative != Path::new("xtask/src/node_free.rs")
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
        "fixture-serve" => crate::e2e_fixture::serve(args),
        "report" => e2e_report(args),
        "verify-artifacts" => e2e_verify_artifacts(args),
        _ => Err(format!("unknown e2e command {command}")),
    }
}

pub fn design(flags: &[String]) -> Result<(), String> {
    if flags.first().map(String::as_str) != Some("capture") {
        return Err("design accepts only: design capture --group 0..9".into());
    }
    e2e_run(&flags[1..])
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
    let mut action_count = 0usize;
    let mut outcome_count = 0usize;
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
            if !scenario.path.starts_with('/')
                || scenario.outcomes.is_empty()
                || !scenario_ids.insert(scenario.id.clone())
            {
                return Err(format!("invalid or duplicate scenario {}", scenario.id));
            }
            validate_scenario_contract(
                scenario,
                manifest
                    .matrices
                    .get(&group.matrix)
                    .ok_or("group matrix is missing")?,
            )?;
            action_count += scenario.actions.len();
            outcome_count += scenario.outcomes.len();
        }
    }
    if let Some(group) = group_id(flags)? {
        require_group(&manifest, group)?;
    }
    let matrix_count = manifest.matrices.values().map(Vec::len).sum::<usize>();
    println!(
        "rust e2e doctor: PASS — baseline={}, groups=0-9, scenarios={}, actions={action_count}, asserted_outcomes={outcome_count}, matrices={matrix_count}",
        lock.commit, scenario_ids.len()
    );
    Ok(())
}

fn validate_scenario_contract(scenario: &Scenario, matrices: &[Matrix]) -> Result<(), String> {
    let state = scenario
        .state
        .as_object()
        .ok_or_else(|| format!("{} state must be an object", scenario.id))?;
    let session = state
        .get("session")
        .and_then(Value::as_str)
        .filter(|session| matches!(*session, "signed-out" | "authenticated"))
        .ok_or_else(|| format!("{} state has an invalid session", scenario.id))?;
    state
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| format!("{} state omitted id", scenario.id))?;
    if session == "authenticated" {
        state
            .get("audience")
            .and_then(Value::as_str)
            .filter(|audience| matches!(*audience, "epsx-frontend" | "epsx-admin"))
            .ok_or_else(|| format!("{} authenticated state omitted audience", scenario.id))?;
        let permissions = state
            .get("permissions")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{} authenticated state omitted permissions", scenario.id))?;
        if permissions
            .iter()
            .any(|permission| permission.as_str().is_none_or(str::is_empty))
        {
            return Err(format!(
                "{} state contains an invalid permission",
                scenario.id
            ));
        }
    }
    if state
        .get("fixtureModeSide")
        .and_then(Value::as_str)
        .is_some_and(|side| !matches!(side, "source" | "target"))
    {
        return Err(format!("{} state has an invalid fixture side", scenario.id));
    }
    if state
        .get("fixtureMode")
        .is_some_and(|mode| mode.as_str().is_none_or(str::is_empty))
    {
        return Err(format!("{} state has an invalid fixture mode", scenario.id));
    }
    if scenario
        .fixture_requirements
        .iter()
        .any(|requirement| requirement.trim().is_empty())
    {
        return Err(format!("{} has an empty fixture requirement", scenario.id));
    }

    let matrix_ids = matrices
        .iter()
        .map(|matrix| matrix.id.as_str())
        .collect::<BTreeSet<_>>();
    for action in &scenario.actions {
        validate_contract_scope(&scenario.id, action, &matrix_ids)?;
        match action_type(action) {
            Some("reload" | "clear-cookies") => {}
            Some("navigate") => {
                let path = require_string(action, "path", &scenario.id, "navigate action")?;
                if !path.starts_with('/') {
                    return Err(format!("{} navigate path must be absolute", scenario.id));
                }
            }
            Some("wait-for" | "click") => {
                validate_selector_syntax(action_selector(action)?)?;
            }
            Some("fill") => {
                validate_selector_syntax(action_selector(action)?)?;
                require_string(action, "value", &scenario.id, "fill action")?;
            }
            Some("set-input-files") => {
                validate_selector_syntax(action_selector(action)?)?;
                let name = require_string(action, "name", &scenario.id, "file action")?;
                if Path::new(name).components().count() != 1 || !safe_relative_path(Path::new(name))
                {
                    return Err(format!("{} file action has an unsafe name", scenario.id));
                }
                let mime = require_string(action, "mimeType", &scenario.id, "file action")?;
                if !mime.contains('/') || mime.bytes().any(|byte| byte.is_ascii_whitespace()) {
                    return Err(format!(
                        "{} file action has an invalid MIME type",
                        scenario.id
                    ));
                }
                let content = require_string(action, "contentBase64", &scenario.id, "file action")?;
                BASE64.decode(content).map_err(|error| {
                    format!("{} file action has invalid base64: {error}", scenario.id)
                })?;
            }
            Some(other) => {
                return Err(format!("{} has unsupported action {other}", scenario.id));
            }
            None => return Err(format!("{} action omitted type", scenario.id)),
        }
    }

    for outcome in &scenario.outcomes {
        validate_contract_scope(&scenario.id, outcome, &matrix_ids)?;
        match action_type(outcome) {
            Some("path") => {
                let path = require_string(outcome, "value", &scenario.id, "path outcome")?;
                if !path.starts_with('/') {
                    return Err(format!("{} outcome path must be absolute", scenario.id));
                }
            }
            Some("query") => {
                require_string(outcome, "key", &scenario.id, "query outcome")?;
                require_string(outcome, "value", &scenario.id, "query outcome")?;
            }
            Some("text" | "text-absent") => {
                require_string(outcome, "value", &scenario.id, "text outcome")?;
            }
            Some("selector") => {
                validate_selector_syntax(require_string(
                    outcome,
                    "value",
                    &scenario.id,
                    "selector outcome",
                )?)?;
            }
            Some("attribute") => {
                validate_selector_syntax(action_selector(outcome)?)?;
                let name = require_string(outcome, "name", &scenario.id, "attribute outcome")?;
                if !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':'))
                {
                    return Err(format!(
                        "{} attribute outcome has an invalid name",
                        scenario.id
                    ));
                }
                require_string(outcome, "value", &scenario.id, "attribute outcome")?;
            }
            Some("status") => {
                let status = outcome
                    .get("value")
                    .and_then(Value::as_u64)
                    .filter(|status| (100..=599).contains(status))
                    .ok_or_else(|| format!("{} status outcome is invalid", scenario.id))?;
                let _ = status;
            }
            Some("no-horizontal-overflow") => {}
            Some(other) => {
                return Err(format!("{} has unsupported outcome {other}", scenario.id));
            }
            None => return Err(format!("{} outcome omitted type", scenario.id)),
        }
    }
    Ok(())
}

fn validate_contract_scope(
    scenario_id: &str,
    value: &Value,
    matrix_ids: &BTreeSet<&str>,
) -> Result<(), String> {
    if value
        .get("side")
        .and_then(Value::as_str)
        .is_some_and(|side| !matches!(side, "source" | "target"))
    {
        return Err(format!("{scenario_id} contract has an invalid side"));
    }
    if let Some(ids) = value.get("matrixIds") {
        let ids = ids
            .as_array()
            .ok_or_else(|| format!("{scenario_id} matrixIds must be an array"))?;
        if ids.is_empty()
            || ids
                .iter()
                .any(|id| id.as_str().is_none_or(|id| !matrix_ids.contains(id)))
        {
            return Err(format!("{scenario_id} contract has an unknown matrix ID"));
        }
    }
    Ok(())
}

fn require_string<'a>(
    value: &'a Value,
    key: &str,
    scenario_id: &str,
    label: &str,
) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{scenario_id} {label} omitted {key}"))
}

fn validate_selector_syntax(selector: &str) -> Result<(), String> {
    for branch in split_selector_branches(selector)? {
        let (branch, _) = selector_nth(&branch)?;
        let branch = branch.replace(":visible", "");
        let (css, _) = selector_text_filter(&branch)?;
        if css.trim().is_empty() {
            return Err("selector resolves to empty CSS".into());
        }
    }
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
        "--fixture-url",
        "--fixture-token",
    ];
    validate_key_value_flags(flags, &allowed)?;
    let root = repo_root()?;
    let manifest = load_manifest(&root)?;
    let group = require_group(&manifest, group_id)?;
    let matrices = manifest
        .matrices
        .get(&group.matrix)
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
    let fixture_url = flag_value(flags, "--fixture-url")
        .map(str::to_owned)
        .or_else(|| env::var("E2E_FIXTURE_URL").ok());
    let fixture_token = flag_value(flags, "--fixture-token")
        .map(str::to_owned)
        .or_else(|| env::var("E2E_FIXTURE_TOKEN").ok())
        .unwrap_or_else(|| "epsx-e2e-local-reset-token".into());
    let provisioner = fixture_url
        .as_deref()
        .map(|endpoint| StateProvisioner::connect(&client, endpoint, &fixture_token))
        .transpose()?;
    require_runtime_state_provisioning(group, provisioner.as_ref())?;
    let mut passed = 0usize;
    for matrix in matrices {
        for repeat in 1..=group.repeat {
            let run_root = output_root
                .join(browser)
                .join(&matrix.id)
                .join(format!("repeat-{repeat}"));
            fs::create_dir_all(&run_root)
                .map_err(|error| format!("could not create {}: {error}", run_root.display()))?;
            for scenario in &group.scenarios {
                let base = if scenario.surface == "admin" {
                    &admin
                } else {
                    &frontend
                };
                let mut session =
                    WebDriverSession::create(&client, &webdriver_url, browser_name, matrix)?;
                let mut provisioned = None;
                let result = (|| {
                    session.set_window(matrix.viewport.width, matrix.viewport.height)?;
                    if let Some(state_authority) = provisioner.as_ref() {
                        provisioned = Some(state_authority.prepare(scenario)?);
                    }
                    if let Some(access_token) = provisioned
                        .as_ref()
                        .and_then(|provisioned| provisioned.access_token.as_deref())
                    {
                        session.install_access_cookie(base, &scenario.surface, access_token)?;
                    }
                    run_scenario(&mut session, scenario, base, matrix, &run_root)
                })();
                let close_result = session.close();
                let reset_result = provisioner.as_ref().map(|state_authority| {
                    if let Some(provisioned) = provisioned.as_ref() {
                        state_authority.finish(scenario, provisioned, &run_root)
                    } else {
                        state_authority.cleanup_failed_setup(scenario)
                    }
                });
                combine_scenario_results(result, close_result, reset_result)?;
                passed += 1;
            }
        }
    }
    println!(
        "rust e2e run: PASS — group={group_id}, browser={browser}, matrices={}, repeats={}, executions={passed}",
        matrices.len(), group.repeat
    );
    Ok(())
}

fn require_runtime_state_provisioning(
    group: &ScenarioGroup,
    provisioner: Option<&StateProvisioner<'_>>,
) -> Result<(), String> {
    let blocked = group
        .scenarios
        .iter()
        .filter(|scenario| {
            let authenticated =
                scenario.state.get("session").and_then(Value::as_str) == Some("authenticated");
            let target_fixture = scenario.state.get("fixtureMode").is_some()
                && scenario
                    .state
                    .get("fixtureModeSide")
                    .and_then(Value::as_str)
                    != Some("source");
            authenticated || target_fixture
        })
        .map(|scenario| scenario.id.as_str())
        .collect::<Vec<_>>();
    if blocked.is_empty() {
        return Ok(());
    }
    if let Some(provisioner) = provisioner {
        return provisioner.supports(group);
    }
    Err(format!(
        "group {} requires authenticated or target-fixture scenario provisioning for {} scenarios (first: {}); refusing an unprovisioned false-positive run",
        group.id,
        blocked.len(),
        blocked.iter().take(5).copied().collect::<Vec<_>>().join(", ")
    ))
}

#[derive(Debug)]
struct ProvisionedScenario {
    reset_before: Value,
    configured_state: Value,
    access_token: Option<String>,
}

impl<'a> StateProvisioner<'a> {
    fn connect(client: &'a Client, endpoint: &str, token: &str) -> Result<Self, String> {
        require_loopback(endpoint, "E2E fixture provisioner")?;
        if token.len() < 16 || token.len() > 256 {
            return Err("E2E fixture token must contain 16 through 256 bytes".into());
        }
        let mut provisioner = Self {
            client,
            endpoint: endpoint.trim_end_matches('/').into(),
            token: token.into(),
            capabilities: FixtureCapabilities {
                schema_version: 0,
                authority: String::new(),
                session_algorithm: String::new(),
                key_id: String::new(),
                supported_groups: Vec::new(),
                supported_modes: Vec::new(),
                reset_proof: false,
            },
        };
        provisioner.capabilities =
            serde_json::from_value(provisioner.control("GET", "/__e2e/capabilities", None)?)
                .map_err(|error| format!("invalid fixture capability contract: {error}"))?;
        let capabilities = &provisioner.capabilities;
        if capabilities.schema_version != 1
            || capabilities.authority != "epsx-rust-e2e-fixture"
            || capabilities.session_algorithm != "RS256"
            || capabilities.key_id != "epsx-e2e-rs256-v1"
            || !capabilities.reset_proof
        {
            return Err(
                "fixture provisioner does not satisfy the Rust E2E authority contract".into(),
            );
        }
        Ok(provisioner)
    }

    fn supports(&self, group: &ScenarioGroup) -> Result<(), String> {
        if !self.capabilities.supported_groups.contains(&group.id) {
            return Err(format!(
                "fixture provisioner does not support migration group {}; supported groups: {}",
                group.id,
                self.capabilities
                    .supported_groups
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        for scenario in &group.scenarios {
            if target_fixture_mode(scenario).is_some_and(|mode| {
                !self
                    .capabilities
                    .supported_modes
                    .iter()
                    .any(|supported| supported == mode)
            }) {
                return Err(format!(
                    "fixture provisioner does not support mode required by {}",
                    scenario.id
                ));
            }
        }
        Ok(())
    }

    fn prepare(&self, scenario: &Scenario) -> Result<ProvisionedScenario, String> {
        let reset_before = self.control("POST", "/__e2e/reset", Some(json!({})))?;
        require_clean_reset(&reset_before, &scenario.id, "pre")?;
        if let Some(mode) = target_fixture_mode(scenario) {
            let configured = self.control("PUT", "/__e2e/mode", Some(json!({"mode":mode})))?;
            if configured.get("mode").and_then(Value::as_str) != Some(mode) {
                return Err(format!(
                    "fixture provisioner did not select {mode} for {}",
                    scenario.id
                ));
            }
        }
        let access_token =
            if scenario.state.get("session").and_then(Value::as_str) == Some("authenticated") {
                Some(self.session(scenario)?)
            } else {
                None
            };
        let configured_state = self.control("GET", "/__e2e/state", None)?;
        Ok(ProvisionedScenario {
            reset_before,
            configured_state,
            access_token,
        })
    }

    fn finish(
        &self,
        scenario: &Scenario,
        provisioned: &ProvisionedScenario,
        output_root: &Path,
    ) -> Result<(), String> {
        let observed_before_reset = self.control("GET", "/__e2e/state", None)?;
        let reset_after = self.control("POST", "/__e2e/reset", Some(json!({})))?;
        require_clean_reset(&reset_after, &scenario.id, "post")?;
        let observed_after_reset = self.control("GET", "/__e2e/state", None)?;
        let clean_after = fixture_state_is_clean(&observed_after_reset);
        let proof = json!({
            "schemaVersion":1,
            "scenarioId":scenario.id,
            "authority":self.capabilities.authority,
            "sessionAlgorithm":self.capabilities.session_algorithm,
            "resetBefore":provisioned.reset_before,
            "configuredState":provisioned.configured_state,
            "observedBeforeReset":observed_before_reset,
            "resetAfter":reset_after,
            "observedAfterReset":observed_after_reset,
            "checks":{
                "preResetClean":true,
                "postResetClean":clean_after,
                "accessTokenProvisioned":provisioned.access_token.is_some(),
                "accessTokenRequired":scenario.state.get("session").and_then(Value::as_str) == Some("authenticated")
            },
            "passed":clean_after
        });
        let path = output_root.join(format!("{}.fixture-reset-proof.json", scenario.id));
        fs::write(
            &path,
            serde_json::to_vec_pretty(&proof)
                .map_err(|error| format!("could not serialize fixture reset proof: {error}"))?,
        )
        .map_err(|error| format!("could not write {}: {error}", path.display()))?;
        if !clean_after {
            return Err(format!(
                "fixture reset proof failed for {}; see {}",
                scenario.id,
                path.display()
            ));
        }
        Ok(())
    }

    fn cleanup_failed_setup(&self, scenario: &Scenario) -> Result<(), String> {
        let reset = self.control("POST", "/__e2e/reset", Some(json!({})))?;
        require_clean_reset(&reset, &scenario.id, "failed-setup")
    }

    fn session(&self, scenario: &Scenario) -> Result<String, String> {
        let audience = scenario
            .state
            .get("audience")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{} authenticated state omitted audience", scenario.id))?;
        let permissions = scenario
            .state
            .get("permissions")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{} authenticated state omitted permissions", scenario.id))?
            .iter()
            .map(|permission| {
                permission
                    .as_str()
                    .ok_or_else(|| format!("{} has a non-text permission", scenario.id))
            })
            .collect::<Result<Vec<_>, String>>()?
            .join(" ");
        let key_id = scenario
            .state
            .get("tokenKeyId")
            .and_then(Value::as_str)
            .unwrap_or(&self.capabilities.key_id);
        let mut url = Url::parse(&format!("{}/__e2e/session", self.endpoint))
            .map_err(|error| format!("invalid fixture session URL: {error}"))?;
        url.query_pairs_mut()
            .append_pair("audience", audience)
            .append_pair("permissions", &permissions)
            .append_pair("key_id", key_id);
        let path = match url.query() {
            Some(query) => format!("{}?{query}", url.path()),
            None => url.path().to_string(),
        };
        let response = self.control("GET", &path, None)?;
        let access_token = response
            .get("accessToken")
            .and_then(Value::as_str)
            .filter(|token| token.split('.').count() == 3 && token.len() < 16 * 1024)
            .ok_or_else(|| format!("fixture session response was invalid for {}", scenario.id))?;
        Ok(access_token.into())
    }

    fn control(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value, String> {
        let url = format!("{}{}", self.endpoint, path);
        let request = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            _ => return Err("unsupported fixture control method".into()),
        }
        .header("x-epsx-e2e-token", &self.token);
        let response = if let Some(body) = body {
            request.json(&body)
        } else {
            request
        }
        .send()
        .map_err(|error| format!("fixture control {method} failed: {error}"))?;
        let status = response.status();
        let value = response
            .json::<Value>()
            .map_err(|error| format!("fixture control {method} returned invalid JSON: {error}"))?;
        if !status.is_success() {
            return Err(format!(
                "fixture control {method} {} failed with {status}: {}",
                path.split('?').next().unwrap_or(path),
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            ));
        }
        Ok(value)
    }
}

fn target_fixture_mode(scenario: &Scenario) -> Option<&str> {
    let side = scenario
        .state
        .get("fixtureModeSide")
        .and_then(Value::as_str)
        .unwrap_or("both");
    (side != "source")
        .then(|| scenario.state.get("fixtureMode").and_then(Value::as_str))
        .flatten()
}

fn require_clean_reset(value: &Value, scenario_id: &str, phase: &str) -> Result<(), String> {
    if value.get("schemaVersion").and_then(Value::as_u64) == Some(1)
        && value.get("reset").and_then(Value::as_bool) == Some(true)
        && value.get("mode").and_then(Value::as_str) == Some("healthy")
        && value.get("requestCount").and_then(Value::as_u64) == Some(0)
        && value.get("mutationCount").and_then(Value::as_u64) == Some(0)
    {
        return Ok(());
    }
    Err(format!(
        "fixture {phase}-reset was not clean for {scenario_id}"
    ))
}

fn fixture_state_is_clean(value: &Value) -> bool {
    value.get("schemaVersion").and_then(Value::as_u64) == Some(1)
        && value.get("mode").and_then(Value::as_str) == Some("healthy")
        && value.get("requestCount").and_then(Value::as_u64) == Some(0)
        && value
            .get("requests")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
        && value
            .get("mutations")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
}

fn combine_scenario_results(
    scenario: Result<(), String>,
    close: Result<(), String>,
    reset: Option<Result<(), String>>,
) -> Result<(), String> {
    let mut failures = Vec::new();
    if let Err(error) = scenario {
        failures.push(format!("scenario failed: {error}"));
    }
    if let Err(error) = close {
        failures.push(format!("WebDriver cleanup failed: {error}"));
    }
    if let Some(Err(error)) = reset {
        failures.push(format!("fixture rollback failed: {error}"));
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

struct WebDriverSession<'a> {
    client: &'a Client,
    endpoint: String,
    id: String,
}

#[derive(Debug, Clone, Copy)]
struct ElementRect {
    x: f64,
    width: f64,
}

impl<'a> WebDriverSession<'a> {
    fn create(
        client: &'a Client,
        endpoint: &str,
        browser: &str,
        matrix: &Matrix,
    ) -> Result<Self, String> {
        let mut always_match = json!({"browserName":browser});
        match browser {
            "chrome" => {
                let mut arguments = vec![
                    "--headless=new".to_string(),
                    "--no-sandbox".to_string(),
                    "--disable-dev-shm-usage".to_string(),
                    format!(
                        "--window-size={},{}",
                        matrix.viewport.width, matrix.viewport.height
                    ),
                ];
                if matrix.color_scheme == "dark" {
                    arguments.push("--force-dark-mode".into());
                }
                always_match["goog:chromeOptions"] = json!({"args": arguments});
            }
            "firefox" => {
                always_match["moz:firefoxOptions"] = json!({
                    "args": ["-headless"],
                    "prefs": {
                        "ui.systemUsesDarkTheme": if matrix.color_scheme == "dark" { 1 } else { 0 }
                    }
                });
            }
            _ => {}
        }
        let response = client
            .post(format!("{}/session", endpoint.trim_end_matches('/')))
            .json(&json!({"capabilities":{"alwaysMatch":always_match}}))
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

    fn find_elements(&self, using: &str, value: &str) -> Result<Vec<String>, String> {
        let response = self.command(
            "POST",
            "/elements",
            Some(json!({"using":using,"value":value})),
        )?;
        response
            .as_array()
            .ok_or("WebDriver elements response was not an array")?
            .iter()
            .map(|element| {
                element
                    .get(W3C_ELEMENT_KEY)
                    .or_else(|| element.get("ELEMENT"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .ok_or_else(|| "WebDriver element omitted its element ID".to_string())
            })
            .collect()
    }

    fn element_text(&self, element: &str) -> Result<String, String> {
        self.command("GET", &format!("/element/{element}/text"), None)?
            .as_str()
            .map(str::to_owned)
            .ok_or("WebDriver element text was not text".into())
    }

    fn element_displayed(&self, element: &str) -> Result<bool, String> {
        self.command("GET", &format!("/element/{element}/displayed"), None)?
            .as_bool()
            .ok_or("WebDriver displayed result was not boolean".into())
    }

    fn element_attribute(&self, element: &str, name: &str) -> Result<Option<String>, String> {
        let value = self.command("GET", &format!("/element/{element}/attribute/{name}"), None)?;
        if value.is_null() {
            Ok(None)
        } else {
            value
                .as_str()
                .map(|value| Some(value.to_string()))
                .ok_or("WebDriver attribute result was not text or null".into())
        }
    }

    fn element_rect(&self, element: &str) -> Result<ElementRect, String> {
        let value = self.command("GET", &format!("/element/{element}/rect"), None)?;
        Ok(ElementRect {
            x: value
                .get("x")
                .and_then(Value::as_f64)
                .ok_or("WebDriver element rect omitted x")?,
            width: value
                .get("width")
                .and_then(Value::as_f64)
                .ok_or("WebDriver element rect omitted width")?,
        })
    }

    fn click(&self, element: &str) -> Result<(), String> {
        self.command(
            "POST",
            &format!("/element/{element}/click"),
            Some(json!({})),
        )
        .map(|_| ())
    }

    fn clear(&self, element: &str) -> Result<(), String> {
        self.command(
            "POST",
            &format!("/element/{element}/clear"),
            Some(json!({})),
        )
        .map(|_| ())
    }

    fn send_keys(&self, element: &str, value: &str) -> Result<(), String> {
        let characters = value.chars().map(|ch| ch.to_string()).collect::<Vec<_>>();
        self.command(
            "POST",
            &format!("/element/{element}/value"),
            Some(json!({"text":value,"value":characters})),
        )
        .map(|_| ())
    }

    fn install_access_cookie(
        &self,
        base_url: &str,
        surface: &str,
        access_token: &str,
    ) -> Result<(), String> {
        if access_token.contains(['\r', '\n', ';']) || access_token.len() >= 16 * 1024 {
            return Err("fixture access token is unsafe for a browser cookie".into());
        }
        self.navigate(base_url)?;
        self.command("DELETE", "/cookie", None)?;
        let name = match surface {
            "frontend" => "epsx.frontend.access_token",
            "admin" => "epsx.admin.access_token",
            _ => return Err("unsupported scenario surface for access cookie".into()),
        };
        self.command(
            "POST",
            "/cookie",
            Some(json!({
                "cookie":{
                    "name":name,
                    "value":access_token,
                    "path":"/",
                    "httpOnly":true,
                    "secure":false,
                    "sameSite":"Lax"
                }
            })),
        )
        .map(|_| ())
    }

    fn http_status(&self, url: &str) -> Result<u16, String> {
        require_loopback(url, "browser status probe")?;
        let cookies = self.command("GET", "/cookie", None)?;
        let cookie_header = cookies
            .as_array()
            .ok_or("WebDriver cookie response was not an array")?
            .iter()
            .map(|cookie| {
                let name = cookie
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or("WebDriver cookie omitted name")?;
                let value = cookie
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or("WebDriver cookie omitted value")?;
                Ok(format!("{name}={value}"))
            })
            .collect::<Result<Vec<_>, String>>()?
            .join("; ");
        let mut request = self.client.get(url);
        if !cookie_header.is_empty() {
            request = request.header(reqwest::header::COOKIE, cookie_header);
        }
        request
            .send()
            .map(|response| response.status().as_u16())
            .map_err(|error| format!("could not probe browser URL status: {error}"))
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
            Some("wait-for") => {
                wait_for_elements(session, action_selector(action)?, Duration::from_secs(10))?;
            }
            Some("click") => {
                let element =
                    wait_for_elements(session, action_selector(action)?, Duration::from_secs(10))?
                        .into_iter()
                        .next()
                        .ok_or("click action resolved no element")?;
                session.click(&element)?;
            }
            Some("fill") => {
                let value = action
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or("fill action omitted value")?;
                let element =
                    wait_for_elements(session, action_selector(action)?, Duration::from_secs(10))?
                        .into_iter()
                        .next()
                        .ok_or("fill action resolved no element")?;
                session.clear(&element)?;
                session.send_keys(&element, value)?;
            }
            Some("set-input-files") => {
                set_input_file(session, scenario, action, output_root)?;
            }
            Some(other) => return Err(format!("unsupported Rust E2E action {other}")),
            None => return Err(format!("{} action omitted type", scenario.id)),
        }
    }
    let expected_path = scenario
        .expected_target_path
        .as_deref()
        .or_else(|| target_path_outcome(scenario))
        .or_else(|| target_navigation_path(scenario, &matrix.id))
        .unwrap_or_else(|| requested_path(&scenario.path));
    let current_url =
        wait_for_url_contract(session, scenario, expected_path, Duration::from_secs(10))?;
    let current =
        Url::parse(&current_url).map_err(|error| format!("invalid browser URL: {error}"))?;
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
                wait_for_rendered_text(session, value, Duration::from_secs(10))?;
            }
            Some("text-absent") => {
                let value = outcome
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or("text-absent outcome omitted value")?;
                if session.source()?.contains(value) {
                    return Err(format!("{} rendered forbidden text", scenario.id));
                }
            }
            Some("selector") => {
                let selector = outcome
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or("selector outcome omitted value")?;
                wait_for_elements(session, selector, Duration::from_secs(10))?;
            }
            Some("attribute") => {
                let selector = action_selector(outcome)?;
                let name = outcome
                    .get("name")
                    .and_then(Value::as_str)
                    .filter(|name| {
                        name.bytes().all(|byte| {
                            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':')
                        })
                    })
                    .ok_or("attribute outcome has an invalid or missing name")?;
                let expected = outcome
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or("attribute outcome omitted value")?;
                let element = wait_for_elements(session, selector, Duration::from_secs(10))?
                    .into_iter()
                    .next()
                    .ok_or("attribute outcome resolved no element")?;
                let actual = session.element_attribute(&element, name)?;
                if actual.as_deref() != Some(expected) {
                    return Err(format!(
                        "{} attribute {name} was {:?}, expected {expected:?}",
                        scenario.id, actual
                    ));
                }
            }
            Some("status") => {
                let expected = outcome
                    .get("value")
                    .and_then(Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok())
                    .ok_or("status outcome omitted a valid HTTP status")?;
                let actual = session.http_status(&current_url)?;
                if actual != expected {
                    return Err(format!(
                        "{} HTTP status {actual} != {expected}",
                        scenario.id
                    ));
                }
            }
            Some("no-horizontal-overflow") => {
                assert_no_horizontal_overflow(session, scenario)?;
            }
            None => return Err(format!("{} outcome omitted type", scenario.id)),
            Some(other) => return Err(format!("unsupported Rust E2E outcome {other}")),
        }
    }
    let screenshot = session.screenshot()?;
    let source = session.source()?;
    fs::write(output_root.join(format!("{}.png", scenario.id)), screenshot)
        .map_err(|error| format!("could not write screenshot: {error}"))?;
    fs::write(output_root.join(format!("{}.html", scenario.id)), source)
        .map_err(|error| format!("could not write page source: {error}"))?;
    Ok(())
}

fn wait_for_url_contract(
    session: &WebDriverSession<'_>,
    scenario: &Scenario,
    expected_path: &str,
    timeout: Duration,
) -> Result<String, String> {
    let start = Instant::now();
    let mut current_url = session.current_url()?;
    loop {
        let current =
            Url::parse(&current_url).map_err(|error| format!("invalid browser URL: {error}"))?;
        let query_matches = scenario.outcomes.iter().all(|outcome| {
            if action_type(outcome) != Some("query") || action_side(outcome) == Some("source") {
                return true;
            }
            let Some(key) = outcome.get("key").and_then(Value::as_str) else {
                return false;
            };
            let Some(value) = outcome.get("value").and_then(Value::as_str) else {
                return false;
            };
            current
                .query_pairs()
                .any(|(candidate, actual)| candidate == key && actual == value)
        });
        if current.path() == expected_path && query_matches {
            return Ok(current_url);
        }
        if start.elapsed() >= timeout {
            return Err(format!(
                "browser URL {} did not satisfy path/query contract for {} within {} seconds",
                current_url,
                scenario.id,
                timeout.as_secs()
            ));
        }
        thread::sleep(Duration::from_millis(100));
        current_url = session.current_url()?;
    }
}

fn target_path_outcome(scenario: &Scenario) -> Option<&str> {
    let mut paths = scenario.outcomes.iter().filter_map(|outcome| {
        (action_type(outcome) == Some("path") && action_side(outcome) != Some("source"))
            .then(|| outcome.get("value").and_then(Value::as_str))
            .flatten()
    });
    let path = paths.next()?;
    paths.next().is_none().then_some(path)
}

fn requested_path(path_and_query: &str) -> &str {
    path_and_query
        .split_once('?')
        .map(|(path, _)| path)
        .unwrap_or(path_and_query)
}

fn target_navigation_path<'a>(scenario: &'a Scenario, matrix_id: &str) -> Option<&'a str> {
    scenario.actions.iter().rev().find_map(|action| {
        (action_type(action) == Some("navigate")
            && action_side(action) != Some("source")
            && matrix_matches(action, matrix_id))
        .then(|| {
            action
                .get("path")
                .and_then(Value::as_str)
                .map(requested_path)
        })
        .flatten()
    })
}

fn action_selector(value: &Value) -> Result<&str, String> {
    value
        .get("selector")
        .and_then(Value::as_str)
        .filter(|selector| !selector.trim().is_empty())
        .ok_or("interactive contract omitted selector".into())
}

fn wait_for_elements(
    session: &WebDriverSession<'_>,
    selector: &str,
    timeout: Duration,
) -> Result<Vec<String>, String> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let elements = resolve_selector(session, selector)?;
        if !elements.is_empty() {
            return Ok(elements);
        }
        thread::sleep(Duration::from_millis(100));
    }
    Err(format!("timed out waiting for {selector}"))
}

fn wait_for_rendered_text(
    session: &WebDriverSession<'_>,
    text: &str,
    timeout: Duration,
) -> Result<(), String> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let body = session
            .find_elements("css selector", "body")?
            .into_iter()
            .next();
        if body.is_some_and(|body| {
            session
                .element_text(&body)
                .is_ok_and(|rendered| rendered.contains(text))
        }) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }
    Err(format!("timed out waiting for rendered text {text:?}"))
}

fn resolve_selector(session: &WebDriverSession<'_>, selector: &str) -> Result<Vec<String>, String> {
    let mut resolved = Vec::new();
    for branch in split_selector_branches(selector)? {
        let (branch, nth) = selector_nth(&branch)?;
        let visible = branch.contains(":visible");
        let branch = branch.replace(":visible", "");
        let (css, required_text) = selector_text_filter(&branch)?;
        let candidates = session.find_elements("css selector", &css)?;
        let mut matching = Vec::new();
        for element in candidates {
            if visible && !session.element_displayed(&element)? {
                continue;
            }
            if let Some(text) = required_text.as_deref() {
                if !session.element_text(&element)?.contains(text) {
                    continue;
                }
            }
            matching.push(element);
        }
        if let Some(index) = nth {
            if let Some(element) = matching.get(index) {
                resolved.push(element.clone());
            }
        } else {
            resolved.extend(matching);
        }
        if !resolved.is_empty() {
            break;
        }
    }
    Ok(resolved)
}

fn split_selector_branches(selector: &str) -> Result<Vec<String>, String> {
    let mut branches = Vec::new();
    let mut start = 0usize;
    let mut parentheses = 0usize;
    let mut brackets = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in selector.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if let Some(active) = quote {
            if character == active {
                quote = None;
            }
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '(' => parentheses += 1,
            ')' => {
                parentheses = parentheses
                    .checked_sub(1)
                    .ok_or("selector has an unmatched parenthesis")?
            }
            '[' => brackets += 1,
            ']' => {
                brackets = brackets
                    .checked_sub(1)
                    .ok_or("selector has an unmatched bracket")?
            }
            ',' if parentheses == 0 && brackets == 0 => {
                let branch = selector[start..index].trim();
                if branch.is_empty() {
                    return Err("selector contains an empty branch".into());
                }
                branches.push(branch.to_string());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    if quote.is_some() || parentheses != 0 || brackets != 0 {
        return Err("selector has an unterminated quote or group".into());
    }
    let branch = selector[start..].trim();
    if branch.is_empty() {
        return Err("selector is empty".into());
    }
    branches.push(branch.to_string());
    Ok(branches)
}

fn selector_nth(selector: &str) -> Result<(String, Option<usize>), String> {
    let Some((selector, index)) = selector.rsplit_once(" >> nth=") else {
        return Ok((selector.to_string(), None));
    };
    let index = index
        .parse::<usize>()
        .map_err(|_| "selector nth index is invalid")?;
    Ok((selector.trim().to_string(), Some(index)))
}

fn selector_text_filter(selector: &str) -> Result<(String, Option<String>), String> {
    if let Some(text) = selector.strip_prefix("text=") {
        let text = text.trim_matches(|character| matches!(character, '"' | '\''));
        if text.is_empty() {
            return Err("text selector is empty".into());
        }
        return Ok(("body *".into(), Some(text.to_string())));
    }
    let Some(start) = selector.find(":has-text(") else {
        return Ok((selector.to_string(), None));
    };
    let text_start = start + ":has-text(".len();
    let end = selector[text_start..]
        .find(')')
        .map(|offset| text_start + offset)
        .ok_or("has-text selector is unterminated")?;
    let text = selector[text_start..end]
        .trim()
        .trim_matches(|character| matches!(character, '"' | '\''));
    if text.is_empty() {
        return Err("has-text selector is empty".into());
    }
    let mut css = selector.to_string();
    css.replace_range(start..=end, "");
    if css.trim().is_empty() {
        css = "body *".into();
    }
    Ok((css, Some(text.to_string())))
}

fn set_input_file(
    session: &WebDriverSession<'_>,
    scenario: &Scenario,
    action: &Value,
    output_root: &Path,
) -> Result<(), String> {
    let name = action
        .get("name")
        .and_then(Value::as_str)
        .ok_or("set-input-files action omitted name")?;
    let name_path = Path::new(name);
    if name_path.components().count() != 1 || !safe_relative_path(name_path) {
        return Err("set-input-files action has an unsafe name".into());
    }
    action
        .get("mimeType")
        .and_then(Value::as_str)
        .filter(|mime| mime.contains('/') && !mime.bytes().any(|byte| byte.is_ascii_whitespace()))
        .ok_or("set-input-files action has an invalid or missing MIME type")?;
    let encoded = action
        .get("contentBase64")
        .and_then(Value::as_str)
        .ok_or("set-input-files action omitted contentBase64")?;
    let bytes = BASE64
        .decode(encoded)
        .map_err(|error| format!("set-input-files content is invalid base64: {error}"))?;
    let upload_root = output_root.join("uploads");
    fs::create_dir_all(&upload_root)
        .map_err(|error| format!("could not create {}: {error}", upload_root.display()))?;
    let path = upload_root.join(format!("{}-{name}", scenario.id));
    fs::write(&path, bytes)
        .map_err(|error| format!("could not write {}: {error}", path.display()))?;
    let element = wait_for_elements(session, action_selector(action)?, Duration::from_secs(10))?
        .into_iter()
        .next()
        .ok_or("set-input-files action resolved no element")?;
    session.send_keys(
        &element,
        path.to_str().ok_or("upload path is not valid UTF-8")?,
    )
}

fn assert_no_horizontal_overflow(
    session: &WebDriverSession<'_>,
    scenario: &Scenario,
) -> Result<(), String> {
    let start = Instant::now();
    loop {
        match assert_no_horizontal_overflow_once(session, scenario) {
            Err(error)
                if (error.contains("stale element reference")
                    || error.contains("document has no html element"))
                    && start.elapsed() < Duration::from_secs(10) =>
            {
                thread::sleep(Duration::from_millis(100));
            }
            result => return result,
        }
    }
}

fn assert_no_horizontal_overflow_once(
    session: &WebDriverSession<'_>,
    scenario: &Scenario,
) -> Result<(), String> {
    let html = session
        .find_elements("css selector", "html")?
        .into_iter()
        .next()
        .ok_or("document has no html element")?;
    let viewport = session.element_rect(&html)?.width;
    if viewport <= 0.0 {
        return Err("document viewport width is not positive".into());
    }
    for element in session.find_elements("css selector", "body, body > *")? {
        if !session.element_displayed(&element)? {
            continue;
        }
        let rect = session.element_rect(&element)?;
        if rect.x < -1.0 || rect.x + rect.width > viewport + 1.0 {
            return Err(format!(
                "{} horizontal overflow: x={} width={} viewport={viewport}",
                scenario.id, rect.x, rect.width
            ));
        }
    }
    Ok(())
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

fn contains_node_command(contents: &str) -> bool {
    contents
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .any(|line| {
            let line = line.to_ascii_lowercase();
            if line.contains("setup-node") || line.contains("setup-bun") {
                return true;
            }
            line.split(|character: char| {
                !(character.is_ascii_alphanumeric()
                    || matches!(character, '-' | '_' | '.' | '/' | '@'))
            })
            .filter(|token| !token.is_empty())
            .filter_map(|token| token.rsplit('/').next())
            .any(|token| ACTIVE_NODE_COMMANDS.contains(&token))
        })
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
    use super::{
        contains_node_command, environment_name, is_hex, matrix_matches,
        require_runtime_state_provisioning, safe_relative_path, selector_nth, selector_text_filter,
        split_selector_branches, Scenario, ScenarioGroup,
    };
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
    fn webdriver_selector_contract_handles_playwright_compatibility_tokens() {
        let branches = split_selector_branches(
            r#"form:has(input[value="a,b"]), h2:has-text("Active Plans"):visible"#,
        )
        .unwrap();
        assert_eq!(branches.len(), 2);
        let (selector, nth) = selector_nth("text=Plan not found >> nth=1").unwrap();
        assert_eq!(selector, "text=Plan not found");
        assert_eq!(nth, Some(1));
        assert_eq!(
            selector_text_filter(r#"h2:has-text("Active Plans")"#).unwrap(),
            ("h2".to_string(), Some("Active Plans".to_string()))
        );
        assert_eq!(
            selector_text_filter("text=Total Credits Outstanding").unwrap(),
            (
                "body *".to_string(),
                Some("Total Credits Outstanding".to_string())
            )
        );
    }

    #[test]
    fn runtime_refuses_unprovisioned_authenticated_scenarios() {
        let scenario = |id: &str, state| Scenario {
            id: id.to_string(),
            surface: "frontend".into(),
            path: "/".into(),
            expected_target_path: None,
            actions: Vec::new(),
            outcomes: Vec::new(),
            state,
            fixture_requirements: Vec::new(),
        };
        let signed_out = ScenarioGroup {
            id: 0,
            slug: "signed-out".into(),
            matrix: "test".into(),
            repeat: 1,
            scenarios: vec![scenario(
                "public",
                json!({"id":"public","session":"signed-out"}),
            )],
        };
        assert!(require_runtime_state_provisioning(&signed_out, None).is_ok());

        let authenticated = ScenarioGroup {
            id: 1,
            slug: "authenticated".into(),
            matrix: "test".into(),
            repeat: 1,
            scenarios: vec![scenario(
                "owner",
                json!({
                    "id":"owner",
                    "session":"authenticated",
                    "audience":"epsx-frontend",
                    "permissions":[]
                }),
            )],
        };
        assert!(require_runtime_state_provisioning(&authenticated, None)
            .unwrap_err()
            .contains("refusing an unprovisioned false-positive run"));
    }

    #[test]
    fn default_environment_is_supported() {
        assert!(matches!(
            environment_name().as_str(),
            "development" | "staging" | "production"
        ));
    }

    #[test]
    fn active_command_audit_uses_exact_runtime_tokens() {
        assert!(contains_node_command("run: node -p version"));
        assert!(contains_node_command("uses: actions/setup-node@v4"));
        assert!(!contains_node_command(
            "run: cargo xtask audit no-node --strict"
        ));
        assert!(!contains_node_command("nodePort: 30080"));
    }
}
