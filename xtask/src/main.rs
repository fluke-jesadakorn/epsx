//! Repository maintenance commands for the Dioxus/Rust migration.
//!
//! This crate intentionally has no runtime dependencies. It is the first
//! migration tool that can run after the Bun/Node toolchain is removed.

mod node_free;
mod workspace_tools;

use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    env,
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const JS_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs", "cjs"];
const RUST_ROOTS: &[&str] = &["apps", "services", "shared/rust", "xtask"];
const NOTIFICATION_LIFECYCLE_TABLES: &[&str] = &[
    "notification_template_versions",
    "notification_preferences",
    "notification_inbox",
    "notification_outbox",
    "notification_channel_jobs",
    "notification_delivery_attempts",
    "notification_dead_letters",
    "notification_replay_cursors",
    "notification_push_subscriptions",
    "notification_request_idempotency",
    "notification_provider_events",
    "notification_engagement",
    "notification_template_audit",
    "notification_expirations",
];
const NOTIFICATION_AUTHORITY_SURFACES: &[&str] = &[
    "owner-list",
    "owner-unread-count",
    "owner-read",
    "owner-unread",
    "owner-delete",
    "owner-mark-all",
    "owner-clear-all",
    "admin-list",
    "admin-template-list",
    "admin-template-write",
    "admin-template-delete",
    "admin-send",
];
const NOTIFICATION_MIGRATIONS: &[(&str, &str)] = &[
    (
        "20260722040000",
        "apps/backend/migrations/notifications/20260722040000_create_notification_service_tables",
    ),
    (
        "20260723120000",
        "apps/backend/migrations/notifications/20260723120000_add_notification_lifecycle_foundation",
    ),
    (
        "20260723130000",
        "apps/backend/migrations/notifications/20260723130000_add_notification_idempotency_provider_events",
    ),
    (
        "20260723140000",
        "apps/backend/migrations/notifications/20260723140000_add_notification_lifecycle_constraints",
    ),
    (
        "20260724120000",
        "apps/backend/migrations/notifications/20260724120000_add_notification_template_audit",
    ),
    (
        "20260724130000",
        "apps/backend/migrations/notifications/20260724130000_add_notification_engagement_acknowledged",
    ),
    (
        "20260724140000",
        "apps/backend/migrations/notifications/20260724140000_add_notification_expirations",
    ),
    (
        "20260724150000",
        "apps/backend/migrations/notifications/20260724150000_add_notification_vapid_key_lineage",
    ),
];
const EMBEDDED_MARKERS: &[&str] = &[
    "<script",
    "onclick=",
    "onerror=",
    "innerHTML",
    "insertAdjacentHTML",
    "document.write",
    "eval(",
    "fetch(",
    "EventSource",
    "window.",
    "document.",
];
const APPROVED_EMBEDDED_RUNTIME_FILES: &[(&str, &str)] = &[
    (
        "shared/rust/browser-runtime/src/lib.rs",
        "rust-wasm-browser-runtime",
    ),
    (
        "shared/rust/service-worker/src/lib.rs",
        "rust-wasm-service-worker",
    ),
    (
        "shared/rust/templates/src/lib.rs",
        "wasm-bootstrap-script-tag",
    ),
];

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MigrationBaseline {
    schema_version: u64,
    source_ref: String,
    source_sha: String,
    target_branch: String,
    target_sha: String,
    evidence_sha: String,
    readiness_level: String,
    staging_ready: bool,
    production_ready: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RustEmbeddedRuntimeReview {
    schema_version: u64,
    hash_algorithm: String,
    inventory_sha256: String,
    approved_runtime_files: Vec<ApprovedEmbeddedRuntimeFile>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApprovedEmbeddedRuntimeFile {
    path: String,
    category: String,
    sha256: String,
    reason: String,
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".to_string());
    let flags: Vec<String> = args.collect();

    let result = match command.as_str() {
        "rust-audit" => rust_audit(&flags),
        "migration-audit" => migration_audit(&flags),
        "authority-audit" => authority_audit(&flags),
        "notification-compatibility-audit" => notification_compatibility_audit(&flags),
        "notification-producer-audit" => notification_producer_audit(&flags),
        "notification-backfill" => notification_backfill(&flags),
        "notification-reconcile" => notification_reconcile(&flags),
        "notification-readiness" => notification_readiness(&flags),
        "notification-privacy-audit" => notification_privacy_audit(&flags),
        "notification-migration-audit" => notification_migration_audit(&flags),
        "sync-audit" => sync_audit(),
        "k8s-audit" => k8s_audit(&flags),
        "audit" => node_free::audit(&flags),
        "e2e" => node_free::e2e(&flags),
        "env" => node_free::env_command(&flags),
        "setup-local" => node_free::setup_local(&flags),
        "dev" => node_free::dev(&flags),
        "build" => node_free::build(&flags),
        "browser-runtime" => node_free::browser_runtime(&flags),
        "test" => node_free::test(&flags),
        "anvil-proxy" => workspace_tools::anvil_proxy(&flags),
        "assets" => workspace_tools::assets(&flags),
        "fixtures" => workspace_tools::fixtures(&flags),
        "design" => node_free::design(&flags),
        "verify" => rust_audit(&["--strict".to_string()]),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        unknown => Err(format!("unknown xtask command {unknown}")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cargo xtask: ERROR: {error}");
            ExitCode::from(1)
        }
    }
}

fn print_help() {
    println!(
        "\
cargo xtask commands:
  audit no-node [--strict]
                         reject tracked Node/JS sources, tooling, active commands, and inline runtimes
  e2e doctor [--group 0..9]
  e2e run --group 0..9 [--webdriver-url URL] [--browser chromium|firefox|safari]
  e2e report [--group 0..9]
  e2e verify-artifacts [--group 0..9]
                         Rust-native migration scenario and evidence commands
  env validate          validate the merged root environment against .env.example
  setup-local           deploy the local Foundry contracts and tokens
  dev --all|--frontend|--admin|--backend
                         run the Rust/Dioxus development surface
  build --profile development|production
  browser-runtime build  compile Rust/WASM and emit untracked wasm-bindgen browser assets
  test --all            run the Rust workspace test suite
  anvil-proxy [--listen ADDR] [--upstream ADDR] [--no-spawn]
                         run Anvil and its Rust HTTP/RPC proxy
  assets verify         verify frozen CSS and committed public assets
  fixtures serve [--root PATH] [--bind LOOPBACK_ADDR]
                         serve local test fixtures without a script runtime
  design capture --group 0..9 [Rust E2E flags]
                         capture design evidence through the Rust WebDriver harness
  rust-audit [--strict]  inventory tracked JS/TS and embedded runtime markers
  migration-audit [--strict] detect colliding versions and destructive SQL
  authority-audit [--strict] verify the notification authority/compatibility matrix
  notification-compatibility-audit [--strict]
                         verify fixture method/path registrations in Rust BFF/admin/service routers
  notification-producer-audit [--strict]
                         verify migrated backend producers use stable NotificationPort event identities
  notification-backfill --dry-run --input <jsonl> [--legacy] [--after <source-event-id>]
                         validate target or explicitly mapped legacy notification records without writes
  notification-reconcile --dry-run --source <jsonl> --target <jsonl>
                         compare bounded source/target records without writes
  notification-readiness --dry-run --input <metrics-json>
                         evaluate a redacted metrics snapshot against N7 thresholds
  notification-privacy-audit [--strict]
                         validate the no-write retention/legal-hold/erasure policy contract
  notification-migration-audit [--strict]
                         verify notification migration chain and ledger evidence
  sync-audit             prove development is a reference ancestor only
  k8s-audit [--strict]   render overlays and enforce environment service inventories
  verify                 run the strict Rust-only audit
"
    );
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NotificationPrivacyPolicy {
    #[serde(rename = "schemaVersion")]
    schema_version: u64,
    #[serde(rename = "contractId")]
    contract_id: String,
    #[serde(rename = "productionReady")]
    production_ready: bool,
    scope: String,
    #[serde(rename = "legalHold")]
    legal_hold: NotificationLegalHold,
    erasure: NotificationErasure,
    channels: BTreeMap<String, NotificationChannelPrivacy>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NotificationLegalHold {
    required: bool,
    #[serde(rename = "activeHoldBlocksPurge")]
    active_hold_blocks_purge: bool,
    release: String,
    #[serde(rename = "payloadLogging")]
    payload_logging: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NotificationErasure {
    #[serde(rename = "ownerIdentity")]
    owner_identity: String,
    #[serde(rename = "ownerRows")]
    owner_rows: String,
    #[serde(rename = "broadcastRows")]
    broadcast_rows: String,
    #[serde(rename = "inFlightJobs")]
    in_flight_jobs: String,
    audit: String,
    #[serde(rename = "runtimeProof")]
    runtime_proof: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NotificationChannelPrivacy {
    #[serde(rename = "payloadRetentionDays")]
    payload_retention_days: u32,
    #[serde(rename = "providerIdentityRetentionDays")]
    provider_identity_retention_days: u32,
    #[serde(rename = "terminalStates")]
    terminal_states: Vec<String>,
    #[serde(rename = "recipientClass")]
    recipient_class: String,
    #[serde(rename = "pendingRule")]
    pending_rule: String,
}

fn notification_privacy_audit(flags: &[String]) -> Result<(), String> {
    let strict = flags.iter().any(|flag| flag == "--strict");
    if flags
        .iter()
        .any(|flag| flag != "--strict" && flag != "--report")
    {
        return Err("notification-privacy-audit accepts only --strict and --report".into());
    }
    let root = repo_root()?;
    let path = root.join("docs/migration/contracts/notification-privacy-policy.json");
    let policy: NotificationPrivacyPolicy = serde_json::from_str(
        &std::fs::read_to_string(&path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?,
    )
    .map_err(|error| format!("notification privacy policy is invalid: {error}"))?;
    let expected_scope = "reviewed policy contract only; runtime purge, legal-hold, and account-erasure execution remain separately gated";
    let expected_states = BTreeSet::from([
        "provider_accepted".to_string(),
        "terminal_failed".to_string(),
        "dead_lettered".to_string(),
        "expired".to_string(),
        "deleted".to_string(),
    ]);
    let expected_channels = BTreeSet::from([
        "email".to_string(),
        "in_app".to_string(),
        "push".to_string(),
    ]);
    let channel_ids = policy.channels.keys().cloned().collect::<BTreeSet<_>>();
    let channels_ok = channel_ids == expected_channels
        && policy.channels.values().all(|channel| {
            (1..=3650).contains(&channel.payload_retention_days)
                && channel.provider_identity_retention_days <= 3650
                && channel.terminal_states.iter().collect::<BTreeSet<_>>()
                    == expected_states.iter().collect::<BTreeSet<_>>()
                && !channel.recipient_class.trim().is_empty()
                && channel.pending_rule.contains("expiry")
                && channel.pending_rule.contains("terminal")
        });
    let sentinels_ok = policy.schema_version == 1
        && policy.contract_id == "A11.7-notification-privacy"
        && !policy.production_ready
        && policy.scope == expected_scope
        && policy.legal_hold.required
        && policy.legal_hold.active_hold_blocks_purge
        && policy.legal_hold.release.contains("explicit operator")
        && policy.legal_hold.payload_logging.contains("never log")
        && policy
            .erasure
            .owner_identity
            .contains("canonical lowercase EVM")
        && policy.erasure.owner_rows.contains("dependent jobs")
        && policy.erasure.broadcast_rows.contains("engagement")
        && policy.erasure.in_flight_jobs.contains("cancel")
        && policy.erasure.audit.contains("redacted")
        && policy.erasure.runtime_proof == "required-before-production";
    println!(
        "notification-privacy-audit: policy={} channels={} legal_hold={} erasure={} writes=0 network=0 database=0",
        if sentinels_ok { "pass" } else { "fail" },
        if channels_ok { "pass" } else { "fail" },
        policy.legal_hold.active_hold_blocks_purge,
        if sentinels_ok { "explicit" } else { "incomplete" }
    );
    if strict && !(sentinels_ok && channels_ok) {
        return Err("notification privacy policy contract is incomplete".into());
    }
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct BackfillRecord {
    source_event_id: String,
    wallet_address: String,
    status: String,
}

/// The legacy `wallet_notifications` row shape is intentionally accepted only
/// through the explicit `--legacy` dry-run mode. It is never written directly
/// to the target schema: the mapper below rejects topic-only rows and unknown
/// lifecycle states instead of guessing at a source-to-target meaning.
#[derive(Debug, serde::Deserialize)]
struct LegacyBackfillRecord {
    id: String,
    #[serde(default)]
    recipient_wallet_address: Option<String>,
    #[serde(default)]
    topic_name: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    notification_type: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    channels: Option<serde_json::Value>,
    #[serde(default)]
    action_url: Option<String>,
    #[serde(default)]
    data_payload: Option<serde_json::Value>,
    #[serde(default)]
    expires_at: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    status: String,
}

fn legacy_status(status: &str) -> Option<&'static str> {
    match status.trim().to_ascii_lowercase().as_str() {
        "created" | "queued" | "scheduled" => Some("pending"),
        "sent" | "delivered" | "read" => Some("sent"),
        "failed" => Some("failed"),
        "suppressed" => Some("suppressed"),
        "cancelled" => Some("cancelled"),
        "expired" => Some("expired"),
        "deleted" => Some("deleted"),
        _ => None,
    }
}

fn valid_uuid_text(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

fn normalize_legacy_wallet(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.eq_ignore_ascii_case("all") {
        return Some("all".to_string());
    }
    if value.len() != 42
        || !(value.starts_with("0x") || value.starts_with("0X"))
        || !value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    Some(format!("0x{}", value[2..].to_ascii_lowercase()))
}

fn valid_legacy_text(value: Option<&String>, max_bytes: usize, allow_empty: bool) -> bool {
    value.is_none_or(|value| {
        value.len() <= max_bytes
            && (allow_empty || !value.is_empty())
            && !value.chars().any(char::is_control)
    })
}

fn valid_legacy_json(value: Option<&serde_json::Value>) -> bool {
    value.is_none_or(|value| {
        value.is_object()
            && serde_json::to_vec(value).is_ok_and(|encoded| encoded.len() <= 64 * 1024)
    })
}

fn valid_legacy_timestamp(value: Option<&String>) -> bool {
    value.is_none_or(|value| chrono::DateTime::parse_from_rfc3339(value).is_ok())
}

fn map_legacy_backfill_record(line: &str) -> Result<(BackfillRecord, usize), ()> {
    let legacy: LegacyBackfillRecord = serde_json::from_str(line).map_err(|_| ())?;
    if !valid_uuid_text(&legacy.id)
        || legacy
            .topic_name
            .as_deref()
            .is_some_and(|topic| !topic.trim().is_empty())
        || !valid_legacy_text(legacy.title.as_ref(), 512, false)
        || !valid_legacy_text(legacy.body.as_ref(), 16 * 1024, false)
        || !valid_legacy_text(legacy.notification_type.as_ref(), 50, false)
        || !valid_legacy_text(legacy.priority.as_ref(), 20, false)
        || !valid_legacy_json(legacy.channels.as_ref())
        || !valid_legacy_text(legacy.action_url.as_ref(), 2 * 1024, false)
        || !valid_legacy_json(legacy.data_payload.as_ref())
        || !valid_legacy_timestamp(legacy.expires_at.as_ref())
        || !valid_legacy_timestamp(legacy.created_at.as_ref())
    {
        return Err(());
    }
    let wallet = normalize_legacy_wallet(legacy.recipient_wallet_address.as_deref()).ok_or(())?;
    let status = legacy_status(&legacy.status).ok_or(())?;
    let preserved_fields = [
        legacy.recipient_wallet_address.is_some(),
        legacy.title.is_some(),
        legacy.body.is_some(),
        legacy.notification_type.is_some(),
        legacy.priority.is_some(),
        legacy.channels.is_some(),
        legacy.action_url.is_some(),
        legacy.data_payload.is_some(),
        legacy.expires_at.is_some(),
        legacy.created_at.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    Ok((
        BackfillRecord {
            source_event_id: format!("legacy.wallet_notification:{}", legacy.id),
            wallet_address: wallet,
            status: status.to_string(),
        },
        preserved_fields,
    ))
}

fn notification_backfill(flags: &[String]) -> Result<(), String> {
    let dry_run = flags.iter().any(|flag| flag == "--dry-run");
    if !dry_run {
        return Err("notification-backfill requires --dry-run; writes are not implemented".into());
    }
    let mut input = None;
    let mut after = None;
    let mut legacy = false;
    let mut index = 0;
    while index < flags.len() {
        match flags[index].as_str() {
            "--dry-run" => index += 1,
            "--input" => {
                input = flags.get(index + 1).cloned();
                index += 2;
            }
            "--after" => {
                after = flags.get(index + 1).cloned();
                index += 2;
            }
            "--legacy" => {
                legacy = true;
                index += 1;
            }
            "--report" => index += 1,
            other => return Err(format!("notification-backfill unknown flag {other}")),
        }
    }
    let root = repo_root()?;
    let input = input.ok_or("notification-backfill requires --input <jsonl>")?;
    let path = root.join(input);
    let file =
        File::open(&path).map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let reader = BufReader::new(file);
    let mut seen = HashSet::new();
    let mut records = 0usize;
    let mut eligible = 0usize;
    let mut invalid = 0usize;
    let mut duplicate = 0usize;
    let mut legacy_records = 0usize;
    let mut legacy_fields_preserved = 0usize;
    let mut mapped_statuses = BTreeMap::<String, usize>::new();
    let mut checkpoint_seen = after.is_none();
    for (line_number, line) in reader.lines().enumerate() {
        if records >= 100_000 {
            return Err("notification-backfill input exceeds the 100000-record bound".into());
        }
        let line =
            line.map_err(|error| format!("could not read line {}: {error}", line_number + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        records += 1;
        let record: BackfillRecord = match if legacy {
            legacy_records += 1;
            map_legacy_backfill_record(&line).map(|(record, preserved_fields)| {
                legacy_fields_preserved += preserved_fields;
                record
            })
        } else {
            serde_json::from_str(&line).map_err(|_| ())
        } {
            Ok(record) => record,
            Err(_) => {
                invalid += 1;
                continue;
            }
        };
        if legacy {
            *mapped_statuses.entry(record.status.clone()).or_insert(0) += 1;
        }
        if !seen.insert(record.source_event_id.clone()) {
            duplicate += 1;
        }
        if after.as_deref() == Some(record.source_event_id.as_str()) {
            checkpoint_seen = true;
            continue;
        }
        if !checkpoint_seen {
            continue;
        }
        if (valid_wallet(&record.wallet_address) || (legacy && record.wallet_address == "all"))
            && matches!(
                record.status.as_str(),
                "pending" | "sent" | "failed" | "suppressed" | "cancelled" | "expired" | "deleted"
            )
            && !record.source_event_id.trim().is_empty()
            && record.source_event_id.len() <= 128
        {
            eligible += 1;
        } else {
            invalid += 1;
        }
    }
    if !checkpoint_seen {
        return Err("notification-backfill checkpoint was not found in the input".into());
    }
    if legacy {
        let mapped_statuses = serde_json::to_string(&mapped_statuses)
            .map_err(|error| format!("could not encode legacy status mapping: {error}"))?;
        println!("notification-backfill: mode=dry-run format=legacy records={records} eligible={eligible} invalid={invalid} duplicate_source_events={duplicate} legacy_records={legacy_records} legacy_fields_preserved={legacy_fields_preserved} mapped_statuses={mapped_statuses}");
    } else {
        println!("notification-backfill: mode=dry-run records={records} eligible={eligible} invalid={invalid} duplicate_source_events={duplicate}");
    }
    println!(
        "notification-backfill: writes=0 network=0 database=0 checkpoint={}",
        after.as_deref().unwrap_or("start")
    );
    if invalid > 0 || duplicate > 0 {
        return Err("notification-backfill dry-run found invalid or duplicate records".into());
    }
    Ok(())
}

fn valid_wallet(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ReconcileRecord {
    source_event_id: String,
    wallet_address: String,
    status: String,
    #[serde(default)]
    provider_message_id: Option<String>,
    #[serde(default)]
    provider_event_id: Option<String>,
    #[serde(default)]
    template_id: Option<String>,
    #[serde(default)]
    preference_hash: Option<String>,
    #[serde(default)]
    broadcast: bool,
}

#[derive(Debug, serde::Serialize)]
struct ReconciliationReport {
    source_records: usize,
    target_records: usize,
    invalid_source_records: usize,
    invalid_target_records: usize,
    duplicate_source_events: usize,
    duplicate_target_events: usize,
    missing_target_events: usize,
    orphan_target_events: usize,
    source_status_distribution: BTreeMap<String, usize>,
    target_status_distribution: BTreeMap<String, usize>,
    source_broadcast_records: usize,
    target_broadcast_records: usize,
    target_records_without_provider_id: usize,
    target_sent_without_provider_id: usize,
    template_identity_drift: usize,
    preference_identity_drift: usize,
    provider_identity_drift: usize,
    source_wallet_checksum: String,
    target_wallet_checksum: String,
    wallet_checksum_match: bool,
    source_target_event_set_match: bool,
    status_distribution_match: bool,
    broadcast_count_match: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NotificationMetricsSnapshot {
    queue_depth: i64,
    queue_age_seconds: Option<i64>,
    suppressed: i64,
    retry_wait: i64,
    terminal_failed: i64,
    dead_lettered: i64,
    provider_accepted: i64,
    attempting: i64,
    channel_outcomes: BTreeMap<String, i64>,
    provider_events: i64,
    delivery_attempts: i64,
    replay_cursors: i64,
    replay_cursor_age_seconds: Option<i64>,
    active_streams: u64,
    stream_connections_total: u64,
    stream_reconnects_total: u64,
    stream_replayed_events_total: u64,
    stream_lag_seconds: Option<u64>,
    stream_query_failures_total: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NotificationReadinessContract {
    #[serde(rename = "schemaVersion")]
    schema_version: u64,
    #[serde(rename = "contractId")]
    contract_id: String,
    #[serde(rename = "productionReady")]
    production_ready: bool,
    scope: String,
    thresholds: NotificationReadinessThresholds,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NotificationReadinessThresholds {
    max_queue_depth: i64,
    max_queue_age_seconds: i64,
    max_retry_wait: i64,
    max_terminal_failed: i64,
    max_dead_lettered: i64,
    max_stream_lag_seconds: u64,
    max_stream_query_failures: u64,
    max_active_streams: u64,
    max_replayed_events_per_connection: u64,
    min_provider_acceptance_percent: u64,
}

#[derive(Debug, serde::Serialize)]
struct NotificationReadinessReport {
    production_ready: bool,
    healthy: bool,
    checks: BTreeMap<String, bool>,
    writes: u8,
    network: u8,
    database: u8,
}

fn notification_readiness(flags: &[String]) -> Result<(), String> {
    if !flags.iter().any(|flag| flag == "--dry-run") {
        return Err("notification-readiness requires --dry-run; writes are not implemented".into());
    }
    let mut input = None;
    let mut index = 0;
    while index < flags.len() {
        match flags[index].as_str() {
            "--dry-run" => index += 1,
            "--input" => {
                input = flags.get(index + 1).cloned();
                index += 2;
            }
            other => return Err(format!("notification-readiness unknown flag {other}")),
        }
    }
    let root = repo_root()?;
    let input_path =
        root.join(input.ok_or("notification-readiness requires --input <metrics-json>")?);
    let metrics: NotificationMetricsSnapshot = serde_json::from_str(
        &std::fs::read_to_string(&input_path)
            .map_err(|error| format!("could not read {}: {error}", input_path.display()))?,
    )
    .map_err(|error| format!("notification metrics snapshot is invalid: {error}"))?;
    let contract_path = root.join("docs/migration/contracts/notification-readiness.json");
    let contract: NotificationReadinessContract = serde_json::from_str(
        &std::fs::read_to_string(&contract_path)
            .map_err(|error| format!("could not read {}: {error}", contract_path.display()))?,
    )
    .map_err(|error| format!("notification readiness contract is invalid: {error}"))?;
    if contract.schema_version != 1
        || contract.contract_id != "A11.7-notification-readiness"
        || contract.production_ready
        || contract.scope != "offline redacted metrics evaluation only; no database, network, provider, or deployment access"
    {
        return Err("notification readiness contract sentinel changed".into());
    }
    let non_negative = [
        metrics.queue_depth,
        metrics.suppressed,
        metrics.retry_wait,
        metrics.terminal_failed,
        metrics.dead_lettered,
        metrics.provider_accepted,
        metrics.attempting,
        metrics.provider_events,
        metrics.delivery_attempts,
        metrics.replay_cursors,
    ]
    .iter()
    .all(|value| *value >= 0);
    let queue_age_ok = metrics.queue_depth == 0
        || metrics
            .queue_age_seconds
            .is_some_and(|age| age >= 0 && age <= contract.thresholds.max_queue_age_seconds);
    let replay_age_ok = metrics
        .replay_cursor_age_seconds
        .is_none_or(|age| age >= 0 && age <= contract.thresholds.max_queue_age_seconds);
    let provider_acceptance_ok = metrics.delivery_attempts == 0
        || (metrics.provider_accepted >= 0
            && (metrics.provider_accepted as u128 * 100)
                >= (metrics.delivery_attempts as u128
                    * contract.thresholds.min_provider_acceptance_percent as u128));
    let channel_values_ok = metrics.channel_outcomes.values().all(|value| *value >= 0);
    let stream_counters_ok = metrics.active_streams <= contract.thresholds.max_active_streams
        && metrics.active_streams <= metrics.stream_connections_total
        && metrics.stream_reconnects_total <= metrics.stream_connections_total
        && metrics.stream_replayed_events_total
            <= metrics
                .stream_connections_total
                .saturating_add(metrics.stream_reconnects_total)
                .saturating_mul(contract.thresholds.max_replayed_events_per_connection);
    let mut checks = BTreeMap::new();
    checks.insert(
        "non_negative_counters".to_string(),
        non_negative && channel_values_ok,
    );
    checks.insert(
        "queue_depth".to_string(),
        metrics.queue_depth >= 0 && metrics.queue_depth <= contract.thresholds.max_queue_depth,
    );
    checks.insert("queue_age".to_string(), queue_age_ok);
    checks.insert(
        "retry_wait".to_string(),
        metrics.retry_wait >= 0 && metrics.retry_wait <= contract.thresholds.max_retry_wait,
    );
    checks.insert(
        "terminal_failed".to_string(),
        metrics.terminal_failed >= 0
            && metrics.terminal_failed <= contract.thresholds.max_terminal_failed,
    );
    checks.insert(
        "dead_lettered".to_string(),
        metrics.dead_lettered >= 0
            && metrics.dead_lettered <= contract.thresholds.max_dead_lettered,
    );
    checks.insert("provider_acceptance".to_string(), provider_acceptance_ok);
    checks.insert("stream_counters".to_string(), stream_counters_ok);
    checks.insert("replay_cursor_age".to_string(), replay_age_ok);
    checks.insert(
        "stream_lag".to_string(),
        metrics
            .stream_lag_seconds
            .is_none_or(|lag| lag <= contract.thresholds.max_stream_lag_seconds),
    );
    checks.insert(
        "stream_query_failures".to_string(),
        metrics.stream_query_failures_total <= contract.thresholds.max_stream_query_failures,
    );
    let healthy = checks.values().all(|check| *check);
    let report = NotificationReadinessReport {
        production_ready: false,
        healthy,
        checks,
        writes: 0,
        network: 0,
        database: 0,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("could not encode readiness report: {error}"))?
    );
    println!(
        "notification-readiness: healthy={} production_ready=false writes=0 network=0 database=0",
        report.healthy
    );
    if !report.healthy {
        return Err("notification readiness thresholds failed".into());
    }
    Ok(())
}

fn notification_reconcile(flags: &[String]) -> Result<(), String> {
    if !flags.iter().any(|flag| flag == "--dry-run") {
        return Err("notification-reconcile requires --dry-run; writes are not implemented".into());
    }
    let mut source = None;
    let mut target = None;
    let mut index = 0;
    while index < flags.len() {
        match flags[index].as_str() {
            "--dry-run" => index += 1,
            "--source" => {
                source = flags.get(index + 1).cloned();
                index += 2;
            }
            "--target" => {
                target = flags.get(index + 1).cloned();
                index += 2;
            }
            other => return Err(format!("notification-reconcile unknown flag {other}")),
        }
    }
    let root = repo_root()?;
    let source_path = root.join(source.ok_or("notification-reconcile requires --source <jsonl>")?);
    let target_path = root.join(target.ok_or("notification-reconcile requires --target <jsonl>")?);
    let source = read_reconcile_records(&source_path)?;
    let target = read_reconcile_records(&target_path)?;
    let report = build_reconciliation_report(&source, &target);
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("could not encode reconciliation report: {error}"))?
    );
    println!("notification-reconcile: writes=0 network=0 database=0");
    if report.invalid_source_records > 0
        || report.invalid_target_records > 0
        || report.duplicate_source_events > 0
        || report.duplicate_target_events > 0
        || report.missing_target_events > 0
        || report.orphan_target_events > 0
        || !report.wallet_checksum_match
        || !report.source_target_event_set_match
        || !report.status_distribution_match
        || !report.broadcast_count_match
        || report.template_identity_drift > 0
        || report.preference_identity_drift > 0
        || report.provider_identity_drift > 0
        || report.target_sent_without_provider_id > 0
    {
        return Err("notification-reconcile found source/target drift".into());
    }
    Ok(())
}

fn notification_migration_audit(flags: &[String]) -> Result<(), String> {
    let strict = flags.iter().any(|flag| flag == "--strict");
    if flags
        .iter()
        .any(|flag| flag != "--strict" && flag != "--report")
    {
        return Err("notification-migration-audit accepts only --strict and --report".into());
    }
    let root = repo_root()?;
    let ledger_path = root.join("docs/migration/contracts/notification-migration-ledger.json");
    let ledger: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&ledger_path)
            .map_err(|error| format!("could not read {}: {error}", ledger_path.display()))?,
    )
    .map_err(|error| format!("notification migration ledger is invalid JSON: {error}"))?;
    let ledger_migrations = ledger
        .get("migrations")
        .and_then(serde_json::Value::as_array)
        .ok_or("notification migration ledger has no migrations array")?;
    let mut static_ok = ledger_migrations.len() == NOTIFICATION_MIGRATIONS.len();
    for (expected_version, relative_dir) in NOTIFICATION_MIGRATIONS {
        let directory = root.join(relative_dir);
        let up = directory.join("up.sql");
        let down = directory.join("down.sql");
        let expected_files = up.is_file() && down.is_file();
        let up_checksum = sha256_file(&up);
        let down_checksum = sha256_file(&down);
        let safe_sql = [up, down]
            .into_iter()
            .filter_map(|path| std::fs::read_to_string(path).ok())
            .all(|sql| !contains_destructive_sql(&sql));
        let listed = ledger_migrations.iter().any(|entry| {
            entry.get("version").and_then(serde_json::Value::as_str) == Some(*expected_version)
                && entry.get("directory").and_then(serde_json::Value::as_str) == Some(*relative_dir)
                && entry.get("upSha256").and_then(serde_json::Value::as_str)
                    == up_checksum.as_deref()
                && entry.get("downSha256").and_then(serde_json::Value::as_str)
                    == down_checksum.as_deref()
        });
        println!(
            "notification-migration-audit: version={} files={} sql_safe={} checksums={} ledger_entry={}",
            expected_version,
            expected_files,
            safe_sql,
            up_checksum.is_some() && down_checksum.is_some(),
            listed
        );
        static_ok &= expected_files && safe_sql && listed;
    }
    let database_evidence = ledger
        .get("databaseLedgerEvidence")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("missing");
    let mut database_evidence_ok = database_evidence == "missing";
    if database_evidence == "verified" {
        let evidence_file = ledger
            .get("databaseLedgerEvidenceFile")
            .and_then(serde_json::Value::as_str)
            .ok_or("verified notification migration evidence has no evidence file")?;
        let evidence_path = Path::new(evidence_file);
        if evidence_path.is_absolute()
            || evidence_path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err("notification migration evidence path is unsafe".into());
        }
        let evidence_path = root.join(evidence_path);
        let evidence: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&evidence_path).map_err(|error| {
                format!(
                    "could not read notification migration evidence {}: {error}",
                    evidence_path.display()
                )
            })?)
            .map_err(|error| format!("notification migration evidence is invalid JSON: {error}"))?;
        let target_checksum = evidence
            .get("targetSchemaChecksum")
            .and_then(serde_json::Value::as_str);
        let recovery_checksum = evidence
            .get("recoverySchemaChecksum")
            .and_then(serde_json::Value::as_str);
        let legacy_before = evidence
            .get("legacyRowsBeforeUpgrade")
            .and_then(serde_json::Value::as_u64);
        let legacy_after = evidence
            .get("legacyRowsAfterUpgrade")
            .and_then(serde_json::Value::as_u64);
        let recovery_rows = evidence
            .get("recoveryRows")
            .and_then(serde_json::Value::as_u64);
        database_evidence_ok = evidence.get("schemaVersion") == Some(&serde_json::json!(1))
            && evidence
                .get("databaseClass")
                .and_then(serde_json::Value::as_str)
                == Some("local-scratch-only")
            && evidence.get("cleanMigrationLedgerRows") == Some(&serde_json::json!(11))
            && evidence.get("targetTables") == Some(&serde_json::json!(16))
            && evidence.get("constraintsVerified") == Some(&serde_json::Value::Bool(true))
            && evidence.get("expiryFilterVerified") == Some(&serde_json::Value::Bool(true))
            && evidence.get("productionReady") == Some(&serde_json::Value::Bool(false))
            && legacy_before.is_some()
            && legacy_before == legacy_after
            && recovery_rows == legacy_after
            && target_checksum.is_some()
            && target_checksum == recovery_checksum;
        println!(
            "notification-migration-audit: live_evidence_file={} live_evidence={}",
            evidence_file,
            if database_evidence_ok { "pass" } else { "fail" }
        );
    }
    println!(
        "notification-migration-audit: static={} database_ledger_evidence={}",
        if static_ok { "pass" } else { "fail" },
        database_evidence
    );
    if !static_ok {
        return Err("notification migration static chain is incomplete".into());
    }
    if strict && (database_evidence != "verified" || !database_evidence_ok) {
        return Err(
            "notification migration database ledger evidence is not verified or failed its report checks"
                .into(),
        );
    }
    Ok(())
}

fn read_reconcile_records(path: &Path) -> Result<(Vec<ReconcileRecord>, usize, usize), String> {
    let file =
        File::open(path).map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut records = Vec::new();
    let mut invalid = 0;
    let mut seen = HashSet::new();
    let mut duplicate = 0;
    for (line_number, line) in BufReader::new(file).lines().enumerate() {
        if records.len() + invalid >= 100_000 {
            return Err(format!(
                "notification-reconcile input {} exceeds the 100000-record bound",
                path.display()
            ));
        }
        let line =
            line.map_err(|error| format!("could not read line {}: {error}", line_number + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(record) = serde_json::from_str::<ReconcileRecord>(&line) else {
            invalid += 1;
            continue;
        };
        if !valid_reconcile_record(&record) {
            invalid += 1;
            continue;
        }
        if !seen.insert(record.source_event_id.clone()) {
            duplicate += 1;
        }
        records.push(record);
    }
    Ok((records, invalid, duplicate))
}

fn valid_reconcile_record(record: &ReconcileRecord) -> bool {
    !record.source_event_id.trim().is_empty()
        && record.source_event_id.len() <= 128
        && (valid_wallet(&record.wallet_address) || record.wallet_address == "all")
        && matches!(
            record.status.as_str(),
            "pending" | "sent" | "failed" | "suppressed" | "cancelled" | "expired" | "deleted"
        )
        && record
            .provider_message_id
            .as_deref()
            .is_none_or(|value| !value.is_empty() && value.len() <= 255)
        && record
            .provider_event_id
            .as_deref()
            .is_none_or(|value| !value.is_empty() && value.len() <= 255)
        && record
            .template_id
            .as_deref()
            .is_none_or(|value| !value.is_empty() && value.len() <= 128)
        && record.preference_hash.as_deref().is_none_or(|value| {
            value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
}

fn build_reconciliation_report(
    source: &(Vec<ReconcileRecord>, usize, usize),
    target: &(Vec<ReconcileRecord>, usize, usize),
) -> ReconciliationReport {
    let source_map = source
        .0
        .iter()
        .map(|record| (record.source_event_id.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let target_map = target
        .0
        .iter()
        .map(|record| (record.source_event_id.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let missing_target_events = source_map
        .keys()
        .filter(|event_id| !target_map.contains_key(*event_id))
        .count();
    let orphan_target_events = target_map
        .keys()
        .filter(|event_id| !source_map.contains_key(*event_id))
        .count();
    let source_status_distribution = status_distribution(source_map.values().copied());
    let target_status_distribution = status_distribution(target_map.values().copied());
    let source_wallet_checksum = wallet_checksum(source_map.values().copied());
    let target_wallet_checksum = wallet_checksum(target_map.values().copied());
    let source_broadcast_records = source.0.iter().filter(|record| record.broadcast).count();
    let target_broadcast_records = target.0.iter().filter(|record| record.broadcast).count();
    let target_records_without_provider_id = target
        .0
        .iter()
        .filter(|record| record.provider_message_id.is_none())
        .count();
    let target_sent_without_provider_id = target
        .0
        .iter()
        .filter(|record| record.status == "sent" && record.provider_message_id.is_none())
        .count();
    let mut template_identity_drift = 0;
    let mut preference_identity_drift = 0;
    let mut provider_identity_drift = 0;
    for (event_id, source_record) in &source_map {
        let Some(target_record) = target_map.get(event_id) else {
            continue;
        };
        if source_record.template_id != target_record.template_id {
            template_identity_drift += 1;
        }
        if source_record.preference_hash != target_record.preference_hash {
            preference_identity_drift += 1;
        }
        if source_record.provider_message_id != target_record.provider_message_id
            || source_record.provider_event_id != target_record.provider_event_id
        {
            provider_identity_drift += 1;
        }
    }
    let status_distribution_match = source_status_distribution == target_status_distribution;
    let broadcast_count_match = source_broadcast_records == target_broadcast_records;
    ReconciliationReport {
        source_records: source.0.len(),
        target_records: target.0.len(),
        invalid_source_records: source.1,
        invalid_target_records: target.1,
        duplicate_source_events: source.2,
        duplicate_target_events: target.2,
        missing_target_events,
        orphan_target_events,
        source_status_distribution,
        target_status_distribution,
        source_broadcast_records,
        target_broadcast_records,
        target_records_without_provider_id,
        target_sent_without_provider_id,
        template_identity_drift,
        preference_identity_drift,
        provider_identity_drift,
        wallet_checksum_match: source_wallet_checksum == target_wallet_checksum,
        source_target_event_set_match: source_map.keys().collect::<BTreeSet<_>>()
            == target_map.keys().collect::<BTreeSet<_>>(),
        status_distribution_match,
        broadcast_count_match,
        source_wallet_checksum,
        target_wallet_checksum,
    }
}

fn status_distribution<'a>(
    records: impl Iterator<Item = &'a ReconcileRecord>,
) -> BTreeMap<String, usize> {
    let mut distribution = BTreeMap::new();
    for status in records.map(|record| record.status.as_str()) {
        *distribution.entry(status.to_string()).or_insert(0) += 1;
    }
    distribution
}

fn wallet_checksum<'a>(records: impl Iterator<Item = &'a ReconcileRecord>) -> String {
    let mut values = records
        .map(|record| format!("{}\0{}", record.source_event_id, record.wallet_address))
        .collect::<Vec<_>>();
    values.sort();
    let mut digest = Sha256::new();
    for value in values {
        digest.update(value.as_bytes());
        digest.update([0]);
    }
    let digest = digest.finalize();
    format!("{digest:x}")
}

fn authority_audit(flags: &[String]) -> Result<(), String> {
    let strict = flags.iter().any(|flag| flag == "--strict");
    if flags
        .iter()
        .any(|flag| flag != "--strict" && flag != "--report")
    {
        return Err("authority-audit accepts only --strict and --report".into());
    }

    let root = repo_root()?;
    let path = root.join("docs/migration/contracts/notification-authority-matrix.json");
    let contract = std::fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let matrix: serde_json::Value = serde_json::from_str(&contract)
        .map_err(|error| format!("invalid authority matrix JSON: {error}"))?;
    if matrix.get("schemaVersion") != Some(&serde_json::Value::from(1))
        || matrix.get("contractId")
            != Some(&serde_json::Value::from("A11.1-notification-authority"))
        || matrix.get("mode") != Some(&serde_json::Value::from("versioned-adapter"))
        || matrix.get("productionReady") != Some(&serde_json::Value::Bool(false))
    {
        return Err("authority matrix sentinel changed".into());
    }
    let matrix_surfaces = matrix
        .get("surfaces")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "authority matrix surfaces are missing".to_string())?;
    let mut matrix_ids = BTreeSet::new();
    for surface in matrix_surfaces {
        let id = surface
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "authority matrix surface id is missing".to_string())?;
        if !matrix_ids.insert(id.to_string()) {
            return Err(format!("duplicate authority surface: {id}"));
        }
        for field in [
            "authority",
            "bff",
            "service",
            "request",
            "success",
            "errors",
        ] {
            if surface
                .get(field)
                .and_then(serde_json::Value::as_str)
                .is_none_or(str::is_empty)
            {
                return Err(format!(
                    "authority surface {id} has no usable {field} contract"
                ));
            }
        }
    }
    let missing: Vec<&str> = NOTIFICATION_AUTHORITY_SURFACES
        .iter()
        .copied()
        .filter(|id| !matrix_ids.contains(*id))
        .collect();
    let id_count = matrix_surfaces.len();
    let fixture_path = root.join(
        matrix
            .get("compatibilityFixtureFile")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "compatibilityFixtureFile is missing".to_string())?,
    );
    let fixture_text = std::fs::read_to_string(&fixture_path)
        .map_err(|error| format!("could not read {}: {error}", fixture_path.display()))?;
    let fixture: serde_json::Value = serde_json::from_str(&fixture_text)
        .map_err(|error| format!("invalid compatibility fixture JSON: {error}"))?;
    if fixture.get("schemaVersion") != Some(&serde_json::Value::from(1))
        || fixture.get("contractId")
            != Some(&serde_json::Value::from(
                "A11.2-notification-compatibility-fixtures",
            ))
        || fixture.get("productionReady") != Some(&serde_json::Value::Bool(false))
    {
        return Err("compatibility fixture sentinel changed".into());
    }
    let fixture_surfaces = fixture
        .get("surfaces")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "compatibility fixture surfaces are missing".to_string())?;
    if fixture_surfaces.len() != NOTIFICATION_AUTHORITY_SURFACES.len() {
        return Err("compatibility fixture surface count changed".into());
    }
    let matrix_by_id = matrix_surfaces
        .iter()
        .filter_map(|surface| {
            surface
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(|id| (id, surface))
        })
        .collect::<std::collections::HashMap<_, _>>();
    let mut fixture_ids = BTreeSet::new();
    let allowed_methods = ["GET", "POST", "PUT", "DELETE"];
    for fixture_surface in fixture_surfaces {
        let id = fixture_surface
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "compatibility fixture surface id is missing".to_string())?;
        if !fixture_ids.insert(id.to_string()) || !matrix_by_id.contains_key(id) {
            return Err(format!(
                "compatibility fixture surface is unknown or duplicated: {id}"
            ));
        }
        let authority = fixture_surface
            .get("authority")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("compatibility fixture authority missing: {id}"))?;
        let matrix_surface = matrix_by_id[id];
        if matrix_surface
            .get("authority")
            .and_then(serde_json::Value::as_str)
            != Some(authority)
        {
            return Err(format!("authority drift for compatibility surface: {id}"));
        }
        if fixture_surface.get("ownerDerived")
            != Some(&serde_json::Value::Bool(authority == "owner"))
        {
            return Err(format!(
                "owner derivation drift for compatibility surface: {id}"
            ));
        }
        let routes = fixture_surface
            .get("routes")
            .and_then(serde_json::Value::as_array)
            .filter(|routes| !routes.is_empty())
            .ok_or_else(|| format!("compatibility routes missing: {id}"))?;
        let matrix_bff = matrix_surface
            .get("bff")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let matrix_service = matrix_surface
            .get("service")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        for route in routes {
            for (side, matrix_text) in [("bff", matrix_bff), ("service", matrix_service)] {
                let route_side = route
                    .get(side)
                    .ok_or_else(|| format!("{id}: {side} route missing"))?;
                let method = route_side
                    .get("method")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                let path = route_side
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                if !allowed_methods.contains(&method)
                    || path.is_empty()
                    || !matrix_text.contains(method)
                    || !matrix_text.contains(path)
                {
                    return Err(format!(
                        "{id}: {side} method/path is not represented by the authority matrix"
                    ));
                }
            }
            let success = route
                .get("success")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| format!("{id}: success status missing"))?;
            if !(200..=299).contains(&success) {
                return Err(format!("{id}: success status is not 2xx"));
            }
            let errors = route
                .get("errors")
                .and_then(serde_json::Value::as_array)
                .filter(|errors| !errors.is_empty())
                .ok_or_else(|| format!("{id}: error statuses missing"))?;
            if errors.iter().any(|status| {
                status
                    .as_u64()
                    .is_none_or(|status| !(400..=599).contains(&status))
            }) {
                return Err(format!("{id}: invalid error status"));
            }
        }
    }
    if fixture_ids != matrix_ids {
        return Err("authority matrix and compatibility fixture surface sets differ".into());
    }

    println!(
        "authority-audit: surfaces={} expected=12 missing={} compatibility_routes=validated",
        id_count,
        missing.len()
    );

    let valid = id_count == NOTIFICATION_AUTHORITY_SURFACES.len() && missing.is_empty();
    if strict && !valid {
        return Err("strict authority gate failed: compatibility matrix is incomplete".into());
    }
    Ok(())
}

/// Verify that every route in the reviewed compatibility fixture is present in
/// the Rust router source that owns it. This is intentionally narrower than a
/// live compatibility test: it catches method/path drift in the checked-in
/// target while leaving payload, envelope, status, source behavior, and live
/// service integration claims to their separate gates.
fn notification_compatibility_audit(flags: &[String]) -> Result<(), String> {
    let strict = flags.iter().any(|flag| flag == "--strict");
    if flags
        .iter()
        .any(|flag| flag != "--strict" && flag != "--report")
    {
        return Err("notification-compatibility-audit accepts only --strict and --report".into());
    }

    let root = repo_root()?;
    let fixture_path =
        root.join("docs/migration/contracts/notification-compatibility-fixtures.json");
    let fixture_text = std::fs::read_to_string(&fixture_path)
        .map_err(|error| format!("could not read {}: {error}", fixture_path.display()))?;
    let fixture: serde_json::Value = serde_json::from_str(&fixture_text)
        .map_err(|error| format!("invalid compatibility fixture JSON: {error}"))?;
    if fixture.get("schemaVersion") != Some(&serde_json::Value::from(1))
        || fixture.get("contractId")
            != Some(&serde_json::Value::from(
                "A11.2-notification-compatibility-fixtures",
            ))
        || fixture.get("productionReady") != Some(&serde_json::Value::Bool(false))
    {
        return Err("compatibility fixture sentinel changed".into());
    }
    let surfaces = fixture
        .get("surfaces")
        .and_then(serde_json::Value::as_array)
        .filter(|surfaces| !surfaces.is_empty())
        .ok_or_else(|| "compatibility fixture surfaces are missing".to_string())?;

    let frontend_router = std::fs::read_to_string(root.join("apps/frontend/src/main.rs"))
        .map_err(|error| format!("could not read frontend router: {error}"))?;
    let admin_router = std::fs::read_to_string(root.join("apps/admin/src/main.rs"))
        .map_err(|error| format!("could not read admin router: {error}"))?;
    let admin_ssr = std::fs::read_to_string(root.join("apps/admin/src/ssr.rs"))
        .map_err(|error| format!("could not read admin SSR router: {error}"))?;
    let service_router = std::fs::read_to_string(root.join("services/notification/src/main.rs"))
        .map_err(|error| format!("could not read notification service router: {error}"))?;

    let mut checked_routes = 0usize;
    let mut checked_sources = BTreeSet::new();
    for surface in surfaces {
        let id = surface
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "compatibility surface id is missing".to_string())?;
        let routes = surface
            .get("routes")
            .and_then(serde_json::Value::as_array)
            .filter(|routes| !routes.is_empty())
            .ok_or_else(|| format!("{id}: compatibility routes are missing"))?;
        for route in routes {
            let bff = route
                .get("bff")
                .ok_or_else(|| format!("{id}: bff route is missing"))?;
            let service = route
                .get("service")
                .ok_or_else(|| format!("{id}: service route is missing"))?;
            let bff_method = bff
                .get("method")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("{id}: bff method is missing"))?;
            let bff_path = bff
                .get("path")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("{id}: bff path is missing"))?;
            let service_method = service
                .get("method")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("{id}: service method is missing"))?;
            let service_path = service
                .get("path")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("{id}: service path is missing"))?;

            let bff_source = if bff_path == "/notifications/manage" {
                checked_sources.insert("apps/admin/src/ssr.rs");
                &admin_ssr
            } else if bff_path.starts_with("/api/v1/notifications/templates")
                || bff_path == "/api/v1/notifications/send"
                || bff_path == "/api/v1/notifications/metrics"
            {
                checked_sources.insert("apps/admin/src/main.rs");
                &admin_router
            } else {
                checked_sources.insert("apps/frontend/src/main.rs");
                &frontend_router
            };
            let bff_registered = if bff_path == "/notifications/manage" {
                bff_source.contains("route_path == \"/notifications/manage\"")
                    && bff_method == "GET"
            } else {
                route_registration_present(bff_source, bff_method, bff_path)
            };
            if !bff_registered {
                return Err(format!(
                    "{id}: BFF route registration missing or method-drifted: {bff_method} {bff_path}"
                ));
            }

            checked_sources.insert("services/notification/src/main.rs");
            if !route_registration_present(service_router.as_str(), service_method, service_path) {
                return Err(format!(
                    "{id}: service route registration missing or method-drifted: {service_method} {service_path}"
                ));
            }
            checked_routes += 1;
        }
    }

    println!(
        "notification-compatibility-audit: routes={} sources={} method_path_registrations=verified",
        checked_routes,
        checked_sources.len()
    );
    if strict && checked_routes == 0 {
        return Err("strict compatibility audit found no routes".into());
    }
    Ok(())
}

/// Verify that the concrete backend producers moved behind the notification
/// port use source-derived event identities. This is a source-wiring check,
/// not proof that every producer is deployed with a provisioned service
/// identity or that a remote provider accepted the event.
fn notification_producer_audit(flags: &[String]) -> Result<(), String> {
    let strict = flags.iter().any(|flag| flag == "--strict");
    if flags
        .iter()
        .any(|flag| flag != "--strict" && flag != "--report")
    {
        return Err("notification-producer-audit accepts only --strict and --report".into());
    }

    let root = repo_root()?;
    let contract_path = root.join("docs/migration/contracts/notification-publisher.json");
    let contract_text = std::fs::read_to_string(&contract_path)
        .map_err(|error| format!("could not read {}: {error}", contract_path.display()))?;
    let contract: serde_json::Value = serde_json::from_str(&contract_text)
        .map_err(|error| format!("invalid notification publisher contract JSON: {error}"))?;
    if contract.get("schemaVersion") != Some(&serde_json::Value::from(1))
        || contract.get("contractId")
            != Some(&serde_json::Value::from("A11.3-notification-publisher"))
        || contract.get("productionReady") != Some(&serde_json::Value::Bool(false))
    {
        return Err("notification publisher contract sentinel changed".into());
    }

    let producers = [
        (
            "apps/backend/src/web/payments/credit_handlers.rs",
            &["send_with_event_id_retry("][..],
        ),
        (
            "apps/backend/src/web/payments/submit_tx_handler.rs",
            &["send_with_event_id_retry("][..],
        ),
        (
            "apps/backend/src/web/admin/permissions/assignments/create.rs",
            &["send_with_event_id_retry("][..],
        ),
        (
            "apps/backend/src/web/admin/permissions/assignments/remove.rs",
            &["send_with_event_id_retry("][..],
        ),
        (
            "apps/backend/src/web/user/chat_handlers.rs",
            &[
                "send_with_event_id_retry(",
                "broadcast_with_event_id_retry(",
            ][..],
        ),
        (
            "apps/backend/src/web/admin/chat_handlers.rs",
            &["send_with_event_id_retry("][..],
        ),
        (
            "apps/backend/src/infrastructure/services/plan_expiration_service.rs",
            &["send_with_event_id_retry("][..],
        ),
    ];
    let mut retry_anchors = 0usize;
    for (relative, required) in producers {
        let path = root.join(relative);
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("could not read {relative}: {error}"))?;
        for anchor in required {
            if !source.contains(anchor) {
                return Err(format!(
                    "{relative}: missing stable producer anchor {anchor}"
                ));
            }
            retry_anchors += 1;
        }
    }

    let stable_event_id_anchors = retry_anchors;

    let notification_service = std::fs::read_to_string(
        root.join("apps/backend/src/infrastructure/services/notification_service.rs"),
    )
    .map_err(|error| format!("could not read notification service shim: {error}"))?;
    let mut legacy_calls = 0usize;
    for relative in [
        "apps/backend/src/web/payments/credit_handlers.rs",
        "apps/backend/src/web/payments/submit_tx_handler.rs",
        "apps/backend/src/web/admin/permissions/assignments/create.rs",
        "apps/backend/src/web/admin/permissions/assignments/remove.rs",
        "apps/backend/src/web/user/chat_handlers.rs",
        "apps/backend/src/web/admin/chat_handlers.rs",
        "apps/backend/src/infrastructure/services/plan_expiration_service.rs",
    ] {
        let source = std::fs::read_to_string(root.join(relative))
            .map_err(|error| format!("could not read {relative}: {error}"))?;
        legacy_calls += source.matches("NotificationService::send(").count();
        legacy_calls += source.matches("NotificationService::broadcast(").count();
    }
    if legacy_calls != 0 {
        return Err(format!(
            "migrated backend producers still call the legacy NotificationService shim: {legacy_calls}"
        ));
    }
    if !notification_service.contains("pub struct NotificationService") {
        return Err("legacy notification service shim sentinel missing".into());
    }

    println!(
        "notification-producer-audit: producers={} stable_event_id_anchors={} retry_anchors={} legacy_shim_calls={} verified",
        producers.len(),
        stable_event_id_anchors,
        retry_anchors,
        legacy_calls
    );
    if strict && stable_event_id_anchors == 0 {
        return Err("strict producer audit found no stable event-id retry anchors".into());
    }
    Ok(())
}

fn route_registration_present(source: &str, method: &str, path: &str) -> bool {
    let Some(mut offset) = source.find(path) else {
        return false;
    };
    while let Some(relative) = source[offset..].find(path) {
        let path_start = offset + relative;
        let start = source[..path_start]
            .rfind(".route(")
            .unwrap_or(path_start.saturating_sub(256));
        let after_path = path_start + path.len();
        let end = source[after_path..]
            .find(".route(")
            .map(|relative| after_path + relative)
            .unwrap_or((after_path + 256).min(source.len()));
        let window = &source[start..end];
        let method_token = match method {
            "GET" => "get(",
            "POST" => "post(",
            "PUT" => "put(",
            "DELETE" => "delete(",
            _ => return false,
        };
        if window.contains(method_token) {
            return true;
        }
        let next = after_path;
        if next >= source.len() {
            break;
        }
        offset = next;
    }
    false
}

fn repo_root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|error| format!("could not run git: {error}"))?;
    if !output.status.success() {
        return Err("git rev-parse --show-toplevel failed".into());
    }
    let root = String::from_utf8(output.stdout)
        .map_err(|_| "git returned a non-UTF-8 repository path".to_string())?;
    Ok(PathBuf::from(root.trim()))
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
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| {
            String::from_utf8(bytes.to_vec())
                .map(PathBuf::from)
                .map_err(|_| "tracked path was not valid UTF-8".to_string())
        })
        .collect()
}

fn extension(path: &Path) -> Option<&str> {
    path.extension().and_then(OsStr::to_str)
}

fn rust_audit(flags: &[String]) -> Result<(), String> {
    let strict = flags.iter().any(|flag| flag == "--strict");
    let report_only = flags.iter().all(|flag| flag == "--report");
    if flags
        .iter()
        .any(|flag| flag != "--strict" && flag != "--report")
    {
        return Err("rust-audit accepts only --strict and --report".into());
    }

    let root = repo_root()?;
    let files = tracked_files(&root)?;
    let authored_scripts: Vec<&Path> = files
        .iter()
        .filter(|path| root.join(path).is_file())
        .filter(|path| extension(path).is_some_and(|ext| JS_EXTENSIONS.contains(&ext)))
        .map(PathBuf::as_path)
        .collect();

    let mut embedded = Vec::new();
    for relative in &files {
        if extension(relative) != Some("rs")
            || !RUST_ROOTS.iter().any(|prefix| relative.starts_with(prefix))
        {
            continue;
        }
        let absolute = root.join(relative);
        let Ok(contents) = std::fs::read_to_string(&absolute) else {
            continue;
        };
        let markers: Vec<&str> = EMBEDDED_MARKERS
            .iter()
            .copied()
            .filter(|marker| contents.contains(marker))
            .collect();
        if !markers.is_empty() {
            embedded.push((relative, markers));
        }
    }
    embedded.sort_by(|left, right| left.0.cmp(right.0));

    let review_path = root.join("docs/migration/contracts/rust-embedded-runtime-review.json");
    let review: RustEmbeddedRuntimeReview = serde_json::from_str(
        &std::fs::read_to_string(&review_path)
            .map_err(|error| format!("could not read {}: {error}", review_path.display()))?,
    )
    .map_err(|error| format!("embedded runtime review is invalid: {error}"))?;
    let inventory_sha256 = embedded_inventory_sha256(&root, &embedded)?;
    let approved_runtime_valid = validate_approved_embedded_runtimes(&root, &review, &embedded)?;
    let inventory_reviewed = review.schema_version == 1
        && review.hash_algorithm == "sha256"
        && is_hex_digest(&review.inventory_sha256)
        && review.inventory_sha256 == inventory_sha256;

    println!(
        "rust-only-audit: tracked authored JS/TS-family files={}",
        authored_scripts.len()
    );
    for path in &authored_scripts {
        println!("  authored-script: {}", path.display());
    }
    println!(
        "rust-only-audit: Rust files with embedded runtime markers={}",
        embedded.len()
    );
    for (path, markers) in &embedded {
        println!(
            "  embedded-runtime: {} [{}]",
            path.display(),
            markers.join(", ")
        );
    }

    println!(
        "rust-only-audit: embedded inventory sha256={inventory_sha256} reviewed={}",
        if inventory_reviewed { "yes" } else { "no" }
    );
    println!(
        "rust-only-audit: approved Rust/WASM runtime files={} marker-only reviewed files={} approval={}",
        review.approved_runtime_files.len(),
        embedded
            .len()
            .saturating_sub(review.approved_runtime_files.len()),
        if approved_runtime_valid { "pass" } else { "fail" }
    );

    if strict && (!authored_scripts.is_empty() || !inventory_reviewed || !approved_runtime_valid) {
        return Err(format!(
            "strict Rust gate failed: authored_scripts={} inventory_reviewed={} approved_runtime_review={}",
            authored_scripts.len(), inventory_reviewed, approved_runtime_valid
        ));
    }
    if !strict && !report_only {
        println!("rust-only-audit: report mode; use --strict for the completion gate");
    }
    Ok(())
}

fn embedded_inventory_sha256(
    root: &Path,
    embedded: &[(&PathBuf, Vec<&str>)],
) -> Result<String, String> {
    let mut digest = Sha256::new();
    for (relative, markers) in embedded {
        let file_sha = sha256_file(&root.join(relative))
            .ok_or_else(|| format!("could not hash embedded marker file {}", relative.display()))?;
        digest.update(relative.to_string_lossy().as_bytes());
        digest.update([0]);
        digest.update(file_sha.as_bytes());
        digest.update([0]);
        digest.update(markers.join(",").as_bytes());
        digest.update([b'\n']);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn validate_approved_embedded_runtimes(
    root: &Path,
    review: &RustEmbeddedRuntimeReview,
    embedded: &[(&PathBuf, Vec<&str>)],
) -> Result<bool, String> {
    let detected = embedded
        .iter()
        .map(|(path, _)| path.to_string_lossy().to_string())
        .collect::<BTreeSet<_>>();
    let expected = APPROVED_EMBEDDED_RUNTIME_FILES
        .iter()
        .map(|(path, category)| ((*path).to_string(), (*category).to_string()))
        .collect::<BTreeMap<_, _>>();
    let mut actual = BTreeMap::new();
    for approved in &review.approved_runtime_files {
        if actual
            .insert(approved.path.clone(), approved.category.clone())
            .is_some()
            || approved.reason.trim().is_empty()
            || !is_hex_digest(&approved.sha256)
            || !detected.contains(&approved.path)
            || sha256_file(&root.join(&approved.path)).as_deref() != Some(&approved.sha256)
        {
            return Ok(false);
        }
    }
    Ok(actual == expected)
}

fn is_hex_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sync_audit() -> Result<(), String> {
    let root = repo_root()?;
    let contract_path = root.join("docs/migration/contracts/migration-baseline.json");
    let contract: MigrationBaseline = serde_json::from_str(
        &std::fs::read_to_string(&contract_path)
            .map_err(|error| format!("could not read {}: {error}", contract_path.display()))?,
    )
    .map_err(|error| format!("migration baseline contract is invalid: {error}"))?;

    if contract.schema_version != 1
        || !is_hex_sha(&contract.source_sha)
        || !is_hex_sha(&contract.target_sha)
        || !is_hex_sha(&contract.evidence_sha)
        || contract.readiness_level != "source-integrated-static-only"
        || contract.staging_ready
        || contract.production_ready
    {
        return Err(
            "migration baseline contract has an unsafe or unsupported readiness claim".into(),
        );
    }

    let branch = git_text(&root, &["branch", "--show-current"])?;
    if !branch.is_empty() && branch != contract.target_branch {
        return Err(format!(
            "migration target branch mismatch: contract={} checkout={branch}",
            contract.target_branch
        ));
    }

    for (label, commit) in [
        ("source", contract.source_sha.as_str()),
        ("target", contract.target_sha.as_str()),
        ("evidence", contract.evidence_sha.as_str()),
    ] {
        if !git_success(&root, &["cat-file", "-e", &format!("{commit}^{{commit}}")])? {
            return Err(format!("migration {label} commit is unavailable: {commit}"));
        }
    }

    if !git_success(
        &root,
        &[
            "merge-base",
            "--is-ancestor",
            &contract.source_sha,
            &contract.target_sha,
        ],
    )? {
        return Err("pinned development source is not an ancestor of the target baseline".into());
    }
    for (label, commit) in [
        ("target", contract.target_sha.as_str()),
        ("evidence", contract.evidence_sha.as_str()),
    ] {
        if !git_success(&root, &["merge-base", "--is-ancestor", commit, "HEAD"])? {
            return Err(format!("pinned {label} commit is not an ancestor of HEAD"));
        }
    }

    let local_source_ref = format!("{}^{{commit}}", contract.source_ref);
    if git_success(
        &root,
        &["rev-parse", "--verify", "--quiet", &local_source_ref],
    )? {
        let local_source = git_text(&root, &["rev-parse", &local_source_ref])?;
        if local_source != contract.source_sha {
            return Err(format!(
                "local {} moved: contract={} checkout={local_source}",
                contract.source_ref, contract.source_sha
            ));
        }
    }

    let head = git_text(&root, &["rev-parse", "HEAD^{commit}"])?;
    println!(
        "sync-audit: branch={}",
        if branch.is_empty() {
            "DETACHED"
        } else {
            &branch
        }
    );
    println!(
        "sync-audit: source={} {}",
        contract.source_ref, contract.source_sha
    );
    println!("sync-audit: target-baseline={}", contract.target_sha);
    println!("sync-audit: evidence-baseline={}", contract.evidence_sha);
    println!("sync-audit: head={head}");
    println!("sync-audit: readiness={}", contract.readiness_level);
    println!("sync-audit: staging_ready=false production_ready=false");
    println!("sync-audit: PASS — source, target, evidence, and HEAD ancestry verified");
    Ok(())
}

fn k8s_audit(flags: &[String]) -> Result<(), String> {
    let strict = flags.iter().any(|flag| flag == "--strict");
    if flags
        .iter()
        .any(|flag| flag != "--strict" && flag != "--report")
    {
        return Err("k8s-audit accepts only --strict and --report".into());
    }

    let root = repo_root()?;
    let migration_services = [
        "epsx-admin",
        "epsx-analytics",
        "epsx-backend",
        "epsx-frontend",
        "epsx-identity",
        "epsx-notification",
        "epsx-pay-bff",
        "epsx-pay-svc",
    ];
    let production_services = ["epsx-admin", "epsx-backend", "epsx-frontend"];
    let mut failed = false;

    for (environment, expected_namespace, expected_services) in [
        ("dev", "epsx-dev", migration_services.as_slice()),
        ("staging", "epsx-staging", migration_services.as_slice()),
        ("prod", "epsx-prod", production_services.as_slice()),
    ] {
        let overlay = format!("infrastructure/kubernetes/overlays/{environment}");
        let output = Command::new("kubectl")
            .args(["kustomize", &overlay])
            .current_dir(&root)
            .output()
            .map_err(|error| format!("could not run kubectl kustomize {overlay}: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("could not render {environment} overlay: {stderr}"));
        }
        let rendered = String::from_utf8(output.stdout)
            .map_err(|_| format!("{environment} overlay rendered non-UTF-8 YAML"))?;
        let (namespace, inventory) = k8s_render_inventory(&rendered)?;
        let expected = expected_services
            .iter()
            .flat_map(|name| [format!("Deployment/{name}"), format!("Service/{name}")])
            .collect::<BTreeSet<_>>();
        let matches = namespace.as_deref() == Some(expected_namespace) && inventory == expected;
        println!(
            "k8s-audit: environment={environment} namespace={} resources={} inventory={}",
            namespace.as_deref().unwrap_or("MISSING"),
            inventory.len(),
            if matches { "pass" } else { "fail" }
        );
        if !matches {
            let missing = expected.difference(&inventory).cloned().collect::<Vec<_>>();
            let unexpected = inventory.difference(&expected).cloned().collect::<Vec<_>>();
            println!("k8s-audit: {environment} missing={}", missing.join(","));
            println!(
                "k8s-audit: {environment} unexpected={}",
                unexpected.join(",")
            );
            failed = true;
        }
    }

    if strict && failed {
        return Err("strict Kubernetes overlay inventory gate failed".into());
    }
    if !failed {
        println!("k8s-audit: PASS — all overlays render with explicit service inventories");
    }
    Ok(())
}

fn k8s_render_inventory(rendered: &str) -> Result<(Option<String>, BTreeSet<String>), String> {
    let mut namespace = None;
    let mut inventory = BTreeSet::new();
    for document in rendered.split("\n---\n") {
        let kind = document
            .lines()
            .find_map(|line| line.strip_prefix("kind: "));
        let name = document
            .lines()
            .skip_while(|line| *line != "metadata:")
            .skip(1)
            .find_map(|line| line.strip_prefix("  name: "));
        match (kind, name) {
            (Some("Namespace"), Some(name)) => namespace = Some(name.to_string()),
            (Some(kind @ ("Deployment" | "Service")), Some(name)) => {
                inventory.insert(format!("{kind}/{name}"));
            }
            (Some("Deployment" | "Service" | "Namespace"), None) => {
                return Err("rendered Kubernetes resource has no metadata.name".into());
            }
            _ => {}
        }
    }
    Ok((namespace, inventory))
}

fn is_hex_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn git_success(root: &Path, args: &[&str]) -> Result<bool, String> {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map(|output| output.status.success())
        .map_err(|error| format!("could not run git {}: {error}", args.join(" ")))
}

fn migration_audit(flags: &[String]) -> Result<(), String> {
    let strict = flags.iter().any(|flag| flag == "--strict");
    if flags
        .iter()
        .any(|flag| flag != "--strict" && flag != "--report")
    {
        return Err("migration-audit accepts only --strict and --report".into());
    }

    let root = repo_root()?;
    let destructive_allowlist = migration_destructive_allowlist(&root)?;
    let mut duplicate_versions = Vec::new();
    let mut destructive_files = Vec::new();
    for migration_root in migration_roots(&root)? {
        let mut versions = std::collections::BTreeMap::<String, Vec<PathBuf>>::new();
        let entries = std::fs::read_dir(&migration_root)
            .map_err(|error| format!("could not read {}: {error}", migration_root.display()))?;
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("could not inspect migration entry: {error}"))?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(OsStr::to_str) else {
                continue;
            };
            if path.is_dir() {
                let version = migration_version(name);
                versions.entry(version).or_default().push(path.clone());
                for sql_file in [path.join("up.sql"), path.join("down.sql")] {
                    if let Ok(sql) = std::fs::read_to_string(&sql_file) {
                        if contains_destructive_sql(&sql) {
                            destructive_files.push(sql_file);
                        }
                    }
                }
            } else if path.extension() == Some(OsStr::new("sql")) {
                let version =
                    migration_version(path.file_stem().and_then(OsStr::to_str).unwrap_or(name));
                versions.entry(version).or_default().push(path.clone());
                if let Ok(sql) = std::fs::read_to_string(&path) {
                    if contains_destructive_sql(&sql) {
                        destructive_files.push(path);
                    }
                }
            }
        }
        for (version, paths) in versions {
            if paths.len() > 1 {
                duplicate_versions.push((migration_root.clone(), version, paths));
            }
        }
    }
    let lifecycle_migration = root.join(
        "apps/backend/migrations/notifications/20260723120000_add_notification_lifecycle_foundation",
    );
    let lifecycle_extensions = root.join(
        "apps/backend/migrations/notifications/20260723130000_add_notification_idempotency_provider_events",
    );
    let lifecycle_constraints = root.join(
        "apps/backend/migrations/notifications/20260723140000_add_notification_lifecycle_constraints",
    );
    let template_audit = root.join(
        "apps/backend/migrations/notifications/20260724120000_add_notification_template_audit",
    );
    let engagement_acknowledged = root.join(
        "apps/backend/migrations/notifications/20260724130000_add_notification_engagement_acknowledged",
    );
    let lifecycle_up = lifecycle_migration.join("up.sql");
    let lifecycle_down = lifecycle_migration.join("down.sql");
    let extensions_up = lifecycle_extensions.join("up.sql");
    let extensions_down = lifecycle_extensions.join("down.sql");
    let constraints_up = lifecycle_constraints.join("up.sql");
    let constraints_down = lifecycle_constraints.join("down.sql");
    let audit_up = template_audit.join("up.sql");
    let audit_down = template_audit.join("down.sql");
    let engagement_up = engagement_acknowledged.join("up.sql");
    let engagement_down = engagement_acknowledged.join("down.sql");
    let expiration_migration = root
        .join("apps/backend/migrations/notifications/20260724140000_add_notification_expirations");
    let expiration_up = expiration_migration.join("up.sql");
    let expiration_down = expiration_migration.join("down.sql");
    let expiration_ready = match (
        std::fs::read_to_string(&expiration_up),
        std::fs::read_to_string(&expiration_down),
    ) {
        (Ok(up), Ok(down)) => {
            up.contains("CREATE TABLE IF NOT EXISTS public.notification_expirations")
                && up.contains("CREATE INDEX IF NOT EXISTS notification_expirations_due_idx")
                && down.trim().ends_with("SELECT 1;")
                && !contains_destructive_sql(&up)
                && !contains_destructive_sql(&down)
        }
        _ => false,
    };
    let lifecycle_ready = match (
        std::fs::read_to_string(&lifecycle_up),
        std::fs::read_to_string(&lifecycle_down),
        std::fs::read_to_string(&extensions_up),
        std::fs::read_to_string(&extensions_down),
        std::fs::read_to_string(&constraints_up),
        std::fs::read_to_string(&constraints_down),
        std::fs::read_to_string(&audit_up),
        std::fs::read_to_string(&audit_down),
        std::fs::read_to_string(&engagement_up),
        std::fs::read_to_string(&engagement_down),
    ) {
        (
            Ok(up),
            Ok(down),
            Ok(extensions),
            Ok(extensions_down),
            Ok(constraints),
            Ok(constraints_down),
            Ok(audit),
            Ok(audit_down),
            Ok(engagement),
            Ok(engagement_down),
        ) => {
            let foundation_tables = &NOTIFICATION_LIFECYCLE_TABLES[..9];
            let extension_tables = &NOTIFICATION_LIFECYCLE_TABLES[9..12];
            foundation_tables
                .iter()
                .all(|table| up.contains(&format!("CREATE TABLE IF NOT EXISTS public.{table}")))
                && extension_tables.iter().all(|table| {
                    extensions.contains(&format!("CREATE TABLE IF NOT EXISTS public.{table}"))
                })
                && audit.contains("CREATE TABLE IF NOT EXISTS public.notification_template_audit")
                && !contains_destructive_sql(&up)
                && !contains_destructive_sql(&extensions)
                && !contains_destructive_sql(&constraints)
                && down.trim().ends_with("SELECT 1;")
                && extensions_down.trim().ends_with("SELECT 1;")
                && constraints_down.trim().ends_with("SELECT 1;")
                && !contains_destructive_sql(&down)
                && !contains_destructive_sql(&extensions_down)
                && !contains_destructive_sql(&constraints_down)
                && !contains_destructive_sql(&audit)
                && !contains_destructive_sql(&audit_down)
                && audit_down.trim().ends_with("SELECT 1;")
                && engagement.contains("ADD COLUMN IF NOT EXISTS acknowledged_at")
                && !contains_destructive_sql(&engagement)
                && !contains_destructive_sql(&engagement_down)
                && engagement_down.trim().ends_with("SELECT 1;")
        }
        _ => false,
    };
    let lifecycle_ready = lifecycle_ready && expiration_ready;

    destructive_files.sort();
    let mut approved_destructive_files = Vec::new();
    let mut unapproved_destructive_files = Vec::new();
    let mut observed_allowlist_paths = BTreeSet::new();
    for path in &destructive_files {
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| format!("migration path escaped the repository: {}", path.display()))?
            .to_path_buf();
        let digest =
            sha256_file(path).ok_or_else(|| format!("could not hash {}", path.display()))?;
        if destructive_allowlist.get(&relative) == Some(&digest) {
            observed_allowlist_paths.insert(relative);
            approved_destructive_files.push(path.clone());
        } else {
            unapproved_destructive_files.push(path.clone());
        }
    }
    let stale_allowlist_entries = destructive_allowlist
        .keys()
        .filter(|path| !observed_allowlist_paths.contains(*path))
        .cloned()
        .collect::<Vec<_>>();

    println!(
        "migration-audit: duplicate active versions={}",
        duplicate_versions.len()
    );
    for (root, version, paths) in &duplicate_versions {
        println!("  duplicate: {} version {}", root.display(), version);
        for path in paths {
            println!("    {}", path.display());
        }
    }
    println!(
        "migration-audit: destructive SQL approved_history={} unapproved={} stale_allowlist={}",
        approved_destructive_files.len(),
        unapproved_destructive_files.len(),
        stale_allowlist_entries.len()
    );
    for path in &unapproved_destructive_files {
        println!("  unapproved-destructive-sql: {}", path.display());
    }
    for path in &stale_allowlist_entries {
        println!("  stale-destructive-allowlist: {}", path.display());
    }
    println!(
        "migration-audit: notification lifecycle foundation={}",
        if lifecycle_ready {
            "additive-static-pass"
        } else {
            "missing-or-unsafe"
        }
    );

    if strict
        && (!duplicate_versions.is_empty()
            || !unapproved_destructive_files.is_empty()
            || !stale_allowlist_entries.is_empty()
            || !lifecycle_ready)
    {
        return Err(format!(
            "strict migration gate failed: {} colliding versions, {} unapproved destructive SQL files, {} stale allowlist entries, lifecycle foundation={}",
            duplicate_versions.len(),
            unapproved_destructive_files.len(),
            stale_allowlist_entries.len(),
            if lifecycle_ready { "present" } else { "missing-or-unsafe" }
        ));
    }
    Ok(())
}

fn migration_destructive_allowlist(root: &Path) -> Result<BTreeMap<PathBuf, String>, String> {
    let path = root.join("docs/migration/contracts/legacy-destructive-migrations.sha256");
    let contents = std::fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut entries = BTreeMap::new();
    for (index, raw) in contents.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split_whitespace();
        let digest = fields.next().unwrap_or("");
        let relative = fields.next().unwrap_or("");
        if fields.next().is_some()
            || digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!(
                "invalid destructive migration allowlist line {}",
                index + 1
            ));
        }
        let relative = PathBuf::from(relative);
        if relative.as_os_str().is_empty()
            || relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir | std::path::Component::RootDir
                )
            })
        {
            return Err(format!(
                "unsafe destructive migration allowlist path on line {}",
                index + 1
            ));
        }
        if entries.insert(relative, digest.to_string()).is_some() {
            return Err(format!(
                "duplicate destructive migration allowlist path on line {}",
                index + 1
            ));
        }
    }
    Ok(entries)
}

fn migration_roots(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut roots = Vec::new();
    let backend = root.join("apps/backend/migrations");
    if backend.is_dir() {
        for entry in std::fs::read_dir(&backend)
            .map_err(|error| format!("could not read {}: {error}", backend.display()))?
        {
            let path = entry
                .map_err(|error| format!("could not inspect migration root: {error}"))?
                .path();
            if path.is_dir() && path.file_name() != Some(OsStr::new("_archive")) {
                roots.push(path);
            }
        }
    }
    let services = root.join("services");
    if services.is_dir() {
        for entry in std::fs::read_dir(&services)
            .map_err(|error| format!("could not read {}: {error}", services.display()))?
        {
            let service = entry
                .map_err(|error| format!("could not inspect service: {error}"))?
                .path();
            let migrations = service.join("migrations");
            if migrations.is_dir() {
                roots.push(migrations);
            }
        }
    }
    roots.sort();
    Ok(roots)
}

fn migration_version(name: &str) -> String {
    name.split('_').next().unwrap_or(name).to_string()
}

fn contains_destructive_sql(sql: &str) -> bool {
    let upper = sql.to_ascii_uppercase();
    [
        "DROP TABLE",
        "DROP SCHEMA",
        "DROP DATABASE",
        "TRUNCATE",
        "DELETE FROM",
        "CASCADE",
    ]
    .iter()
    .any(|marker| upper.contains(marker))
}

fn sha256_file(path: &Path) -> Option<String> {
    let contents = std::fs::read(path).ok()?;
    let mut digest = Sha256::new();
    digest.update(contents);
    Some(format!("{:x}", digest.finalize()))
}

fn git_text(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("could not run git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!("git {} failed", args.join(" ")));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|_| format!("git {} returned non-UTF-8 output", args.join(" ")))
}

#[cfg(test)]
mod tests {
    use super::{
        build_reconciliation_report, contains_destructive_sql, k8s_render_inventory,
        map_legacy_backfill_record, migration_version, route_registration_present,
        valid_reconcile_record, ReconcileRecord,
    };

    #[test]
    fn kubernetes_inventory_reads_only_top_level_resources() {
        let rendered = r#"apiVersion: v1
kind: Namespace
metadata:
  name: epsx-staging
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: epsx-backend
spec:
  template:
    spec:
      containers:
        - name: nested-container-name
---
apiVersion: v1
kind: Service
metadata:
  name: epsx-backend
"#;
        let (namespace, inventory) = k8s_render_inventory(rendered).expect("valid inventory");
        assert_eq!(namespace.as_deref(), Some("epsx-staging"));
        assert_eq!(
            inventory.into_iter().collect::<Vec<_>>(),
            vec!["Deployment/epsx-backend", "Service/epsx-backend"]
        );
    }

    #[test]
    fn migration_versions_use_the_prefix_before_the_name() {
        assert_eq!(
            migration_version("00000000000001_consolidated_baseline_v6"),
            "00000000000001"
        );
        assert_eq!(
            migration_version("20260723000000_bind_refresh_tokens_to_client"),
            "20260723000000"
        );
    }

    #[test]
    fn destructive_sql_markers_are_case_insensitive() {
        assert!(contains_destructive_sql("drop table users cascade"));
        assert!(contains_destructive_sql("DELETE FROM users"));
        assert!(!contains_destructive_sql(
            "ALTER TABLE users ADD COLUMN active BOOLEAN"
        ));
    }

    #[test]
    fn reconciliation_report_detects_event_and_wallet_drift() {
        let source = vec![ReconcileRecord {
            source_event_id: "event-1".into(),
            wallet_address: "0x1111111111111111111111111111111111111111".into(),
            status: "sent".into(),
            provider_message_id: None,
            provider_event_id: None,
            template_id: None,
            preference_hash: None,
            broadcast: false,
        }];
        let target = vec![ReconcileRecord {
            source_event_id: "event-2".into(),
            wallet_address: "all".into(),
            status: "pending".into(),
            provider_message_id: None,
            provider_event_id: None,
            template_id: None,
            preference_hash: None,
            broadcast: true,
        }];
        let report = build_reconciliation_report(&(source, 0, 0), &(target, 0, 0));
        assert_eq!(report.missing_target_events, 1);
        assert_eq!(report.orphan_target_events, 1);
        assert!(!report.wallet_checksum_match);
        assert!(!report.source_target_event_set_match);
        assert!(!report.status_distribution_match);
        assert!(!report.broadcast_count_match);
    }

    #[test]
    fn reconciliation_report_accepts_matching_provider_backed_records() {
        let records = vec![
            ReconcileRecord {
                source_event_id: "event-1".into(),
                wallet_address: "0x1111111111111111111111111111111111111111".into(),
                status: "sent".into(),
                provider_message_id: Some("provider-1".into()),
                provider_event_id: Some("provider-event-1".into()),
                template_id: Some("template-1".into()),
                preference_hash: Some("a".repeat(64)),
                broadcast: false,
            },
            ReconcileRecord {
                source_event_id: "event-2".into(),
                wallet_address: "all".into(),
                status: "pending".into(),
                provider_message_id: None,
                provider_event_id: None,
                template_id: None,
                preference_hash: None,
                broadcast: true,
            },
        ];
        let report = build_reconciliation_report(&(records.clone(), 0, 0), &(records, 0, 0));
        assert!(report.wallet_checksum_match);
        assert!(report.source_target_event_set_match);
        assert!(report.status_distribution_match);
        assert!(report.broadcast_count_match);
        assert_eq!(report.target_sent_without_provider_id, 0);
    }

    #[test]
    fn reconciliation_report_detects_template_preference_and_provider_identity_drift() {
        let source = ReconcileRecord {
            source_event_id: "event-identity".into(),
            wallet_address: "0x1111111111111111111111111111111111111111".into(),
            status: "sent".into(),
            provider_message_id: Some("provider-1".into()),
            provider_event_id: Some("provider-event-1".into()),
            template_id: Some("template-1".into()),
            preference_hash: Some("a".repeat(64)),
            broadcast: false,
        };
        let mut target = source.clone();
        target.provider_message_id = Some("provider-2".into());
        target.provider_event_id = Some("provider-event-2".into());
        target.template_id = Some("template-2".into());
        target.preference_hash = Some("b".repeat(64));
        let report = build_reconciliation_report(&(vec![source], 0, 0), &(vec![target], 0, 0));
        assert_eq!(report.template_identity_drift, 1);
        assert_eq!(report.preference_identity_drift, 1);
        assert_eq!(report.provider_identity_drift, 1);
    }

    #[test]
    fn reconciliation_accepts_durable_preference_suppression_status() {
        assert!(valid_reconcile_record(&ReconcileRecord {
            source_event_id: "event-suppressed".into(),
            wallet_address: "0x1111111111111111111111111111111111111111".into(),
            status: "suppressed".into(),
            provider_message_id: None,
            provider_event_id: None,
            template_id: None,
            preference_hash: None,
            broadcast: false,
        }));
    }

    #[test]
    fn legacy_mapper_preserves_target_notification_fields_without_writing() {
        let (record, preserved_fields) = map_legacy_backfill_record(
            r#"{
                "id":"11111111-1111-4111-8111-111111111111",
                "recipient_wallet_address":"0XAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "topic_name":null,
                "title":"Legacy payment",
                "body":"Payment received",
                "notification_type":"payment",
                "priority":"high",
                "channels":{"in_app":true},
                "action_url":"/payments/legacy-1",
                "data_payload":{"amount":"10"},
                "expires_at":"2026-08-01T00:00:00Z",
                "created_at":"2026-07-24T00:00:00Z",
                "status":"created"
            }"#,
        )
        .expect("rich legacy row should map");
        assert_eq!(
            record.source_event_id,
            "legacy.wallet_notification:11111111-1111-4111-8111-111111111111"
        );
        assert_eq!(
            record.wallet_address,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(record.status, "pending");
        assert_eq!(preserved_fields, 10);
    }

    #[test]
    fn legacy_mapper_rejects_invalid_preserved_fields() {
        let invalid_timestamp = r#"{
            "id":"11111111-1111-4111-8111-111111111111",
            "recipient_wallet_address":"0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "topic_name":null,
            "body":"body",
            "created_at":"not-a-timestamp",
            "status":"created"
        }"#;
        assert!(map_legacy_backfill_record(invalid_timestamp).is_err());

        let invalid_payload = r#"{
            "id":"11111111-1111-4111-8111-111111111111",
            "recipient_wallet_address":"0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "topic_name":null,
            "data_payload":["not","an","object"],
            "status":"created"
        }"#;
        assert!(map_legacy_backfill_record(invalid_payload).is_err());
    }

    #[test]
    fn compatibility_route_probe_requires_registered_method_and_path() {
        let source = r#"
            Router::new()
                .route("/api/v1/notification/list", get(list_notifications))
                .route(
                    "/api/v1/notification/{id}/acknowledge",
                    put(acknowledge),
                )
        "#;
        assert!(route_registration_present(
            source,
            "GET",
            "/api/v1/notification/list"
        ));
        assert!(route_registration_present(
            source,
            "PUT",
            "/api/v1/notification/{id}/acknowledge"
        ));
        assert!(!route_registration_present(
            source,
            "POST",
            "/api/v1/notification/list"
        ));
        assert!(!route_registration_present(
            source,
            "GET",
            "/api/v1/notification/missing"
        ));
    }
}
